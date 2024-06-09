use serde_json;

use crate::pypi::PypiResponse;

fn response_from_file(filename: impl AsRef<str>) -> PypiResponse {
    let input = std::fs::read(filename.as_ref()).unwrap();
    serde_json::from_slice(&input).unwrap()
}

#[test]
fn test_response() {
    let fauxmo = response_from_file("tests/files/fauxmo_response.json");
    assert_eq!(fauxmo.info.version, "0.8.0");
    let keyring = response_from_file("tests/files/keyring_response.json");
    assert_eq!(keyring.info.version, "25.2.0");
}
