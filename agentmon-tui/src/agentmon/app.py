from __future__ import annotations

import argparse
import os
import shlex
import subprocess
import tempfile
import threading
from collections.abc import Iterable
from dataclasses import replace
from datetime import datetime, timezone
from pathlib import Path

from rich.console import Console
from rich.text import Text
from textual import work
from textual.app import App, ComposeResult, SystemCommand
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.events import Key
from textual.screen import ModalScreen, Screen
from textual.widgets import Button, DataTable, Footer, Input, Label, Select, Static

from . import __version__
from .model import (
    AgentRun,
    DEFAULT_LAUNCH_AGENT,
    LAUNCH_AGENT_LABELS,
    LaunchDraft,
    LaunchStep,
    PromptHistory,
    Repository,
    normalize_launch_agent,
)
from .quota import QuotaReport, QuotaService, QuotaWindow
from .services import AgentmonService, CommandError, discover_repository, discover_socket
from .transcript import Transcript, TranscriptError, TranscriptMessage


STATE_EMOJI = {
    "working": "🔄",
    "blocked": "✋",
    "idle": "💤",
    "exited": "🏁",
    "none": "🪟",
}

AGENT_BADGE = {
    "codex": "CODX",
    "claude": "CLDE",
    "claude-code": "CLDE",
    "finished": "----",
    "window": "----",
    "app": "????",
}

WORKTREE_MARK = {
    "merged": "✓ merged",
    "unmerged": "↑ unmerged",
    "dirty": "! dirty",
    "unknown": "? unknown",
    "not-git": "—",
}


def agent_badge(agent: str) -> str:
    """Return a compact, terminal-safe label for an hmux agent identifier."""
    known = AGENT_BADGE.get(agent)
    if known is not None:
        return known
    compact = "".join(character for character in agent if character.isalnum())
    return (compact[:4] or "????").upper().ljust(4)


class NavigableDataTable(DataTable):
    """A data table that also supports Vim-style vertical navigation."""

    BINDINGS = [
        Binding("j", "cursor_down", "Cursor down", show=False),
        Binding("k", "cursor_up", "Cursor up", show=False),
    ]

    def __init__(self, **kwargs) -> None:
        super().__init__(**kwargs)
        self.repository_rows: set[int] = set()

    def _move_worktree_cursor(self, offset: int) -> None:
        if not self.row_count:
            return
        row = self.cursor_row
        while True:
            candidate = row + offset
            if candidate < 0 or candidate >= self.row_count:
                return
            row = candidate
            if row not in self.repository_rows:
                self.move_cursor(row=row)
                return

    def action_cursor_down(self) -> None:
        self._move_worktree_cursor(1)

    def action_cursor_up(self) -> None:
        self._move_worktree_cursor(-1)


class HistorySearchInput(Input):
    """Keep search active while arrow keys move through matching prompts."""

    def on_key(self, event: Key) -> None:
        screen = self.screen
        if not isinstance(screen, HistoryScreen):
            return
        if event.key == "up":
            screen.move_selection(-1)
        elif event.key == "down":
            screen.move_selection(1)
        else:
            return
        event.prevent_default()
        event.stop()


