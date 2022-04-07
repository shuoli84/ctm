use crate::config::{Config, CrateOpt, Profile, ToolchainConfig};

// get target folder for tuple (crate, toolchain)
pub fn target_folder(
    k: &CrateOpt,
    profile: &Profile,
    toolchain: &ToolchainConfig,
    config: &Config,
) -> std::path::PathBuf {
    let crate_folder = config.global.build_root().join(&k.name);
    crate_folder.join(format!("target/{}_{}", toolchain.name, profile.name))
}
