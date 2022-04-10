pub struct ToolChainOpts {
    pub name: String,
    pub rust_rev: String,
    pub patches: Vec<String>,
    pub patch_folder: std::path::PathBuf,
    /// prefix folder is the folder to install all toolchains
    pub toolchains_root: std::path::PathBuf,
    /// whether force to build
    pub force: bool,
}

pub fn build_toolchain(
    rust_repo: &std::path::Path,
    toolchain: &ToolChainOpts,
) -> anyhow::Result<()> {
    log::info!("build toolchain for {}", toolchain.name);

    let base_rev = &toolchain.rust_rev;
    let name = toolchain.name.clone();
    let toolchain_folder = toolchain.toolchains_root.join(&name);

    if !toolchain_folder.join("bin/rustc").exists() || toolchain.force {
        // reset rust repo to base rev
        log::debug!("reset repo {:?} to {}", rust_repo, base_rev);
        cmd_lib::run_cmd! (
            cd $rust_repo;
            git reset --hard $base_rev;
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
prefix = "{prefix}"
sysconfdir = "etc"

[build]
docs = false
extended = true
tools = ["cargo", "src"]

[rust]
description = "{toolchain_name}+{base_rev}"
"#,
                prefix = toolchain_folder.to_str().unwrap().to_string(),
                toolchain_name = toolchain.name,
                base_rev = base_rev,
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
