fn main() {
    for library in ["ncursesw", "utf8proc", "utempter", "systemd", "m", "resolv"] {
        println!("cargo:rustc-link-lib={library}");
    }
}
