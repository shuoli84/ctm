use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)]
#[clap(name = "git")]
#[clap(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// create an example config.toml in current folder
    Init {
        #[clap(help = "if omitted, emit config.toml in current folder")]
        folder: Option<String>,
    },

    /// bootstrap helps checkout rust to repo folder
    /// and some setup steps
    Bootstrap {
        #[clap(long = "config", default_value = "config.toml")]
        config: String,
    },

    BuildToolchain {
        #[clap(long = "config", default_value = "config.toml")]
        config: String,
    },

    #[clap(arg_required_else_help = true)]
    BuildCrate {
        #[clap(long = "config", default_value = "config.toml")]
        config: String,

        #[clap(long = "crate", help = "if provided, only build this crate")]
        krate: Option<String>,

        #[clap(long = "output", help = "output path", default_value = "-")]
        output: String,
    },

    /// run the command in number of times, before run, will inject krate target path into PATH for
    /// each tool chain
    #[clap(arg_required_else_help = true, trailing_var_arg = true)]
    Run {
        #[clap(long = "config", default_value = "config.toml")]
        config: String,

        #[clap(long = "crate")]
        krate: String,

        #[clap(long = "output", help = "output path", default_value = "-")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Init { folder } => {
            let folder = folder
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap());
            init::init_project(folder)?;
        }
        Commands::Bootstrap { config } => {
            let config = config::load_from_file(config.as_str())?;
            bootstrap::bootstrap(&config)?;
        }

        Commands::BuildToolchain { config } => {
            let config = config::load_from_file(config.as_str())?;

            let global_option = config.global;
            let rust_repo = global_option.rust_repo();
            let rust_rev = global_option.rust_rev.clone();
            let toolchains = config.toolchains;

            for toolchain in toolchains.into_iter() {
                let toolchain = build_toolchain::ToolChainOpts {
                    name: toolchain.name,
                    patches: toolchain.patches,
                    patch_folder: global_option.patches_root(),
                    toolchains_root: global_option.toolchains_root(),
                };
                build_toolchain::build_toolchain(&rust_repo, &rust_rev, &toolchain)?;
            }
        }

        Commands::BuildCrate {
            config,
            krate,
            output,
        } => {
            let config = config::load_from_file(config.as_str())?;

            let mut crates_iter: Box<dyn Iterator<Item = _>> = Box::new(config.crates.iter());
            if let Some(name) = krate.as_ref() {
                crates_iter = Box::new(crates_iter.filter(|k| k.name.eq(name)));
            }

            let mut rows = vec![];
            for krate in crates_iter {
                log::info!("building {}", krate.name);
                let artifacts = build_crate::build_crate_for_all_profile(krate, &config)?;
                rows.append(&mut report::report_artifacts(&artifacts));
            }
            write_json_to_output(rows, output)?;
        }

        Commands::Run {
            config,
            krate,
            output,
        } => {
            let config = config::load_from_file(config.as_str())?;
            let krate = config
                .crates
                .iter()
                .filter(|k| k.name.eq(&krate))
                .nth(0)
                .ok_or(anyhow::anyhow!("Not able to find crate"))?;
            let run_result = run::run_cmds(&config, &krate)?;
            let rows = report::report_run_results(run_result);
            write_json_to_output(rows, output)?;
        }
    }

    Ok(())
}

mod bootstrap;
mod build_crate;
mod build_toolchain;
mod config;
mod init;
mod report;
mod run;
mod utils;

/// write json to output file
fn write_json_to_output(x: impl serde::Serialize, output: String) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(&x).unwrap();
    if output.eq("-") {
        println!("{}", content);
    } else {
        std::fs::write(output, content)?;
    }
    Ok(())
}
