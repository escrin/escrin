use std::process::Command;

fn main() -> eyre::Result<()> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    let repo = out_dir.join("merkle-tree");
    if !repo.is_dir() {
        eyre::ensure!(
            Command::new("git")
                .args([
                    "clone",
                    "--depth=1",
                    "https://github.com/openzeppelin/merkle-tree"
                ])
                .current_dir(&out_dir)
                .output()?
                .status
                .success(),
            "clone failed"
        );
    }

    if !repo.join("node_modules").is_dir() {
        eyre::ensure!(
            Command::new("npm")
                .args(["ci"])
                .current_dir(&repo)
                .output()?
                .status
                .success(),
            "npm install failed"
        );
    }

    let bundle_file = out_dir.join("merkle-tree.js");

    if !bundle_file.is_file() {
        eyre::ensure!(
            Command::new("npm")
                .args([
                    "exec",
                    "--",
                    "esbuild",
                    "dist/index.js",
                    "--bundle",
                    "--platform=node",
                    "--target=node16",
                    &format!("--outfile={}", bundle_file.display()),
                ])
                .current_dir(&repo)
                .output()?
                .status
                .success(),
            "bundling failed"
        );
    }

    println!(
        "cargo:rustc-env=OZ_MERKLE_TREE_JS={}",
        bundle_file.display()
    );

    Ok(())
}
