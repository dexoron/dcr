use crate::core::config::Config;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub deps: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub members: Vec<WorkspaceMember>,
}

pub fn parse_workspace(config: &Config, root: &Path) -> Result<Option<Workspace>, String> {
    let table = match config.get("workspace").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return Ok(None),
    };

    let mut members = Vec::new();
    for (name, value) in table {
        let tbl = value
            .as_table()
            .ok_or_else(|| format!("workspace.{name} must be a table with path and deps"))?;
        let path_raw = tbl
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("workspace.{name}.path is required"))?;
        let path = resolve_path(root, path_raw)?;
        if !path.join("dcr.toml").is_file() {
            return Err(format!(
                "workspace.{name}.path does not contain dcr.toml: {}",
                path.display()
            ));
        }
        let deps = parse_deps(tbl.get("deps"))?;
        members.push(WorkspaceMember {
            name: name.to_string(),
            path,
            deps,
        });
    }

    let mut ws = Workspace { members };
    ws.members = topo_sort(&ws.members)?;
    Ok(Some(ws))
}

fn resolve_path(root: &Path, raw: &str) -> Result<PathBuf, String> {
    let p = Path::new(raw);
    let full = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    Ok(full)
}

fn parse_deps(value: Option<&Value>) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let arr = value
        .as_array()
        .ok_or_else(|| "workspace deps must be an array of strings".to_string())?;
    let mut out = Vec::new();
    for item in arr {
        let s = item
            .as_str()
            .ok_or_else(|| "workspace deps must be an array of strings".to_string())?;
        out.push(s.to_string());
    }
    Ok(out)
}

fn topo_sort(members: &[WorkspaceMember]) -> Result<Vec<WorkspaceMember>, String> {
    let mut map = HashMap::new();
    for m in members {
        map.insert(m.name.clone(), m.clone());
    }
    let mut state: HashMap<String, u8> = HashMap::new();
    let mut order: Vec<WorkspaceMember> = Vec::new();

    for name in map.keys() {
        if *state.get(name).unwrap_or(&0) == 0 {
            visit(name, &map, &mut state, &mut order)?;
        }
    }
    Ok(order)
}

fn visit(
    name: &str,
    map: &HashMap<String, WorkspaceMember>,
    state: &mut HashMap<String, u8>,
    order: &mut Vec<WorkspaceMember>,
) -> Result<(), String> {
    match state.get(name).copied().unwrap_or(0) {
        1 => return Err(format!("workspace dependency cycle at {name}")),
        2 => return Ok(()),
        _ => {}
    }
    state.insert(name.to_string(), 1);
    let member = map
        .get(name)
        .ok_or_else(|| format!("unknown workspace member {name}"))?;
    let mut seen = HashSet::new();
    for dep in &member.deps {
        if !seen.insert(dep) {
            continue;
        }
        if !map.contains_key(dep) {
            return Err(format!("workspace dependency '{dep}' not found"));
        }
        visit(dep, map, state, order)?;
    }
    state.insert(name.to_string(), 2);
    order.push(member.clone());
    Ok(())
}
