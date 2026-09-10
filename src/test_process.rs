//! Process isolation for unit tests that exercise process-global C APIs.
//!
//! The ordinary test harness stays parallel. Each selected test re-executes
//! only itself, so it never holds a lock that another test needs.

const ISOLATED_TEST: &str = "HMUX_ISOLATED_UNIT_TEST";

/// Returns true in the parent after the isolated test has passed. Callers
/// return immediately in that case; the child executes the original body.
pub(crate) fn run() -> bool {
    let thread = std::thread::current();
    let name = thread.name().expect("libtest names each test thread");
    if std::env::var(ISOLATED_TEST).as_deref() == Ok(name) {
        return false;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(ISOLATED_TEST, name)
        .output()
        .expect("start isolated unit test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains("1 passed; 0 failed"),
        "isolated test {name} failed ({}):\n{stdout}\n{stderr}",
        output.status,
    );
    true
}

/// Keep new tests from accidentally racing a process-global C buffer.
pub(crate) fn require(reason: &str) {
    assert!(
        std::env::var_os(ISOLATED_TEST).is_some(),
        "process isolation required ({reason}); start this test with `if crate::test_process::run() {{ return; }}`",
    );
}
