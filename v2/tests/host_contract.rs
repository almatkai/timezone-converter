#[path = "contract/pinned.rs"]
mod pinned;
use serde_json::{json, Value};
#[test]
fn actual_host_manifest_validator_positive_and_negatives() {
    let m: Value = serde_json::from_str(include_str!("../extension.json")).unwrap();
    pinned::validate_fixture(&m).unwrap();
    for (k, v) in [
        ("schemaVersion", json!(2)),
        ("runtime", json!("javascript")),
        ("execution", json!("embedded")),
        ("protocolVersion", json!(2)),
        ("id", json!("bad")),
        ("capabilities", json!(["invented"])),
        ("migrationPaths", json!({"x":"../outside"})),
    ] {
        let mut n = m.clone();
        n[k] = v;
        assert!(pinned::validate_fixture(&n).is_err(), "{k}");
    }
    for (k, v) in [
        ("timeoutMs", json!(0)),
        ("concurrency", json!("parallel")),
        ("refreshMs", json!(100)),
    ] {
        let mut n = m.clone();
        n["commands"][0][k] = v;
        assert!(pinned::validate_fixture(&n).is_err(), "{k}");
    }
    let mut n = m.clone();
    n["commands"]
        .as_array_mut()
        .unwrap()
        .push(m["commands"][0].clone());
    assert!(pinned::validate_fixture(&n).is_err());
}
#[test]
fn actual_host_producer_null_launch_and_annotation() {
    let p = pinned::open_params(json!({}));
    assert_eq!(p["searchText"], Value::Null);
    assert_eq!(p["argumentValues"], json!({}));
    assert_eq!(p["sessionId"], "fixture-session");
    let v = pinned::annotate_fixture(
        json!({"ok":true,"mode":"view","root":{"type":"List"},"actions":[]}),
    )
    .unwrap();
    assert_eq!(v["extensionId"], "com.tezbar.native.timezone-converter");
    assert_eq!(v["refreshMs"], Value::Null);
    assert_eq!(v["generation"], "fixture-generation");
    assert_eq!(
        pinned::annotate_fixture(json!({"ok":true,"mode":"unchanged"})).unwrap()["mode"],
        "unchanged"
    );
    assert!(pinned::annotate_fixture(json!({"ok":true,"mode":"invented"})).is_err());
    assert!(pinned::annotate_fixture(json!({"mode":"view"})).is_err());
}
