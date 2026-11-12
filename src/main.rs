use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "configer", version, about = "Store named config paths")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Add or update a config
    Add {
        /// Name of the config
        #[arg(short, long)]
        name: String,

        /// Path to the config (absolute or relative)
        #[arg(short, long)]
        path: PathBuf,
    },

    /// Print the path of a stored config
    Show {
        /// Name of the config
        name: String,
    },

    /// Copy the path of a stored config to the clipboard
    Copy {
        /// Name of the config
        name: String,
    },
}

fn config_file() -> Result<PathBuf> {
    let dir = dirs::config_dir().context("could not determine the user config directory")?;
    Ok(dir.join("configer").join("configs.json"))
}

fn load(path: &Path) -> Result<BTreeMap<String, PathBuf>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    if contents.trim().is_empty() {
        return Ok(BTreeMap::new());
    }

    serde_json::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

fn save(path: &Path, configs: &BTreeMap<String, PathBuf>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let contents = serde_json::to_string_pretty(configs)?;
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    std::path::absolute(path).with_context(|| format!("failed to resolve path {}", path.display()))
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = config_file()?;
    let mut configs = load(&file)?;

    match args.command {
        Command::Add { name, path } => {
            if name.trim().is_empty() {
                bail!("config name cannot be empty");
            }

            let resolved = absolute_path(&path)?;
            configs.insert(name.clone(), resolved.clone());
            save(&file, &configs)?;
            println!("Added {name} -> {}", resolved.display());
        }
        Command::Show { name } => {
            let stored = configs
                .get(&name)
                .with_context(|| format!("no config named {name:?}"))?;
            println!("{}", stored.display());
        }
        Command::Copy { name } => {
            let stored = configs
                .get(&name)
                .with_context(|| format!("no config named {name:?}"))?;
            let mut clipboard =
                arboard::Clipboard::new().context("failed to access the clipboard")?;
            clipboard
                .set_text(stored.to_string_lossy().into_owned())
                .context("failed to write to the clipboard")?;
            println!("Copied {}", stored.display());
        }
    }

    Ok(())
}
