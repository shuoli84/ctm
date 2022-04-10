use crate::config::{Config, CrateOpt, Profile, ToolchainConfig};

#[derive(Debug)]
pub struct RunResult {
    pub results: Vec<OneRunResult>,
}

#[derive(Debug)]
pub struct OneRunResult {
    /// crate name
    pub krate: String,

    /// toolchain name
    pub toolchain: String,

    /// profile name
    pub profile: String,

    /// cmd name
    pub cmd: String,

    /// how large the binary is, useful for report
    pub binary_size: u64,

    /// how long it taks for one cmd run
    pub duration_ms: u64,
}

/// run command with each toolchain and krate by inject PATH
pub fn run_cmds(
    profile_to_run: Option<&str>,
    config: &crate::config::Config,
    krate: &crate::config::CrateOpt,
) -> anyhow::Result<RunResult> {
    let mut run_results = vec![];

    for toolchain in &config.toolchains {
        for profile in toolchain.profiles.iter() {
            let should_run = profile_to_run
                .map(|p| p.eq(profile.as_str()))
                .unwrap_or(true);
            if !should_run {
                continue;
            }

            let profile = config
                .profiles
                .iter()
                .filter(|p| p.name.eq(profile))
                .nth(0)
                .unwrap();

            let mut results = run_cmd_step(krate, toolchain, profile, config)?;
            run_results.append(&mut results);
        }
    }

    Ok(RunResult {
        results: run_results,
    })
}

fn run_cmd_step(
    krate: &CrateOpt,
    toolchain: &ToolchainConfig,
    profile: &Profile,
    config: &Config,
) -> anyhow::Result<Vec<OneRunResult>> {
    let target_folder = crate::utils::target_folder(krate, profile, toolchain, config)
        .into_os_string()
        .to_str()
        .unwrap()
        .to_owned();
    let program = std::path::PathBuf::from(&target_folder).join(&krate.output_path);
    log::info!("running program: {:?}", program);

    let file_size = {
        let meta = std::fs::metadata(&program)?;
        meta.len()
    };

    let mut run_results = vec![];

    for run in krate.runs.iter() {
        for _i in 0..run.count {
            let start = std::time::Instant::now();
            let output = std::process::Command::new(program.clone())
                .args(run.args.as_slice())
                .output()?;

            let duration_ms = std::time::Instant::now().duration_since(start).as_millis() as u64;

            run_results.push(OneRunResult {
                krate: krate.name.clone(),
                toolchain: toolchain.name.clone(),
                profile: profile.name.clone(),
                cmd: run.name.clone(),
                binary_size: file_size,
                duration_ms,
            });

            log::info!("program done with status {:?}", output.status);
        }
    }
    Ok(run_results)
}
