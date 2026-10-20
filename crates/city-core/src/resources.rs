//! Conservative admission estimates, separate from measured visualizer budgets.
use crate::*;
pub fn local_budget_bytes() -> u64 {
    #[cfg(target_os = "macos")]
    {
        let total = std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(16 * 1024 * 1024 * 1024);
        // Estimate reclaimable capacity conservatively. memory_pressure's percentage is not available RAM.
        let vm = std::process::Command::new("/usr/bin/vm_stat")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();
        let page_size = vm
            .lines()
            .next()
            .and_then(|line| line.split("page size of ").nth(1))
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(16384);
        let pages = |key: &str| {
            vm.lines()
                .find_map(|line| line.strip_prefix(key))
                .and_then(|s| s.trim().trim_end_matches('.').parse::<u64>().ok())
                .unwrap_or(0)
        };
        let reclaimable =
            pages("Pages free:") + pages("Pages purgeable:") + pages("File-backed pages:") / 2;
        reclaimable
            .saturating_mul(page_size)
            .saturating_sub(2 * 1024 * 1024 * 1024)
            .min(total / 2)
    }
    #[cfg(not(target_os = "macos"))]
    {
        8 * 1024 * 1024 * 1024
    }
}
pub fn estimated_bytes(profile: &Profile) -> u64 {
    if profile.provider == "ollama" {
        profile.size.saturating_mul(3) / 2 + 2 * 1024 * 1024 * 1024
    } else {
        0
    }
}
pub fn choose<'a>(
    profiles: &'a [Profile],
    world: &World,
    preferred: Option<&String>,
    budget: u64,
) -> Option<&'a Profile> {
    let active: std::collections::HashSet<&str> = world
        .runs
        .iter()
        .filter(|r| r.ended_at.is_none())
        .map(|r| r.model.as_str())
        .collect();
    let occupied = profiles
        .iter()
        .filter(|p| active.contains(p.model.as_str()) && !p.loaded)
        .map(estimated_bytes)
        .sum::<u64>();
    let eligible = |p: &&Profile| {
        p.available
            && !world.runs.iter().any(|r| {
                r.model == p.model
                    && r.failure_code.as_deref() == Some("provider_incompatible")
                    && r.ended_at
                        .as_ref()
                        .and_then(|at| chrono::DateTime::parse_from_rfc3339(at).ok())
                        .is_some_and(|at| {
                            (chrono::Utc::now() - at.with_timezone(&chrono::Utc)).num_minutes() < 10
                        })
            })
            && p.tools
            && (p.provider != "ollama" || p.context_length >= 16384)
            && (p.provider == "ollama"
                || (world.settings.policy == "auto"
                    && preferred.is_some_and(|m| m.starts_with("codex:"))))
            && (p.provider != "ollama"
                || occupied.saturating_add(if p.loaded {
                    1024 * 1024 * 1024
                } else {
                    estimated_bytes(p)
                }) <= budget)
    };
    preferred
        .and_then(|name| profiles.iter().filter(eligible).find(|p| p.model == *name))
        .or_else(|| {
            profiles
                .iter()
                .filter(eligible)
                .filter(|p| p.provider == "ollama")
                .min_by_key(|p| p.size)
        })
}
