//! The git plugin: where a pane's working directory sits in a git worktree,
//! and what the repository holding it is in the middle of.
//!
//! Every value comes out of files: the upward walk for `.git`, the `HEAD` it
//! names, and the marker files an interrupted operation leaves behind. Nothing
//! here runs git or reads the index, so a sweep costs one `readlink` per pane
//! and two `stat`s per repository, and the repositories are shared — a dozen
//! panes in one worktree are one entry, which is what keeps the sweep flat in
//! panes rather than in repositories.
//!
//! Values are computed on the tick and `resolve` reads what the last one
//! published, so expanding a status format never touches the filesystem.

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::osdep_linux::osdep_get_cwd;
use crate::window::window_pane_find_by_id;

use super::{Host, PaneId, Plugin};

/// The variables this plugin answers for, in the spelling `README.md` fixes.
const VARIABLES: &[&str] = &[
    "git_worktree",
    "git_worktree_path",
    "git_subdir",
    "git_repo",
    "git_branch",
    "git_head",
    "git_action",
    "git_action_step",
    "git_action_total",
];

/// How often every pane's working directory is read back. Slower than the
/// agent sweep: a `cd` is a keystroke away from being noticed either way, and
/// this one is doing filesystem work rather than reading `/proc`.
const INTERVAL: Duration = Duration::from_millis(500);

/// What the plugin publishes for one pane. Every field is empty for a pane
/// that is not in a repository, which is what lets a format branch on
/// `#{?git_worktree,…}` rather than on the plugin being enabled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PaneGit {
    worktree: String,
    worktree_path: String,
    subdir: String,
    repo: String,
    branch: String,
    head: String,
    action: String,
    action_step: String,
    action_total: String,
}

/// The per-pane values, plus the two caches that keep the sweep cheap: the
/// upward walk memoised by working directory, and the repository state
/// memoised by git directory.
pub struct GitPlugin {
    published: HashMap<PaneId, PaneGit>,
    roots: HashMap<PathBuf, Discovery>,
    repos: HashMap<PathBuf, Repo>,
}

impl GitPlugin {
    pub fn new() -> Self {
        GitPlugin {
            published: HashMap::new(),
            roots: HashMap::new(),
            repos: HashMap::new(),
        }
    }

    /// The worktree a directory belongs to, walking up for it only the first
    /// time a directory is seen. A directory in no repository is not cached,
    /// so a repository created later is still found.
    fn discover(&mut self, cwd: &Path) -> Option<Discovery> {
        if let Some(found) = self.roots.get(cwd) {
            return Some(found.clone());
        }
        let found = Discovery::find(cwd)?;
        self.roots.insert(cwd.to_path_buf(), found.clone());
        Some(found)
    }

    /// The repository state behind a discovery, re-read only when the git
    /// directory has moved since the last read. A repository mid-operation is
    /// re-read every sweep instead: its progress counters live in a
    /// subdirectory, which the git directory's own timestamp says nothing
    /// about. `None` means the git directory is gone.
    fn repo(&mut self, found: &Discovery) -> Option<Repo> {
        let stamp = stamp(&found.gitdir)?;
        if let Some(repo) = self.repos.get(&found.gitdir)
            && repo.stamp == stamp
            && repo.action.is_empty()
        {
            return Some(repo.clone());
        }
        let repo = Repo::read(found, stamp);
        self.repos.insert(found.gitdir.clone(), repo.clone());
        Some(repo)
    }

    /// Everything published for a pane sitting in `cwd`.
    fn pane_git(&mut self, cwd: &Path) -> PaneGit {
        let Some(found) = self.discover(cwd) else {
            return PaneGit::default();
        };
        let Some(repo) = self.repo(&found) else {
            self.roots.remove(cwd);
            self.repos.remove(&found.gitdir);
            return PaneGit::default();
        };
        PaneGit {
            worktree: file_name(&found.worktree),
            worktree_path: found.worktree.to_string_lossy().into_owned(),
            subdir: subdir(&found.worktree, cwd),
            repo: found.repo,
            head: match repo.branch.is_empty() {
                true => repo.commit.clone(),
                false => repo.branch.clone(),
            },
            branch: repo.branch,
            action: repo.action,
            action_step: repo.step,
            action_total: repo.total,
        }
    }
}

impl Default for GitPlugin {
    fn default() -> Self {
        GitPlugin::new()
    }
}

