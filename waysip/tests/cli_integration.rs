//! Integration tests that exercise the compiled `waysip` binary directly.
//!
//! These cover clap's `--help`/`--version`/`--completions` early-exit paths
//! (which never touch Wayland), the clap usage-error exit path for
//! conflicting flags, and the "no compositor available" error path --
//! verifying the binary fails cleanly instead of panicking when it can't
//! connect to Wayland. None of this requires a running compositor, so it's
//! safe in headless CI. `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` are stripped
//! explicitly so the tests behave the same on a machine that does have a
//! compositor running.

use std::io::Write;
use std::process::{Command, Stdio};

fn waysip_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_waysip"));
    cmd.env_remove("WAYLAND_DISPLAY");
    cmd.env_remove("WAYLAND_SOCKET");
    cmd
}

#[test]
fn help_flag_exits_successfully_without_a_compositor() {
    let output = waysip_cmd()
        .arg("--help")
        .output()
        .expect("failed to run waysip binary");
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn version_flag_prints_version_and_exits_successfully() {
    let output = waysip_cmd()
        .arg("--version")
        .output()
        .expect("failed to run waysip binary");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("waysip"));
}

#[cfg(feature = "completions")]
#[test]
fn completions_flag_prints_a_script_without_touching_wayland() {
    let output = waysip_cmd()
        .args(["--completions", "bash"])
        .output()
        .expect("failed to run waysip binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("waysip"));
}

#[test]
fn conflicting_flags_exit_with_a_usage_error_before_touching_wayland() {
    let output = waysip_cmd()
        .args(["-p", "-d"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
}

#[test]
fn without_a_compositor_it_fails_cleanly_instead_of_panicking() {
    let output = waysip_cmd()
        .arg("-p")
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

// ─── Bad CLI-value error paths ────────────────────────────────────────────
//
// These fail during argument validation in `run_selection`, before
// `WaySip::get()` ever tries to connect to Wayland, so they're safe to run
// without a compositor even though they use a selection-mode flag.

#[test]
fn invalid_background_color_fails_cleanly_before_touching_wayland() {
    let output = waysip_cmd()
        .args(["-p", "-b", "not-a-color"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn invalid_border_weight_fails_cleanly_before_touching_wayland() {
    let output = waysip_cmd()
        .args(["-p", "-w", "not-a-number"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn invalid_aspect_ratio_format_fails_cleanly_before_touching_wayland() {
    // `-a` conflicts with `-p`, so this needs a mode compatible with it.
    // Three ':'-separated parts hits the "wrong part count" branch.
    let output = waysip_cmd()
        .args(["-d", "-a", "not:a:ratio"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn invalid_aspect_ratio_width_fails_cleanly_before_touching_wayland() {
    // Two parts, but the width half isn't a number.
    let output = waysip_cmd()
        .args(["-d", "-a", "bad:9"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[test]
fn invalid_aspect_ratio_height_fails_cleanly_before_touching_wayland() {
    // Two parts, but the height half isn't a number.
    let output = waysip_cmd()
        .args(["-d", "-a", "16:bad"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

// ─── Flags that reach further into `run_selection` before failing ────────
//
// These pass argument validation cleanly and reach `edit_selection`/
// `bench`/`freeze` handling (and, for `--freeze`, its own no-compositor
// fallback inside `capture_backgrounds`) before failing at the same
// `WaySip::get()` connection step as the plain no-compositor case.

#[test]
fn edit_selection_flag_still_fails_cleanly_without_a_compositor() {
    // `-e` conflicts with `-p`, so this needs a mode compatible with it.
    let output = waysip_cmd()
        .args(["-d", "-e", "--edit-selection-key", "5"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[cfg(feature = "benchmark")]
#[test]
fn bench_flag_still_fails_cleanly_without_a_compositor() {
    let output = waysip_cmd()
        .args(["-p", "--bench"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

#[cfg(feature = "freeze")]
#[test]
fn freeze_flag_still_fails_cleanly_without_a_compositor() {
    let output = waysip_cmd()
        .args(["-p", "--freeze"])
        .output()
        .expect("failed to run waysip binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
}

// ─── Predefined boxes (stdin) ──────────────────────────────────────────────

#[test]
fn boxes_flag_reads_piped_stdin_then_fails_cleanly_without_a_compositor() {
    let mut child = waysip_cmd()
        .arg("-r")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn waysip binary");

    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(b"10,10 50x50\n")
        .expect("failed to write to child stdin");

    let output = child
        .wait_with_output()
        .expect("failed to wait on waysip binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty());
    assert!(!stderr.contains("panicked at"));
    // Specifically: it got past stdin parsing (not the "no piped stdin" /
    // "stdin is empty" early-exit messages) and failed at the Wayland
    // connection step instead.
    assert!(!stderr.contains("No piped stdin"));
    assert!(!stderr.contains("Stdin is empty"));
}
