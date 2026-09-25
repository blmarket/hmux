//! Signal and fork contracts run in a dedicated, single-threaded test executable.
use hmux_rt::mio;
use hmux_rt::{AsyncRead, Handle, Runtime, Signals};
use std::cell::Cell;
use std::future::Future;
use std::io::{self, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

fn tick(runtime: &mut mio::Runtime) {
    runtime.poll(Some(Duration::ZERO)).unwrap();
}
fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut Context::from_waker(Waker::noop()))
}
fn wait_child(pid: libc::pid_t) {
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert!(libc::WIFEXITED(status));
    assert_eq!(libc::WEXITSTATUS(status), 0);
}

fn signals() {
    let runtime = mio::Runtime::new().unwrap();
    let handle = runtime.handle();
    for set in [
        &[][..],
        &[libc::SIGKILL][..],
        &[libc::SIGUSR1, libc::SIGUSR1][..],
        &[999_999][..],
    ] {
        assert!(matches!(handle.signals(set), Err(e) if e.kind() == io::ErrorKind::InvalidInput));
    }
    let mut subscription = handle.signals(&[libc::SIGUSR1, libc::SIGUSR2]).unwrap();
    let other = mio::Runtime::new().unwrap();
    assert!(
        matches!(other.handle().signals(&[libc::SIGUSR1]), Err(e) if e.kind() == io::ErrorKind::AlreadyExists)
    );
    let mut cancelled = subscription.recv();
    assert!(poll(&mut cancelled).is_pending());
    drop(cancelled);
    unsafe {
        libc::raise(libc::SIGUSR2);
        libc::raise(libc::SIGUSR1);
        libc::raise(libc::SIGUSR2);
    }
    let mut seen = Vec::new();
    for _ in 0..2 {
        match poll(&mut subscription.recv()) {
            Poll::Ready(Ok(signal)) => seen.push(signal),
            _ => panic!("pending signal lost"),
        }
    }
    seen.sort();
    assert_eq!(seen, [libc::SIGUSR1, libc::SIGUSR2]);
    drop(runtime);
    assert!(
        matches!(poll(&mut subscription.recv()), Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::BrokenPipe)
    );
    let _replacement = other
        .handle()
        .signals(&[libc::SIGUSR1, libc::SIGUSR2])
        .unwrap();
}

fn delayed_signal() {
    let mut runtime = mio::Runtime::new().unwrap();
    let mut subscription = runtime.handle().signals(&[libc::SIGUSR1]).unwrap();
    let received = Rc::new(Cell::new(false));
    let mark = received.clone();
    let _task = runtime
        .handle()
        .spawn(async move {
            assert_eq!(subscription.recv().await.unwrap(), libc::SIGUSR1);
            mark.set(true);
        })
        .unwrap();
    tick(&mut runtime);
    let parent = unsafe { libc::getpid() };
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        std::thread::sleep(Duration::from_millis(20));
        unsafe {
            libc::kill(parent, libc::SIGUSR1);
            libc::_exit(0);
        }
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while !received.get() && Instant::now() < deadline {
        runtime.poll(Some(Duration::from_millis(100))).unwrap();
    }
    wait_child(pid);
    assert!(received.get());
}

