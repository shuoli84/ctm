use crate::config::{Config, CrateOpt, Profile, ToolchainConfig};
use std::{ffi::OsString, str::FromStr};
use which::which_in;

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
    config: &crate::config::Config,
    krate: &crate::config::CrateOpt,
) -> anyhow::Result<RunResult> {
    let mut run_results = vec![];

    for toolchain in &config.toolchains {
        for profile in toolchain.profiles.iter() {
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
    let output_path = std::path::PathBuf::from(&target_folder).join(&krate.output_path);
    let output_folder = output_path.parent().unwrap().to_str().unwrap().to_owned();
    let program = output_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let path_env = std::env::var("PATH").unwrap_or_default();
    let modified_path_env = format!("{output_folder}:{path_env}");
    let cwd = std::env::current_dir()?;

    let expanded_program = which_in(
        OsString::from_str(program.as_str())?,
        Some(modified_path_env),
        cwd.clone(),
    )?;
    log::info!("expanded program: {:?}", expanded_program);

    let file_size = {
        let meta = std::fs::metadata(&expanded_program)?;
        meta.len()
    };

    let mut run_results = vec![];

    for run in krate.runs.iter() {
        for _i in 0..run.count {
            let start = std::time::Instant::now();
            let output = std::process::Command::new(expanded_program.clone())
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
