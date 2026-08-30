fn main() {
    lalrpop::process_src().unwrap();
    println!("cargo:rustc-link-lib=ncursesw");
    println!("cargo:rustc-link-lib=utf8proc");
    println!("cargo:rustc-link-lib=utempter");
    println!("cargo:rustc-link-lib=systemd");
    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=resolv");
}
