#![cfg(unix)]

#[test]
fn narrow_terminal_layout_with_captured_stdout() {
    let output = std::process::Command::new("python3")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/terminal_layout.py"))
        .arg(env!("CARGO_BIN_EXE_fozzie"))
        .output()
        .expect("Python 3 is required for the Unix PTY regression test");
    assert!(
        output.status.success(),
        "Narrow-terminal PTY regression failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