class DashboardScreen(Screen):
    BINDINGS = [
        ("n", "new_run", "New run"),
        ("p", "populate_run", "Populate from run"),
        Binding("enter", "open_run", "Open window", priority=True),
        ("w", "new_window", "New shell at CWD"),
        ("x", "cleanup", "Clean up worktree"),
        ("r", "reload", "Refresh"),
        ("t", "view_transcript", "Full transcript"),
        ("q", "quit", "Quit"),
    ]

    def compose(self) -> ComposeResult:
        yield Static(id="title")
        with Horizontal(id="dashboard-main"):
            with Vertical(id="runs-pane"):
                yield NavigableDataTable(
                    id="runs",
                    cursor_type="row",
                    zebra_stripes=True,
                    show_header=False,
                )
                yield Static("", id="notice")
            with Vertical(id="transcript-pane"):
                yield Static("Transcript", id="transcript-title")
                yield Static(
                    "Select a run to inspect its agent session.", id="transcript-meta"
                )
                with Vertical(id="transcript-scroll"):
                    yield Static("", id="transcript-content")
        yield Footer()

    def on_mount(self) -> None:
        self._subscription_stop = threading.Event()
        self._subscription_started = False
        self._transcript_session_id: str | None = None
        self._transcript_signature: tuple[object, ...] | None = None
        self._loaded_transcript: Transcript | None = None
        self._row_runs: list[AgentRun | None] = []
        self._row_repositories: list[Repository | None] = []
        table = self.query_one("#runs", DataTable)
        table.add_column("WORKTREE", width=36)
        table.add_column("GIT", width=12)
        table.add_column("HMUX", width=9)
        self.set_interval(30.0, self.refresh_runs)
        self.refresh_runs()

    def on_unmount(self) -> None:
        self._subscription_stop.set()

    def refresh_runs(self) -> None:
        # The hmux/git scans run in a worker thread instead of blocking the event
        # loop, which keeps keypresses responsive during snapshot reconciliation.
        self._load_runs()

    @work(thread=True, exclusive=True, group="runs")
    def _load_runs(self) -> None:
        """Gather run status off the event loop; apply results on the UI thread."""
        service = self.app.service
        try:
            active = service.runs()
            runs = active + service.recent_finished_all(active)
        except CommandError as exc:
            self.app.call_from_thread(self._show_disconnected, str(exc))
        else:
            self.app.call_from_thread(self._apply_runs, runs)

    @work(thread=True, exclusive=True, group="subscription")
    def _watch_run_changes(self) -> None:
        """Refresh snapshots when tmux format subscriptions report changes."""

        def changed() -> None:
            if not self._subscription_stop.is_set():
                self.app.call_from_thread(self.refresh_runs)

        try:
            self.app.service.watch_runs(changed, self._subscription_stop)
        except CommandError:
            # The periodic snapshot remains authoritative if the advisory
            # control-mode channel cannot be established.
            return

    def _apply_runs(self, runs: list[AgentRun]) -> None:
        selected = self._selected_run()
        selected_pane = selected.pane_id if selected is not None else None
        table = self.query_one("#runs", NavigableDataTable)
        selected_repository = (
            self._selected_repository()
            if table.cursor_row in table.repository_rows
            else None
        )
        selected_repository_key = (
            selected_repository.common_dir
            if selected_repository is not None
            else None
        )
        self.app.runs = runs
        table.clear()
        table.repository_rows.clear()
        self._row_runs = []
        self._row_repositories = []

        default_repository = self.app.service.repo
        grouped: dict[Path | None, tuple[Repository | None, list[AgentRun]]] = {}
        if default_repository is not None:
            grouped[default_repository.common_dir] = (default_repository, [])
        for run in runs:
            repository = run.repository
            key = repository.common_dir if repository is not None else None
            grouped.setdefault(key, (repository, []))[1].append(run)

        counts: dict[str, int] = {}
        selected_row: int | None = None
        first_worktree_row: int | None = None
        for group_index, (repository, repository_runs) in enumerate(grouped.values()):
            repository_row = len(self._row_runs)
            repository_key = (
                repository.common_dir if repository is not None else None
            )
            table.repository_rows.add(repository_row)
            if repository is None:
                group_label = Text("▾ non-Git windows", style="bold #8ec8ff")
                base_label = Text("directories", style="bold #8ec8ff")
            else:
                project_root = repository.project_root
                group_label = Text(
                    f"▾ {project_root.name}  {project_root}", style="bold #8ec8ff"
                )
                base_label = Text(
                    f"base:{repository.branch}", style="bold #8ec8ff"
                )
            table.add_row(
                group_label,
                base_label,
                "",
                key=f"repository:{group_index}",
            )
            self._row_runs.append(None)
            self._row_repositories.append(repository)
            if (
                selected_pane is None
                and repository_key is not None
                and repository_key == selected_repository_key
            ):
                selected_row = repository_row

            for index, run in enumerate(repository_runs):
                counts[run.state] = counts.get(run.state, 0) + 1
                connector = "└─" if index == len(repository_runs) - 1 else "├─"
                status = STATE_EMOJI.get(run.state, "❔")
                badge = agent_badge(run.agent)
                label = run.branch
                if (
                    run.state == "none"
                    and run.window_name
                    and run.window_name != run.branch
                ):
                    label = f"{run.window_name} · {run.branch}"
                table.add_row(
                    f"  {connector} {status} {badge} {label}",
                    WORKTREE_MARK.get(run.worktree_state, run.worktree_state),
                    run.location if run.state != "exited" else "—",
                    key=run.pane_id,
                )
                row = len(self._row_runs)
                self._row_runs.append(run)
                self._row_repositories.append(run.repository)
                if first_worktree_row is None:
                    first_worktree_row = row
                if run.pane_id == selected_pane:
                    selected_row = row

        summary = "  ".join(
            f"{n} {'window' if state == 'none' else state}"
            for state, n in counts.items()
        )
        self.query_one("#title", Static).update(f"agentmon    {summary or 'no active runs'}")
        notice = self.app.startup_warning
        if not runs and not notice:
            notice = "No agent runs found — press n to prepare one"
        self._set_notice(notice)
        if first_worktree_row is not None:
            table.move_cursor(
                row=selected_row if selected_row is not None else first_worktree_row
            )
        self._refresh_transcript()
        if not self._subscription_started:
            self._subscription_started = True
            self._watch_run_changes()

    def _set_notice(self, message: str) -> None:
        notice = self.query_one("#notice", Static)
        notice.update(message)
        notice.display = bool(message)

    def _show_disconnected(self, message: str) -> None:
        self.app.runs = []
        self.query_one("#runs", DataTable).clear()
        self._row_runs = []
        self._row_repositories = []
        self.query_one("#title", Static).update("agentmon    disconnected")
        self._set_notice(f"Could not read hmux: {message}")
        self._clear_transcript("Agent runs are unavailable while hmux is disconnected.")

    def _selected_run(self) -> AgentRun | None:
        table = self.query_one("#runs", DataTable)
        row = table.cursor_row
        if row >= len(self._row_runs):
            return None
        return self._row_runs[row]

    def _selected_repository(self) -> Repository | None:
        table = self.query_one("#runs", DataTable)
        row = table.cursor_row
        if row >= len(self._row_repositories):
            return None
        return self._row_repositories[row]

    def _set_transcript_meta(self, message: str | None) -> None:
        meta = self.query_one("#transcript-meta", Static)
        meta.update(message or "")
        meta.display = message is not None

    def _clear_transcript(self, message: str) -> None:
        self._transcript_session_id = None
        self._transcript_signature = None
        self._loaded_transcript = None
        self.query_one("#transcript-title", Static).update("Transcript")
        self._set_transcript_meta(None)
        self.query_one("#transcript-content", Static).update(message)

    def _refresh_transcript(self) -> None:
        run = self._selected_run()
        if run is None:
            self._clear_transcript("Select a run to inspect its agent session.")
            return

        agent_key = "claude" if run.agent == "claude-code" else run.agent
        agent_name = LAUNCH_AGENT_LABELS.get(agent_key, run.agent)
        if run.state == "none":
            self._transcript_session_id = None
            self._transcript_signature = None
            self._loaded_transcript = None
            self.query_one("#transcript-title", Static).update(
                f"Window · {run.window_name or run.branch}"
            )
            self._set_transcript_meta(
                f"HMUX       {run.location}\nWorktree   {run.worktree}"
            )
            self.query_one("#transcript-content", Static).update(
                "No coding agent is running in this window."
            )
            return
        self.query_one("#transcript-title", Static).update(
            f"Transcript · {run.branch} · {agent_name}"
        )
        self._set_transcript_meta(None)
        if not run.session_id:
            self._transcript_session_id = None
            self._transcript_signature = None
            self._loaded_transcript = None
            message = (
                "This finished run no longer has an active agent session."
                if run.state == "exited"
                else "Waiting for hmux to discover this agent's session id."
            )
            self.query_one("#transcript-content", Static).update(message)
            return

        changed_session = run.session_id != self._transcript_session_id
        self._transcript_session_id = run.session_id
        if changed_session:
            self._transcript_signature = None
            self._loaded_transcript = None
            self.query_one("#transcript-content", Static).update("Loading transcript…")
        self._load_transcript(run)

    @work(thread=True, exclusive=True, group="transcript")
    def _load_transcript(self, run: AgentRun) -> None:
        try:
            loaded = self.app.service.run_transcript(run)
        except (TranscriptError, OSError) as exc:
            self.app.call_from_thread(
                self._show_transcript_error, run.pane_id, run.session_id, str(exc)
            )
        else:
            self.app.call_from_thread(
                self._apply_transcript, run.pane_id, run.session_id, loaded
            )

    def _is_selected_session(self, pane_id: str, session_id: str | None) -> bool:
        selected = self._selected_run()
        return bool(
            selected
            and selected.pane_id == pane_id
            and selected.session_id == session_id
        )

    def _show_transcript_error(
        self, pane_id: str, session_id: str | None, message: str
    ) -> None:
        if not self._is_selected_session(pane_id, session_id):
            return
        self._transcript_signature = None
        self._loaded_transcript = None
        self.query_one("#transcript-content", Static).update(
            Text.assemble(("Transcript unavailable\n", "bold #ffcb6b"), message)
        )

    def _apply_transcript(
        self, pane_id: str, session_id: str | None, loaded: Transcript
    ) -> None:
        if not self._is_selected_session(pane_id, session_id):
            return
        signature: tuple[object, ...] = (
            loaded.rollout_path,
            loaded.skipped_lines,
            *((message.timestamp, message.text) for message in loaded.messages),
        )
        self._loaded_transcript = loaded
        if signature == self._transcript_signature:
            return
        self._transcript_signature = signature
        self.call_after_refresh(self._update_transcript_preview)

    def _update_transcript_preview(self) -> None:
        loaded = self._loaded_transcript
        if loaded is None:
            return
        preview = self.query_one("#transcript-scroll", Vertical)
        content = self.query_one("#transcript-content", Static)
        padding = content.styles.padding
        width = max(
            1,
            preview.content_region.width - padding.left - padding.right,
        )
        max_lines = max(
            1, preview.content_region.height - padding.top - padding.bottom
        )
        self.query_one("#transcript-content", Static).update(
            self._render_transcript_preview(
                loaded.messages,
                loaded.skipped_lines,
                console=self.app.console,
                width=width,
                max_lines=max_lines,
            )
        )

    @staticmethod
    def _render_transcript(
        messages: tuple[TranscriptMessage, ...], skipped_lines: tuple[int, ...]
    ) -> Text:
        rendered = Text()
        if not messages:
            rendered.append(
                "No user interactions were found.", style="italic #8492a0"
            )
        for index, message in enumerate(messages):
            if index:
                rendered.append("\n\n")
            timestamp = (
                f" · {DashboardScreen._local_timestamp(message.timestamp)}"
                if message.timestamp
                else ""
            )
            rendered.append(
                f"{message.role.upper()}{timestamp}\n", style="bold #8ec8ff"
            )
            rendered.append(message.text)
        if skipped_lines:
            lines = ", ".join(str(line) for line in skipped_lines)
            rendered.append(
                f"\n\nWarning: skipped malformed JSONL line(s): {lines}",
                style="#ffcb6b",
            )
        return rendered

    @staticmethod
    def _local_timestamp(timestamp: str) -> str:
        try:
            parsed = datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
        except ValueError:
            return timestamp
        if parsed.tzinfo is None:
            return timestamp
        return parsed.astimezone().strftime("%Y-%m-%d %H:%M %Z")

    @classmethod
    def _render_transcript_preview(
        cls,
        messages: tuple[TranscriptMessage, ...],
        skipped_lines: tuple[int, ...],
        *,
        console: Console,
        width: int,
        max_lines: int,
    ) -> Text:
        """Fill the preview in last, first, second-last priority order."""

        width = max(1, width)
        full = cls._render_transcript(messages, skipped_lines)
        full_lines = list(full.wrap(console, width))
        if len(full_lines) <= max_lines:
            return full
        if not messages:
            return cls._join_visual_lines(full_lines[:max_lines])

        if len(messages) == 1:
            visible = full_lines[:max_lines]
            if len(full_lines) > max_lines and visible:
                visible[-1].append(" …", style="italic #8492a0")
            return cls._join_visual_lines(visible)

        blocks = [
            list(
                cls._render_transcript((message,), ()).wrap(console, width)
            )
            for message in messages
        ]
        priority = [len(messages) - 1, 0, *range(len(messages) - 2, 0, -1)]
        selected: dict[int, list[Text]] = {}

        for message_index in priority:
            block = blocks[message_index]
            trial = {**selected, message_index: block}
            visible = cls._compose_prioritized_preview(
                trial,
                message_count=len(messages),
                skipped_lines=skipped_lines,
                width=width,
            )
            if len(visible) <= max_lines:
                selected = trial
                continue

            partial: list[Text] | None = None
            for line_count in range(1, len(block)):
                shortened_trial = {
                    **selected,
                    message_index: cls._ellipsize_visual_lines(
                        block[:line_count], width
                    ),
                }
                candidate = cls._compose_prioritized_preview(
                    shortened_trial,
                    message_count=len(messages),
                    skipped_lines=skipped_lines,
                    width=width,
                )
                if len(candidate) > max_lines:
                    break
                partial = candidate
            if partial is not None:
                return cls._join_visual_lines(partial)
            if selected:
                return cls._join_visual_lines(
                    cls._compose_prioritized_preview(
                        selected,
                        message_count=len(messages),
                        skipped_lines=skipped_lines,
                        width=width,
                    )[:max_lines]
                )
            return cls._join_visual_lines(block[:max_lines])

        return cls._join_visual_lines(
            cls._compose_prioritized_preview(
                selected,
                message_count=len(messages),
                skipped_lines=skipped_lines,
                width=width,
            )[:max_lines]
        )

    @classmethod
    def _compose_prioritized_preview(
        cls,
        selected: dict[int, list[Text]],
        *,
        message_count: int,
        skipped_lines: tuple[int, ...],
        width: int,
    ) -> list[Text]:
        hidden = message_count - len(selected)
        marker = cls._compact_preview_marker(hidden, skipped_lines)
        marker.truncate(width, overflow="ellipsis")
        visible: list[Text] = []

        first = selected.get(0)
        if first is not None:
            visible.extend(first)
            visible.append(Text())
        visible.append(marker)

        recent = sorted(index for index in selected if index)
        for index in recent:
            visible.append(Text())
            visible.extend(selected[index])
        return visible

    @staticmethod
    def _compact_preview_marker(
        hidden: int,
        skipped_lines: tuple[int, ...],
    ) -> Text:
        details = []
        if hidden:
            details.append(f"{hidden} hidden")
        if skipped_lines:
            details.append(f"{len(skipped_lines)} malformed")
        detail = ", ".join(details)
        message = (
            f"{detail}, press t: transcript"
            if detail
            else "press t: transcript"
        )
        return Text(
            message,
            style="italic #8492a0",
        )

    @staticmethod
    def _ellipsize_visual_lines(lines: list[Text], width: int) -> list[Text]:
        shortened = [line.copy() for line in lines]
        if shortened:
            shortened[-1].append(" …", style="italic #8492a0")
            shortened[-1].truncate(width, overflow="ellipsis")
        return shortened

    @staticmethod
    def _join_visual_lines(lines: list[Text]) -> Text:
        return Text("\n").join(lines)

    def on_resize(self) -> None:
        if getattr(self, "_loaded_transcript", None) is not None:
            self.call_after_refresh(self._update_transcript_preview)

    def on_data_table_row_highlighted(self, event: DataTable.RowHighlighted) -> None:
        if event.data_table.id == "runs":
            self._refresh_transcript()

    def action_reload(self) -> None:
        self.refresh_runs()

    def action_new_run(self) -> None:
        repository = self._selected_repository() or self.app.service.repo
        if repository is None:
            self._set_notice(
                "Select a Git repository or Git-backed window to create a run"
            )
            return
        self.app.push_screen(NewRunScreen(repository=repository))

    def action_populate_run(self) -> None:
        run = self._selected_run()
        if run is None:
            return
        if run.repository is None:
            self._set_notice("This window is not associated with a Git repository")
            return
        service = self.app.service.for_run(run)
        try:
            draft = service.populate_draft(run)
        except CommandError as exc:
            self._set_notice(str(exc))
            return
        if draft.restart_worktree:
            self.app.push_screen(NewRunScreen(restart_draft=draft))
        else:
            self.app.push_screen(
                NewRunScreen(
                    branch=draft.branch,
                    prompt=draft.prompt,
                    agent=draft.agent,
                    repository=draft.repository or service.repo,
                )
            )

    def action_open_run(self) -> None:
        run = self._selected_run()
        if run is None:
            return
        try:
            self.app.service.select_window(run)
            self._set_notice(f"Switched to {run.location}")
        except CommandError as exc:
            self._set_notice(str(exc))

    def action_new_window(self) -> None:
        run = self._selected_run()
        if run is None:
            return
        try:
            window = self.app.service.open_shell_window(run)
            self._set_notice(
                f"Opened shell {window or 'window'} at {run.worktree}"
            )
        except CommandError as exc:
            self._set_notice(f"Could not open shell: {exc}")

    def action_cleanup(self) -> None:
        run = self._selected_run()
        if run is None:
            return
        if run.state != "exited" or run.worktree_state != "merged":
            self._set_notice(
                "Cleanup is available only for finished, clean, merged worktrees"
            )
            return
        self.app.push_screen(CleanupScreen(run))

    def action_view_transcript(self) -> None:
        loaded = self._loaded_transcript
        if loaded is None:
            self._set_notice("No transcript is loaded for the selected run")
            return
        transcript = self._render_transcript(
            loaded.messages, loaded.skipped_lines
        ).plain
        document = (
            f"Session ID  {loaded.session_id}\n"
            f"Source      {loaded.rollout_path}\n\n"
            f"{transcript}\n"
        )
        try:
            with self.app.suspend():
                result = subprocess.run(
                    ["less"],
                    input=document,
                    text=True,
                    check=False,
                )
        except OSError as exc:
            self._set_notice(f"Could not open transcript in less: {exc}")
            return
        if result.returncode not in (0, 130):
            self._set_notice(f"less exited with status {result.returncode}")

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
        if event.data_table.id == "runs":
            self.action_open_run()

    def action_quit(self) -> None:
        self.app.exit()


