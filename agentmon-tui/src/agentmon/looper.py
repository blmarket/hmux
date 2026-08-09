"""Headless loop runner: re-run one prompt for as long as pacing allows.

`looper` is started from a pane inside hmux, usually as `cat prompt | looper`.
It splits its own window and hands the bottom half to a coding agent, then
repeats one cycle: wait until quota pacing allows a run, start the agent on the
prompt, end the run once the agent goes idle, and commit whatever it changed.

`--preset` picks which agent runs, with its model and effort; the presets are
named in `PRESETS`.

Pacing comes from the same `QuotaService` the dashboard shows. A window is
"on pace" while the share of it spent is no larger than the share of it
elapsed; over pace, the loop sleeps until an even burn would have caught up,
which for an exhausted window is its reset. Only the agent's own provider
gates it — a spent Claude subscription has no bearing on a codex loop. `-f`
drops the gate entirely and runs the budget back to back.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
import tempfile
import time
from collections.abc import Callable
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path

from . import __version__
from .quota import QuotaReport, QuotaService, QuotaWindow
from .services import AgentmonService, CommandError, discover_context


@dataclass(frozen=True)
class Preset:
    """One named agent configuration a loop can run with.

    `provider` is the quota provider that paces it. A provider `QuotaService`
    knows nothing about — antigravity has no usage endpoint — leaves the loop
    unpaced, which `pacing_decision` reports on every run.
    """

    agent: str
    model: str
    effort: str
    provider: str


PRESETS = {
    "codex": Preset(
        agent="codex", model="gpt-5.6-luna", effort="max", provider="codex"
    ),
    "agy": Preset(
        agent="agy", model="gemini-3.6-flash", effort="high", provider="antigravity"
    ),
}
DEFAULT_PRESET = "codex"

# States that end a run: the agent is sitting at its prompt with nothing left
# to do, or it wants a human. An interactive agent never exits on its own, so
# the loop is what ends the run.
DONE_STATES = ("idle", "blocked")


@dataclass(frozen=True)
class PacingDecision:
    """Whether a new run may start, and when to look again if not."""

    ready: bool
    wait_until: datetime | None
    reason: str


def pacing_ready_at(window: QuotaWindow) -> datetime | None:
    """Return when an even burn would catch up with what this window spent.

    `QuotaWindow.pace_percent` reads the elapsed share of the window off the
    reset time; this inverts it. Spending u% of a W-long window is on pace once
    only W*(100-u)/100 of it remains, so a fully spent window is caught up only
    at its reset. None when the provider did not date the window.
    """
    if window.resets_at is None or not window.window_seconds:
        return None
    used = min(100.0, max(0.0, window.used_percent))
    return window.resets_at - timedelta(
        seconds=window.window_seconds * (100.0 - used) / 100.0
    )


def _describe(window: QuotaWindow, now: datetime) -> str:
    pace = window.pace_percent(now)
    if pace is None:
        return f"{window.label} {window.used_percent:.0f}% used"
    return f"{window.label} {window.used_percent:.0f}% used / {pace:.0f}% elapsed"


def pacing_decision(
    report: QuotaReport, *, provider: str, now: datetime
) -> PacingDecision:
    """Decide whether `provider`'s quota windows all allow another run.

    Every dated window must be at or under its elapsed share; `wait_until` is
    the latest catch-up time among those that are not, since the loop waits for
    all of them. Windows the provider left undated cannot be paced and are
    passed over, as is a provider that reported nothing at all — a quota
    lookup that is merely unavailable should not stall the loop forever.
    """
    windows = [quota for quota in report.quotas if quota.provider == provider]
    if not windows:
        return PacingDecision(True, None, f"no {provider} quota reported")

    behind: list[tuple[datetime, QuotaWindow]] = []
    for window in windows:
        pace = window.pace_percent(now)
        ready_at = pacing_ready_at(window)
        if pace is None or ready_at is None or window.used_percent <= pace:
            continue
        behind.append((ready_at, window))

    summary = " · ".join(_describe(window, now) for window in windows)
    if not behind:
        return PacingDecision(True, None, summary)
    wait_until, _worst = max(behind, key=lambda item: item[0])
    return PacingDecision(False, wait_until, summary)


def format_duration(seconds: float) -> str:
    seconds = max(0, int(seconds))
    if seconds < 60:
        return f"{seconds}s"
    minutes, seconds = divmod(seconds, 60)
    if minutes < 60:
        return f"{minutes}m{seconds:02d}s"
    hours, minutes = divmod(minutes, 60)
    if hours < 24:
        return f"{hours}h{minutes:02d}m"
    days, hours = divmod(hours, 24)
    return f"{days}d{hours:02d}h"


@dataclass(frozen=True)
class LooperConfig:
    agent: str = PRESETS[DEFAULT_PRESET].agent
    model: str = PRESETS[DEFAULT_PRESET].model
    effort: str = PRESETS[DEFAULT_PRESET].effort
    provider: str = PRESETS[DEFAULT_PRESET].provider
    # 0 means "until the user stops it" for both limits below.
    max_runs: int = 0
    run_timeout: float = 0.0
    agent_size_percent: int = 75
    startup_timeout: float = 120.0
    poll_seconds: float = 2.0
    quota_poll_seconds: float = 300.0
    commit: bool = True
    devshell: bool = False
    # Run back to back without consulting quota at all.
    ignore_pacing: bool = False


class Looper:
    """Run one prompt over and over in a split pane, paced by quota usage."""

    def __init__(
        self,
        service: AgentmonService,
        *,
        worktree: Path,
        pane: str,
        instruction_file: Path,
        config: LooperConfig | None = None,
        quota_service: QuotaService | None = None,
        log: Callable[[str], None] = print,
        sleep: Callable[[float], None] = time.sleep,
        now: Callable[[], datetime] = lambda: datetime.now(timezone.utc),
    ) -> None:
        self.service = service
        self.worktree = worktree
        self.pane = pane
        self.instruction_file = instruction_file
        self.config = config or LooperConfig()
        self.quota_service = quota_service or QuotaService()
        self._log_line = log
        self._sleep = sleep
        self._now = now
        self._reported_errors: set[str] = set()

    def log(self, message: str) -> None:
        self._log_line(f"[{self._now().astimezone():%H:%M:%S}] {message}")

    def run(self) -> int:
        """Loop until the run budget is spent or a run needs a human."""
        index = 0
        while not self.config.max_runs or index < self.config.max_runs:
            index += 1
            self.await_pacing()
            pane = self.start_agent(index)
            outcome = self.await_agent(pane)
            if outcome == "blocked":
                self.log(
                    f"run {index}: agent is waiting on you; leaving pane {pane} "
                    "open and stopping the loop"
                )
                return 0
            if outcome == "exited":
                self.log(f"run {index}: agent pane closed on its own")
            else:
                if outcome == "timeout":
                    self.log(
                        f"run {index}: still going after "
                        f"{format_duration(self.config.run_timeout)}, ending it"
                    )
                clean = self.service.exit_agent_pane(pane)
                self.log(
                    f"run {index}: agent ended"
                    if clean
                    else f"run {index}: agent ignored /exit, pane killed"
                )
            self.harvest(index)
        self.log(f"finished {index} run(s)")
        return 0

    def await_pacing(self) -> None:
        """Block until every gating quota window is at or under its pace."""
        if self.config.ignore_pacing:
            self.log("pacing ignored (-f): starting the run now")
            return
        while True:
            report = self.quota_service.report()
            self.report_errors(report)
            now = self._now()
            decision = pacing_decision(
                report, provider=self.config.provider, now=now
            )
            if decision.ready:
                self.log(f"on pace: {decision.reason}")
                return
            remaining = (
                self.config.quota_poll_seconds
                if decision.wait_until is None
                else (decision.wait_until - now).total_seconds()
            )
            # Nap in bounded steps so the next loop refetches: usage moves while
            # this process waits, and the catch-up time moves with it.
            nap = max(1.0, min(remaining, self.config.quota_poll_seconds))
            self.log(
                f"over pace: {decision.reason}; caught up in "
                f"{format_duration(remaining)}, sleeping {format_duration(nap)}"
            )
            self._sleep(nap)

    def report_errors(self, report: QuotaReport) -> None:
        """Surface quota lookup failures once each, not on every poll."""
        for error in report.errors:
            if error not in self._reported_errors:
                self._reported_errors.add(error)
                self.log(f"quota lookup failed: {error}")

    def start_agent(self, index: int) -> str:
        pane = self.service.split_agent_pane(
            target=self.pane,
            worktree=self.worktree,
            agent=self.config.agent,
            model=self.config.model,
            effort=self.config.effort,
            instruction_file=self.instruction_file,
            size_percent=self.config.agent_size_percent,
            devshell=self.config.devshell,
            # Focus the agent, not this log: status that follows the active
            # pane only reports the run while the agent pane holds it. Focus
            # comes back here on its own when the run's pane goes away.
            focus=True,
        )
        self.log(
            f"run {index}: started {self.config.agent} ({self.config.model}, "
            f"effort {self.config.effort}) in pane {pane}"
        )
        return pane

    def await_agent(self, pane: str) -> str:
        """Wait out one run, returning idle, blocked, exited, or timeout.

        The agent is given a grace period to be recognized as working before
        idleness counts, so a pane hmux has not classified yet is not mistaken
        for a run that already finished. A run short enough to finish inside
        that grace period still resolves, because the wait below sees the idle
        state immediately.
        """
        state = self.service.wait_for_pane_state(
            pane,
            ("working",),
            timeout=self.config.startup_timeout,
            poll=self.config.poll_seconds,
        )
        if state == "exited":
            return "exited"
        if state is None:
            status = self.service.pane_status(pane)
            if status is not None and not status.agent:
                # Without agent detection there is no idle to wait for, so say
                # so rather than blocking on a state that will never arrive.
                self.log(
                    f"pane {pane} is not recognized as an agent "
                    f"(state {status.state}); waiting anyway"
                )
        state = self.service.wait_for_pane_state(
            pane,
            DONE_STATES,
            timeout=self.config.run_timeout or None,
            poll=self.config.poll_seconds,
        )
        return state or "timeout"

    def harvest(self, index: int) -> None:
        """Commit whatever the run left behind in the worktree."""
        if not self.config.commit:
            return
        try:
            commit = self.service.commit_all_changes(
                self.worktree, f"looper: auto-commit run {index}"
            )
        except CommandError as exc:
            self.log(f"run {index}: auto-commit failed: {exc}")
            return
        self.log(
            f"run {index}: committed {commit}"
            if commit
            else f"run {index}: worktree was clean, nothing to commit"
        )


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="looper",
        description="Re-run one prompt in a split hmux pane while quota pacing allows.",
        epilog="The prompt is read from standard input: cat prompt.md | looper",
    )
    parser.add_argument("--socket", help="hmux socket path (otherwise auto-detected)")
    parser.add_argument("--tmux", default="tmux", help="tmux-compatible client")
    parser.add_argument(
        "--pane",
        help="pane to split (otherwise $TMUX_PANE, the pane looper runs in)",
    )
    parser.add_argument(
        "--preset", choices=sorted(PRESETS), default=DEFAULT_PRESET,
        help="agent, model and effort to run (default: %(default)s)",
    )
    parser.add_argument(
        "-n", "--max-runs", type=int, default=0,
        help="stop after this many runs (default: keep going)",
    )
    parser.add_argument(
        "-f", "--force", action="store_true",
        help="run regardless of quota pacing, without waiting between runs",
    )
    parser.add_argument(
        "--run-timeout", type=float, default=0.0, metavar="SECONDS",
        help="end a run that has not finished in this long (default: no limit)",
    )
    parser.add_argument(
        "--size", type=int, default=LooperConfig.agent_size_percent,
        metavar="PERCENT", help="share of the window given to the agent pane",
    )
    parser.add_argument(
        "--devshell", action="store_true",
        help="start the agent inside the worktree's nix dev shell",
    )
    parser.add_argument(
        "--no-commit", action="store_true",
        help="leave changes uncommitted instead of committing after each run",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)

    prompt = "" if sys.stdin.isatty() else sys.stdin.read()
    if not prompt.strip():
        print(
            "looper: no prompt on standard input; try `cat prompt.md | looper`",
            file=sys.stderr,
        )
        return 2

    pane = args.pane or os.environ.get("TMUX_PANE", "")
    if not pane:
        print(
            "looper: no pane to split; run looper inside an hmux pane or pass --pane",
            file=sys.stderr,
        )
        return 2

    try:
        context = discover_context(socket=args.socket, tmux=args.tmux)
    except CommandError as exc:
        print(f"looper: {exc}", file=sys.stderr)
        return 1
    if context.warning:
        print(f"looper: {context.warning}", file=sys.stderr)
    if context.repository is None:
        print("looper: not inside a git worktree", file=sys.stderr)
        return 1

    preset = PRESETS[args.preset]
    config = LooperConfig(
        agent=preset.agent,
        model=preset.model,
        effort=preset.effort,
        provider=preset.provider,
        max_runs=max(0, args.max_runs),
        run_timeout=max(0.0, args.run_timeout),
        agent_size_percent=args.size,
        commit=not args.no_commit,
        devshell=args.devshell,
        ignore_pacing=args.force,
    )
    # The prompt is kept outside the worktree so auto-commit never picks it up.
    directory = Path(tempfile.mkdtemp(prefix="looper-"))
    try:
        instruction_file = directory / "prompt.md"
        instruction_file.write_text(prompt, encoding="utf-8")
        looper = Looper(
            context.service(),
            worktree=context.repository.root,
            pane=pane,
            instruction_file=instruction_file,
            config=config,
        )
        looper.log(
            f"looping in {context.repository.root} on {context.repository.branch}"
        )
        return looper.run()
    except KeyboardInterrupt:
        print("\nlooper: stopped; any running agent pane was left alone")
        return 130
    except CommandError as exc:
        print(f"looper: {exc}", file=sys.stderr)
        return 1
    finally:
        shutil.rmtree(directory, ignore_errors=True)


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
