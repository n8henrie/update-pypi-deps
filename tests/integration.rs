use assert_cmd::Command;

#[test]
fn test() {
    let mut cmd = Command::cargo_bin("update-pypi-deps").unwrap();
    cmd.assert().success();
}
