/// bootstrap
/// 1. clone rust
/// 2. git submodule update
pub fn bootstrap(config: &crate::config::Config) -> anyhow::Result<()> {
    let rust_repo = config.global.rust_repo();

    if !rust_repo.exists() {
        log::info!("cloning rust into {:?}", rust_repo);
        cmd_lib::run_cmd!(
            git clone "https://github.com/rust-lang/rust.git" $rust_repo;
        )?;
    } else {
        log::info!("skip cloning {:?} already exists", rust_repo);
    }
    cmd_lib::run_cmd!(
        cd $rust_repo;
        git submodule update --init --recursive;
    )?;

    Ok(())
}
