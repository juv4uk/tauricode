//! Self-reported layer (provenance: self-reported) for Slice 2. Reads the
//! Agent Identity Contract v1 — arbitrary JSON files written by external
//! agent runtimes, consumed read-only here. This crate never writes this
//! format; it only reads it. See the coordinator's Slice 2 design notes
//! for the full contract (format, lifecycle, staleness, PID-reuse guard).
//!
//! Nothing here is ecosystem-wide policy yet — this is Tauricode's own
//! consumer-side reading of a format it hopes other agent runtimes will
//! someday write, not a ratified cross-repo contract.
//!
//! **Known, deliberately-unfixed technical debt (adversarial review,
//! 2026-08-20):** this module never checks who wrote a `*.json` file —
//! not file ownership, not directory permissions. `identity_contract::default_base_dir`'s
//! fallback (`/tmp/ecosystem-agents-<uid>`) is created by whatever writes
//! the first file into it, typically under the process's default umask,
//! which on a machine where multiple agents run under the same Unix
//! *group* (not just the same user — the common case in this
//! environment) can leave it group-writable. A different UID in that
//! same group could write a file claiming any PID/identity it likes.
//! This does not let anyone fake OS-observed facts (those come only from
//! this crate's own `/proc` reads) or gain any capability — self-reported
//! data is informational-only by design (see `snapshot::IdentityStatus`'s
//! doc comment) — but it does mean a `Fresh`/`Stale` status is *not* proof
//! that the named model/role/task is genuine, only that some file's PID
//! claim currently lines up with a real process. Not fixed here;
//! producer-side hardening (e.g. the directory being created `0700`) is
//! future scope.

use crate::process_observe::OsProcess;
use crate::snapshot::{IdentityStatus, SelfReportedIdentity};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// How stale `updated_at` may be before a correlated-but-quiet record is
/// downgraded from `Fresh` to `Stale`.
const STALE_THRESHOLD: Duration = Duration::from_secs(60);

/// How far apart the self-reported `started_at` and the OS-observed start
/// time may be before they're considered a mismatch — used only as a
/// fallback when `process_start_token` is absent (e.g. a non-Linux agent).
/// Generous on purpose: clock skew and formatting-precision differences
/// between an agent's own clock and this crate's derivation from
/// `/proc/stat` are expected; a real PID-reuse gap is almost always much
/// larger than this.
const STARTED_AT_TOLERANCE: Duration = Duration::from_secs(5);

/// Raw shape of an Agent Identity Contract v1 file — mirrors the JSON
/// schema exactly, nothing renamed or reinterpreted at this layer.
#[derive(Debug, Deserialize)]
struct RawRecord {
    #[allow(dead_code)]
    schema_version: u32,
    pid: u32,
    process_start_token: Option<String>,
    started_at: Option<String>,
    updated_at: Option<String>,
    model: Option<String>,
    role: Option<String>,
    repository: Option<String>,
    instance: Option<String>,
    task: Option<String>,
    declared_capabilities: Option<Vec<String>>,
}

/// `$XDG_RUNTIME_DIR/ecosystem-agents`, falling back to
/// `/tmp/ecosystem-agents-<uid>` when `XDG_RUNTIME_DIR` isn't set.
/// Deliberately outside any git repository — see the design notes for why
/// (no `.gitignore` changes needed across every repo in the ecosystem, no
/// risk of an identity file ever being accidentally committed).
pub fn default_base_dir() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime_dir.is_empty() {
            return PathBuf::from(runtime_dir).join("ecosystem-agents");
        }
    }
    let uid = current_uid().unwrap_or(0);
    PathBuf::from(format!("/tmp/ecosystem-agents-{uid}"))
}

fn current_uid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|l| l.starts_with("Uid:"))?;
    line.split_whitespace().nth(1)?.parse().ok()
}

/// Every `*.json` file in `base_dir`, keyed by the `pid` field *inside*
/// each file's content — not by filename. A file's own claimed PID is
/// what gets correlated against the real OS process; trusting the
/// filename instead would let a stale/misnamed file silently attach to
/// the wrong process. Missing directory or unparseable files are not
/// errors: they simply contribute nothing (an agent that never started,
/// or wrote malformed JSON, is indistinguishable from one that never
/// reported at all).
fn read_all(base_dir: &Path) -> HashMap<u32, RawRecord> {
    let mut result = HashMap::new();
    let Ok(entries) = std::fs::read_dir(base_dir) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(record) = serde_json::from_str::<RawRecord>(&text) {
            result.insert(record.pid, record);
        }
    }
    result
}