class NewRunScreen(Screen):
    BINDINGS = [("escape", "back", "Back")]

    def __init__(
        self, *, branch: str = "", prompt: str = "", agent: str = "codex",
        restart_draft: LaunchDraft | None = None,
        repository: Repository | None = None,
    ) -> None:
        super().__init__()
        self.restart_draft = restart_draft
        self.repository = (
            restart_draft.repository if restart_draft is not None else repository
        )
        self.prompt = restart_draft.prompt if restart_draft else prompt
        self.initial_branch = restart_draft.branch if restart_draft else branch
        self.initial_agent = normalize_launch_agent(
            restart_draft.agent if restart_draft else agent
        )

    @property
    def service(self) -> AgentmonService:
        return self.app.service.for_repository(self.repository)

    def compose(self) -> ComposeResult:
        with VerticalScroll(id="dialog", classes="new-run"):
            yield Label("New run", classes="dialog-title")
            yield Label(f"Repository     {self.service.repo.root}")
            with Horizontal(id="draft-fields"):
                with Vertical(classes="draft-field"):
                    yield Label("Agent")
                    yield Select(
                        [
                            (label, agent)
                            for agent, label in LAUNCH_AGENT_LABELS.items()
                        ],
                        value=self.initial_agent,
                        allow_blank=False,
                        id="agent",
                    )
                with Vertical(classes="draft-field"):
                    yield Label("Branch")
                    yield Input(
                        value=self.initial_branch,
                        placeholder="feature-name",
                        id="branch",
                        disabled=self.restart_draft is not None,
                    )
            yield Static("Worktree       —", id="worktree")
            yield Static("Prompt         Not written", id="prompt-status")
            with Horizontal(classes="buttons"):
                yield Button("Open $EDITOR", id="edit")
                yield Button("Choose historical commit", id="history")
            yield Static("", id="error")
            with Horizontal(classes="buttons"):
                yield Button("Continue", id="continue", classes="dialog-action", disabled=True)
                yield Button("Cancel", id="cancel")

    def on_mount(self) -> None:
        self._update_summary()
        self.query_one("#branch", Input).focus()

    def on_input_changed(self, event: Input.Changed) -> None:
        if event.input.id == "branch":
            self._update_summary()

    def _update_summary(self) -> None:
        branch = self.query_one("#branch", Input).value if self.is_mounted else self.initial_branch
        path = self.service.suggested_worktree(branch)
        self.query_one("#worktree", Static).update(f"Worktree       {path}")
        lines = len(self.prompt.splitlines())
        if self.prompt:
            preview = next((x.strip() for x in self.prompt.splitlines() if x.strip()), "")
            self.query_one("#prompt-status", Static).update(
                f"Prompt         Ready · {lines} lines\n               {preview[:68]}"
            )
        else:
            self.query_one("#prompt-status", Static).update("Prompt         Not written")
        self._update_continue_state(branch)

    def _update_continue_state(self, branch: str) -> None:
        """Keep the primary action available only for a valid draft."""
        button = self.query_one("#continue", Button)
        error = self.query_one("#error", Static)
        if self.restart_draft is not None:
            button.disabled = not self.prompt.strip()
            error.remove_class("guidance", "validation-error")
            error.add_class("overwrite-info")
            error.update(
                f"Existing merged worktree will be restarted: {self.restart_draft.worktree}"
            )
            return
        try:
            draft = self.service.validate_draft(branch, self.prompt)
        except ValueError as exc:
            button.disabled = True
            message = str(exc)
            incomplete = message in {"Enter a branch name", "Write a prompt before continuing"}
            error.remove_class("overwrite-info")
            error.set_class(incomplete, "guidance")
            error.set_class(not incomplete, "validation-error")
            error.update(message)
        else:
            button.disabled = False
            error.remove_class("guidance", "validation-error")
            if draft.overwrite_worktree:
                error.add_class("overwrite-info")
                error.update(f"Existing worktree will be replaced: {draft.worktree}")
            elif draft.existing_branch:
                error.add_class("overwrite-info")
                error.update(f"Existing branch will be used: {draft.branch}")
            else:
                error.remove_class("overwrite-info")
                error.update("")

    def _edit_prompt(self) -> None:
        editor = shlex.split(os.environ.get("EDITOR", "vi"))
        fd, name = tempfile.mkstemp(prefix="agentmon-prompt-", suffix=".md")
        path = Path(name)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(self.prompt)
            with self.app.suspend():
                result = subprocess.run([*editor, str(path)], check=False)
            if result.returncode == 0:
                self.prompt = path.read_text(encoding="utf-8")
                self._update_summary()
        except OSError as exc:
            self.query_one("#error", Static).update(f"Could not open editor: {exc}")
        finally:
            path.unlink(missing_ok=True)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "edit":
            self._edit_prompt()
        elif event.button.id == "history":
            self.app.push_screen(
                HistoryScreen(repository=self.service.repo),
                self._history_selected,
            )
        elif event.button.id == "continue":
            branch = self.query_one("#branch", Input).value
            agent = self.query_one("#agent", Select).value
            selected_agent = agent if isinstance(agent, str) else DEFAULT_LAUNCH_AGENT
            if self.restart_draft is not None:
                draft = LaunchDraft(
                    branch=self.restart_draft.branch,
                    worktree=self.restart_draft.worktree,
                    prompt=self.prompt,
                    overwrite_worktree=True,
                    restart_worktree=True,
                    agent=selected_agent,
                    repository=self.service.repo,
                )
                self.app.push_screen(ConfirmScreen(draft))
            else:
                try:
                    draft = self.service.validate_draft(branch, self.prompt)
                except ValueError as exc:
                    self.query_one("#error", Static).update(str(exc))
                else:
                    self.app.push_screen(ConfirmScreen(replace(draft, agent=selected_agent)))
        elif event.button.id == "cancel":
            self.app.pop_screen()

    def _history_selected(self, item: PromptHistory | None) -> None:
        if item is not None:
            self.prompt = item.prompt
            self._update_summary()
            self._edit_prompt()

    def action_back(self) -> None:
        self.app.pop_screen()


