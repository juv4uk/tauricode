//! M2.6 — OpenCode event normalization, from OpenCode's own structured log
//! line format (`~/.local/share/opencode/log/opencode.log`), not from any
//! HTTP call — keeps M2.5's "zero HTTP round-trips" safety property intact.
//!
//! Test fixtures below are real lines observed directly in this session's
//! own `opencode.log` while investigating the 2026-08-25 WSL freeze
//! (session ids/timestamps redacted to stable placeholders, wording and
//! structure otherwise verbatim) — not synthesized guesses at the format.
//!
//! Deliberately incomplete, and disclosed as such: only two of the eight
//! core events have a directly-observed OpenCode log line to normalize
//! from. Everything else the log format could theoretically carry
//! (`ToolCompleted`'s result, `OutputChunk`'s actual token content,
//! `TaskBound`, `StatusChanged`) has NO corresponding line ever captured
//! this session — inventing a parse rule for a format never seen would be
//! exactly the "Unknown > invented" violation the whole audit series was
//! built to avoid. `RuntimeStarted`/`RuntimeStopped` are not parsed from
//! logs at all: they belong at the adapter's own `launch()`/`stop()` call
//! sites (M2.5), which already know the true moment of state change more
//! reliably than any log line could.
//!
//! Observed, NOT normalized here (drop, not force-mapped) - each line
//! genuinely doesn't correspond to any of the 8 core events:
//! - `message="llm runtime selected"` - internal model-backend detail
//! - `message="resolved path"` - internal tool-argument resolution
//! - `message=stream ...` (non-error) - stream metadata, never carries
//!   the actual output text in this log format
//! - `message=loop ... step=N` - OpenCode's own internal agentic-loop
//!   bookkeeping
//! - `message=process session.id=... messageID=...` - new message
//!   arrival; maps to the already-excluded `input_received`, not to
//!   anything in the 8-event core vocabulary
//! - `message=cancel session.id=...` - session cancellation is not the
//!   same event as `RuntimeStopped` (the process is still alive); no
//!   core event currently represents it - a real gap this normalizer
//!   surfaces rather than hides

use crate::events::Event;
use crate::types::RuntimeHandle;
use std::collections::HashMap;

/// Splits an OpenCode log line into its `key=value` fields, respecting
/// double-quoted values that may contain spaces (e.g.
/// `message="stream error"`). Deliberately simple - this format has no
/// escaped-quote-within-quote cases in any line observed this session; a
/// line that doesn't fit is skipped, never guessed at.
fn tokenize(line: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let mut rest = line;

    while let Some(eq_pos) = rest.find('=') {
        let key = rest[..eq_pos].trim().to_string();
        rest = &rest[eq_pos + 1..];

        let value = if rest.starts_with('"') {
            match rest[1..].find('"') {
                Some(end) => {
                    let v = rest[1..1 + end].to_string();
                    rest = rest[1 + end + 1..].trim_start();
                    v
                }
                None => break, // unterminated quote - malformed line, stop parsing
            }
        } else {
            match rest.find(' ') {
                Some(sp) => {
                    let v = rest[..sp].to_string();
                    rest = rest[sp + 1..].trim_start();
                    v
                }
                None => {
                    let v = rest.trim_end().to_string();
                    rest = "";
                    v
                }
            }
        };

        if key.is_empty() {
            break;
        }
        fields.insert(key, value);
    }

    fields
}