/// Every PID that has a self-reported record in `base_dir` right now —
/// used one layer up, in `discover`, purely to decide *relevance* (should
/// this OS process be included in `local_processes` at all), before any
/// correlation/staleness logic runs.
pub fn known_pids(base_dir: &Path) -> HashSet<u32> {
    read_all(base_dir).into_keys().collect()
}

/// Correlates one OS-observed process against the self-reported records
/// found in `base_dir`, returning the status and (when trustworthy) the
/// identity. `now` is a parameter, not `SystemTime::now()` internally, so
/// staleness is deterministically testable without waiting on a clock.
pub fn correlate(
    process: &OsProcess,
    base_dir: &Path,
    now: SystemTime,
) -> (IdentityStatus, SelfReportedIdentity) {
    let records = read_all(base_dir);
    correlate_with(process, &records, now)
}

fn correlate_with(
    process: &OsProcess,
    records: &HashMap<u32, RawRecord>,
    now: SystemTime,
) -> (IdentityStatus, SelfReportedIdentity) {
    let Some(record) = records.get(&process.pid) else {
        return (IdentityStatus::NotFound, SelfReportedIdentity::default());
    };

    let matches = match (&record.process_start_token, &process.start_token) {
        (Some(claimed), Some(actual)) => claimed == actual,
        _ => {
            started_at_within_tolerance(record.started_at.as_deref(), process.started_at.as_deref())
        }
    };
    if !matches {
        return (IdentityStatus::Orphaned, SelfReportedIdentity::default());
    }

    let identity = SelfReportedIdentity {
        model: record.model.clone(),
        role: record.role.clone(),
        repository_identity: record.repository.clone(),
        instance: record.instance.clone(),
        task: record.task.clone(),
        declared_capabilities: record.declared_capabilities.clone(),
    };

    let status = match record.updated_at.as_deref().and_then(parse_iso8601_seconds) {
        Some(updated_at_secs) => {
            let updated_at =
                SystemTime::UNIX_EPOCH + Duration::from_secs(updated_at_secs.max(0) as u64);
            match now.duration_since(updated_at) {
                Ok(age) if age > STALE_THRESHOLD => IdentityStatus::Stale,
                Ok(_) => IdentityStatus::Fresh,
                Err(_) => IdentityStatus::Fresh, // updated_at in the future — clock skew, not staleness
            }
        }
        // no parseable updated_at at all: correlated by PID/token, but we
        // can't judge freshness — treat as Stale, not Fresh, since "we
        // can't tell" must never look identical to "confirmed recent".
        None => IdentityStatus::Stale,
    };
    (status, identity)
}

fn started_at_within_tolerance(claimed: Option<&str>, observed: Option<&str>) -> bool {
    match (
        claimed.and_then(parse_iso8601_seconds),
        observed.and_then(parse_iso8601_seconds),
    ) {
        (Some(c), Some(o)) => (c - o).unsigned_abs() <= STARTED_AT_TOLERANCE.as_secs(),
        // if either side has no parseable timestamp at all, there is
        // nothing to correlate on — fail closed (Orphaned), not open.
        _ => false,
    }
}

/// Minimal `YYYY-MM-DDTHH:MM:SS(.mmm)?Z` parser — only what this crate's
/// own `time_util::iso8601_now`/`iso8601_from_unix_seconds` ever produce,
/// not a general ISO-8601 parser. Deliberately strict: a self-reported
/// timestamp in an unexpected shape is treated as absent, never guessed.
fn parse_iso8601_seconds(s: &str) -> Option<i64> {
    let s = s.strip_suffix('Z')?;
    let (date, time) = s.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;
    let time = time.split('.').next()?; // drop optional .mmm
    let mut time_parts = time.split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: i64 = time_parts.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60 + second)
}

