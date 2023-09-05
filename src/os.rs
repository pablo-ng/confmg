use std::fmt;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Linux,
    Windows,
    MacOS,
}

lazy_static! {
    pub static ref CURRENT_OS: OS = {
        match std::env::consts::OS {
            "linux" => OS::Linux,
            "macos" => OS::MacOS,
            "windows" => OS::Windows,
            _ => {
                eprintln!("Current OS '{}' is not supported.", std::env::consts::OS);
                std::process::exit(1);
            }
        }
    };
}

impl fmt::Display for OS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