fn fork_parent_survives(reset: bool) {
    let mut runtime = mio::Runtime::new().unwrap();
    let old_handle = runtime.handle();
    let (mut sender, receiver) = UnixStream::pair().unwrap();
    receiver.set_nonblocking(true).unwrap();
    let fd = Rc::new(OwnedFd::from(receiver));
    let source = old_handle.io(fd.clone()).unwrap();
    let ready = Rc::new(Cell::new(false));
    let mark = ready.clone();
    let _task = old_handle
        .spawn(async move {
            source.read(&mut [0; 1]).await.unwrap();
            mark.set(true);
        })
        .unwrap();
    tick(&mut runtime);
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        assert!(
            matches!(runtime.poll(Some(Duration::ZERO)), Err(e) if e.kind() == io::ErrorKind::BrokenPipe)
        );
        if reset {
            let mut saved = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            assert_eq!(
                unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut saved) },
                0
            );
            let limited = libc::rlimit {
                rlim_cur: 0,
                rlim_max: saved.rlim_max,
            };
            assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limited) }, 0);
            assert!(runtime.reset_after_fork().is_err());
            assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &saved) }, 0);
            runtime.reset_after_fork().unwrap();
            assert!(old_handle.spawn(async {}).is_err());
            let (mut child_sender, child_receiver) = UnixStream::pair().unwrap();
            child_receiver.set_nonblocking(true).unwrap();
            let child_io = runtime.handle().io(Rc::new(child_receiver.into())).unwrap();
            let done = Rc::new(Cell::new(false));
            let mark = done.clone();
            let _child_task = runtime
                .handle()
                .spawn(async move {
                    child_io.read(&mut [0; 1]).await.unwrap();
                    mark.set(true);
                })
                .unwrap();
            child_sender.write_all(b"child").unwrap();
            tick(&mut runtime);
            assert!(done.get());
        }
        drop(runtime);
        unsafe {
            libc::_exit(0);
        }
    }
    wait_child(pid);
    sender.write_all(b"parent").unwrap();
    tick(&mut runtime);
    assert!(
        ready.get(),
        "child cleanup changed parent's readiness registration"
    );
    // The supplied fd was never duplicated and remains owned by the caller.
    assert!(unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFD) } >= 0);
}

fn resource_churn() {
    #[cfg(target_os = "linux")]
    {
        fn count() -> usize {
            std::fs::read_dir("/proc/self/fd").unwrap().count()
        }
        // Warm up the signal library before measuring retained descriptors.
        let runtime = mio::Runtime::new().unwrap();
        drop(runtime.handle().signals(&[libc::SIGUSR1]).unwrap());
        drop(runtime);
        let baseline = count();
        for _ in 0..100 {
            let runtime = mio::Runtime::new().unwrap();
            let subscription = runtime
                .handle()
                .signals(&[libc::SIGUSR1, libc::SIGUSR2])
                .unwrap();
            drop(runtime);
            drop(subscription);
        }
        assert_eq!(count(), baseline);
    }
}

fn interrupted_wait_keeps_original_deadline() {
    let mut runtime = mio::Runtime::new().unwrap();
    let sleep = runtime
        .handle()
        .sleep_until(Instant::now() + Duration::from_secs(60));
    let _task = runtime
        .handle()
        .spawn(async move {
            sleep.await.unwrap();
        })
        .unwrap();
    tick(&mut runtime);
    // No self-pipe wake: repeated signals interrupt the underlying poll.
    let registration = unsafe { signal_hook::low_level::register(libc::SIGUSR2, || {}) }.unwrap();
    let parent = unsafe { libc::getpid() };
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        for _ in 0..80 {
            std::thread::sleep(Duration::from_millis(10));
            unsafe {
                libc::kill(parent, libc::SIGUSR2);
            }
        }
        unsafe {
            libc::_exit(0);
        }
    }
    let start = Instant::now();
    runtime.poll(Some(Duration::from_millis(50))).unwrap();
    let elapsed = start.elapsed();
    wait_child(pid);
    signal_hook::low_level::unregister(registration);
    assert!(
        elapsed < Duration::from_millis(400),
        "poll restarted its timeout: {elapsed:?}"
    );
}

fn main() {
    match std::env::var("HMUX_RT_CONTRACT_CASE").ok().as_deref() {
        Some("signals") => signals(),
        Some("delayed-signal") => delayed_signal(),
        Some("fork-reset") => fork_parent_survives(true),
        Some("fork-drop") => fork_parent_survives(false),
        Some("resource-churn") => resource_churn(),
        Some("interrupted-wait") => interrupted_wait_keeps_original_deadline(),
        Some(other) => panic!("unknown scenario {other}"),
        None => {
            for scenario in [
                "signals",
                "delayed-signal",
                "fork-reset",
                "fork-drop",
                "resource-churn",
                "interrupted-wait",
            ] {
                let status = std::process::Command::new(std::env::current_exe().unwrap())
                    .env("HMUX_RT_CONTRACT_CASE", scenario)
                    .status()
                    .unwrap();
                assert!(status.success(), "scenario {scenario} failed: {status}");
                println!("{scenario}: passed");
            }
        }
    }
}
