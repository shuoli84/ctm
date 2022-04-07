use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GlobalConfig {
    /// the project root
    #[serde(default)]
    pub project_root: String,

    /// rust repo path, all patches and build will be applied to it.
    rust_repo: String,

    /// rust repo rev the patches based. Each toolchain can override it
    pub rust_rev: String,

    /// where the toolchain should be installed
    toolchains_root: String,

    /// where to find patch
    patch_root: String,

    /// where to put all built crates
    build_root: String,
}

impl GlobalConfig {
    /// rust repo path, all patches and build will be applied to it.
    pub fn rust_repo(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.project_root).join(&self.rust_repo)
    }

    /// where the toolchain should be installed
    pub fn toolchains_root(&self) -> std::path::PathBuf {
        self.to_absolute_path(self.toolchains_root.as_str())
    }

    /// where to find patch
    pub fn patches_root(&self) -> std::path::PathBuf {
        self.to_absolute_path(self.patch_root.as_str())
    }

    /// where to put all built crates
    pub fn build_root(&self) -> std::path::PathBuf {
        self.to_absolute_path(self.build_root.as_str())
    }

    fn to_absolute_path(&self, path: &str) -> std::path::PathBuf {
        let path = std::path::PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            std::path::PathBuf::from(&self.project_root).join(&path)
        }
    }
}

/// toolchain build config
#[derive(Deserialize, Debug)]
pub struct ToolchainConfig {
    /// toolchain name, used when rustup link
    pub name: String,

    /// the patch file name, if not provided, then no patch will be applied
    #[serde(default)]
    pub patches: Vec<String>,

    /// which rev this patch based on, if provided,
    /// will overwrite global setting
    #[serde(default)]
    pub rust_rev: Option<String>,

    /// which profiles this toolchain uses to build crate
    /// defined in profile section
    #[serde(default)]
    pub profiles: Vec<String>,
}

/// Crate opt loaded from config file
/// defines how to build one crate and where is the final
/// output
#[derive(Deserialize, Debug)]
pub struct CrateOpt {
    /// crate name
    pub name: String,

    /// git url to clone
    #[serde(default)]
    pub git: Option<String>,

    /// local path to project root
    #[serde(default)]
    pub path: Option<String>,

    /// if provided, override default build cmd
    #[serde(default)]
    pub build_cmd: Option<String>,

    /// output artifact path relative to target folder
    pub output_path: String,

    /// runs, all runs defined for this crate, they can be executed
    /// for each toolchain + profile permutation, and will collect
    /// run duration etc to indicate toolchain + profile perf difference
    #[serde(default)]
    pub runs: Vec<Run>,
}

#[derive(Deserialize, Debug)]
pub struct Profile {
    /// the name for profile, should be unique
    pub name: String,

    /// the config.toml content
    pub environ: std::collections::HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Run {
    /// run name, can be used as param to just run this one
    pub name: String,

    /// how many times to run
    pub count: u64,

    /// args passed to program
    pub args: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    /// global config
    pub global: GlobalConfig,

    /// toolchains
    pub toolchains: Vec<ToolchainConfig>,

    /// profiles
    pub profiles: Vec<Profile>,

    /// crates
    pub crates: Vec<CrateOpt>,
}

/// load crate from config file
pub fn load_from_file(file: &str) -> anyhow::Result<Config> {
    let file = std::path::PathBuf::from(file).canonicalize().unwrap();
    let parent = file.parent().unwrap();
    let content = std::fs::read_to_string(&file)?;
    let mut config: Config = toml::from_str(content.as_str())?;
    if config.global.project_root.is_empty() {
        config.global.project_root = parent.to_str().unwrap().to_string();
    }

    Ok(config)
}
