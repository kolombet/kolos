// Stamps the binary with its git commit for `kolos-bar --version`; "-dirty" means bar/ had
// uncommitted changes when it was built.
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let dirty = git(&["status", "--porcelain", "--", "."]).is_some_and(|s| !s.is_empty());
    println!("cargo:rustc-env=KOLOS_BAR_COMMIT={commit}{}", if dirty { "-dirty" } else { "" });
    // Setting any rerun-if-changed turns off cargo's default package scan, so list the sources too.
    for path in ["src", "build.rs", "Cargo.toml", "../.git/HEAD", "../.git/logs/HEAD"] {
        println!("cargo:rerun-if-changed={path}");
    }
}
