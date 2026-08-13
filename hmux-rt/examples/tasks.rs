//! Demo for the `hmux-rt` task executor.
//!
//! Run with `cargo run --example tasks`. Each scene mirrors a shape from the
//! hmux daemon: consecutive suspensions on fresh descriptors (the pipelined
//! wedge), a client task waiting on a command task without any kernel object,
//! and independent tasks interleaving on one thread.
//!
//! The runtime here is the same executor the daemon embeds, driven by the
//! standalone `block_on` host, so what these scenes show is what `run-shell`
//! and `if-shell` actually do.

use std::io::{self, Read as _};
use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use hmux_rt::{sleep, AsyncFd, Interest, TaskHandle, TaskRuntime};

fn set_nonblocking(fd: BorrowedFd<'_>) -> io::Result<()> {
    let raw = fd.as_raw_fd();
    let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// The `run-shell` suspension, in miniature: spawn the child, wait for its
/// output on a fresh descriptor, capture, reap.
///
/// The daemon's version is `server::command::suspend::run_shell`; this one drops
/// the flag parsing and the tmux-shaped reporting to leave the waiting visible.
async fn run_shell(tasks: &TaskHandle, command: &str) -> io::Result<Vec<u8>> {
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdout = child.stdout.take().expect("piped stdout");
    set_nonblocking(stdout.as_fd())?;
    let source = AsyncFd::new(tasks, stdout.as_fd(), Interest::READABLE)?;
    let mut output = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        match stdout.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => output.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                source.readiness().await;
            }
            Err(error) => return Err(error),
        }
    }
    drop(source);
    // End of file means the child closed stdout; it has exited or is exiting,
    // so this reap blocks at most momentarily. Good enough for a demo — the
    // daemon backs off on the loop's timer queue instead.
    child.wait()?;
    Ok(output)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim_end().to_string()
}

fn main() -> io::Result<()> {
    let mut runtime = TaskRuntime::new()?;
    let handle = runtime.handle();
    let started = Instant::now();
    let stamp = move || format!("[{:>6.1}ms]", started.elapsed().as_secs_f64() * 1e3);

    // Scene 1 — the pipelined-wedge shape: one task runs two suspending
    // commands back to back. Each suspension registers a brand-new descriptor
    // owned by the `AsyncFd` that reads it; nothing is re-pointed, nothing is
    // versioned.
    println!("== consecutive suspensions, fresh fd each ==");
    let scene = handle.clone();
    let print = stamp.clone();
    runtime.block_on(async move {
        let one = run_shell(&scene, "echo one").await.expect("first");
        println!("{} first suspension:  {:?}", print(), text(&one));
        let two = run_shell(&scene, "echo two").await.expect("second");
        println!("{} second suspension: {:?}", print(), text(&two));
    });

    // Scene 2 — the control-client shape: a "client" task waits on a "command"
    // task through its join handle. Waiting on another task costs no kernel
    // object at all.
    println!("== client waits on command, no fd ==");
    let scene = handle.clone();
    let print = stamp.clone();
    let reply = runtime.block_on(async move {
        let shell = scene.clone();
        let command = scene.spawn_join(async move {
            let output = run_shell(&shell, "sleep 0.05; echo done").await;
            output.expect("shell output")
        });
        println!("{} client parked on the command's join handle", print());
        command.await.expect("command task completed")
    });
    println!("{} client resumed with {:?}", stamp(), text(&reply));

    // Scene 3 — interleaving: three tasks suspended at once on one thread — two
    // on child stdout, one on the loop's timer queue. Completion order follows
    // event order, not spawn order.
    println!("== three tasks interleave ==");
    let scene = handle.clone();
    let print = stamp.clone();
    runtime.block_on(async move {
        let shell = scene.clone();
        let slow = scene.spawn_join(async move {
            let output = run_shell(&shell, "sleep 0.10; echo slow").await;
            output.expect("slow shell")
        });
        let shell = scene.clone();
        let fast = scene.spawn_join(async move {
            let output = run_shell(&shell, "sleep 0.02; echo fast").await;
            output.expect("fast shell")
        });
        sleep(&scene, Duration::from_millis(50)).await;
        println!("{} timer fired between the two shells", print());
        let fast = fast.await.expect("fast task");
        println!("{} fast shell: {:?}", print(), text(&fast));
        let slow = slow.await.expect("slow task");
        println!("{} slow shell: {:?}", print(), text(&slow));
    });

    Ok(())
}