impl Plugin for GitPlugin {
    fn name(&self) -> &'static str {
        "git"
    }

    fn variables(&self) -> &'static [&'static str] {
        VARIABLES
    }

    fn interval(&self) -> Option<Duration> {
        Some(INTERVAL)
    }

    fn tick(&mut self, host: &dyn Host) {
        let Ok(ids) = host.pane_ids() else {
            return;
        };
        let mut panes: HashMap<PaneId, PaneGit> = HashMap::new();
        let mut live_cwds: HashSet<PathBuf> = HashSet::new();
        let mut live_gitdirs: HashSet<PathBuf> = HashSet::new();
        for id in ids {
            let Some(cwd) = pane_cwd(id) else {
                continue;
            };
            let value = self.pane_git(&cwd);
            if let Some(found) = self.roots.get(&cwd) {
                live_gitdirs.insert(found.gitdir.clone());
            }
            live_cwds.insert(cwd);
            panes.insert(id, value);
        }
        self.roots.retain(|cwd, _| live_cwds.contains(cwd));
        self.repos.retain(|gitdir, _| live_gitdirs.contains(gitdir));
        // Redraw only what moved: a pane whose values appeared, changed or
        // went away is one whose window is now drawing something stale.
        for (id, value) in &panes {
            if self.published.get(id) != Some(value) {
                host.invalidate(*id);
            }
        }
        for id in self.published.keys() {
            if !panes.contains_key(id) {
                host.invalidate(*id);
            }
        }
        self.published = panes;
    }

    fn resolve(&self, pane: PaneId, key: &str) -> Option<String> {
        let value = self.published.get(&pane);
        let field = |pick: fn(&PaneGit) -> &str| Some(value.map_or("", pick).to_string());
        match key {
            "git_worktree" => field(|git| &git.worktree),
            "git_worktree_path" => field(|git| &git.worktree_path),
            "git_subdir" => field(|git| &git.subdir),
            "git_repo" => field(|git| &git.repo),
            "git_branch" => field(|git| &git.branch),
            "git_head" => field(|git| &git.head),
            "git_action" => field(|git| &git.action),
            "git_action_step" => field(|git| &git.action_step),
            "git_action_total" => field(|git| &git.action_total),
            _ => None,
        }
    }
}

/// Where the upward walk stopped: the worktree root, the git directory it
/// names, and the repository that directory belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Discovery {
    worktree: PathBuf,
    gitdir: PathBuf,
    repo: String,
    /// Whether refs live in a reftable rather than in files, which is what
    /// makes reading `HEAD` as a file meaningless.
    reftable: bool,
}

impl Discovery {
    /// The first worktree at or above `cwd`, or nothing if the walk reaches
    /// the filesystem root without finding one.
    fn find(cwd: &Path) -> Option<Discovery> {
        let mut dir = cwd;
        loop {
            if let Some(gitdir) = git_dir_at(dir) {
                let common = common_dir(&gitdir);
                return Some(Discovery {
                    worktree: dir.to_path_buf(),
                    repo: repo_name(&common),
                    reftable: common.join("reftable").is_dir(),
                    gitdir,
                });
            }
            dir = dir.parent()?;
        }
    }
}

/// The modification times a repository's cheap state hangs off: the git
/// directory itself, which moves when a marker file appears or goes, and
/// `HEAD`, which moves on checkout.
type Stamp = (Option<SystemTime>, Option<SystemTime>);

/// What the last read of a repository found. `commit` is only filled in for a
/// detached HEAD, since a branch name is the better answer whenever there is
/// one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Repo {
    stamp: Stamp,
    branch: String,
    commit: String,
    action: String,
    step: String,
    total: String,
}

impl Repo {
    fn read(found: &Discovery, stamp: Stamp) -> Repo {
        let (action, step, total) = read_action(&found.gitdir);
        let (mut branch, commit) = match found.reftable {
            true => (String::new(), String::new()),
            false => read_head(&found.gitdir),
        };
        if branch.is_empty() {
            branch = rebased_branch(&found.gitdir);
        }
        Repo {
            stamp,
            branch,
            commit,
            action,
            step,
            total,
        }
    }
}

/// The timestamps a repository's state is re-read on, or nothing once the git
/// directory has gone — a worktree removed under the pane's feet.
fn stamp(gitdir: &Path) -> Option<Stamp> {
    let dir = fs::metadata(gitdir).ok()?;
    if !dir.is_dir() {
        return None;
    }
    let head = fs::metadata(gitdir.join("HEAD"))
        .ok()
        .and_then(|head| head.modified().ok());
    Some((dir.modified().ok(), head))
}

