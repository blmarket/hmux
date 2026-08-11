//! Demo for the prototype `Future` executor in `hmux::future_rt`.
//!
//! Run with `cargo run --example future_rt`. Each scene mirrors a shape from
//! the daemon: consecutive suspensions on fresh descriptors (the pipelined
//! wedge), a client task waiting on a command task without any kernel
//! object, and independent tasks interleaving on one thread.

use std::io::{self, Read as _};
use std::os::fd::{AsFd as _, AsRawFd as _, BorrowedFd};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use hmux::future_rt::{oneshot, sleep, AsyncFd, Handle, Interest, Runtime};

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

/// The run-shell suspension as an `async fn`: spawn the child, wait for its
/// output on a fresh descriptor, capture, reap. The equivalent of what takes
/// `RunningShell` plus a driver as a hand-written state machine.
async fn run_shell(handle: &Handle, command: &str) -> io::Result<Vec<u8>> {
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdout = child.stdout.take().expect("piped stdout");
    set_nonblocking(stdout.as_fd())?;
    let fd = AsyncFd::new(handle, stdout.as_fd(), Interest::READABLE)?;
    let mut output = Vec::new();
    'eof: loop {
        fd.readiness().await;
        loop {
            let mut chunk = [0u8; 4096];
            match stdout.read(&mut chunk) {
                Ok(0) => break 'eof,
                Ok(count) => output.extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }
    }
    drop(fd);
    // EOF means the child closed stdout, so this reap blocks at most
    // momentarily.
    child.wait()?;
    Ok(output)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim_end().to_string()
}

fn main() -> io::Result<()> {
    let runtime = Runtime::new()?;
    let handle = runtime.handle();
    let started = Instant::now();
    let stamp = move || format!("[{:>6.1}ms]", started.elapsed().as_secs_f64() * 1e3);

    // Scene 1 — the pipelined-wedge shape: one task runs two suspending
    // commands back to back. Each suspension registers a brand-new
    // descriptor; nothing is re-pointed, nothing is versioned.
    println!("== consecutive suspensions, fresh fd each ==");
    let scene = handle.clone();
    let print = stamp.clone();
    runtime.block_on(async move {
        let one = run_shell(&scene, "echo one").await.expect("first");
        println!("{} first suspension:  {:?}", print(), text(&one));
        let two = run_shell(&scene, "echo two").await.expect("second");
        println!("{} second suspension: {:?}", print(), text(&two));
    });

    // Scene 2 — the control-client shape: a "client" task waits on a
    // "command" task through a oneshot. While only the oneshot wait is
    // pending, the reactor holds zero registrations: waiting on another task
    // costs no kernel object.
    println!("== client waits on command, no fd ==");
    let scene = handle.clone();
    let probe = handle.clone();
    let print = stamp.clone();
    let reply = runtime.block_on(async move {
        let (sender, receiver) = oneshot();
        let shell = scene.clone();
        scene.spawn(async move {
            let output = run_shell(&shell, "sleep 0.05; echo done").await;
            sender.send(output.expect("shell output"));
        });
        println!(
            "{} client parked on oneshot ({} fds registered so far)",
            print(),
            probe.registered_fds(),
        );
        receiver.await.expect("command task completed")
    });
    println!("{} client resumed with {:?}", stamp(), text(&reply));

    // Scene 3 — interleaving: three tasks suspended at once on one thread —
    // two on child stdout, one on the timer wheel. Completion order follows
    // event order, not spawn order.
    println!("== three tasks interleave ==");
    let scene = handle.clone();
    let print = stamp.clone();
    runtime.block_on(async move {
        let (slow_sender, slow_receiver) = oneshot();
        let (fast_sender, fast_receiver) = oneshot();
        let shell = scene.clone();
        scene.spawn(async move {
            let output = run_shell(&shell, "sleep 0.10; echo slow").await;
            slow_sender.send(output.expect("slow shell"));
        });
        let shell = scene.clone();
        scene.spawn(async move {
            let output = run_shell(&shell, "sleep 0.02; echo fast").await;
            fast_sender.send(output.expect("fast shell"));
        });
        sleep(&scene, Duration::from_millis(50)).await;
        println!("{} timer fired between the two shells", print());
        let fast = fast_receiver.await.expect("fast task");
        println!("{} fast shell: {:?}", print(), text(&fast));
        let slow = slow_receiver.await.expect("slow task");
        println!("{} slow shell: {:?}", print(), text(&slow));
    });

    Ok(())
}
