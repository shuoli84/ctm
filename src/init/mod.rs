/// create an init config file in folder
pub fn init_project(folder: std::path::PathBuf) -> anyhow::Result<std::path::PathBuf> {
    log::info!("initing project at {:?}", folder);

    let config_path = folder.join("config.toml");
    if config_path.exists() {
        anyhow::bail!("config file {config_path:?} already exists");
    }
    std::fs::write(config_path, include_str!("init_config.toml"))?;

    Ok(folder)
}
