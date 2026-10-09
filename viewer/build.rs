use std::process::Command;
fn git(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".into())
}
fn main() {
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/index");
    println!(
        "cargo:rustc-env=NV_BUILD_REVISION={}",
        git(&["rev-parse", "HEAD"])
    );
    println!(
        "cargo:rustc-env=NV_BUILD_DIRTY={}",
        !git(&["status", "--porcelain", "--untracked-files=normal"]).is_empty()
    );
}