class CleanupScreen(Screen):
    BINDINGS = [
        ("enter", "confirm", "Remove worktree"),
        ("escape", "cancel", "Cancel"),
    ]

    def __init__(self, run: AgentRun) -> None:
        super().__init__()
        self.run = run

    def compose(self) -> ComposeResult:
        with VerticalScroll(id="dialog"):
            yield Label(f"Clean up {self.run.branch}?", classes="dialog-title")
            yield Static(
                f"Remove registered worktree\n  {self.run.worktree}\n\n"
                "The branch will be preserved. Git state will be checked again."
            )
            yield Static("", id="error")
            with Horizontal(classes="buttons"):
                yield Button("Remove worktree", id="remove", classes="dialog-action")
                yield Button("Cancel", id="cancel")

    def on_mount(self) -> None:
        self.query_one("#remove", Button).focus()

    def _confirm(self) -> None:
        try:
            self.app.service.for_run(self.run).cleanup_worktree(self.run)
        except CommandError as exc:
            self.query_one("#error", Static).update(str(exc))
            return
        self.app.pop_screen()
        dashboard = self.app.screen
        if isinstance(dashboard, DashboardScreen):
            dashboard.refresh_runs()
            dashboard._set_notice(
                f"Removed worktree {self.run.worktree}; branch {self.run.branch} was preserved"
            )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "remove":
            self._confirm()
        else:
            self.app.pop_screen()

    def action_confirm(self) -> None:
        self._confirm()

    def action_cancel(self) -> None:
        self.app.pop_screen()


