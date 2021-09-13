use std::path::{Path, PathBuf};

use cargo_toml::Manifest;

pub(crate) fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

pub(crate) fn get_version() -> String {
    let root = project_root();
    let cli_toml = root
        .join("crates")
        .join("github_scbot_cli")
        .join("Cargo.toml");
    let manifest = Manifest::from_path(cli_toml).expect("Cargo.toml should exist");
    manifest.package.expect("should have package").version
}
