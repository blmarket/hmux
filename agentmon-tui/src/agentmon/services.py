from __future__ import annotations

import os
import re
import shlex
import stat
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from . import claude_transcript, transcript as codex_transcript
from .model import (
    AgentRun,
    LaunchDraft,
    LaunchStep,
    PromptHistory,
    Repository,
    normalize_launch_agent,
)
from .transcript import Transcript, TranscriptError


class CommandError(RuntimeError):
    pass


# Foreground commands hmux reports for a bare interactive shell. Anything else
# running in an agentless pane is treated as "some app" rather than an idle
# shell, so it earns a distinct badge in the dashboard.
SHELL_COMMANDS = frozenset(
    {"sh", "bash", "zsh", "fish", "dash", "ksh", "mksh", "tcsh", "csh", "ash"}
)


def _is_shell_command(command: str) -> bool:
    """Return whether a pane's foreground command is just an interactive shell."""
    # Older hmux builds do not expose pane_current_command. Preserve their
    # previous agentless-window behavior instead of labelling every pane an app.
    if not command:
        return True
    # Login shells are reported with a leading dash (e.g. "-zsh").
    return command.lstrip("-") in SHELL_COMMANDS


Runner = Callable[..., subprocess.CompletedProcess[str]]


def _run(
    args: list[str], *, cwd: Path | None = None, check: bool = True
) -> subprocess.CompletedProcess[str]:
    try:
        result = subprocess.run(
            args, cwd=cwd, text=True, capture_output=True, check=False
        )
    except OSError as exc:
        raise CommandError(str(exc)) from exc
    if check and result.returncode:
        message = result.stderr.strip() or result.stdout.strip() or "command failed"
        raise CommandError(f"{shlex.join(args)}: {message}")
    return result


def discover_repository(start: Path = Path.cwd()) -> Repository:
    root = Path(_run(["git", "-C", str(start), "rev-parse", "--show-toplevel"]).stdout.strip())
    common_raw = _run(
        ["git", "-C", str(root), "rev-parse", "--git-common-dir"]
    ).stdout.strip()
    common_dir = Path(common_raw)
    if not common_dir.is_absolute():
        common_dir = root / common_dir
    branch = _run(
        ["git", "-C", str(root), "branch", "--show-current"]
    ).stdout.strip() or "(detached)"
    return Repository(root=root.resolve(), common_dir=common_dir.resolve(), branch=branch)


@dataclass(frozen=True)
class SocketSelection:
    path: str
    warning: str = ""


def default_socket() -> str:
    """Return the socket pathname used by a plain tmux client."""
    base = Path(os.environ.get("TMUX_TMPDIR") or "/tmp")
    return str(base / f"tmux-{os.geteuid()}" / "default")


def _tmux_socket(value: str) -> str | None:
    """Extract the socket path from tmux's `socket,pid,index` value."""
    parts = value.rsplit(",", 2)
    return parts[0] if len(parts) == 3 and parts[0] else None


def _supports_hmux_agent_status(path: str, tmux: str) -> bool:
    """Return whether this server evaluates hmux's agent-status format vars."""
    result = _run(
        [tmux, "-S", path, "list-panes", "-a", "-F", "#{pane_agent_state}"],
        check=False,
    )
    return result.returncode == 0 and any(
        line.strip() for line in result.stdout.splitlines()
    )


def discover_socket(
    *, requested: str | None = None, tmux: str = "tmux"
) -> SocketSelection:
    """Select and verify an hmux/tmux-compatible server socket."""
    hmux_socket = os.environ.get("HMUX_SOCKET")
    tmux_socket = _tmux_socket(os.environ.get("TMUX", ""))
    candidates: list[tuple[str, str]] = []
    if requested:
        candidates.append((requested, "explicit"))
    elif hmux_socket:
        candidates.append((hmux_socket, "hmux"))
    elif tmux_socket:
        candidates.append((tmux_socket, "session"))
    else:
        default = Path(default_socket())
        candidates.append((str(default), "discovered"))
        try:
            siblings = sorted(default.parent.iterdir())
        except OSError:
            siblings = []
        for path in siblings:
            try:
                is_socket = stat.S_ISSOCK(path.stat().st_mode)
            except OSError:
                continue
            if is_socket and str(path) != str(default):
                candidates.append((str(path), "discovered"))

    failures: list[str] = []
    for path, _source in candidates:
        result = _run([tmux, "-S", path, "list-sessions"], check=False)
        if result.returncode == 0:
            warning = ""
            if not _supports_hmux_agent_status(path, tmux):
                warning = (
                    "Selected tmux server does not expose hmux agent status; "
                    "agent runs may not be visible."
                )
            return SocketSelection(path, warning)
        failures.append(path)

    detail = ", ".join(failures) if failures else "no candidate sockets"
    raise CommandError(
        "no reachable hmux socket found "
        f"({detail}); use --socket PATH, set HMUX_SOCKET, or start hmux at a "
        "tmux-discoverable socket"
    )


