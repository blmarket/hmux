//! End-to-end bracketed-paste coverage through the client's terminal path.

#![cfg(unix)]

use std::ffi::CString;
use std::fs;
use std::io::{self, ErrorKind, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TEST_TIMEOUT: Duration = Duration::from_secs(8);

struct PtyClient {
    master: Option<std::fs::File>,
    pid: Option<libc::pid_t>,
    binary: PathBuf,
    socket: PathBuf,
    directory: PathBuf,
}

impl PtyClient {
    fn new(binary: PathBuf, socket: PathBuf, directory: PathBuf) -> io::Result<Self> {
        let mut master = -1;
        let window = libc::winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let pid =
            unsafe { libc::forkpty(&mut master, std::ptr::null_mut(), std::ptr::null(), &window) };
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        if pid == 0 {
            let mut terminal: libc::termios = unsafe { std::mem::zeroed() };
            if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut terminal) } == 0 {
                terminal.c_lflag &= !(libc::ECHO | libc::ECHONL);
                unsafe {
                    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &terminal);
                }
            }
            for (name, value) in [
                ("TERM", "xterm-256color"),
                ("SHELL", "/bin/sh"),
                ("TMUX", ""),
                ("LC_ALL", "C"),
            ] {
                let name = CString::new(name).expect("static environment name");
                let value = CString::new(value).expect("static environment value");
                unsafe {
                    libc::setenv(name.as_ptr(), value.as_ptr(), 1);
                }
            }
            let args = [
                CString::new("hmux2").expect("static argv0"),
                CString::new("-f").expect("static option"),
                CString::new("/dev/null").expect("static config"),
                CString::new("-S").expect("static option"),
                CString::new(socket.as_os_str().as_encoded_bytes())
                    .expect("socket path has no NUL"),
                CString::new("new-session").expect("static command"),
                CString::new("-s").expect("static option"),
                CString::new("bracketed-paste-test").expect("static session name"),
                CString::new("printf READY; exec cat -v").expect("static pane command"),
            ];
            let mut argv = args.iter().map(|arg| arg.as_ptr()).collect::<Vec<_>>();
            argv.push(std::ptr::null());
            let binary = CString::new(binary.as_os_str().as_encoded_bytes())
                .expect("binary path has no NUL");
            unsafe {
                libc::execv(binary.as_ptr(), argv.as_ptr());
                libc::_exit(127);
            }
        }

        let master_file = unsafe { std::fs::File::from_raw_fd(master) };
        let flags = unsafe { libc::fcntl(master_file.as_raw_fd(), libc::F_GETFL) };
        let nonblocking = flags >= 0
            && unsafe {
                libc::fcntl(
                    master_file.as_raw_fd(),
                    libc::F_SETFL,
                    flags | libc::O_NONBLOCK,
                )
            } >= 0;
        if !nonblocking {
            let error = io::Error::last_os_error();
            unsafe {
                libc::kill(pid, libc::SIGTERM);
                libc::waitpid(pid, std::ptr::null_mut(), 0);
            }
            return Err(error);
        }
        Ok(Self {
            master: Some(master_file),
            pid: Some(pid),
            binary,
            socket,
            directory,
        })
    }

    fn poll(&self, events: libc::c_short, timeout: Duration) -> io::Result<()> {
        let fd = self
            .master
            .as_ref()
            .expect("PTY master is open")
            .as_raw_fd();
        let timeout = timeout.as_millis().min(i32::MAX as u128) as libc::c_int;
        let mut descriptor = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        loop {
            let result = unsafe { libc::poll(&mut descriptor, 1, timeout) };
            if result >= 0 {
                return Ok(());
            }
            let error = io::Error::last_os_error();
            if error.kind() != ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8], deadline: Instant) -> io::Result<()> {
        let mut written = 0;
        while written < bytes.len() {
            if Instant::now() >= deadline {
                return Err(io::Error::new(ErrorKind::TimedOut, "PTY write timed out"));
            }
            let result = self
                .master
                .as_mut()
                .expect("PTY master is open")
                .write(&bytes[written..]);
            match result {
                Ok(0) => return Err(io::Error::new(ErrorKind::WriteZero, "PTY closed")),
                Ok(count) => written += count,
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    self.poll(
                        libc::POLLOUT,
                        deadline.saturating_duration_since(Instant::now()),
                    )?;
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn read_available(&mut self, output: &mut Vec<u8>) -> io::Result<()> {
        let mut chunk = [0u8; 4096];
        loop {
            let result = self
                .master
                .as_mut()
                .expect("PTY master is open")
                .read(&mut chunk);
            match result {
                Ok(0) => return Ok(()),
                Ok(count) => output.extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(()),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) if error.raw_os_error() == Some(libc::EIO) => return Ok(()),
                Err(error) => return Err(error),
            }
        }
    }

    fn read_until(&mut self, needle: &[u8], deadline: Instant) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        while Instant::now() < deadline {
            self.read_available(&mut output)?;
            if output.windows(needle.len()).any(|window| window == needle) {
                return Ok(output);
            }
            self.poll(
                libc::POLLIN,
                deadline.saturating_duration_since(Instant::now()),
            )?;
        }
        Err(io::Error::new(
            ErrorKind::TimedOut,
            format!("did not receive {needle:?}; output was {output:?}"),
        ))
    }

    fn wait_for_exit(&mut self, deadline: Instant) -> bool {
        let Some(pid) = self.pid else {
            return true;
        };
        while Instant::now() < deadline {
            let mut status = 0;
            let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
            if result == pid || result < 0 {
                self.pid = None;
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    fn stop_server(&self) {
        if !self.socket.exists() {
            return;
        }
        let Ok(mut command) = Command::new(&self.binary)
            .args([
                "-f",
                "/dev/null",
                "-S",
                self.socket.to_str().expect("socket path is UTF-8"),
                "kill-server",
            ])
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .env("TMUX", "")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match command.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                Ok(None) => {
                    let _ = command.kill();
                    let _ = command.wait();
                    return;
                }
                Err(_) => return,
            }
        }
    }

    fn command_output(&self, args: &[&str]) -> io::Result<std::process::Output> {
        Command::new(&self.binary)
            .args(["-f", "/dev/null", "-S"])
            .arg(&self.socket)
            .args(args)
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .env("TMUX", "")
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .output()
    }
}

impl Drop for PtyClient {
    fn drop(&mut self) {
        if let Some(master) = self.master.as_mut() {
            let _ = master.write_all(b"\x04");
        }
        if self.pid.is_some() && !self.wait_for_exit(Instant::now() + Duration::from_millis(500)) {
            if let Some(pid) = self.pid {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }
            if !self.wait_for_exit(Instant::now() + Duration::from_millis(500)) {
                if let Some(pid) = self.pid {
                    unsafe {
                        libc::kill(pid, libc::SIGKILL);
                        libc::waitpid(pid, std::ptr::null_mut(), 0);
                    }
                }
                self.pid = None;
            }
        }
        self.master.take();
        self.stop_server();
        let _ = fs::remove_file(&self.socket);
        let _ = fs::remove_file(self.socket.with_extension("lock"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn unique_directory() -> io::Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("hmux2-bracketed-paste-{stamp}"));
    fs::create_dir(&directory)?;
    Ok(directory)
}

fn contains(output: &[u8], needle: &[u8]) -> bool {
    output.windows(needle.len()).any(|window| window == needle)
}

#[test]
fn split_end_boundary_delivers_paste_bytes_without_boundary_bytes() {
    let directory = unique_directory().expect("create private PTY directory");
    let socket = directory.join("socket");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_hmux2"));
    let mut client = PtyClient::new(binary, socket, directory).expect("start hmux2 in a PTY");
    let deadline = Instant::now() + TEST_TIMEOUT;

    client
        .read_until(b"READY", deadline)
        .expect("wait for the pane command");

    let payload = b"inside-\x1b[20x\n";
    client
        .write_bytes(b"\x1b[200~", deadline)
        .expect("write paste start");
    client
        .write_bytes(payload, deadline)
        .expect("write paste payload");
    client
        .write_bytes(b"\x1b[201", deadline)
        .expect("write partial paste end");
    thread::sleep(Duration::from_millis(100));
    client
        .write_bytes(b"~AFTER\n", deadline)
        .expect("finish paste end and write trailing bytes");

    let output = client
        .read_until(b"AFTER", deadline)
        .expect("wait for bytes delivered after the paste");
    assert!(
        contains(&output, b"inside-^[[20x"),
        "paste payload missing: {output:?}"
    );
    assert!(
        !contains(&output, b"^[[200~"),
        "paste start leaked: {output:?}"
    );
    assert!(
        !contains(&output, b"^[[201~"),
        "paste end leaked: {output:?}"
    );
}

#[test]
fn clipboard_reply_decodes_counted_base64_and_preserves_first_nul() {
    let directory = unique_directory().expect("create private PTY directory");
    let socket = directory.join("socket");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_hmux2"));
    let mut client = PtyClient::new(binary, socket, directory).expect("start hmux2 in a PTY");
    let deadline = Instant::now() + TEST_TIMEOUT;

    client
        .read_until(b"READY", deadline)
        .expect("wait for the pane command");
    let clients = client
        .command_output(&["list-clients", "-F", "#{client_name}"])
        .expect("list attached clients");
    assert!(clients.status.success(), "{clients:?}");
    let name = String::from_utf8(clients.stdout).expect("client name is UTF-8");
    let name = name.trim();
    assert!(!name.is_empty(), "attached client has a name");

    let refresh = client
        .command_output(&["refresh-client", "-l", "-t", name])
        .expect("request terminal clipboard");
    assert!(refresh.status.success(), "{refresh:?}");
    client
        .write_bytes(b"\x1b]52;c;QQBC\x07", deadline)
        .expect("reply with binary clipboard data");
    loop {
        let shown = client
            .command_output(&["show-buffer"])
            .expect("read paste buffer");
        if shown.status.success() && shown.stdout == b"A\0B" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "clipboard not decoded: {shown:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }

    let refresh = client
        .command_output(&["refresh-client", "-l", "-t", name])
        .expect("request terminal clipboard again");
    assert!(refresh.status.success(), "{refresh:?}");
    client
        .write_bytes(b"\x1b]52;c;QQ==\0QkI=\x1b\\", deadline)
        .expect("reply with embedded NUL in the encoded input");
    loop {
        let shown = client
            .command_output(&["show-buffer"])
            .expect("read paste buffer");
        if shown.status.success() && shown.stdout == b"A" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "first NUL not preserved: {shown:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn set_buffer_writes_base64_selection_to_attached_terminal() {
    let directory = unique_directory().expect("create private PTY directory");
    let socket = directory.join("socket");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_hmux2"));
    let mut client = PtyClient::new(binary, socket, directory).expect("start hmux2 in a PTY");
    let deadline = Instant::now() + TEST_TIMEOUT;

    client
        .read_until(b"READY", deadline)
        .expect("wait for the pane command");
    let clients = client
        .command_output(&["list-clients", "-F", "#{client_name}"])
        .expect("list attached clients");
    assert!(clients.status.success(), "{clients:?}");
    let name = String::from_utf8(clients.stdout).expect("client name is UTF-8");
    let name = name.trim();
    assert!(!name.is_empty(), "attached client has a name");

    let result = client
        .command_output(&["set-buffer", "-w", "-t", name, "A+B/!"])
        .expect("set clipboard buffer");
    assert!(result.status.success(), "{result:?}");
    client
        .read_until(b"\x1b]52;;QStCLyE=\x07", deadline)
        .expect("receive encoded clipboard data");

    let path = client.directory.join("binary-clipboard");
    fs::write(&path, b"A\0B").expect("write binary clipboard data");
    let result = client
        .command_output(&[
            "load-buffer",
            "-w",
            "-t",
            name,
            path.to_str().expect("clipboard path is UTF-8"),
        ])
        .expect("load binary clipboard buffer");
    assert!(result.status.success(), "{result:?}");
    client
        .read_until(b"\x1b]52;;QQBC\x07", deadline)
        .expect("receive encoded binary clipboard data");
    fs::remove_file(path).expect("remove binary clipboard file");
}
