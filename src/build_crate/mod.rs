use crate::utils;

#[derive(Debug)]
pub struct Artifact {
    pub toolchain: String,
    pub profile: String,
    pub crate_name: String,
    pub output_path: std::path::PathBuf,
}

pub fn build_crate_for_all_profile(
    krate: &crate::config::CrateOpt,
    config: &crate::config::Config,
) -> anyhow::Result<Vec<Artifact>> {
    let mut artifacts = vec![];
    // build the crate for each toolchain and profile
    for toolchain in config.toolchains.iter() {
        for profile in toolchain.profiles.iter() {
            let profile = config
                .profiles
                .iter()
                .filter(|p| p.name.eq(profile))
                .nth(0)
                .unwrap();

            let artifact = build_crate_step(krate, toolchain, profile, config)?;
            artifacts.push(artifact);
        }
    }

    Ok(artifacts)
}

pub fn build_crate_step(
    krate: &crate::config::CrateOpt,
    toolchain: &crate::config::ToolchainConfig,
    profile: &crate::config::Profile,
    config: &crate::config::Config,
) -> anyhow::Result<Artifact> {
    let name = krate.name.clone();
    let git = krate.git.clone();
    let path = krate.path.clone();
    let build_root = config.global.build_root().clone();

    cmd_lib::run_cmd!(
        mkdir -p $build_root;
    )?;

    let folder_name = name;
    let folder_path = std::path::PathBuf::from(&build_root).join(&folder_name);

    if !std::path::PathBuf::from(&folder_path).exists() {
        if let Some(git) = git {
            cmd_lib::run_cmd!(
                cd $build_root;
                git clone $git $folder_name;
            )?;
        } else if let Some(path) = path {
            cmd_lib::run_cmd!(
                cd $build_root;
                cp -R $path $folder_name;
            )?;
        } else {
            anyhow::bail!("Neither git nor path provided");
        }
    }

    // create target foldr for each toolchain, to make build artifact
    // more easier
    let target_folder = utils::target_folder(krate, profile, toolchain, config);

    {
        let target_folder = target_folder.to_str().unwrap().to_string();
        let toolchain_name = toolchain.name.clone();

        let mut environs = profile.environ.clone();
        environs.insert("CARGO_TARGET_DIR".to_string(), target_folder);
        environs.insert("RUSTUP_TOOLCHAIN".to_string(), toolchain_name);

        let build_cmd = krate
            .build_cmd
            .as_ref()
            .cloned()
            .unwrap_or_else(|| "cargo build --release".to_string());

        let output = std::process::Command::new("bash")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .envs(environs)
            .current_dir(folder_path)
            .args(&["-c", &build_cmd])
            .output()?;
        if !output.status.success() {
            anyhow::bail!("failed to build with output: {output:?}");
        }
    }

    let output_path = target_folder.join(krate.output_path.clone());

    // strip the binary
    {
        std::process::Command::new("strip")
            .args(&[output_path.as_os_str()])
            .output()?;
    }

    Ok(Artifact {
        toolchain: toolchain.name.clone(),
        profile: profile.name.clone(),
        crate_name: krate.name.clone(),
        output_path,
    })
}