class AgentmonService:
    def __init__(
        self, repo: Repository | None, *, socket: str, tmux: str = "tmux"
    ) -> None:
        self.repo = repo
        self.socket = socket
        self.tmux = tmux
        self._repositories = {} if repo is None else {repo.common_dir: repo}
        self._worktree_contexts = (
            {}
            if repo is None
            else {repo.root.resolve(): (repo, repo.branch)}
        )

    def for_repository(self, repo: Repository | None) -> AgentmonService:
        """Return a service whose Git operations are scoped to ``repo``."""
        if repo is None or (
            self.repo is not None and repo.common_dir == self.repo.common_dir
        ):
            return self
        return AgentmonService(repo, socket=self.socket, tmux=self.tmux)

    def for_run(self, run: AgentRun) -> AgentmonService:
        return self.for_repository(run.repository)

    def suggested_worktree(self, branch: str) -> Path:
        # Keep worktrees as siblings while allowing conventional feature/foo refs.
        directory = re.sub(r"[^A-Za-z0-9._-]+", "-", branch).strip("-")
        return self.repo.root.parent / (directory or "new-run")

    def _registered_worktrees(self) -> set[Path]:
        result = _run(
            ["git", "-C", str(self.repo.root), "worktree", "list", "--porcelain"]
        )
        return {
            Path(line.removeprefix("worktree ")).resolve()
            for line in result.stdout.splitlines()
            if line.startswith("worktree ")
        }

    def validate_draft(self, branch: str, prompt: str) -> LaunchDraft:
        branch = branch.strip()
        if not branch:
            raise ValueError("Enter a branch name")
        check = _run(["git", "check-ref-format", "--branch", branch], check=False)
        if check.returncode:
            raise ValueError(check.stderr.strip() or "Invalid branch name")
        exists = _run(
            ["git", "-C", str(self.repo.root), "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"],
            check=False,
        )
        existing_branch = exists.returncode == 0
        worktree = self.suggested_worktree(branch)
        overwrite_worktree = False
        if worktree.exists():
            resolved = worktree.resolve()
            registered = self._registered_worktrees()
            if resolved == self.repo.root.resolve() or resolved not in registered:
                raise ValueError(f"ERROR: Cannot overwrite non-worktree path: {worktree}")
            if self._worktree_state(resolved) == "dirty":
                raise ValueError(f"ERROR: Cannot overwrite dirty worktree: {worktree}")
            overwrite_worktree = True
        if not prompt.strip():
            raise ValueError("Write a prompt before continuing")
        return LaunchDraft(
            branch=branch,
            worktree=worktree,
            prompt=prompt,
            overwrite_worktree=overwrite_worktree,
            existing_branch=existing_branch,
            repository=self.repo,
        )

    def _git_common_dir(self, cwd: Path) -> Path | None:
        result = _run(
            ["git", "-C", str(cwd), "rev-parse", "--git-common-dir"], check=False
        )
        if result.returncode:
            return None
        path = Path(result.stdout.strip())
        return (cwd / path).resolve() if not path.is_absolute() else path.resolve()

    def _branch_at(self, cwd: Path) -> str:
        result = _run(
            ["git", "-C", str(cwd), "branch", "--show-current"], check=False
        )
        return result.stdout.strip() or "(detached)"

    def _worktree_root(self, cwd: Path) -> Path:
        result = _run(
            ["git", "-C", str(cwd), "rev-parse", "--show-toplevel"], check=False
        )
        if result.returncode == 0 and result.stdout.strip():
            return Path(result.stdout.strip()).resolve()
        return cwd.resolve()

    def _repository_for_worktree(
        self, cwd: Path, common_dir: Path
    ) -> Repository:
        cached = self._repositories.get(common_dir)
        if cached is not None:
            return cached
        worktree = self._worktree_root(cwd)
        project_root = common_dir.parent if common_dir.name == ".git" else worktree
        repository = Repository(
            root=project_root.resolve(),
            common_dir=common_dir,
            branch=self._branch_at(project_root),
        )
        self._repositories[common_dir] = repository
        return repository

    @staticmethod
    def _pane_directory(raw: str) -> Path:
        """Normalize Linux's marker for a process cwd whose directory was removed."""
        return Path(raw.removesuffix(" (deleted)")).resolve()

    def _prompt_preview(self, cwd: Path, limit: int = 120) -> str:
        """Return a single-line glimpse of the prompt belonging to this run."""
        prompt = self._worktree_root(cwd) / "prompt.md"
        try:
            text = prompt.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            return "—"
        preview = " ".join(text.split())
        if not preview:
            return "(empty)"
        return preview if len(preview) <= limit else preview[: limit - 1].rstrip() + "…"

    def _worktree_state(self, cwd: Path) -> str:
        """Describe whether a worktree needs attention before removal."""
        status = _run(
            ["git", "-C", str(cwd), "status", "--porcelain"], check=False
        )
        if status.returncode:
            return "unknown"
        if status.stdout:
            return "dirty"
        merged = _run(
            [
                "git", "-C", str(cwd), "merge-base", "--is-ancestor", "HEAD",
                self.repo.branch,
            ],
            check=False,
        )
        return "merged" if merged.returncode == 0 else "unmerged"

    def runs(self) -> list[AgentRun]:
        fields = "\t".join(
            (
                "#{pane_id}",
                "#{session_name}:#{window_index}.#{pane_index}",
                "#{window_name}",
                "#{pane_agent}",
                "#{pane_agent_state}",
                "#{pane_current_path}",
                "#{pane_agent_session_id}",
                "#{pane_active}",
                "#{pane_current_command}",
            )
        )
        result = _run(
            [self.tmux, "-S", self.socket, "list-panes", "-a", "-F", fields]
        )
        windows: dict[str, list[tuple[tuple[bool, bool], list[str]]]] = {}
        for line in result.stdout.splitlines():
            parts = line.split("\t", 8)
            if len(parts) != 9:
                continue
            (
                _pane_id,
                location,
                _name,
                agent,
                _state,
                cwd_raw,
                _session_id,
                active,
                _command,
            ) = parts
            if not cwd_raw:
                continue
            window_location = location.rsplit(".", 1)[0]
            score = (bool(agent), active == "1")
            windows.setdefault(window_location, []).append((score, parts))

        runs: list[AgentRun] = []
        for window_location, candidates in windows.items():
            selected: tuple[list[str], Path, Path] | None = None
            ordered = sorted(candidates, key=lambda item: item[0], reverse=True)
            for _score, parts in ordered:
                cwd = self._pane_directory(parts[5])
                common_dir = self._git_common_dir(cwd)
                if common_dir is not None:
                    selected = (parts, cwd, common_dir)
                    break
            if selected is None:
                parts = ordered[0][1]
                cwd = self._pane_directory(parts[5])
                common_dir = None
            else:
                parts, cwd, common_dir = selected
            (
                _pane_id,
                _location,
                window_name,
                agent,
                state,
                _cwd_raw,
                session_id,
                _active,
                pane_command,
            ) = parts
            # hmux only reports an agent name for recognized coding agents. When
            # a pane has none, distinguish an idle shell (badge "----") from any
            # other running program (badge "????") by its foreground command.
            effective_agent = agent or (
                "window" if _is_shell_command(pane_command) else "app"
            )
            if common_dir is None:
                cached_context = self._worktree_contexts.get(cwd)
                if cached_context is None:
                    repository = None
                    branch = cwd.name or str(cwd)
                    worktree_state = "not-git"
                else:
                    repository, branch = cached_context
                    worktree_state = "unknown"
                prompt_preview = "—"
            else:
                repository = self._repository_for_worktree(cwd, common_dir)
                scoped_service = self.for_repository(repository)
                branch = self._branch_at(cwd)
                prompt_preview = self._prompt_preview(cwd)
                worktree_state = scoped_service._worktree_state(cwd)
                self._worktree_contexts[cwd] = (repository, branch)
            runs.append(
                AgentRun(
                    pane_id=f"window:{window_location}",
                    location=window_location,
                    branch=branch,
                    state=state or "none",
                    agent=effective_agent,
                    worktree=cwd,
                    prompt_preview=prompt_preview,
                    worktree_state=worktree_state,
                    session_id=session_id or None,
                    repository=repository,
                    window_name=window_name,
                )
            )
        return runs

    def run_transcript(self, run: AgentRun) -> Transcript:
        """Load the local user-interaction transcript for an active agent run."""
        if not run.session_id:
            raise TranscriptError("This run does not expose an agent session id yet")
        if run.agent == "codex":
            return codex_transcript.load_transcript(run.session_id)
        if run.agent in {"claude", "claude-code"}:
            return claude_transcript.load_transcript(run.session_id)
        raise TranscriptError(f"Unsupported transcript agent: {run.agent}")

    def recent_finished(self, active: list[AgentRun], limit: int = 5) -> list[AgentRun]:
        """Recover recent agent worktrees whose hmux agent pane is gone."""
        active_paths = {run.worktree.resolve() for run in active}
        candidates: list[tuple[int, AgentRun]] = []
        for worktree in self._registered_worktrees():
            if worktree == self.repo.root.resolve() or any(
                worktree == path or worktree in path.parents for path in active_paths
            ):
                continue
            prompt = worktree / "prompt.md"
            if not prompt.is_file() or self._git_common_dir(worktree) != self.repo.common_dir:
                continue
            timestamp = _run(
                [
                    "git", "-C", str(worktree), "log", "-1", "--format=%ct", "--",
                    "prompt.md",
                ],
                check=False,
            ).stdout.strip()
            if not timestamp.isdigit():
                continue
            candidates.append(
                (
                    int(timestamp),
                    AgentRun(
                        pane_id=f"finished:{worktree}",
                        location="0:",
                        branch=self._branch_at(worktree),
                        state="exited",
                        agent="finished",
                        worktree=worktree,
                        prompt_preview=self._prompt_preview(worktree),
                        worktree_state=self._worktree_state(worktree),
                        repository=self.repo,
                    ),
                )
            )
        candidates.sort(key=lambda item: item[0], reverse=True)
        return [run for _, run in candidates[:limit]]

    def recent_finished_all(
        self, active: list[AgentRun], limit_per_repository: int = 5
    ) -> list[AgentRun]:
        """Recover finished runs for every repository represented by active panes."""
        repositories = (
            {} if self.repo is None else {self.repo.common_dir: self.repo}
        )
        for run in active:
            if run.repository is not None:
                repositories[run.repository.common_dir] = run.repository

        finished: list[AgentRun] = []
        for common_dir, repository in repositories.items():
            scoped = self.for_repository(repository)
            repository_active = [
                run
                for run in active
                if run.repository is not None
                and run.repository.common_dir == common_dir
            ]
            finished.extend(
                scoped.recent_finished(
                    repository_active,
                    limit=limit_per_repository,
                )
            )
        return finished

    def select_window(self, run: AgentRun) -> None:
        if run.state == "exited":
            raise CommandError("The agent window is closed; press w to open a shell at its worktree")
        window = run.location.split(":", 1)[1].split(".", 1)[0]
        session = run.location.split(":", 1)[0]
        _run([self.tmux, "-S", self.socket, "select-window", "-t", f"{session}:{window}"])

    def open_shell_window(self, run: AgentRun) -> str:
        """Create and select a shell window in the agent's current directory."""
        session = run.location.split(":", 1)[0]
        result = _run(
            [
                self.tmux,
                "-S",
                self.socket,
                "new-window",
                "-t",
                f"{session}:",
                "-P",
                "-F",
                "#{session_name}:#{window_index}",
                "-c",
                str(run.worktree),
            ]
        )
        return result.stdout.strip()

    def cleanup_worktree(self, run: AgentRun) -> None:
        """Remove a finished worktree after rechecking that cleanup is safe."""
        worktree = run.worktree.resolve()
        if run.state != "exited":
            raise CommandError("Only finished agent worktrees can be cleaned up")
        if worktree == self.repo.root.resolve() or worktree not in self._registered_worktrees():
            raise CommandError(f"Not a removable registered worktree: {worktree}")
        state = self._worktree_state(worktree)
        if state != "merged":
            raise CommandError(
                f"Worktree cleanup refused: state is {state}, not clean and merged"
            )
        _run(["git", "-C", str(self.repo.root), "worktree", "remove", str(worktree)])

    def restart_draft(self, run: AgentRun) -> LaunchDraft:
        """Build a new launch from a finished, safely replaceable run."""
        worktree = run.worktree.resolve()
        if run.state != "exited":
            raise CommandError("Only finished agent runs can be restarted")
        if worktree == self.repo.root.resolve() or worktree not in self._registered_worktrees():
            raise CommandError(f"Not a removable registered worktree: {worktree}")
        state = self._worktree_state(worktree)
        if state != "merged":
            raise CommandError(
                f"Worktree cleanup refused: state is {state}, not clean and merged"
            )
        try:
            prompt = (worktree / "prompt.md").read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise CommandError(f"Could not read {worktree / 'prompt.md'}: {exc}") from exc
        return LaunchDraft(
            branch=run.branch,
            worktree=worktree,
            prompt=prompt,
            overwrite_worktree=True,
            restart_worktree=True,
            repository=self.repo,
        )

    def populate_draft(self, run: AgentRun) -> LaunchDraft:
        """Build a launch form draft from a selected run's prompt."""
        worktree = self._worktree_root(run.worktree)
        agent = normalize_launch_agent(run.agent)
        try:
            prompt = (worktree / "prompt.md").read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            raise CommandError(f"Could not read {worktree / 'prompt.md'}: {exc}") from exc

        registered = self._registered_worktrees()
        if (
            run.state == "exited"
            and worktree != self.repo.root.resolve()
            and worktree in registered
            and self._worktree_state(worktree) == "merged"
        ):
            return LaunchDraft(
                branch=run.branch,
                worktree=worktree,
                prompt=prompt,
                overwrite_worktree=True,
                restart_worktree=True,
                agent=agent,
                repository=self.repo,
            )

        return LaunchDraft(
            branch=run.branch,
            worktree=self.suggested_worktree(run.branch),
            prompt=prompt,
            agent=agent,
            repository=self.repo,
        )

    def prompt_history(self, limit: int = 50) -> list[PromptHistory]:
        result = _run(
            [
                "git", "-C", str(self.repo.root), "log", "--all",
                f"--max-count={limit}", "--format=%H%x09%cs%x09%s", "--", "prompt.md",
            ]
        )
        history: list[PromptHistory] = []
        for line in result.stdout.splitlines():
            parts = line.split("\t", 2)
            if len(parts) != 3:
                continue
            commit, date, subject = parts
            shown = _run(
                ["git", "-C", str(self.repo.root), "show", f"{commit}:prompt.md"],
                check=False,
            )
            if shown.returncode == 0:
                history.append(PromptHistory(commit, date, subject, shown.stdout))
        return history

    def launch(self, draft: LaunchDraft, progress: Callable[[LaunchStep], None]) -> str:
        if draft.overwrite_worktree:
            state = self._worktree_state(draft.worktree)
            if draft.restart_worktree:
                if state != "merged":
                    raise CommandError(
                        f"Worktree cleanup refused: state is {state}, not clean and merged"
                    )
            elif state == "dirty":
                raise CommandError(
                    f"Worktree overwrite refused: state is {state}, not clean"
                )
            _run(
                [
                    "git", "-C", str(self.repo.root), "worktree", "remove",
                    *([] if draft.restart_worktree else ["--force"]),
                    str(draft.worktree),
                ]
            )
            progress(LaunchStep("Existing worktree removed", str(draft.worktree)))
        worktree_args = ["git", "-C", str(self.repo.root), "worktree", "add"]
        if draft.restart_worktree:
            worktree_args.extend(["-B", draft.branch, str(draft.worktree)])
        elif draft.existing_branch:
            worktree_args.extend([str(draft.worktree), draft.branch])
        else:
            worktree_args.extend(["-b", draft.branch, str(draft.worktree)])
        _run(worktree_args)
        progress(LaunchStep("Branch and worktree created", str(draft.worktree)))

        (draft.worktree / "prompt.md").write_text(draft.prompt, encoding="utf-8")
        _run(["git", "add", "prompt.md"], cwd=draft.worktree)
        _run(
            ["git", "commit", "-m", f"agentmon: add prompt for {draft.branch}"],
            cwd=draft.worktree,
        )
        commit = _run(["git", "rev-parse", "--short", "HEAD"], cwd=draft.worktree).stdout.strip()
        progress(LaunchStep("prompt.md committed", commit))

        command = self._agent_command(draft.agent)
        result = _run(
            [
                self.tmux, "-S", self.socket, "new-window", "-d", "-t", "0:",
                "-P", "-F", "#{window_index}", "-n", draft.branch,
                "-c", str(draft.worktree), command,
            ]
        )
        window = result.stdout.strip()
        progress(LaunchStep("hmux window and agent started", window))
        return window

    def _agent_command(self, agent: str) -> str:
        if agent == "codex":
            return 'exec codex --yolo "$(cat prompt.md)"'
        if agent == "claude":
            return 'exec claude --dangerously-skip-permissions "$(cat prompt.md)"'
        raise CommandError(f"Unsupported launch agent: {agent}")
