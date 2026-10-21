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

/// Sets up DCR registries by loading the registry manager and printing the list of registries.
pub fn setup(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help") {
        let mut help = crate::utils::help::HelpBuilder::new("dcr setup");
        help.section("USAGE").example("dcr setup");
        help.description("Show configured package registries and their priorities.");
        help.render();
        return 0;
    }

    // Load and process the registries, returning 0 on success or 1 on error
    match crate::core::registry::RegistryManager::load() {
        Ok(manager) => {
            let mut lines = vec![format!(
                "Loaded {} registries (configured)",
                manager.config.registry.len()
            )];
            for (name, reg) in manager.config.registry {
                lines.push(format!("  {name}: {} (priority {})", reg.url, reg.priority));
            }
            crate::say!("{}", lines.join("\n"));
            0
        }
        Err(e) => {
            crate::utils::output::error!("{e}");
            1
        }
    }
}