class HistoryScreen(Screen[PromptHistory | None]):
    BINDINGS = [
        ("escape", "cancel", "Back"),
        ("/", "search", "Search"),
    ]

    def __init__(self, *, repository: Repository | None = None) -> None:
        super().__init__()
        self.repository = repository

    def compose(self) -> ComposeResult:
        with VerticalScroll(id="dialog", classes="wide"):
            yield Label("Choose a historical prompt as a template", classes="dialog-title")
            yield HistorySearchInput(placeholder="Filter by commit, run, or prompt text", id="search")
            yield NavigableDataTable(id="history", cursor_type="row", zebra_stripes=True)
            yield Static(
                "↑/↓ or j/k select  ·  Enter choose  ·  / search  ·  Esc back",
                classes="key-hint",
            )
            yield Static("", id="error")
            with Horizontal(classes="buttons"):
                yield Button("Use as template and edit", id="use", classes="dialog-action")
                yield Button("Back", id="cancel")

    def on_mount(self) -> None:
        table = self.query_one("#history", DataTable)
        table.add_columns("COMMIT", "DATE", "RUN", "PROMPT")
        try:
            self.items = self.app.service.for_repository(
                self.repository
            ).prompt_history()
        except CommandError as exc:
            self.items = []
            self.query_one("#error", Static).update(str(exc))
        self._fill("")
        table.focus()

    def _fill(self, search: str) -> None:
        table = self.query_one("#history", DataTable)
        table.clear()
        needle = search.casefold()
        self.filtered_items = [
            item for item in self.items
            if needle in f"{item.commit} {item.date} {item.subject} {item.prompt}".casefold()
        ]
        for item in self.filtered_items:
            table.add_row(item.commit[:8], item.date, item.subject, item.preview[:60], key=item.commit)

    def on_input_changed(self, event: Input.Changed) -> None:
        self._fill(event.value)

    def on_input_submitted(self, _event: Input.Submitted) -> None:
        self._choose()

    def on_data_table_row_selected(self, _event: DataTable.RowSelected) -> None:
        self._choose()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "use":
            self._choose()
        else:
            self.dismiss(None)

    def _choose(self) -> None:
        row = self.query_one("#history", DataTable).cursor_row
        if self.filtered_items and row < len(self.filtered_items):
            self.dismiss(self.filtered_items[row])

    def move_selection(self, offset: int) -> None:
        table = self.query_one("#history", DataTable)
        if not self.filtered_items:
            return
        table.move_cursor(row=max(0, min(table.cursor_row + offset, len(self.filtered_items) - 1)))

    def action_search(self) -> None:
        self.query_one("#search", Input).focus()

    def action_cancel(self) -> None:
        self.dismiss(None)


