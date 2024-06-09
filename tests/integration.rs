use std::process::Command;

const EXE: &str = env!("CARGO_BIN_EXE_update-pypi-deps");

#[test]
fn test_help() {
    let cmd = Command::new(EXE).args(["--help"]).output().unwrap();
    assert!(cmd.status.success());
}
