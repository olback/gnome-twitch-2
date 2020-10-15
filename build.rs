use {
    std::{process::Command, path::PathBuf},
    last_git_commit::LastGitCommit
};

fn path(file: &str) -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(file)
}

fn rerun() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=data/*");
    println!("cargo:rerun-if-changed=data/**");
}

fn version() {

    let lgc = LastGitCommit::new().set_path(".").build().unwrap();
    let cargo_version = env!("CARGO_PKG_VERSION");

    std::fs::write(path("version.txt"), format!("{}-{}", cargo_version, lgc.id().short()));

}

fn compile_resources() {

    const COMMAND: &'static str = "glib-compile-resources";
    const INPUT: &'static str = "data/net.olback.GnomeTwitch2.gresource.xml";
    let output = path("gnome-twitch.gresource");

    let exists = Command::new("which").arg(COMMAND).output().unwrap();
    if !exists.status.success() {
        panic!(format!("Command '{}' not found", COMMAND));
    }

    let resources = Command::new(COMMAND)
    .args(&[INPUT, &format!("--target={}", output.to_str().unwrap())])
    .output()
    .unwrap();

    if !resources.status.success() {
        panic!(format!("Failed to generate resources: {}", String::from_utf8_lossy(&resources.stderr)))
    }

}

fn main() {
    rerun();
    version();
    compile_resources();
}
