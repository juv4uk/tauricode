//! OS-observed layer (provenance: OS-observed) for Slice 2. Reads `/proc`
//! only — no signals sent to any process, nothing written anywhere.
//!
//! Linux-only by construction (`/proc` is a Linux-specific interface).
//! On any other platform every function here degrades to returning empty/
//! `None`, per the crate's existing "unknown stays visible as unknown, not
//! silently defaulted to a false fact" principle — it is not an error to
//! run this crate on a non-Linux host, it just observes nothing.
//!
//! **Known, deliberately-unfixed technical debt (adversarial review,
//! 2026-08-20): vanished-during-scan race.** `list_processes` first lists
//! `/proc`'s PID directories, then reads each one's `cmdline`/`cwd`/`stat`
//! separately; a process that exits in between shows up with an empty
//! `command` and `cwd`/`start_token`/`started_at` all `None` instead of
//! being excluded from the result entirely. This never panics (every read
//! here degrades via `.ok()`), and if that PID also has a self-reported
//! record the correlation in `identity_contract` safely resolves to
//! `Orphaned` rather than a false `Fresh` — but the resulting
//! `OsObservedFacts { command: "", .. }` can read as "we observed a real
//! process with no command" rather than "this process vanished mid-scan".
//! Not fixed here; a future slice could special-case an all-fields-empty
//! read as its own state instead of a degraded-but-present one.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// One process as observed directly from `/proc`, before any correlation
/// with a self-reported identity record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsProcess {
    pub pid: u32,
    pub command: String,
    pub cwd: Option<String>,
    /// Human-readable wall-clock start time, derived from `/proc/<pid>/stat`'s
    /// `starttime` (clock ticks since boot) plus `/proc/stat`'s `btime`
    /// (boot time, seconds since epoch). Display-only.
    pub started_at: Option<String>,
    /// The raw `starttime` tick count as a string — the same value a
    /// self-reporting process would read from its own `/proc/self/stat`.
    /// This, not `started_at`, is what `identity_contract` uses to guard
    /// against PID reuse: two processes can never share both the same PID
    /// *and* the same `starttime` at once, and a process's own `starttime`
    /// never changes during its lifetime.
    pub start_token: Option<String>,
}

/// Every process in `/proc` right now. Cheap enough for a single pass on
/// a typical development machine (a few hundred entries, a handful of
/// small file reads each) — this crate does not attempt to filter here;
/// filtering to what's actually relevant (repo cwd match or a
/// self-reported record) happens one layer up, in `discover`, once it has
/// both this list and the current scan's repository paths to compare
/// against.
pub fn list_processes() -> Vec<OsProcess> {
    let clk_tck = clock_ticks_per_second();
    let btime = boot_time_epoch_seconds();

    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return result;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue; // not a PID directory (e.g. /proc/self, /proc/cpuinfo)
        };
        let command = read_command(pid);
        let cwd = read_cwd(pid);
        let start_token = read_start_token(pid);
        let started_at = match (start_token.as_deref(), clk_tck, btime) {
            (Some(token), Some(clk_tck), Some(btime)) => token
                .parse::<u64>()
                .ok()
                .map(|ticks| btime + ticks / clk_tck)
                .and_then(unix_seconds_to_iso8601),
            _ => None,
        };
        result.push(OsProcess {
            pid,
            command,
            cwd,
            started_at,
            start_token,
        });
    }
    result
}

/// `starttime` (field 22 of `/proc/<pid>/stat`) as a plain decimal string.
/// Parsed past the *last* `)` in the line, since the `comm` field (2, in
/// parens) can itself contain spaces or parens — splitting on whitespace
/// naively would misalign every field after it.
pub fn read_start_token(pid: u32) -> Option<String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_comm = stat.rsplit_once(')')?.1;
    after_comm.split_whitespace().nth(19).map(|s| s.to_string())
}

fn read_command(pid: u32) -> String {
    if let Ok(cmdline) = fs::read_to_string(format!("/proc/{pid}/cmdline")) {
        let joined = cmdline
            .split('\0')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !joined.is_empty() {
            return joined;
        }
    }
    // Kernel threads and some short-lived processes have empty cmdline;
    // comm (the short process name) is the best remaining fact.
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn read_cwd(pid: u32) -> Option<String> {
    fs::read_link(format!("/proc/{pid}/cwd"))
        .ok()
        .map(|p: PathBuf| p.display().to_string())
}

/// `getconf CLK_TCK` — the number of `starttime` ticks per second. Shelled
/// out rather than hardcoded: 100 is true on effectively every real Linux
/// system, but `CLK_TCK` is a kernel-configurable constant, not a
/// language-level guarantee, and this value is cheap to ask for directly
/// instead of assuming it.
fn clock_ticks_per_second() -> Option<u64> {
    std::process::Command::new("getconf")
        .arg("CLK_TCK")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .or(Some(100)) // near-universal default, used only if getconf itself is unavailable
}

fn boot_time_epoch_seconds() -> Option<u64> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    stat.lines()
        .find_map(|line| line.strip_prefix("btime "))
        .and_then(|s| s.trim().parse().ok())
}

fn unix_seconds_to_iso8601(seconds: u64) -> Option<String> {
    let duration = SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(seconds))?
        .duration_since(UNIX_EPOCH)
        .ok()?;
    Some(crate::time_util::iso8601_from_unix_seconds(
        duration.as_secs() as i64,
    ))
}