/// The working directory of the pane's foreground process, as the pane's own
/// `#{pane_current_path}` reads it.
fn pane_cwd(pane: PaneId) -> Option<PathBuf> {
    let wp = window_pane_find_by_id(pane.0);
    if wp.is_null() {
        return None;
    }
    let cwd = osdep_get_cwd(unsafe { (*wp).fd })?;
    let path = PathBuf::from(OsString::from_vec(cwd.into_bytes()));
    path.is_absolute().then_some(path)
}

/// The git directory a worktree root holds: `.git` itself when it is a
/// directory, or the directory the `.git` file points at when the worktree is
/// a linked one.
fn git_dir_at(dir: &Path) -> Option<PathBuf> {
    let dot = dir.join(".git");
    let meta = fs::metadata(&dot).ok()?;
    if meta.is_dir() {
        return Some(dot);
    }
    if !meta.is_file() {
        return None;
    }
    let text = fs::read_to_string(&dot).ok()?;
    let named = text
        .lines()
        .find_map(|line| line.strip_prefix("gitdir:"))?
        .trim();
    match named.is_empty() {
        true => None,
        false => Some(normalize(&dir.join(named))),
    }
}

/// The git directory the whole repository shares, which is the given one
/// unless it is a linked worktree's.
fn common_dir(gitdir: &Path) -> PathBuf {
    let named = fs::read_to_string(gitdir.join("commondir"))
        .ok()
        .and_then(|text| text.lines().next().map(str::trim).map(str::to_string))
        .unwrap_or_default();
    match named.is_empty() {
        true => gitdir.to_path_buf(),
        false => normalize(&gitdir.join(named)),
    }
}

/// The repository's name: the directory holding its common git directory, or
/// the bare directory's own name with the conventional suffix dropped.
fn repo_name(common: &Path) -> String {
    let name = file_name(common);
    if name == ".git" {
        return common.parent().map(file_name).unwrap_or_default();
    }
    name.strip_suffix(".git").unwrap_or(&name).to_string()
}

/// The branch `HEAD` names and, when it names none, the short form of the
/// commit it is parked on.
fn read_head(gitdir: &Path) -> (String, String) {
    let Ok(text) = fs::read_to_string(gitdir.join("HEAD")) else {
        return (String::new(), String::new());
    };
    let line = text.lines().next().unwrap_or_default().trim();
    if let Some(reference) = line.strip_prefix("ref:") {
        let reference = reference.trim();
        let branch = reference.strip_prefix("refs/heads/").unwrap_or(reference);
        // What a reftable repository leaves in the file for readers that
        // predate it; there is no branch name here to be had.
        if branch == ".invalid" {
            return (String::new(), String::new());
        }
        return (branch.to_string(), String::new());
    }
    match line.len() >= 7 && line.chars().all(|byte| byte.is_ascii_hexdigit()) {
        true => (String::new(), line[..7].to_string()),
        false => (String::new(), String::new()),
    }
}

/// The branch an interrupted rebase is rebuilding. HEAD is detached for the
/// length of the operation, and this is the name the pane was on before it
/// started.
fn rebased_branch(gitdir: &Path) -> String {
    for dir in ["rebase-merge", "rebase-apply"] {
        let Ok(text) = fs::read_to_string(gitdir.join(dir).join("head-name")) else {
            continue;
        };
        let name = text.lines().next().unwrap_or_default().trim();
        if let Some(branch) = name.strip_prefix("refs/heads/") {
            return branch.to_string();
        }
    }
    String::new()
}

/// The operation the repository is in the middle of, with the step it has
/// reached and the number of steps it has, for the ones that count.
///
/// The two rebase backends are one `rebase` here. The marker that looks like
/// it separates an interactive rebase from a plain one — `rebase-merge/
/// interactive` — is written for every rebase the merge backend runs, so
/// reporting an interactive rebase from it would be wrong for the common
/// case rather than right for the rare one.
fn read_action(gitdir: &Path) -> (String, String, String) {
    let merge = gitdir.join("rebase-merge");
    if merge.is_dir() {
        return (
            "rebase".to_string(),
            count(&merge.join("msgnum")),
            count(&merge.join("end")),
        );
    }
    let apply = gitdir.join("rebase-apply");
    if apply.is_dir() {
        let action = match apply.join("rebasing").exists() {
            true => "rebase",
            false => "am",
        };
        return (
            action.to_string(),
            count(&apply.join("next")),
            count(&apply.join("last")),
        );
    }
    for (marker, action) in [
        ("MERGE_HEAD", "merge"),
        ("BISECT_LOG", "bisect"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("REVERT_HEAD", "revert"),
    ] {
        if gitdir.join(marker).exists() {
            return (action.to_string(), String::new(), String::new());
        }
    }
    (String::new(), String::new(), String::new())
}

/// A progress counter, or nothing when the file is missing or holds something
/// that is not one.
fn count(path: &Path) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    let value = text.lines().next().unwrap_or_default().trim();
    match !value.is_empty() && value.chars().all(|byte| byte.is_ascii_digit()) {
        true => value.to_string(),
        false => String::new(),
    }
}

