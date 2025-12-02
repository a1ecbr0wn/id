fn main() {
    env_git_hash();
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs");
    println!("cargo:rerun-if-changed=build.rs");
}

fn env_git_hash() {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command to get the hash");

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout);
        println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
        println!(
            "cargo:rustc-env=GIT_HASH_SHORT={}",
            hash.trim().chars().take(7).collect::<String>()
        );
    } else {
        println!("cargo:rustc-env=GIT_HASH=unknown");
    }
}