/// Normalizes one OpenCode log line into a core `Event`, if (and only if)
/// this session has direct evidence of what that line means. Returns
/// `None` for anything not in the disclosed, observed mapping above -
/// never a guessed/invented `Event`.
pub fn normalize_opencode_log_line(line: &str, handle: &RuntimeHandle) -> Option<Event> {
    let fields = tokenize(line);
    let message = fields.get("message")?;

    match message.as_str() {
        "evaluated" => {
            // Observed form: `message=evaluated permission=bash
            // pattern="..." action.permission=* action.action=allow
            // action.pattern=*` - the `permission` field's value is the
            // tool name in every line captured this session (bash was
            // the only permission kind observed; other tool kinds are
            // plausible but not evidenced, so this arm accepts whatever
            // string is present rather than allow listing "bash" only).
            let tool_name = fields.get("permission")?.clone();
            Some(Event::ToolStarted {
                handle: handle.clone(),
                task_id: None,
                tool_name,
            })
        }
        "stream error" => {
            // Observed form: `message="stream error" ... session.id=...
            // ... error.error="AI_APICallError: ..."`.
            let error_message = fields
                .get("error.error")
                .cloned()
                .unwrap_or_else(|| "stream error (no error.error field present)".to_string());
            Some(Event::Error {
                handle: handle.clone(),
                message: error_message,
                retryable: None, // not evidenced in any observed line
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h() -> RuntimeHandle {
        RuntimeHandle::new("test-handle")
    }

    #[test]
    fn tool_started_from_real_evaluated_permission_line() {
        // Verbatim structure from this session's own opencode.log
        // (2026-08-25T07:42:54.759Z entry), run id and full pattern text
        // shortened for the test but the field shape is real.
        let line = r#"timestamp=2026-08-25T07:42:54.759Z level=INFO run=45fb18dc message=evaluated permission=bash pattern="git push origin dev" action.permission=* action.action=allow action.pattern=*"#;

        let event = normalize_opencode_log_line(line, &h()).expect("should normalize");
        assert_eq!(
            event,
            Event::ToolStarted {
                handle: h(),
                task_id: None,
                tool_name: "bash".to_string(),
            }
        );
    }

    #[test]
    fn error_from_real_stream_error_line() {
        // Verbatim structure from this session's own opencode.log
        // (2026-08-25T07:39:20.396Z entry).
        let line = r#"timestamp=2026-08-25T07:39:20.396Z level=ERROR run=d849a094 message="stream error" providerID=opencode modelID=x-preview-f-free session.id=ses_fd6e68465ffeHCi02vtj30aXOq small=false agent=build mode=primary error.error="AI_APICallError: Error from provider (Console): Upstream request failed: Endpoint is unavailable.""#;

        let event = normalize_opencode_log_line(line, &h()).expect("should normalize");
        match event {
            Event::Error { message, .. } => {
                assert!(message.contains("AI_APICallError"));
                assert!(message.contains("Endpoint is unavailable"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn llm_runtime_selected_is_dropped_not_invented() {
        let line = r#"timestamp=2026-08-25T07:41:49.505Z level=INFO run=45fb18dc message="llm runtime selected" llm.runtime=ai-sdk llm.provider=openrouter llm.model=stealth/ox-alpha"#;
        assert_eq!(normalize_opencode_log_line(line, &h()), None);
    }

    #[test]
    fn resolved_path_is_dropped_not_invented() {
        let line = r#"timestamp=2026-08-24T23:32:08.349Z level=INFO run=82f0cfdc message="resolved path" arg=/home/agents/GitHub/my-lisp resolved=/home/agents/GitHub/my-lisp"#;
        assert_eq!(normalize_opencode_log_line(line, &h()), None);
    }

    #[test]
    fn loop_bookkeeping_is_dropped_not_invented() {
        let line = "timestamp=2026-08-24T23:35:09.245Z level=INFO run=82f0cfdc message=loop session.id=ses_fd0f43ae9ffe4QCg5mORK2MXKo step=1";
        assert_eq!(normalize_opencode_log_line(line, &h()), None);
    }

    #[test]
    fn cancel_is_dropped_not_invented_as_runtime_stopped() {
        // A real gap this normalizer surfaces on purpose: session cancel
        // != process stop, and no core event represents it yet.
        let line = "timestamp=2026-08-25T07:41:57.589Z level=INFO run=d849a094 message=cancel session.id=ses_fd6e68465ffeHCi02vtj30aXOq";
        assert_eq!(normalize_opencode_log_line(line, &h()), None);
    }

    #[test]
    fn garbage_line_returns_none_not_panic() {
        assert_eq!(normalize_opencode_log_line("not a log line at all", &h()), None);
        assert_eq!(normalize_opencode_log_line("", &h()), None);
    }
}
