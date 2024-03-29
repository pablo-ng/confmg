use std::{
    collections::{hash_map, HashMap},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use diffy::{create_patch, PatchFormatter};
use serde::{Deserialize, Serialize};

use crate::{
    fs::{_write_file, expand_tilde, is_file, read_file},
    os::{CURRENT_OS, OS},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(rename = "diffCommand")]
    pub diff_command: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config: Configuration,
    configs: HashMap<String, ConfigEntry>,
}

impl Config {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content_str = read_file(path)?;
        Ok(serde_json::from_str::<Config>(&content_str).context("Failed to parse config file")?)
    }

    pub fn _write_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content_str = serde_json::to_string_pretty(self).context("Failed to parse config")?;
        Ok(_write_file(path, content_str)?)
    }

    pub fn get_entries(&self) -> hash_map::Iter<'_, String, ConfigEntry> {
        self.configs.iter()
    }

    pub fn get_entry(&self, label: &String) -> Option<&ConfigEntry> {
        self.configs.get(label)
    }

    pub fn _get_labels(&self) -> hash_map::Keys<'_, String, ConfigEntry> {
        self.configs.keys()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigEntry {
    source: PathBuf,
    targets: HashMap<OS, PathBuf>,
}

impl ConfigEntry {
    pub fn get_source_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        let mut path = PathBuf::new().join(base);
        path.push(&self.source);
        path
    }

    pub fn get_source_content<P: AsRef<Path>>(&self, base: P) -> Result<String> {
        let source_path = self.get_source_path(base);
        read_file(&source_path)
    }

    pub fn get_current_target_path(&self) -> Option<PathBuf> {
        self.targets.get(&CURRENT_OS).cloned()
    }

    pub fn get_current_target_content(&self) -> Option<Result<String>> {
        let current_target_path = self.get_current_target_path()?;
        match is_file(&current_target_path) {
            Ok(is_file) => {
                if is_file {
                    Some(read_file(current_target_path))
                } else {
                    None
                }
            }
            Err(err) => return Some(Err(err)),
        }
    }

    pub fn get_diff<P: AsRef<Path>>(
        &self,
        source_base: P,
        diff_command: &Option<String>,
    ) -> Option<Result<String>> {
        if let Some(diff_command) = diff_command {
            Command::new(diff_command)
                .arg(expand_tilde(self.get_current_target_path()?).ok()?)
                .arg(expand_tilde(self.get_source_path(source_base)).ok()?)
                .status()
                .unwrap();
            None
        } else {
            // get file contents
            let current_target_content = match self.get_current_target_content()? {
                Ok(current_target_content) => current_target_content,
                Err(err) => return Some(Err(err)),
            };
            let source_content = match self.get_source_content(source_base) {
                Ok(source_content) => source_content,
                Err(err) => return Some(Err(err)),
            };

            // get diff
            let patch = create_patch(&source_content, &current_target_content);
            if patch.hunks().len() > 0 {
                Some(Ok(PatchFormatter::new()
                    .with_color()
                    .fmt_patch(&patch)
                    .to_string()))
            } else {
                None
            }
        }
    }
}
