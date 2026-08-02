from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


DEFAULT_LAUNCH_AGENT = "codex"
LAUNCH_AGENT_LABELS = {
    "codex": "Codex",
    "claude": "Claude Code",
}

# "default" means the launcher passes no model/effort flag and the agent CLI
# decides. The remaining entries are a curated list; extend them as the CLIs
# grow new models or effort levels.
DEFAULT_LAUNCH_CHOICE = "default"
LAUNCH_AGENT_MODELS = {
    "codex": ("default", "gpt-5-codex", "gpt-5"),
    "claude": ("default", "sonnet", "opus"),
}
LAUNCH_AGENT_EFFORTS = {
    "codex": ("default", "low", "medium", "high", "xhigh"),
    "claude": ("default",),
}


def normalize_launch_agent(agent: str) -> str:
    if agent == "claude-code":
        return "claude"
    if agent in LAUNCH_AGENT_LABELS:
        return agent
    return DEFAULT_LAUNCH_AGENT


@dataclass(frozen=True)
class Repository:
    root: Path
    common_dir: Path
    branch: str

    @property
    def project_root(self) -> Path:
        """Return the main worktree that owns this repository's common Git data."""
        if self.common_dir.name == ".git":
            return self.common_dir.parent
        return self.root


@dataclass(frozen=True)
class AgentRun:
    pane_id: str
    location: str
    branch: str
    state: str
    agent: str
    worktree: Path
    prompt_preview: str = "—"
    worktree_state: str = "unknown"
    session_id: str | None = None
    repository: Repository | None = None
    window_name: str = ""


@dataclass(frozen=True)
class PromptHistory:
    commit: str
    date: str
    subject: str
    prompt: str

    @property
    def preview(self) -> str:
        for line in self.prompt.splitlines():
            if line.strip():
                return line.strip()
        return "(empty prompt)"


@dataclass(frozen=True)
class LaunchDraft:
    branch: str
    worktree: Path
    prompt: str
    overwrite_worktree: bool = False
    restart_worktree: bool = False
    existing_branch: bool = False
    agent: str = DEFAULT_LAUNCH_AGENT
    model: str = DEFAULT_LAUNCH_CHOICE
    effort: str = DEFAULT_LAUNCH_CHOICE
    repository: Repository | None = None


@dataclass(frozen=True)
class LaunchStep:
    label: str
    detail: str = ""