class ConfirmScreen(Screen):
    BINDINGS = [("escape", "back", "Back")]

    def __init__(self, draft: LaunchDraft) -> None:
        super().__init__()
        self.draft = draft

    @property
    def service(self) -> AgentmonService:
        return self.app.service.for_repository(self.draft.repository)

    def compose(self) -> ComposeResult:
        preview = next((x.strip() for x in self.draft.prompt.splitlines() if x.strip()), "")
        agent_name = LAUNCH_AGENT_LABELS.get(self.draft.agent, self.draft.agent)
        if self.draft.restart_worktree:
            branch_summary = (
                f"Restart branch  {self.draft.branch} "
                f"(from {self.service.repo.branch})"
            )
        elif self.draft.existing_branch:
            branch_summary = f"Existing branch {self.draft.branch}"
        else:
            branch_summary = (
                f"New branch      {self.draft.branch} "
                f"(from {self.service.repo.branch})"
            )
        overwrite = (
            f"Existing worktree at {self.draft.worktree} will be deleted.\n"
            if self.draft.overwrite_worktree
            else ""
        )
        first_step = (
            "  1. Recreate the existing branch/worktree from the dashboard branch\n"
            if self.draft.restart_worktree
            else "  1. Delete the existing worktree and create a new branch/worktree\n"
            if self.draft.overwrite_worktree
            else "  1. Create worktree from the existing branch\n"
            if self.draft.existing_branch
            else "  1. Create branch and worktree\n"
        )
        with VerticalScroll(id="dialog", classes="wide"):
            yield Label(f"Launch {self.draft.branch}?", classes="dialog-title")
            yield Static(
                f"Repository     {self.service.repo.root}\n"
                f"{branch_summary}\n"
                f"{'Reused worktree' if self.draft.restart_worktree else 'New worktree'}   "
                f"{self.draft.worktree}\n"
                f"Agent          {agent_name}\n"
                f"Prompt         {len(self.draft.prompt.splitlines())} lines · {preview[:60]}"
            )
            yield Static(
                overwrite
                + "Will perform\n"
                + first_step
                + "  2. Write prompt.md and commit it\n"
                + "  3. Create an hmux window in the worktree\n"
                + f"  4. Run {agent_name} with the prompt",
                classes="steps",
            )
            with Horizontal(classes="buttons"):
                yield Button("Launch", id="launch", classes="dialog-action")
                yield Button("Back", id="back")
                yield Button("Cancel", id="cancel")

    def on_mount(self) -> None:
        self.query_one("#launch", Button).focus()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "launch":
            self.app.push_screen(LaunchScreen(self.draft))
        elif event.button.id == "back":
            self.app.pop_screen()
        else:
            self.app.pop_screen()
            self.app.pop_screen()

    def action_back(self) -> None:
        self.app.pop_screen()


class LaunchScreen(Screen):
    def __init__(self, draft: LaunchDraft) -> None:
        super().__init__()
        self.draft = draft
        self.lines: list[str] = []
        self.failed = False

    def compose(self) -> ComposeResult:
        agent_name = LAUNCH_AGENT_LABELS.get(self.draft.agent, self.draft.agent)
        with VerticalScroll(id="dialog", classes="wide"):
            yield Label(
                f"Launching {self.draft.branch} with {agent_name}",
                classes="dialog-title",
            )
            yield Static("Starting…", id="progress")
            with Horizontal(classes="buttons", id="done-buttons"):
                yield Button("Return to dashboard", id="dashboard", disabled=True)

    def on_mount(self) -> None:
        self._launch()

    def _record(self, step: LaunchStep) -> None:
        suffix = f" ({step.detail})" if step.detail else ""
        self.lines.append(f"✓ {step.label}{suffix}")
        self.query_one("#progress", Static).update("\n".join(self.lines))

    def _failed(self, message: str) -> None:
        self.failed = True
        self.lines.append(f"! {message}")
        self.query_one("#progress", Static).update("\n".join(self.lines))
        button = self.query_one("#dashboard", Button)
        button.label = "Back to form"
        button.disabled = False
        button.focus()

    def _succeeded(self, window: str) -> None:
        self.lines.append(f"\nAgent started in hmux window {window}")
        self.query_one("#progress", Static).update("\n".join(self.lines))
        button = self.query_one("#dashboard", Button)
        button.disabled = False
        button.focus()

    @work(thread=True, exclusive=True)
    def _launch(self) -> None:
        try:
            window = self.app.service.for_repository(
                self.draft.repository
            ).launch(
                self.draft,
                lambda step: self.app.call_from_thread(self._record, step),
            )
        except (CommandError, OSError) as exc:
            self.app.call_from_thread(self._failed, str(exc))
        else:
            self.app.call_from_thread(self._succeeded, window)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "dashboard":
            self.app.pop_screen()
            self.app.pop_screen()
            if not self.failed:
                self.app.pop_screen()
            dashboard = self.app.screen
            if isinstance(dashboard, DashboardScreen):
                dashboard.refresh_runs()


