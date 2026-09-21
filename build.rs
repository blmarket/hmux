fn main() {
    for library in ["ncursesw", "event_core", "utf8proc", "utempter", "systemd", "m", "resolv"] {
        println!("cargo:rustc-link-lib={library}");
    }
}
