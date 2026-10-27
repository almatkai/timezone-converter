use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};
use tezbar_extension_sdk::{Extension, ExtensionPaths, TestContext};
use timezone_converter_v2::{Adapter, Clock};
struct Fixed(AtomicI64);
impl Clock for Fixed {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn local_zone(&self) -> Result<String, String> {
        Ok("America/Los_Angeles".into())
    }
}
fn owned() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qa/tests");
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn context(p: &std::path::Path) -> TestContext {
    TestContext::new(
        ExtensionPaths {
            package_dir: owned(),
            data_dir: p.into(),
            temp_dir: owned(),
        },
        &[],
    )
}
async fn call(a: &Adapter, t: &TestContext, sid: &str, op: &str, mut p: Value) -> Value {
    p["sessionId"] = json!(sid);
    a.handle(&t.context, "index", op, &p, &mut json!({}))
        .await
        .unwrap()
}
async fn open(a: &Adapter, t: &TestContext, sid: &str, alpha: bool, hide: bool) -> Value {
    call(a,t,sid,"extension:run-command",json!({"searchText":null,"preferences":{"sortTimezones":if alpha{"alphabetically"}else{"manually"},"hideWorldMap":hide}})).await
}
fn id(v: &Value, title: &str) -> String {
    v["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"] == title)
        .unwrap_or_else(|| panic!("missing {title}"))["id"]
        .as_str()
        .unwrap()
        .into()
}
async fn act(a: &Adapter, t: &TestContext, sid: &str, v: &Value, title: &str) -> Value {
    call(
        a,
        t,
        sid,
        "extension:invoke-action",
        json!({"actionId":id(v,title)}),
    )
    .await
}
#[tokio::test]
async fn owned_clock_snapshot_offsets_reset_and_map() {
    let dir = tempfile::tempdir_in(owned()).unwrap();
    let t = context(dir.path());
    let c = Arc::new(Fixed(AtomicI64::new(1705322096789)));
    let a = Adapter::new(c.clone());
    let home = open(&a, &t, "clock", false, false).await;
    assert!(home["root"]["props"]["markdown"]
        .as_str()
        .unwrap()
        .contains("4:34 AM"));
    let add = act(&a, &t, "clock", &home, "Add Timezones").await;
    assert_eq!(
        add["root"]["children"][1]["children"]
            .as_array()
            .unwrap()
            .len(),
        419
    );
    let added = act(&a, &t, "clock", &add, "Add Asia - Katmandu (GMT+5.75)").await;
    let home = act(&a, &t, "clock", &added, "Done").await;
    let form = act(&a, &t, "clock", &home, "Set Custom Time").await;
    let custom=call(&a,&t,"clock","extension:invoke-action",json!({"actionId":id(&form,"Confirm"),"formValues":{"customTime":"2024-07-01T04:34:56.789","timezone":"Asia/Katmandu"}})).await;
    assert!(custom["root"]["props"]["markdown"]
        .as_str()
        .unwrap()
        .contains("Custom time"));
    assert!(custom["root"]["props"]["markdown"]
        .as_str()
        .unwrap()
        .contains("no map"));
    c.0.store(1721044800000, Ordering::SeqCst);
    assert_eq!(
        call(&a, &t, "clock", "extension:refresh-session", json!({})).await["mode"],
        "unchanged"
    );
    let reset = act(&a, &t, "clock", &custom, "Clear Custom Time").await;
    let markdown = reset["root"]["props"]["markdown"].as_str().unwrap();
    assert!(markdown.contains("Jul 15, 2024, 5:00 AM"));
    assert!(markdown.contains("data:image/png;base64,"));
    assert!(t.calls().is_empty());
}
#[tokio::test]
async fn drafts_tokens_confirmations_stale_storage_and_order() {
    let dir = tempfile::tempdir_in(owned()).unwrap();
    let t = context(dir.path());
    let a = Adapter::new(Arc::new(Fixed(AtomicI64::new(1705322096789))));
    let home = open(&a, &t, "a", false, true).await;
    let b = open(&a, &t, "b", false, true).await;
    let form = act(&a, &t, "a", &home, "Set Custom Time").await;
    let old = id(&form, "Confirm");
    let invalid = call(
        &a,
        &t,
        "a",
        "extension:invoke-action",
        json!({"actionId":old,"formValues":{"customTime":"invalid B","timezone":"UTC"}}),
    )
    .await;
    assert!(invalid["message"]
        .as_str()
        .unwrap()
        .contains("Enter local date/time"));
    assert!(invalid["root"]["children"][0]["props"]
        .get("value")
        .is_none());
    assert_eq!(
        call(
            &a,
            &t,
            "a",
            "extension:invoke-action",
            json!({"actionId":old})
        )
        .await["mode"],
        "unchanged"
    );
    assert_eq!(
        call(
            &a,
            &t,
            "b",
            "extension:invoke-action",
            json!({"actionId":id(&invalid,"Confirm")})
        )
        .await["mode"],
        "unchanged"
    );
    let valid=call(&a,&t,"a","extension:invoke-action",json!({"actionId":id(&invalid,"Confirm"),"formValues":{"customTime":"2024-01-15T04:34:56.789","timezone":"UTC"}})).await;
    assert_eq!(valid["root"]["type"], "List");
    let add = act(&a, &t, "a", &valid, "Add Timezones").await;
    let add = act(&a, &t, "a", &add, "Add UTC (GMT+0)").await;
    let add = act(&a, &t, "a", &add, "Add Asia - Tokyo (GMT+9)").await;
    let badd = act(&a, &t, "b", &b, "Add Timezones").await;
    let conflict = act(&a, &t, "b", &badd, "Add Europe - Paris (GMT+1)").await;
    assert!(conflict["message"]
        .as_str()
        .unwrap()
        .contains("another session"));
    let home = act(&a, &t, "a", &add, "Done").await;
    let order = act(&a, &t, "a", &home, "Reorder Timezones").await;
    let moved = act(&a, &t, "a", &order, "Move Down").await;
    assert_eq!(moved["root"]["children"][0]["props"]["title"], "Asia/Tokyo");
    let confirm = act(&a, &t, "a", &moved, "Remove Timezone").await;
    let cancel = act(&a, &t, "a", &confirm, "Cancel").await;
    assert_eq!(cancel["root"]["children"].as_array().unwrap().len(), 2);
    let confirm = act(&a, &t, "a", &cancel, "Remove Timezone").await;
    let removed = act(&a, &t, "a", &confirm, "Delete").await;
    assert_eq!(removed["root"]["children"].as_array().unwrap().len(), 1);
    let home = act(&a, &t, "a", &removed, "Done").await;
    let confirm = act(&a, &t, "a", &home, "Remove Added Timezones").await;
    let cleared = act(&a, &t, "a", &confirm, "Remove All").await;
    assert_eq!(cleared["root"]["children"][0]["type"], "List.EmptyView");
    let bytes = std::fs::read(dir.path().join("selected-timezones.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap()["selected"],
        json!([])
    );
}
#[tokio::test]
async fn corrupt_and_failed_writes_preserve_bytes() {
    let dir = tempfile::tempdir_in(owned()).unwrap();
    let path = dir.path().join("selected-timezones.json");
    std::fs::write(&path, b"{broken").unwrap();
    let t = context(dir.path());
    let a = Adapter::new(Arc::new(Fixed(AtomicI64::new(1705322096789))));
    let v = open(&a, &t, "corrupt", false, true).await;
    assert!(v["message"].as_str().unwrap().contains("Corrupt"));
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    let v = act(&a, &t, "corrupt", &v, "Retry Loading Saved Timezones").await;
    assert!(v["message"].as_str().unwrap().contains("Corrupt"));
    std::fs::remove_file(&path).unwrap();
    let home = act(&a, &t, "corrupt", &v, "Retry Loading Saved Timezones").await;
    assert_eq!(home["root"]["type"], "List");
    // A regular file in place of dataDir fails without changing that file.
    let f = dir.path().join("blocked");
    std::fs::write(&f, b"preserve me").unwrap();
    let blocked = context(&f);
    let v = open(&a, &blocked, "blocked", false, true).await;
    assert!(v["message"].as_str().unwrap().contains("Cannot read"));
    assert_eq!(std::fs::read(f).unwrap(), b"preserve me");
}

#[cfg(unix)]
#[tokio::test]
async fn atomic_storage_failure_keeps_valid_previous_file() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir_in(owned()).unwrap();
    let path = dir.path().join("selected-timezones.json");
    let bytes = b"{\"version\":1,\"selected\":[\"UTC\"]}";
    std::fs::write(&path, bytes).unwrap();
    let t = context(dir.path());
    let a = Adapter::new(Arc::new(Fixed(AtomicI64::new(1705322096789))));
    let home = open(&a, &t, "atomic", false, true).await;
    let add = act(&a, &t, "atomic", &home, "Add Timezones").await;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let failed = act(&a, &t, "atomic", &add, "Add Asia - Tokyo (GMT+9)").await;
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(failed["message"]
        .as_str()
        .unwrap()
        .contains("atomic storage replacement"));
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[tokio::test]
async fn foreign_adapter_tokens_do_not_match_even_when_session_names_are_reused() {
    let dir = tempfile::tempdir_in(owned()).unwrap();
    let t = context(dir.path());
    let clock = Arc::new(Fixed(AtomicI64::new(1705322096789)));
    let a = Adapter::new(clock.clone());
    let b = Adapter::new(clock);
    let av = open(&a, &t, "same-name", false, true).await;
    let bv = open(&b, &t, "same-name", false, true).await;
    assert_ne!(id(&av, "Add Timezones"), id(&bv, "Add Timezones"));
    assert_eq!(
        call(
            &b,
            &t,
            "same-name",
            "extension:invoke-action",
            json!({"actionId":id(&av,"Add Timezones")})
        )
        .await["mode"],
        "unchanged"
    );
    let valid = act(&b, &t, "same-name", &bv, "Add Timezones").await;
    assert_eq!(valid["root"]["props"]["navigationTitle"], "Add Timezones");
}
