use std::process::Command;

#[test]
fn help_is_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_linux-clipboard-history"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("clear-all"));
}
