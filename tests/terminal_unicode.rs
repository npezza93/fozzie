#![cfg(unix)]

#[test]
fn unicode_rendering_in_a_controlling_terminal() {
    let output = std::process::Command::new("python3")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/terminal_unicode.py"))
        .arg(env!("CARGO_BIN_EXE_fozzie"))
        .output()
        .expect("Python 3 is required for the Unix PTY regression test");
    assert!(
        output.status.success(),
        "PTY regression failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
