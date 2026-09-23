//! Exercise environment names in startup path expansion through a real server.

#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Server {
    directory: PathBuf,
    socket_base: PathBuf,
    config_root: PathBuf,
    home: PathBuf,
    label: String,
}

impl Server {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        // Keep the socket below the Unix domain path length limit.
        let directory = PathBuf::from("/tmp").join(format!("hxe{:x}{stamp:x}", std::process::id()));
        let socket_base = directory.join(OsString::from_vec(b"socket-\xff".to_vec()));
        let config_root = directory.join("xdg");
        let home = directory.join("home");
        fs::create_dir_all(&socket_base).expect("create socket base");
        fs::create_dir_all(config_root.join("tmux")).expect("create config directory");
        fs::create_dir_all(&home).expect("create home directory");
        fs::write(
            config_root.join("tmux/tmux.conf"),
            b"set -g @expanded-config loaded\n",
        )
        .expect("write config");
        Self {
            directory,
            socket_base,
            config_root,
            home,
            label: format!("e{:x}{stamp:x}", std::process::id()),
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_hmux2"))
            .arg("-L")
            .arg(&self.label)
            .args(args)
            .env("TMUX_TMPDIR", &self.socket_base)
            .env("XDG_CONFIG_HOME", &self.config_root)
            .env("HOME", &self.home)
            .env_remove("TMUX")
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .output()
            .expect("run hmux2")
    }

    fn run_without_tmpdir(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_hmux2"))
            .arg("-L")
            .arg(&self.label)
            .args(args)
            .env_remove("TMUX_TMPDIR")
            .env("XDG_CONFIG_HOME", &self.config_root)
            .env("HOME", &self.home)
            .env_remove("TMUX")
            .env("TERM", "xterm-256color")
            .env("SHELL", "/bin/sh")
            .output()
            .expect("run hmux2 without TMUX_TMPDIR")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.run(&["kill-server"]);
        let _ = self.run_without_tmpdir(&["kill-server"]);
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn variable_only_socket_base_and_suffixed_config_path_expand() {
    let server = Server::new();
    let create = server.run(&["new-session", "-d", "-s", "expansion-test"]);
    assert!(
        create.status.success(),
        "new-session failed: {:?}",
        create.stderr
    );

    let socket = server
        .socket_base
        .join(format!("tmux-{}", unsafe { libc::getuid() }))
        .join(&server.label);
    assert!(socket.exists(), "socket path was not expanded: {socket:?}");

    let option = server.run(&["show-options", "-gqv", "@expanded-config"]);
    assert!(
        option.status.success(),
        "show-options failed: {:?}",
        option.stderr
    );
    assert_eq!(option.stdout, b"loaded\n");
}

#[test]
fn missing_tmpdir_falls_back_to_tmp() {
    let server = Server::new();
    let create = server.run_without_tmpdir(&["new-session", "-d", "-s", "expansion-test"]);
    assert!(
        create.status.success(),
        "new-session failed: {:?}",
        create.stderr
    );

    let socket = PathBuf::from("/tmp")
        .join(format!("tmux-{}", unsafe { libc::getuid() }))
        .join(&server.label);
    assert!(socket.exists(), "fallback socket missing: {socket:?}");
}
