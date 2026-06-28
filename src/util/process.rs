//! Helpers for bounding wall-clock time spent on subprocess invocations.
//!
//! Every external `Command::new(...).output()` / `.status()` in this codebase
//! is a potential indefinite block if the child hangs (e.g. nix-daemon, NFS
//! mount, scripts that wait on `fuser`). Use the helpers in this module
//! instead of calling `.output()` / `.status()` directly so that runaway
//! subprocesses get killed after a bounded timeout.

use std::process::{Command, ExitStatus, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Run `cmd` and capture its stdout/stderr, killing it if it does not
/// finish within `timeout`. Returns the captured `Output` on success or
/// an `Err` containing a short diagnostic message on spawn, wait, or
/// timeout failure. On timeout the child is sent `SIGKILL`.
///
/// Stdout and stderr are forced to `Stdio::piped()` so they can be
/// captured into the returned `Output`; callers do not need to set
/// `Stdio::piped()` themselves. Use [`run_with_timeout_status`] for
/// commands that should stream their output to the parent (inherit).
pub fn run_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<Output, String> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(timeout) {
        Ok(result) => result.map_err(|e| format!("wait: {e}")),
        Err(_) => {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            Err(format!("timeout after {timeout:?}"))
        }
    }
}

/// Like [`run_with_timeout`] but waits only for the exit status. Use this
/// when the command has its stdio set to `Stdio::inherit()` (live
/// streaming) or when the caller does not need to read stdout/stderr.
pub fn run_with_timeout_status(
    cmd: &mut Command,
    timeout: Duration,
) -> Result<ExitStatus, String> {
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {e}"))?;
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = child.wait();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(timeout) {
        Ok(result) => result.map_err(|e| format!("wait: {e}")),
        Err(_) => {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            Err(format!("timeout after {timeout:?}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    #[test]
    fn run_with_timeout_returns_output_for_fast_command() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("echo hello");
        let out = run_with_timeout(&mut cmd, Duration::from_secs(2)).expect("fast cmd ok");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello");
    }

    #[test]
    fn run_with_timeout_kills_hanging_command() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("sleep 30");
        let result = run_with_timeout(&mut cmd, Duration::from_millis(200));
        assert!(result.is_err(), "expected timeout error, got {result:?}");
        let msg = result.unwrap_err();
        assert!(msg.contains("timeout"), "unexpected error message: {msg}");
    }

    #[test]
    fn run_with_timeout_status_returns_status_for_fast_command() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("exit 0");
        let status =
            run_with_timeout_status(&mut cmd, Duration::from_secs(2)).expect("fast cmd ok");
        assert!(status.success());
    }

    #[test]
    fn run_with_timeout_status_kills_hanging_command() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("sleep 30");
        let result = run_with_timeout_status(&mut cmd, Duration::from_millis(200));
        assert!(result.is_err(), "expected timeout error, got {result:?}");
    }

    #[test]
    fn run_with_timeout_propagates_nonzero_exit() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("echo bad >&2; exit 7");
        let out = run_with_timeout(&mut cmd, Duration::from_secs(2)).expect("cmd ran");
        assert_eq!(out.status.code(), Some(7));
        assert!(String::from_utf8_lossy(&out.stderr).contains("bad"));
    }

    #[test]
    fn run_with_timeout_handles_missing_program() {
        let mut cmd = Command::new("definitely-not-a-real-binary-xyz");
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let result = run_with_timeout(&mut cmd, Duration::from_secs(2));
        assert!(result.is_err(), "missing program should error");
        assert!(result.unwrap_err().contains("spawn"));
    }

    #[test]
    fn run_with_timeout_survives_orphan_reaper_thread() {
        // After a timeout the helper kills the child but the wait thread is
        // still running. Ensure the helper returns quickly and the next call
        // to the helper works fine (i.e. the orphan thread does not poison
        // global state).
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("sleep 30");
        let _ = run_with_timeout(&mut cmd, Duration::from_millis(100));

        let mut follow_up = Command::new("sh");
        follow_up.arg("-c").arg("echo ok");
        let out = run_with_timeout(&mut follow_up, Duration::from_secs(2))
            .expect("follow-up cmd ok");
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ok");
    }
}