class QuotaScreen(ModalScreen):
    """Floating dialog summarizing subscription quota usage."""

    BINDINGS = [
        ("escape", "close", "Close"),
        ("q", "close", "Close"),
        ("r", "refresh", "Refresh now"),
    ]

    BAR_WIDTH = 24

    def compose(self) -> ComposeResult:
        with Vertical(id="quota-dialog"):
            yield Label("Subscription quotas", classes="dialog-title")
            yield Static("Loading quota usage…", id="quota-content")
            yield Static("", id="quota-meta")
            yield Static("r refresh  ·  Esc close", classes="key-hint")

    def on_mount(self) -> None:
        self._load_report(False)

    @work(thread=True, exclusive=True, group="quota")
    def _load_report(self, force: bool) -> None:
        report = self.app.quota_service.report(force=force)
        self.app.call_from_thread(self._apply_report, report)

    def _apply_report(self, report: QuotaReport) -> None:
        if not self.is_mounted:
            return
        self.query_one("#quota-content", Static).update(self._render_report(report))
        fetched = report.fetched_at.astimezone().strftime("%H:%M:%S")
        self.query_one("#quota-meta", Static).update(
            Text(f"Fetched {fetched} · cached between refreshes", style="#8492a0")
        )

    @classmethod
    def _render_report(cls, report: QuotaReport) -> Text:
        rendered = Text()
        label_width = max((len(quota.label) for quota in report.quotas), default=0) + 2
        for index, quota in enumerate(report.quotas):
            if index:
                rendered.append("\n")
            rendered.append_text(cls._render_quota(quota, label_width))
        if report.errors:
            if report.quotas:
                rendered.append("\n\n")
            for index, error in enumerate(report.errors):
                if index:
                    rendered.append("\n")
                rendered.append(f"! {error}", style="#ffcb6b")
        if not report.quotas and not report.errors:
            rendered.append("No quota information is available.", style="#8492a0")
        return rendered

    @classmethod
    def _render_quota(cls, quota: QuotaWindow, label_width: int) -> Text:
        used = max(0.0, min(100.0, quota.used_percent))
        filled = round(used / 100 * cls.BAR_WIDTH)
        if used >= 90:
            color = "#ff6b6b"
        elif used >= 70:
            color = "#ffcb6b"
        else:
            color = "#7ec97e"
        row = Text()
        row.append(quota.label.ljust(label_width))
        row.append("█" * filled, style=color)
        row.append("░" * (cls.BAR_WIDTH - filled), style="#33414f")
        row.append(f"{used:4.0f}% used")
        if quota.resets_at is not None:
            resets = quota.resets_at.astimezone().strftime("%m-%d %H:%M")
            row.append(f"  resets {resets}", style="#8492a0")
        return row

    def action_close(self) -> None:
        self.dismiss()

    def action_refresh(self) -> None:
        self.query_one("#quota-content", Static).update("Refreshing quota usage…")
        self._load_report(True)


class DemoService(AgentmonService):
    def watch_runs(self, on_change, stop) -> None:
        return None

    def runs(self) -> list[AgentRun]:
        parent = self.repo.root.parent
        return [
            AgentRun(
                "%1", "0:3.0", "auth-cleanup", "working", "codex",
                parent / "auth-cleanup",
                "Fix authentication cleanup behavior and add regression tests.",
                "dirty",
                session_id="019f6c99-2762-7dc2-9d93-a8a4b48a3a5e",
                repository=self.repo,
            ),
            AgentRun(
                "%2", "0:4.0", "flaky-test", "blocked", "claude",
                parent / "flaky-test",
                "Investigate the intermittent timeout in the integration suite.",
                "unmerged",
                session_id="874085ab-4a17-4018-ac19-f0381bb7940a",
                repository=self.repo,
            ),
            AgentRun(
                "%3", "0:5.0", "docs-pass", "idle", "codex",
                parent / "docs-pass",
                "Refresh contributor documentation for the new API.",
                "merged",
                session_id="019f6c99-2762-7dc2-9d93-a8a4b48a3a5f",
                repository=self.repo,
            ),
        ]

    def select_window(self, run: AgentRun) -> None:
        return None

    def recent_finished(self, active: list[AgentRun], limit: int = 5) -> list[AgentRun]:
        return [
            AgentRun(
                "finished:/demo/old-fix", "0:", "old-fix", "exited", "finished",
                self.repo.root.parent / "old-fix",
                "Fix the parser edge case found during review.",
                "merged",
                repository=self.repo,
            )
        ]

    def open_shell_window(self, run: AgentRun) -> str:
        return "0:6"

    def run_transcript(self, run: AgentRun) -> Transcript:
        if not run.session_id:
            raise TranscriptError("Demo run has no active agent session")
        messages = {
            "auth-cleanup": (
                TranscriptMessage(
                    "user",
                    "Fix authentication cleanup behavior and add regression tests.",
                    "2026-07-16T19:01:04Z",
                ),
                TranscriptMessage(
                    "user",
                    "Keep the existing CLI behavior unchanged while fixing it.",
                    "2026-07-16T19:18:31Z",
                ),
            ),
            "flaky-test": (
                TranscriptMessage(
                    "user",
                    "Investigate the intermittent timeout in the integration suite.",
                    "2026-07-16T18:42:10Z",
                ),
                TranscriptMessage(
                    "user",
                    "The last failure was in test_reconnect_after_heartbeat.",
                    "2026-07-16T19:06:55Z",
                ),
            ),
            "docs-pass": (
                TranscriptMessage(
                    "user",
                    "Refresh contributor documentation for the new API.",
                    "2026-07-16T17:55:22Z",
                ),
            ),
        }.get(run.branch, ())
        return Transcript(
            session_id=run.session_id,
            rollout_path=Path(f"/demo/transcripts/{run.session_id}.jsonl"),
            messages=messages,
        )

    def cleanup_worktree(self, run: AgentRun) -> None:
        return None

    def populate_draft(self, run: AgentRun) -> LaunchDraft:
        prompt = {
            "auth-cleanup": "Fix authentication cleanup behavior and add regression tests.\n",
            "flaky-test": "Investigate the intermittent timeout in the integration suite.\n",
            "docs-pass": "Refresh contributor documentation for the new API.\n",
            "old-fix": "Fix the parser edge case found during review.\n",
        }.get(run.branch, f"{run.prompt_preview}\n")
        agent = normalize_launch_agent(run.agent)
        return LaunchDraft(
            run.branch,
            self.suggested_worktree(run.branch),
            prompt,
            agent=agent,
            repository=self.repo,
        )

    def prompt_history(self, limit: int = 50) -> list[PromptHistory]:
        return [
            PromptHistory("a" * 40, "2026-07-09", "auth-timeout", "Fix timeout handling in the auth client.\n"),
            PromptHistory("b" * 40, "2026-06-28", "login-tests", "Add coverage for expired credentials.\n"),
        ]

    def launch(self, draft: LaunchDraft, progress) -> str:
        raise CommandError("Demo mode does not make changes")


