// DCR — Cargo-like C/C++ project manager.
//
// Copyright (C) 2026 Dexoron (Bezotechestvo Vladimir) <main@dexoron.su>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub url: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub registry: HashMap<String, Registry>,
}

pub struct RegistryManager {
    pub config: Config,
    #[allow(dead_code)]
    pub path: PathBuf,
}

fn home_dir() -> Option<PathBuf> {
    crate::utils::fs::home_dir()
}

impl RegistryManager {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let dcr_dir = home_dir()
            .ok_or("Cannot determine home directory")?
            .join(".dcr");
        let config_path = dcr_dir.join("config.toml");

        if !config_path.exists() {
            return Err("Config file ~/.dcr/config.toml not found".into());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(Self {
            config,
            path: dcr_dir,
        })
    }
}
