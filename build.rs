// build.rs
use std::process::Command;

fn main() {
    // rerun if src/*.rs has any changes
    println!("cargo:rerun-if-changed=src");
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let datestr = Command::new("date").output().unwrap();
    let build_date  = String::from_utf8(datestr.stdout).unwrap();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
