pub struct ToolChainOpts {
    pub name: String,
    pub patches: Vec<String>,
    pub patch_folder: std::path::PathBuf,
    /// prefix folder is the folder to install all toolchains
    pub toolchains_root: std::path::PathBuf,
}

pub fn build_toolchain(
    rust_repo: &std::path::Path,
    base_rev: &str,
    toolchain: &ToolChainOpts,
) -> anyhow::Result<()> {
    log::info!("build toolchain for {}", toolchain.name);

    let name = toolchain.name.clone();
    let toolchain_folder = toolchain.toolchains_root.join(&name);

    if !toolchain_folder.join("bin/rustc").exists() {
        // reset rust repo to base rev
        log::debug!("reset repo {:?} to {}", rust_repo, base_rev);
        cmd_lib::run_cmd! (
            cd $rust_repo;
            git reset --hard $base_rev;
        )?;

        log::debug!("checkout master");
        cmd_lib::run_cmd! (
            cd $rust_repo;
            git checkout master;
        )?;

        for patch_name in toolchain.patches.iter() {
            let patch_file = toolchain.patch_folder.join(patch_name);
            log::debug!("apply patch {:?}", patch_file);
            cmd_lib::run_cmd! (
                cd $rust_repo;
                git apply $patch_file;
            )?;
        }

        let config_toml = rust_repo.join("config.toml");
        // setup config.toml
        std::fs::write(
            config_toml,
            format!(
                r#"
# Includes one of the default files in src/bootstrap/defaults
profile = "user"
changelog-seen = 2

[install]
prefix = "{}"
sysconfdir = "etc"

[build]
docs = false
extended = true
tools = ["cargo", "src"]

[rust]
description = "{}"
"#,
                toolchain_folder.to_str().unwrap().to_string(),
                toolchain.name
            ),
        )?;

        cmd_lib::run_cmd!(
            cd $rust_repo;
            python x.py install
        )?;
    } else {
        // todo: maybe add more checks? Now we strongly depends on
        // name uniqueness
        log::info!("toolchain already build");
    }

    cmd_lib::run_cmd!(
        rustup toolchain link $name $toolchain_folder;
    )?;

    Ok(())
}
