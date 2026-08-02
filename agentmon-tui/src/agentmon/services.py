from __future__ import annotations

import os
import re
import selectors
import shlex
import stat
import subprocess
import threading
import time
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


# The launch instruction lives in instruction.md; prompt.md is the legacy name
# still honored when reading so older worktrees keep working.
INSTRUCTION_FILENAMES = ("instruction.md", "prompt.md")

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

RUN_FORMAT = "\t".join(
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

_SUBSCRIPTION_NAME = "agentmon"
_RUN_CHANGE_NOTIFICATIONS = (
    f"%subscription-changed {_SUBSCRIPTION_NAME} ",
    "%layout-change ",
    "%session-changed ",
    "%session-closed ",
    "%session-renamed ",
    "%sessions-changed",
    "%unlinked-window-",
    "%window-add ",
    "%window-close ",
    "%window-pane-changed ",
    "%window-renamed ",
)


def _run_change_notification(line: str) -> bool:
    return line.startswith(_RUN_CHANGE_NOTIFICATIONS)


def _subscription_command() -> str:
    value = f"{_SUBSCRIPTION_NAME}:%*:{RUN_FORMAT}"
    return f"refresh-client -B {shlex.quote(value)}\n"


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

    def _validated_branch(self, branch: str) -> str:
        branch = branch.strip()
        if not branch:
            raise ValueError("Enter a branch name")
        check = _run(["git", "check-ref-format", "--branch", branch], check=False)
        if check.returncode:
            raise ValueError(check.stderr.strip() or "Invalid branch name")
        return branch

    def branch_exists(self, branch: str) -> bool:
        result = _run(
            [
                "git", "-C", str(self.repo.root), "show-ref", "--verify",
                "--quiet", f"refs/heads/{branch}",
            ],
            check=False,
        )
        return result.returncode == 0

    def validate_draft(self, branch: str, prompt: str) -> LaunchDraft:
        branch = self._validated_branch(branch)
        existing_branch = self.branch_exists(branch)
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
            raise ValueError("Write an instruction before continuing")
        return LaunchDraft(
            branch=branch,
            worktree=worktree,
            prompt=prompt,
            overwrite_worktree=overwrite_worktree,
            existing_branch=existing_branch,
            repository=self.repo,
        )

    def prepare_worktree(
        self, branch: str, *, base: str | None = None, reset: bool = False
    ) -> Path:
        """Create a sibling worktree only; instruction and agent come later.

        ``base`` is the start-point a freshly forked branch inherits; when
        omitted the new branch forks from the main worktree's HEAD. Pass the
        cursor's worktree branch to fork from it instead of the project base.
        """
        branch = self._validated_branch(branch)
        worktree = self.suggested_worktree(branch)
        if worktree.exists():
            raise ValueError(f"Worktree path already exists: {worktree}")
        args = ["git", "-C", str(self.repo.root), "worktree", "add"]
        if reset:
            args.extend(["-B", branch, str(worktree)])
            if base is not None:
                args.append(base)
        elif self.branch_exists(branch):
            # Reuse the existing branch untouched; base does not apply.
            args.extend([str(worktree), branch])
        else:
            args.extend(["-b", branch, str(worktree)])
            if base is not None:
                args.append(base)
        _run(args)
        return worktree

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

    def read_instruction(self, worktree: Path) -> str | None:
        """Return the worktree's instruction, or None when neither file exists."""
        for name in INSTRUCTION_FILENAMES:
            try:
                return (worktree / name).read_text(encoding="utf-8")
            except (OSError, UnicodeError):
                continue
        return None

    def save_instruction(
        self, worktree: Path, branch: str, text: str | None = None
    ) -> str | None:
        """Persist instruction.md, retire prompt.md, and commit both.

        With ``text`` the content is written first; with ``None`` whatever is
        already on disk is committed. Returns the short commit hash, or None
        when nothing changed.
        """
        path = worktree / "instruction.md"
        if text is not None:
            path.write_text(text, encoding="utf-8")
        if path.exists():
            _run(["git", "add", "instruction.md"], cwd=worktree)
        legacy = worktree / "prompt.md"
        if legacy.exists():
            _run(
                ["git", "rm", "-f", "--ignore-unmatch", "--", "prompt.md"],
                cwd=worktree,
            )
            legacy.unlink(missing_ok=True)
        status = _run(
            ["git", "status", "--porcelain", "--", *INSTRUCTION_FILENAMES],
            cwd=worktree,
        )
        # Committing by pathspec keeps unrelated staged changes out of the
        # commit; unknown names would make git error, so use what changed.
        changed = [line[3:] for line in status.stdout.splitlines() if len(line) > 3]
        if not changed:
            return None
        _run(
            [
                "git", "commit", "-m",
                f"agentmon: add instruction for {branch}",
                "--", *changed,
            ],
            cwd=worktree,
        )
        return _run(
            ["git", "rev-parse", "--short", "HEAD"], cwd=worktree
        ).stdout.strip()

    def instruction_edit_target(self, run: AgentRun) -> Path:
        """Return the instruction.md to edit, seeded from a legacy prompt.md."""
        worktree = self._worktree_root(run.worktree)
        path = worktree / "instruction.md"
        if not path.exists():
            legacy_text = self.read_instruction(worktree)
            if legacy_text is not None:
                path.write_text(legacy_text, encoding="utf-8")
        return path

    def run_instruction(self, run: AgentRun) -> str | None:
        """Return the instruction text belonging to a dashboard run."""
        return self.read_instruction(self._worktree_root(run.worktree))

    def commit_instruction(self, run: AgentRun) -> str | None:
        """Commit the on-disk instruction.md after an external edit."""
        worktree = self._worktree_root(run.worktree)
        if not (worktree / "instruction.md").exists():
            return None
        return self.save_instruction(worktree, run.branch)

    def _prompt_preview(self, cwd: Path, limit: int = 120) -> str:
        """Return a single-line glimpse of the instruction belonging to this run."""
        text = self.read_instruction(self._worktree_root(cwd))
        if text is None:
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

    def _session_names(self) -> set[str]:
        result = _run(
            [
                self.tmux,
                "-S",
                self.socket,
                "list-sessions",
                "-F",
                "#{session_name}",
            ]
        )
        return {line for line in result.stdout.splitlines() if line}

    def watch_runs(
        self, on_change: Callable[[], None], stop: threading.Event
    ) -> None:
        """Notify when tmux reports a dashboard-relevant format change."""
        selector = selectors.DefaultSelector()
        clients: dict[str, subprocess.Popen[bytes]] = {}
        buffers: dict[str, bytearray] = {}
        command = _subscription_command().encode()

        def remove(session: str) -> None:
            process = clients.pop(session, None)
            buffers.pop(session, None)
            if process is None:
                return
            if process.stdout is not None:
                try:
                    selector.unregister(process.stdout)
                except KeyError:
                    pass
            process.terminate()
            try:
                process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
            if process.stdin is not None:
                process.stdin.close()
            if process.stdout is not None:
                process.stdout.close()

        def add(session: str) -> None:
            try:
                process = subprocess.Popen(
                    [
                        self.tmux,
                        "-S",
                        self.socket,
                        "-C",
                        "attach-session",
                        "-f",
                        "no-output,ignore-size",
                        "-t",
                        session,
                    ],
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.DEVNULL,
                    bufsize=0,
                )
            except OSError as exc:
                raise CommandError(str(exc)) from exc
            if process.stdin is None or process.stdout is None:
                process.terminate()
                raise CommandError("tmux control client did not expose pipes")
            try:
                process.stdin.write(command)
                process.stdin.flush()
            except OSError as exc:
                process.terminate()
                process.wait()
                raise CommandError(str(exc)) from exc
            os.set_blocking(process.stdout.fileno(), False)
            clients[session] = process
            buffers[session] = bytearray()
            selector.register(process.stdout, selectors.EVENT_READ, session)

        def reconcile() -> None:
            sessions = self._session_names()
            for session in set(clients) - sessions:
                remove(session)
            for session in sessions - set(clients):
                add(session)

        try:
            reconcile()
            next_reconcile = time.monotonic() + 10
            while not stop.is_set():
                changed = False
                needs_reconcile = False
                for key, _mask in selector.select(timeout=0.5):
                    session = key.data
                    process = clients.get(session)
                    if process is None or process.stdout is None:
                        continue
                    try:
                        chunk = os.read(process.stdout.fileno(), 65536)
                    except BlockingIOError:
                        continue
                    if not chunk:
                        remove(session)
                        needs_reconcile = True
                        changed = True
                        continue
                    buffer = buffers[session]
                    buffer.extend(chunk)
                    while (newline := buffer.find(b"\n")) >= 0:
                        line = bytes(buffer[: newline + 1]).decode(
                            "utf-8", errors="replace"
                        )
                        del buffer[: newline + 1]
                        if _run_change_notification(line):
                            changed = True
                        if line.startswith(("%sessions-changed", "%session-closed ")):
                            needs_reconcile = True

                now = time.monotonic()
                if needs_reconcile or now >= next_reconcile:
                    reconcile()
                    next_reconcile = now + 10
                if changed:
                    on_change()
        finally:
            for session in list(clients):
                remove(session)
            selector.close()

    def runs(self) -> list[AgentRun]:
        result = _run(
            [self.tmux, "-S", self.socket, "list-panes", "-a", "-F", RUN_FORMAT]
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
        """Recover registered worktrees that no hmux window is sitting in.

        This covers both finished agent runs and freshly prepared worktrees
        that have no instruction yet, so the prepare/instruct/launch commands
        can target them.
        """
        active_paths = {run.worktree.resolve() for run in active}
        candidates: list[tuple[int, AgentRun]] = []
        for worktree in self._registered_worktrees():
            if worktree == self.repo.root.resolve() or any(
                worktree == path or worktree in path.parents for path in active_paths
            ):
                continue
            if self._git_common_dir(worktree) != self.repo.common_dir:
                continue
            timestamp = _run(
                [
                    "git", "-C", str(worktree), "log", "-1", "--format=%ct", "--",
                    *INSTRUCTION_FILENAMES,
                ],
                check=False,
            ).stdout.strip()
            if not timestamp.isdigit():
                timestamp = _run(
                    ["git", "-C", str(worktree), "log", "-1", "--format=%ct"],
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
        prompt = self.read_instruction(worktree)
        if prompt is None:
            raise CommandError(f"Could not read {worktree / 'instruction.md'}")
        return LaunchDraft(
            branch=run.branch,
            worktree=worktree,
            prompt=prompt,
            overwrite_worktree=True,
            restart_worktree=True,
            repository=self.repo,
        )

    def populate_draft(self, run: AgentRun) -> LaunchDraft:
        """Build a launch form draft from a selected run's instruction."""
        worktree = self._worktree_root(run.worktree)
        agent = normalize_launch_agent(run.agent)
        prompt = self.read_instruction(worktree)
        if prompt is None:
            raise CommandError(f"Could not read {worktree / 'instruction.md'}")

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
                f"--max-count={limit}", "--format=%H%x09%cs%x09%s", "--",
                *INSTRUCTION_FILENAMES,
            ]
        )
        history: list[PromptHistory] = []
        for line in result.stdout.splitlines():
            parts = line.split("\t", 2)
            if len(parts) != 3:
                continue
            commit, date, subject = parts
            for name in INSTRUCTION_FILENAMES:
                shown = _run(
                    ["git", "-C", str(self.repo.root), "show", f"{commit}:{name}"],
                    check=False,
                )
                if shown.returncode == 0:
                    history.append(PromptHistory(commit, date, subject, shown.stdout))
                    break
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

        commit = self.save_instruction(draft.worktree, draft.branch, draft.prompt)
        progress(LaunchStep("instruction.md committed", commit or "unchanged"))

        command = self._agent_command(draft.agent, draft.model, draft.effort)
        window = self._open_agent_window(draft.worktree, draft.branch, command)
        progress(LaunchStep("hmux window and agent started", window))
        return window

    def launch_agent(
        self,
        run: AgentRun,
        *,
        agent: str,
        model: str = "default",
        effort: str = "default",
        instruction: str = "",
    ) -> str:
        """Start an agent in an existing worktree, without touching branches."""
        worktree = self._worktree_root(run.worktree)
        with_instruction = bool(instruction.strip())
        if with_instruction:
            self.save_instruction(worktree, run.branch, instruction)
        command = self._agent_command(
            agent, model, effort, with_instruction=with_instruction
        )
        return self._open_agent_window(worktree, run.branch, command)

    def _open_agent_window(self, worktree: Path, name: str, command: str) -> str:
        result = _run(
            [
                self.tmux, "-S", self.socket, "new-window", "-d", "-t", "0:",
                "-P", "-F", "#{window_index}", "-n", name,
                "-c", str(worktree), command,
            ]
        )
        return result.stdout.strip()

    def _agent_command(
        self,
        agent: str,
        model: str = "default",
        effort: str = "default",
        *,
        with_instruction: bool = True,
    ) -> str:
        if agent == "codex":
            parts = ["codex", "--yolo"]
            if model != "default":
                parts.extend(["-m", model])
            if effort != "default":
                parts.extend(["-c", f"model_reasoning_effort={effort}"])
        elif agent == "claude":
            parts = ["claude", "--dangerously-skip-permissions"]
            if model != "default":
                parts.extend(["--model", model])
        else:
            raise CommandError(f"Unsupported launch agent: {agent}")
        command = "exec " + shlex.join(parts)
        if with_instruction:
            command += ' "$(cat instruction.md)"'
        return command
