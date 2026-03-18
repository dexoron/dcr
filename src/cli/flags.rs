use crate::config::{PROFILE, flags};
use crate::utils::log::warn;

pub struct BuildRunFlags {
    pub profile: String,
    pub force: bool,
    pub clean: bool,
}

pub fn parse_build_run_flags(args: &[String]) -> Result<BuildRunFlags, i32> {
    let mut profile = PROFILE.to_string();
    let mut force = false;
    let mut clean = false;

    for arg in args {
        if !arg.starts_with("--") {
            warn("Unknown argument");
            return Err(1);
        }
        let candidate = arg.trim_start_matches("--");
        if candidate == "force" {
            force = true;
            continue;
        }
        if candidate == "clean" {
            clean = true;
            continue;
        }
        if flags(candidate).is_some() {
            if profile != PROFILE {
                warn("Duplicate profile flag");
                return Err(1);
            }
            profile = candidate.to_string();
            continue;
        }
        warn("Unknown build flag");
        return Err(1);
    }

    Ok(BuildRunFlags {
        profile,
        force,
        clean,
    })
}
