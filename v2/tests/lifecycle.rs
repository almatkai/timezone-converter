//! Test-only wrapper widens the SDK's post-handle/pre-commit cancellation window.
//! It still uses the pinned real SDK dispatcher and native Adapter on JSONL pipes.
use serde_json::{json, Value};
use tezbar_extension_sdk::{BoxFuture, CoreResult, Extension, ExtensionContext};
use timezone_converter_v2::Adapter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
struct AfterHandle(Adapter);
impl Extension for AfterHandle {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn version(&self) -> &'static str {
        self.0.version()
    }
    fn commands(&self) -> &'static [&'static str] {
        self.0.commands()
    }
    fn handle<'a>(
        &'a self,
        c: &'a ExtensionContext,
        cmd: &'a str,
        op: &'a str,
        p: &'a Value,
        s: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            let result = self.0.handle(c, cmd, op, p, s).await;
            if (op == "extension:run-command" && p["argumentValues"]["hold"] == true)
                || p["testHold"] == true
            {
                c.emit("test.after-handle", json!({}))?;
                std::future::pending::<()>().await;
            }
            result
        })
    }
    fn shutdown<'a>(&'a self, c: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        self.0.shutdown(c)
    }
}
async fn send(w: &mut tokio::io::DuplexStream, id: usize, method: &str, params: Value) {
    let bytes=serde_json::to_vec(&json!({"type":"request","id":format!("h:00000000-0000-4000-8000-{id:012x}"),"method":method,"params":params})).unwrap();
    w.write_all(&bytes).await.unwrap();
    w.write_all(b"\n").await.unwrap();
}
async fn read(r: &mut BufReader<tokio::io::DuplexStream>) -> Value {
    let mut s = String::new();
    tokio::time::timeout(std::time::Duration::from_secs(5), r.read_line(&mut s))
        .await
        .unwrap()
        .unwrap();
    serde_json::from_str(&s).unwrap()
}
#[tokio::test]
async fn more_than_128_post_handle_canceled_opens_then_128_live() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qa/lifecycle");
    std::fs::create_dir_all(&root).unwrap();
    let dir = tempfile::tempdir_in(root).unwrap();
    let (mut host_write, extension_read) = tokio::io::duplex(65536);
    let (extension_write, host_read) = tokio::io::duplex(65536);
    let mut r = BufReader::new(host_read);
    let task = tokio::spawn(tezbar_extension_sdk::serve_io(
        AfterHandle(Adapter::default()),
        extension_read,
        extension_write,
    ));
    assert_eq!(read(&mut r).await["type"], "hello");
    send(&mut host_write,1,"initialize",json!({"protocol":1,"extensionId":"com.tezbar.native.timezone-converter","platform":"aarch64-apple-darwin","packageDir":dir.path(),"dataDir":dir.path().join("data"),"tempDir":dir.path().join("tmp"),"grantedCapabilities":[]})).await;
    assert_eq!(read(&mut r).await["result"]["ready"], true);
    for i in 0..140 {
        let id = i + 2;
        send(&mut host_write,id,"session.open",json!({"sessionId":format!("canceled-{i}"),"commandName":"index","preferences":{"sortTimezones":"manually","hideWorldMap":true},"argumentValues":{"hold":true},"searchText":null})).await;
        assert_eq!(read(&mut r).await["event"], "test.after-handle");
        host_write.write_all(format!("{{\"type\":\"cancel\",\"requestId\":\"h:00000000-0000-4000-8000-{id:012x}\"}}\n").as_bytes()).await.unwrap();
        assert!(read(&mut r).await.get("error").is_some());
    }
    send(&mut host_write,180,"session.open",json!({"sessionId":"action-cancel","commandName":"index","preferences":{"sortTimezones":"manually","hideWorldMap":true}})).await;
    let home = read(&mut r).await;
    let add = home["result"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"] == "Add Timezones")
        .unwrap()["id"]
        .clone();
    send(&mut host_write,181,"session.call",json!({"sessionId":"action-cancel","operation":"extension:invoke-action","payload":{"actionId":add}})).await;
    let add = read(&mut r).await;
    let toggle = add["result"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"] == "Add UTC (GMT+0)")
        .unwrap()["id"]
        .clone();
    send(&mut host_write,182,"session.call",json!({"sessionId":"action-cancel","operation":"extension:invoke-action","payload":{"actionId":toggle,"testHold":true}})).await;
    assert_eq!(read(&mut r).await["event"], "test.after-handle");
    host_write
        .write_all(
            b"{\"type\":\"cancel\",\"requestId\":\"h:00000000-0000-4000-8000-0000000000b6\"}\n",
        )
        .await
        .unwrap();
    assert!(read(&mut r).await.get("error").is_some());
    send(&mut host_write,183,"session.call",json!({"sessionId":"action-cancel","operation":"extension:invoke-action","payload":{"actionId":toggle}})).await;
    assert_eq!(read(&mut r).await["result"]["mode"], "unchanged");
    let bytes = std::fs::read(dir.path().join("data/selected-timezones.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap()["selected"],
        json!(["UTC"])
    );
    send(
        &mut host_write,
        184,
        "session.dispose",
        json!({"sessionId":"action-cancel"}),
    )
    .await;
    read(&mut r).await;
    for i in 0..128 {
        send(&mut host_write,200+i,"session.open",json!({"sessionId":format!("live-{i}"),"commandName":"index","preferences":{"sortTimezones":"manually","hideWorldMap":true}})).await;
        assert_eq!(read(&mut r).await["result"]["root"]["type"], "List");
    }
    // EOF cancels every registered session/task and terminates without a dispose frame.
    drop(host_write);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(5), task)
            .await
            .unwrap()
            .unwrap()
            .is_ok()
    );
}
