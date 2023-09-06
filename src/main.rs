use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

use config::Config;
use fs::open_file;
use os::CURRENT_OS;

use crate::fs::{copy_file, is_file};

mod config;
mod fs;
mod os;

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
    /// Show and resolve diffs between source and target files
    Diff(LabelsArgs),
    /// Edit the confmg config file
    EditConfig,
    /// Edit a source file
    EditSource(LabelArgs),
    /// Edit a target file if it exists
    EditTarget(LabelArgs),
    /// Overwrite existing config files with source files
    ApplySource(LabelsArgs),
    /// Overwrite source files with existing config files
    ApplyTarget(LabelsArgs),
    /// List available configs for this platform
    List,
    /// Print information
    Info,
}

#[derive(Args)]
struct LabelArgs {
    /// Label of the config
    label: String,
}

#[derive(Args)]
struct LabelsArgs {
    /// Labels of the configs or none to run on all
    labels: Vec<String>,
}

fn main() -> Result<()> {
    // parse CLI
    let cli = Cli::parse();

    // info command
    if let Commands::Info = cli.command {
        println!("Current OS: {}", *CURRENT_OS);
        println!("Confmg Config File: {}", cli.config_file.display());
        return Ok(());
    }

    // parse config file
    let config = Config::read_file(&cli.config_file)?;
    let source_base = cli
        .config_file
        .parent()
        .ok_or(anyhow!("Failed to get config file directory."))?
        .to_path_buf();

    if let Commands::List = cli.command {
        for (label, entry) in config.get_entries() {
            if entry.get_current_target_path().is_some() {
                println!("{}", label);
            }
        }
    } else if let Commands::Diff(labels_args)
    | Commands::ApplySource(labels_args)
    | Commands::ApplyTarget(labels_args) = &cli.command
    {
        let labels: Vec<&String> = if labels_args.labels.len() > 0 {
            labels_args.labels.iter().collect()
        } else {
            config.get_labels().collect()
        };
        for label in labels {
            if let Some(entry) = config.get_entry(&label) {
                match &cli.command {
                    Commands::Diff(_) => {
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
                    Commands::ApplySource(_) | Commands::ApplyTarget(_) => {
                        if let Some(target_path) = entry.get_current_target_path() {
                            if is_file(&target_path)? {
                                let source_path = entry.get_source_path(&source_base);
                                let (from_path, to_path) = match &cli.command {
                                    Commands::ApplySource(_) => (source_path, target_path),
                                    Commands::ApplyTarget(_) => (target_path, source_path),
                                    _ => unreachable!(),
                                };
                                println!(
                                    "Copying from '{}' to '{}'",
                                    from_path.display(),
                                    to_path.display()
                                );
                                copy_file(from_path, to_path)?;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        return Ok(());
    } else if let Commands::EditConfig | Commands::EditSource(_) | Commands::EditTarget(_) =
        &cli.command
    {
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
