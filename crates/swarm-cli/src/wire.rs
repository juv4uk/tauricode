//! Wire client for the swarm/1 newline-sexpr protocol: one request, one
//! balanced response. Deliberately thin — the protocol's authority is
//! swarm-node itself; this is a transport adapter for Level 0 agents.

use crate::sexpr::{self, Sexp};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

pub struct Client {
    addr: String,
    pub timeout: Duration,
}

impl Client {
    pub fn new(addr: String) -> Client {
        Client { addr, timeout: Duration::from_secs(20) }
    }

    /// Sends one op and returns the parsed response form. Responses are
    /// single balanced s-expressions; the server keeps the connection open,
    /// so we read until balance, then close (one-shot pattern from the
    /// swarm docs' debugging notes).
    pub fn call(&self, op_form: &str) -> Result<Sexp, String> {
        let mut stream = std::net::TcpStream::connect(&self.addr)
            .map_err(|e| format!("cannot reach swarm node at {}: {e}", self.addr))?;
        stream.set_read_timeout(Some(self.timeout)).map_err(|e| e.to_string())?;
        stream.write_all(format!("{op_form}\n").as_bytes()).map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        let mut line = String::new();
        // The server answers ops with exactly one line-framed sexpr; empty
        // lines are skipped on both sides by convention.
        loop {
            line.clear();
            let n = reader.read_line(&mut line).map_err(|e| format!("read error: {e}"))?;
            if n == 0 {
                return Err("connection closed without a response".to_string());
            }
            if line.trim().is_empty() {
                continue;
            }
            break;
        }
        let _ = stream.flush();
        let parsed = sexpr::parse(line.trim())
            .map_err(|e| format!("unparseable response from {addr}: {e}", addr = self.addr))?;
        Ok(parsed)
    }
}

/// Extracts `(key value...)` fields out of a response form.
pub fn field<'a>(resp: &'a Sexp, key: &str) -> Option<&'a Sexp> {
    resp.field(key)?.first()
}

/// Renders any response subtree back to canonical text (for passthrough
/// values an L0 agent may want verbatim).
pub fn text(v: &Sexp) -> String {
    v.to_text()
}