/// The path from the worktree root down to the pane's directory, empty at the
/// root itself.
fn subdir(worktree: &Path, cwd: &Path) -> String {
    cwd.strip_prefix(worktree)
        .map(|rest| rest.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The last component of a path, as a string.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// A path with its `.` and `..` components resolved by name alone. The paths
/// this resolves are `commondir` and `.git`-file targets, which are written
/// relative to the file that names them and are never followed as a symlink
/// by git either.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() && !out.has_root() {
                    out.push(Component::ParentDir);
                }
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::types::u_int;

    /// A directory tree that goes away with the test, for the file layouts
    /// these functions read.
    ///
    /// Directories are created one component at a time and given their mode
    /// explicitly, because the umask is process-wide: another test starting a
    /// server sets one that leaves a new directory without its execute bit,
    /// and anything created inside it in that window fails.
    struct Scratch {
        root: PathBuf,
    }

    impl Scratch {
        fn new() -> Scratch {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "tmux-c2rs-git-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&root);
            make_dir(&root);
            Scratch { root }
        }

        /// Write `text` to `path` under the scratch root, making the
        /// directories above it.
        fn write(&self, path: &str, text: &str) -> PathBuf {
            let full = self.root.join(path);
            make_dir(full.parent().expect("a parent"));
            fs::write(&full, text).expect("a file");
            full
        }

        fn dir(&self, path: &str) -> PathBuf {
            let full = self.root.join(path);
            make_dir(&full);
            full
        }
    }

    /// Make a directory and everything above it, each with a mode of its own.
    fn make_dir(path: &Path) {
        let mut at = PathBuf::new();
        for part in path.components() {
            at.push(part);
            if fs::create_dir(&at).is_ok() {
                let mode = fs::Permissions::from_mode(0o755);
                fs::set_permissions(&at, mode).expect("a directory mode");
            }
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// The layout `git worktree add` leaves behind: a `.git` file in the
    /// linked worktree pointing into the main repository's git directory,
    /// which points back with `commondir`. The worktree is named by its own
    /// directory and the repository by the one holding the common git dir,
    /// which is the distinction the window label needs.
    #[test]
    fn a_linked_worktree_is_named_apart_from_its_repository() {
        let scratch = Scratch::new();
        let gitdir = scratch.dir("hmux/.git/worktrees/h1");
        fs::write(gitdir.join("commondir"), "../..\n").expect("commondir");
        fs::write(gitdir.join("HEAD"), "ref: refs/heads/h1\n").expect("HEAD");
        scratch.write(
            "h1/.git",
            &format!("gitdir: {}\n", gitdir.to_string_lossy()),
        );
        let deep = scratch.dir("h1/tmux-c2rs/src");

        let found = Discovery::find(&deep).expect("a worktree");
        assert_eq!(found.worktree, scratch.root.join("h1"));
        assert_eq!(found.gitdir, gitdir);
        assert_eq!(found.repo, "hmux", "the repository, not the worktree");
        assert_eq!(file_name(&found.worktree), "h1");
        assert_eq!(subdir(&found.worktree, &deep), "tmux-c2rs/src");
        assert_eq!(
            subdir(&found.worktree, &scratch.root.join("h1")),
            "",
            "nothing below the root"
        );

        let repo = Repo::read(&found, (None, None));
        assert_eq!(repo.branch, "h1");
        assert_eq!(repo.action, "");
    }

    /// A plain checkout: `.git` is the git directory, and the repository and
    /// the worktree are the same directory.
    #[test]
    fn a_plain_checkout_is_its_own_repository() {
        let scratch = Scratch::new();
        scratch.write("hmux/.git/HEAD", "ref: refs/heads/master\n");

        let found = Discovery::find(&scratch.dir("hmux")).expect("a worktree");
        assert_eq!(found.repo, "hmux");
        assert_eq!(found.gitdir, scratch.root.join("hmux/.git"));
        assert_eq!(Repo::read(&found, (None, None)).branch, "master");
    }

    /// A directory in no repository at all, which is every pane whose window
    /// label falls back to its working directory.
    #[test]
    fn a_directory_outside_a_repository_finds_nothing() {
        let scratch = Scratch::new();
        assert_eq!(Discovery::find(&scratch.dir("elsewhere")), None);
    }

    /// A detached HEAD reports the commit rather than a branch, and `git_head`
    /// is whichever of the two there is.
    #[test]
    fn a_detached_head_reports_its_commit() {
        let scratch = Scratch::new();
        scratch.write(
            "repo/.git/HEAD",
            "38b63b01c0ffee00c0ffee00c0ffee00c0ffee00\n",
        );

        let found = Discovery::find(&scratch.dir("repo")).expect("a worktree");
        let repo = Repo::read(&found, (None, None));
        assert_eq!(repo.branch, "");
        assert_eq!(repo.commit, "38b63b0");
    }

    /// A rebase stopped part way through, which is the state a multiplexer is
    /// best placed to show: the operation, how far it has got, and the branch
    /// it is rebuilding — HEAD is detached throughout, so the branch comes
    /// from the operation's own state.
    #[test]
    fn an_interrupted_rebase_reports_its_progress() {
        let scratch = Scratch::new();
        scratch.write(
            "repo/.git/HEAD",
            "38b63b01c0ffee00c0ffee00c0ffee00c0ffee00\n",
        );
        scratch.write("repo/.git/rebase-merge/head-name", "refs/heads/h1\n");
        scratch.write("repo/.git/rebase-merge/msgnum", "2\n");
        scratch.write("repo/.git/rebase-merge/end", "7\n");

        let found = Discovery::find(&scratch.dir("repo")).expect("a worktree");
        let repo = Repo::read(&found, (None, None));
        assert_eq!(repo.action, "rebase");
        assert_eq!((repo.step.as_str(), repo.total.as_str()), ("2", "7"));
        assert_eq!(repo.branch, "h1", "the branch being rebased");
    }

    /// The markers an interrupted merge, bisect, cherry-pick or revert leave,
    /// none of which count steps.
    #[test]
    fn the_other_operations_are_named_by_their_markers() {
        let scratch = Scratch::new();
        scratch.write("repo/.git/HEAD", "ref: refs/heads/master\n");
        let gitdir = scratch.root.join("repo/.git");

        for (marker, action) in [
            ("MERGE_HEAD", "merge"),
            ("BISECT_LOG", "bisect"),
            ("CHERRY_PICK_HEAD", "cherry-pick"),
            ("REVERT_HEAD", "revert"),
        ] {
            fs::write(gitdir.join(marker), "").expect("a marker");
            assert_eq!(
                read_action(&gitdir),
                (action.to_string(), String::new(), String::new())
            );
            fs::remove_file(gitdir.join(marker)).expect("a marker");
        }
        assert_eq!(
            read_action(&gitdir),
            (String::new(), String::new(), String::new()),
            "an idle repository"
        );
    }

    /// A repository whose refs are in a reftable has no branch name in a file
    /// to read, and says nothing rather than reporting the placeholder git
    /// leaves in `HEAD` for readers that predate it.
    #[test]
    fn a_reftable_repository_reports_no_branch() {
        let scratch = Scratch::new();
        scratch.write("repo/.git/HEAD", "ref: refs/heads/.invalid\n");
        scratch.dir("repo/.git/reftable");

        let found = Discovery::find(&scratch.dir("repo")).expect("a worktree");
        assert!(found.reftable);
        let repo = Repo::read(&found, (None, None));
        assert_eq!((repo.branch.as_str(), repo.commit.as_str()), ("", ""));
    }

    /// A pane the last sweep had nothing for reads as empty everywhere, and
    /// every variable the plugin claims is one it answers — a name in one list
    /// and not the other would expand to nothing at all.
    #[test]
    fn every_claimed_variable_is_answered() {
        let plugin = GitPlugin::new();
        for key in plugin.variables() {
            assert_eq!(
                plugin.resolve(PaneId(u_int::MAX), key).as_deref(),
                Some(""),
                "{key} is claimed but not answered"
            );
        }
        assert_eq!(plugin.resolve(PaneId(u_int::MAX), "pane_agent"), None);
    }

    /// Relative paths in `commondir` and in a linked worktree's `.git` are
    /// resolved by name, without asking the filesystem.
    #[test]
    fn relative_git_paths_are_resolved_by_name() {
        assert_eq!(
            normalize(Path::new("/a/b/.git/worktrees/h1/../..")),
            Path::new("/a/b/.git")
        );
        assert_eq!(normalize(Path::new("/a/./b/")), Path::new("/a/b"));
        assert_eq!(repo_name(Path::new("/a/hmux/.git")), "hmux");
        assert_eq!(repo_name(Path::new("/a/hmux.git")), "hmux", "a bare repo");
    }
}