/// Inverse of `time_util`'s `civil_from_days` (Howard Hinnant's algorithm,
/// public domain) — days since the Unix epoch for a given
/// proleptic-Gregorian date.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let mp = ((m + 9) % 12) as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os_process(pid: u32, start_token: Option<&str>, started_at: Option<&str>) -> OsProcess {
        OsProcess {
            pid,
            command: "test".to_string(),
            cwd: None,
            started_at: started_at.map(|s| s.to_string()),
            start_token: start_token.map(|s| s.to_string()),
        }
    }

    fn record(
        pid: u32,
        token: Option<&str>,
        started_at: Option<&str>,
        updated_at: Option<&str>,
    ) -> RawRecord {
        RawRecord {
            schema_version: 1,
            pid,
            process_start_token: token.map(|s| s.to_string()),
            started_at: started_at.map(|s| s.to_string()),
            updated_at: updated_at.map(|s| s.to_string()),
            model: Some("Claude Sonnet 5".to_string()),
            role: Some("Ecosystem Lead".to_string()),
            repository: Some("tauricode".to_string()),
            instance: None,
            task: Some("testing".to_string()),
            declared_capabilities: Some(vec!["bash".to_string()]),
        }
    }

    #[test]
    fn iso8601_seconds_round_trips_through_this_crates_own_formatter() {
        for secs in [0i64, 1_787_173_325, -1, 1_000_000_000] {
            let formatted = crate::time_util::iso8601_from_unix_seconds(secs);
            assert_eq!(parse_iso8601_seconds(&formatted), Some(secs), "{formatted}");
        }
    }

    #[test]
    fn not_found_when_no_record_for_pid() {
        let process = os_process(100, Some("123"), None);
        let records = HashMap::new();
        let (status, identity) = correlate_with(&process, &records, SystemTime::now());
        assert_eq!(status, IdentityStatus::NotFound);
        assert_eq!(identity, SelfReportedIdentity::default());
    }

    #[test]
    fn fresh_when_token_matches_and_recently_updated() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000_000);
        let updated_at = crate::time_util::iso8601_from_unix_seconds(1_999_999_990); // 10s ago
        let process = os_process(100, Some("42"), None);
        let mut records = HashMap::new();
        records.insert(100, record(100, Some("42"), None, Some(&updated_at)));
        let (status, identity) = correlate_with(&process, &records, now);
        assert_eq!(status, IdentityStatus::Fresh);
        assert_eq!(identity.model.as_deref(), Some("Claude Sonnet 5"));
    }

    #[test]
    fn stale_when_token_matches_but_updated_at_too_old() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000_000);
        let updated_at = crate::time_util::iso8601_from_unix_seconds(1_999_999_000); // 1000s ago
        let process = os_process(100, Some("42"), None);
        let mut records = HashMap::new();
        records.insert(100, record(100, Some("42"), None, Some(&updated_at)));
        let (status, identity) = correlate_with(&process, &records, now);
        assert_eq!(status, IdentityStatus::Stale);
        // facts already reported are still preserved during Stale, same
        // "don't discard what was read" principle as Slice 1's GitState.
        assert_eq!(identity.model.as_deref(), Some("Claude Sonnet 5"));
    }

    #[test]
    fn orphaned_when_start_token_mismatches_pid_reuse() {
        let process = os_process(100, Some("999-different"), None);
        let mut records = HashMap::new();
        records.insert(
            100,
            record(100, Some("42"), None, Some("2026-01-01T00:00:00.000Z")),
        );
        let (status, identity) = correlate_with(&process, &records, SystemTime::now());
        assert_eq!(status, IdentityStatus::Orphaned);
        assert_eq!(identity, SelfReportedIdentity::default());
    }

    #[test]
    fn fallback_started_at_tolerance_accepts_small_clock_skew_without_token() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(2_000_000_000);
        let observed_started = crate::time_util::iso8601_from_unix_seconds(1_000_000_000);
        let claimed_started = crate::time_util::iso8601_from_unix_seconds(1_000_000_003); // 3s skew
        let updated_at = crate::time_util::iso8601_from_unix_seconds(1_999_999_990);
        let process = os_process(100, None, Some(&observed_started));
        let mut records = HashMap::new();
        records.insert(
            100,
            record(100, None, Some(&claimed_started), Some(&updated_at)),
        );
        let (status, _) = correlate_with(&process, &records, now);
        assert_eq!(status, IdentityStatus::Fresh);
    }

    #[test]
    fn fallback_started_at_beyond_tolerance_is_orphaned() {
        let observed_started = crate::time_util::iso8601_from_unix_seconds(1_000_000_000);
        let claimed_started = crate::time_util::iso8601_from_unix_seconds(1_000_000_500); // 500s off — real PID reuse
        let process = os_process(100, None, Some(&observed_started));
        let mut records = HashMap::new();
        records.insert(
            100,
            record(
                100,
                None,
                Some(&claimed_started),
                Some("2026-01-01T00:00:00.000Z"),
            ),
        );
        let (status, identity) = correlate_with(&process, &records, SystemTime::now());
        assert_eq!(status, IdentityStatus::Orphaned);
        assert_eq!(identity, SelfReportedIdentity::default());
    }

    #[test]
    fn missing_updated_at_is_stale_not_fresh_when_otherwise_correlated() {
        let process = os_process(100, Some("42"), None);
        let mut records = HashMap::new();
        records.insert(100, record(100, Some("42"), None, None));
        let (status, _) = correlate_with(&process, &records, SystemTime::now());
        assert_eq!(
            status,
            IdentityStatus::Stale,
            "unknown freshness must never look like confirmed-fresh"
        );
    }
}
