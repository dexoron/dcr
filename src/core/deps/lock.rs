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

use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DepLock {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub source: String,
}

pub fn write_lock(
    project_root: &Path,
    project_name: &str,
    project_version: &str,
    packages: &[DepLock],
) -> Result<(), String> {
    let mut out = String::new();
    out.push_str("[[package]]\n");
    out.push_str(&format!("name = \"{}\"\n", escape_value(project_name)));
    out.push_str(&format!(
        "version = \"{}\"\n",
        escape_value(project_version)
    ));
    if !packages.is_empty() {
        out.push_str(&format!(
            "dependencies = [{}]\n",
            quote_list(&packages.iter().map(|p| p.name.clone()).collect::<Vec<_>>())
        ));
    }
    out.push('\n');

    for pkg in packages {
        out.push_str("[[package]]\n");
        out.push_str(&format!("name = \"{}\"\n", escape_value(&pkg.name)));
        out.push_str(&format!("version = \"{}\"\n", escape_value(&pkg.version)));
        out.push_str(&format!("source = \"{}\"\n", escape_value(&pkg.source)));
        out.push_str(&format!("checksum = \"{}\"\n", escape_value(&pkg.checksum)));
        out.push('\n');
    }
    fs::write(project_root.join("dcr.lock"), out)
        .map_err(|err| format!("Failed to write dcr.lock: {err}"))?;
    Ok(())
}

fn quote_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("\"{}\"", escape_value(s)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_value(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}