class DemoQuotaService(QuotaService):
    def report(self, *, force: bool = False) -> QuotaReport:
        now = datetime.now(timezone.utc)
        return QuotaReport(
            fetched_at=now,
            quotas=(
                QuotaWindow("codex", "Codex weekly", 37.0, now),
                QuotaWindow("codex", "Codex 5h", 12.0, now),
                QuotaWindow("claude", "Claude 5h", 41.0, now),
                QuotaWindow("claude", "Claude weekly", 9.0, now),
                QuotaWindow("claude", "Claude Fable weekly", 14.0, now),
            ),
        )


class AgentmonApp(App):
    CSS = """
    Screen { background: #101419; color: #d7dde5; }
    #title { height: 3; padding: 1 2; background: #263442; color: #ffffff; text-style: bold; }
    #dashboard-main { height: 1fr; }
    #runs-pane { width: 3fr; }
    #runs { height: 1fr; margin: 0 1; border: tall #557086; }
    #runs > .datatable--header { background: #263442; color: #b9dcf5; text-style: bold; }
    #runs > .datatable--cursor { background: #315b76; color: #ffffff; }
    #notice { height: auto; max-height: 3; padding: 0 2; color: #ffcb6b; }
    #transcript-pane { width: 2fr; margin: 0 1 0 0; border: tall #557086; }
    #transcript-title { height: 3; padding: 1 2; background: #263442;
                        color: #b9dcf5; text-style: bold; }
    #transcript-meta { height: auto; min-height: 3; padding: 1 2; color: #8ec8ff; }
    #transcript-scroll { height: 1fr; overflow: hidden; }
    #transcript-content { height: auto; padding: 1 2 2 2; }
    #dialog { width: 72; height: auto; max-width: 100%; max-height: 100%;
              margin: 1 2; padding: 1 2;
              border: round #6aa9d8; background: #171d24; }
    #dialog.wide { width: 92; }
    #dialog.new-run { scrollbar-size: 1 1; }
    .dialog-title { height: 2; color: #ffffff; text-style: bold; }
    #draft-fields { height: 4; }
    #draft-fields .draft-field { width: 1fr; }
    .buttons { height: 3; margin-top: 0; }
    .buttons Button { margin-right: 1; }
    .buttons .dialog-action { background: #293440; color: #d7dde5; text-style: none; }
    .buttons .dialog-action:hover { background: #354555; color: #ffffff; }
    .buttons .dialog-action:focus { background: #3f8fc4; color: #ffffff; text-style: bold; }
    .buttons .dialog-action:disabled { background: #202832; color: #65717e; text-style: none; }
    #worktree, #prompt-status { min-height: 2; }
    #error { min-height: 1; }
    #error.guidance { color: #8492a0; }
    #error.validation-error { color: #ff6b6b; }
    #error.overwrite-info { color: #ffcb6b; }
    DataTable#history { height: 14; margin-top: 1; }
    .key-hint { height: 2; padding-top: 1; color: #8492a0; }
    .steps { margin-top: 1; padding: 1; background: #202832; }
    #progress { min-height: 8; padding: 1; background: #202832; }
    Footer { background: #202832; }
    QuotaScreen { align: center middle; background: transparent; }
    #quota-dialog { width: 84; height: auto; max-width: 100%; max-height: 100%;
                    padding: 1 2; border: round #6aa9d8; background: #171d24; }
    #quota-content { min-height: 5; text-wrap: nowrap; text-overflow: ellipsis; }
    #quota-meta { margin-top: 1; }
    """

    def __init__(
        self,
        service: AgentmonService,
        *,
        startup_warning: str = "",
        quota_service: QuotaService | None = None,
    ) -> None:
        super().__init__()
        self.service = service
        self.quota_service = quota_service or QuotaService()
        self.startup_warning = startup_warning
        self.runs: list[AgentRun] = []

    def on_mount(self) -> None:
        self.push_screen(DashboardScreen())

    def get_system_commands(self, screen: Screen) -> Iterable[SystemCommand]:
        yield from super().get_system_commands(screen)
        yield SystemCommand(
            "Subscription quotas",
            "Toggle the Codex/Claude quota usage dialog",
            self.action_toggle_quotas,
        )

    def action_toggle_quotas(self) -> None:
        for existing in self.screen_stack:
            if isinstance(existing, QuotaScreen):
                existing.dismiss()
                return
        self.push_screen(QuotaScreen())


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="agentmon")
    parser.add_argument("--socket", help="hmux socket path (otherwise auto-detected)")
    parser.add_argument("--tmux", default="tmux", help="tmux-compatible client")
    parser.add_argument("--demo", action="store_true", help="show sample data without changing anything")
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    try:
        repo = discover_repository()
    except CommandError:
        repo = None
    if args.demo and repo is None:
        root = Path.cwd().resolve()
        repo = Repository(root=root, common_dir=root / ".git", branch="demo")
    service_type = DemoService if args.demo else AgentmonService
    if args.demo:
        socket = args.socket or "/tmp/agentmon-demo.sock"
        warning = ""
    else:
        try:
            selection = discover_socket(requested=args.socket, tmux=args.tmux)
        except CommandError as exc:
            print(f"agentmon: {exc}")
            return 1
        socket = selection.path
        warning = selection.warning
    AgentmonApp(
        service_type(repo, socket=socket, tmux=args.tmux),
        startup_warning=warning,
        quota_service=DemoQuotaService() if args.demo else None,
    ).run()
    return 0
