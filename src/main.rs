use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

use config::Config;
use os::CURRENT_OS;
use utils::open_file;

mod config;
mod os;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Confmg config file path
    #[arg(short, long, default_value = "~/.confmg/confmg.json")]
    config_file: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Show and resolve diffs between confmg and local config
    Diff,
    /// Edit the config file
    EditConfig,
    /// Edit a source file
    EditSource(LabelArgs),
    /// Edit a target file if it exists
    EditTarget(LabelArgs),
    /// List available configs
    List,
    /// Print information
    Info,
}

#[derive(Args)]
struct LabelArgs {
    /// Label of the config
    label: String,
}

fn main() -> Result<()> {
    // parse CLI
    let cli = Cli::parse();

    // info command
    if let Commands::Info = cli.command {
        println!("Current OS: {}", *CURRENT_OS);
        return Ok(());
    }

    // parse config file
    let config = Config::read_file(&cli.config_file)?;
    let source_base = cli
        .config_file
        .parent()
        .ok_or(anyhow!("Failed to get config file directory."))?
        .to_path_buf();

    // list command
    if let Commands::List = cli.command {
        for label in config.get_labels() {
            println!("{}", label);
        }
        return Ok(());
    }

    // diff command
    if let Commands::Diff = cli.command {
        for (label, entry) in config.get_entries() {
            if let Some(diff) = entry.get_diff(&source_base) {
                println!("Diff for '{}':", label);
                match diff {
                    Ok(diff) => {
                        println!("{}", diff);
                    }
                    Err(err) => {
                        eprintln!("{}\n", err);
                    }
                }
            }
        }
        return Ok(());
    }

    // edit commands
    if let Commands::EditConfig | Commands::EditSource(_) | Commands::EditTarget(_) = &cli.command {
        let editor = std::env::var_os("EDITOR").ok_or(anyhow!(
            "Default editor could not be found. Please set the EDITOR environment variable."
        ))?;
        let path = match &cli.command {
            Commands::EditConfig => cli.config_file,
            Commands::EditSource(label_args) | Commands::EditTarget(label_args) => {
                let entry = config
                    .get_entry(&label_args.label)
                    .ok_or(anyhow!("No config with label '{}' found", label_args.label))?;
                match &cli.command {
                    Commands::EditSource(_) => entry.get_source_path(&source_base),
                    Commands::EditTarget(_) => entry.get_current_target_path().ok_or(anyhow!(
                        "There is no current target file for config '{}'",
                        label_args.label
                    ))?,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        };
        println!("Opening file at '{}'", &path.display());
        open_file(editor, &path)?;
    }

    Ok(())
}
