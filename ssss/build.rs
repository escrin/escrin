fn main() {
    if std::env::var_os("ABI_DIR").is_none() {
        println!(
            "cargo:rustc-env=ABI_DIR={}",
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../evm/abi")
                .display()
        );
    }
}
