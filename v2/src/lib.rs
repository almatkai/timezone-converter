mod maps;
mod storage;
pub mod time;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex, Weak},
};
use tezbar_extension_sdk::{unchanged, view, BoxFuture, CoreResult, Extension, ExtensionContext};

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> i64;
    fn local_zone(&self) -> Result<String, String>;
}
struct SystemClock;
impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
    fn local_zone(&self) -> Result<String, String> {
        iana_time_zone::get_timezone().map_err(|e| format!("Cannot detect OS local timezone: {e}"))
    }
}
#[derive(Clone)]
enum Screen {
    Home,
    Add,
    Order,
    Custom,
    Confirm(Option<String>),
}
#[derive(Clone)]
enum Intent {
    Home,
    Add,
    Order,
    Custom,
    Reset,
    Toggle(String),
    Move(String, i8),
    Ask(Option<String>),
    Confirm(Option<String>),
    Submit,
    Reload,
}
struct Session {
    owner: String,
    life: u64,
    next: u64,
    actions: HashMap<String, Intent>,
    screen: Screen,
    local: String,
    zone: String,
    ms: i64,
    custom: bool,
    selected: Vec<String>,
    baseline: Option<Vec<u8>>,
    ready: bool,
    alpha: bool,
    hide_map: bool,
    draft_time: String,
    draft_zone: String,
    initial_form: bool,
    error: Option<String>,
}
#[derive(Default)]
struct Sessions {
    next: u64,
    items: BTreeMap<String, Session>,
}
pub struct Adapter {
    owner: String,
    sessions: Arc<Mutex<Sessions>>,
    clock: Arc<dyn Clock>,
}
impl Default for Adapter {
    fn default() -> Self {
        Self::new(Arc::new(SystemClock))
    }
}
impl Adapter {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            owner: uuid::Uuid::new_v4().to_string(),
            sessions: Arc::default(),
            clock,
        }
    }
}
struct Cleanup {
    sessions: Weak<Mutex<Sessions>>,
    sid: String,
    life: u64,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(s) = self.sessions.upgrade() {
            if let Ok(mut m) = s.lock() {
                if m.items.get(&self.sid).is_some_and(|s| s.life == self.life) {
                    m.items.remove(&self.sid);
                }
            }
        }
    }
}
impl Session {
    fn action(
        &mut self,
        sid: &str,
        title: &str,
        intent: Intent,
        kind: &str,
        actions: &mut Vec<Value>,
    ) -> String {
        self.next += 1;
        let id = format!("{sid}:{}:{}:{}", self.owner, self.life, self.next);
        self.actions.insert(id.clone(), intent);
        actions.push(json!({"id":id,"title":title,"kind":kind}));
        id
    }
    fn load(&mut self, ctx: &ExtensionContext) -> Result<(), String> {
        let (b, mut selected) = storage::read(&ctx.paths.data_dir)?;
        if self.alpha {
            selected.sort()
        };
        self.baseline = b;
        self.selected = selected;
        self.ready = true;
        Ok(())
    }
    fn save(&mut self, ctx: &ExtensionContext, mut selected: Vec<String>) -> Result<(), String> {
        if self.alpha {
            selected.sort()
        };
        let b = storage::write(&ctx.paths.data_dir, &self.baseline, &selected)?;
        self.baseline = b;
        self.selected = selected;
        Ok(())
    }
    fn apply(
        &mut self,
        ctx: &ExtensionContext,
        intent: Intent,
        payload: &Value,
        clock: &dyn Clock,
    ) -> Result<(), String> {
        match intent {
            Intent::Home => {
                self.screen = Screen::Home;
            }
            Intent::Add => self.screen = Screen::Add,
            Intent::Order => self.screen = Screen::Order,
            Intent::Custom => {
                self.draft_time = time::wall(self.ms, &self.local)?;
                self.draft_zone = self.zone.clone();
                self.initial_form = true;
                self.screen = Screen::Custom;
            }
            Intent::Reset => {
                let ms = clock.now_ms();
                time::instant(ms)?;
                self.ms = ms;
                self.zone = self.local.clone();
                self.custom = false;
                self.screen = Screen::Home;
            }
            Intent::Toggle(z) => {
                let mut selected = self.selected.clone();
                if let Some(i) = selected.iter().position(|v| v == &z) {
                    selected.remove(i);
                } else {
                    selected.push(z)
                }
                self.save(ctx, selected)?;
            }
            Intent::Move(z, d) => {
                let mut selected = self.selected.clone();
                if let Some(i) = selected.iter().position(|v| v == &z) {
                    let j = i as isize + isize::from(d);
                    if j >= 0 && (j as usize) < selected.len() {
                        selected.swap(i, j as usize);
                        self.save(ctx, selected)?;
                    }
                }
            }
            Intent::Ask(z) => self.screen = Screen::Confirm(z),
            Intent::Confirm(z) => {
                let single = z.is_some();
                let selected = if let Some(z) = z {
                    self.selected.iter().filter(|s| **s != z).cloned().collect()
                } else {
                    vec![]
                };
                self.save(ctx, selected)?;
                self.screen = if single { Screen::Order } else { Screen::Home };
            }
            Intent::Submit => {
                let f = payload
                    .get("formValues")
                    .and_then(Value::as_object)
                    .ok_or("Custom Time form values must be an object")?;
                // Preserve every submitted draft, even invalid ones. Oversized inputs have precise bounds.
                if let Some(t) = f.get("customTime").and_then(Value::as_str) {
                    if t.len() > 40 {
                        return Err(
                            "Custom time exceeds 40 UTF-8 bytes; client draft retained".into()
                        );
                    }
                    self.draft_time = t.into();
                }
                if let Some(z) = f.get("timezone").and_then(Value::as_str) {
                    if z.len() > 128 {
                        return Err(
                            "Timezone exceeds 128 UTF-8 bytes; client draft retained".into()
                        );
                    }
                    self.draft_zone = z.into();
                }
                if !f.get("customTime").is_some_and(Value::is_string)
                    || !f.get("timezone").is_some_and(Value::is_string)
                {
                    return Err(
                        "Custom time and timezone must both be strings; client draft retained"
                            .into(),
                    );
                }
                if !time::catalog().contains(&self.draft_zone) && self.draft_zone != self.local {
                    return Err("Choose a timezone from the complete pinned Intl catalog".into());
                }
                let picker = if self.draft_time == time::wall(self.ms, &self.local)? {
                    self.ms
                } else {
                    time::picker(&self.draft_time, &self.local)?
                };
                let ms = time::custom(picker, &self.draft_zone, &self.local)?;
                for z in self
                    .selected
                    .iter()
                    .chain(std::iter::once(&self.draft_zone))
                {
                    time::row(ms, z)?;
                }
                self.ms = ms;
                self.zone = self.draft_zone.clone();
                self.custom = true;
                self.screen = Screen::Home;
            }
            Intent::Reload => {
                self.load(ctx)?;
            }
        }
        Ok(())
    }
    fn render(&mut self, sid: &str, now: i64) -> CoreResult {
        // This map lives outside SDK staged Value: rollback cannot revive a consumed ID.
        self.actions.clear();
        let mut actions = vec![];
        let root = if !self.ready {
            let retry = self.action(
                sid,
                "Retry Loading Saved Timezones",
                Intent::Reload,
                "action",
                &mut actions,
            );
            json!({"type":"Detail","props":{"markdown":"# Saved timezones could not be loaded\nPrevious storage bytes were preserved. Use Retry Loading Saved Timezones after resolving the error.","retryActionId":retry}})
        } else {
            match self.screen.clone() {
                Screen::Home => {
                    self.action(sid, "Add Timezones", Intent::Add, "push", &mut actions);
                    self.action(
                        sid,
                        if self.custom {
                            "Clear Custom Time"
                        } else {
                            "Set Custom Time"
                        },
                        if self.custom {
                            Intent::Reset
                        } else {
                            Intent::Custom
                        },
                        "action",
                        &mut actions,
                    );
                    if !self.alpha {
                        self.action(
                            sid,
                            "Reorder Timezones",
                            Intent::Order,
                            "push",
                            &mut actions,
                        );
                    }
                    if !self.selected.is_empty() {
                        self.action(
                            sid,
                            "Remove Added Timezones",
                            Intent::Ask(None),
                            "push",
                            &mut actions,
                        );
                    }
                    self.action(
                        sid,
                        "Reload Saved Timezones",
                        Intent::Reload,
                        "action",
                        &mut actions,
                    );
                    let mut rows = vec![];
                    let mut metadata = vec![];
                    for z in &self.selected {
                        let r = time::row(self.ms, z)?;
                        let title = format!("{} ({})", time::name(z), r.short);
                        rows.push(json!({"type":"List.Item","props":{"id":z,"title":title,"accessories":[{"tag":r.ff}]}}));
                        metadata.push(json!({"type":"Detail.Metadata.TagList","props":{"title":title},"children":[{"type":"Detail.Metadata.TagList.Item","props":{"text":r.ff}}]}));
                        metadata.push(json!({"type":"Detail.Metadata.Separator"}));
                    }
                    if self.selected.is_empty() {
                        rows.push(json!({"type":"List.EmptyView","props":{"title":"No Timezones Added","description":"Added timezones appear here"}}));
                        metadata.push(json!({"type":"Detail.Metadata.Label","props":{"title":"No Timezones Added","text":"Added timezones appear here"}}));
                    }
                    if self.hide_map {
                        json!({"type":"List","props":{"searchBarPlaceholder":"Search Timezones"},"children":rows})
                    } else {
                        let r = time::row(self.ms, &self.zone)?;
                        let heading = if self.custom {
                            "Custom time"
                        } else {
                            "Local time"
                        };
                        let image=match maps::image(r.offset_seconds){Some(uri)=>format!("![{}]({uri})",time::name(&self.zone)),None=>format!("World map unavailable: the original assets contain no map for offset {} seconds.",r.offset_seconds)};
                        json!({"type":"Detail","props":{"markdown":format!("\n# {heading}\n## {}\n{} ({})\n{image}\n",r.ff,time::name(&self.zone),r.short)},"metadata":{"type":"Detail.Metadata","children":metadata}})
                    }
                }
                Screen::Add => {
                    let back = self.action(sid, "Done", Intent::Home, "pop", &mut actions);
                    let mut sections = vec![];
                    for (title, zones) in [
                        ("Selected", self.selected.clone()),
                        (
                            "Others",
                            time::catalog()
                                .iter()
                                .filter(|z| !self.selected.contains(z))
                                .cloned()
                                .collect(),
                        ),
                    ] {
                        let mut rows = vec![];
                        for z in zones {
                            let label = format!("{} ({})", time::name(&z), time::gmt(now, &z)?);
                            let id = self.action(
                                sid,
                                &format!(
                                    "{} {label}",
                                    if title == "Selected" { "Remove" } else { "Add" }
                                ),
                                Intent::Toggle(z.clone()),
                                "action",
                                &mut actions,
                            );
                            rows.push(json!({"type":"List.Item","props":{"id":z,"title":label,"icon":{"source":if title=="Selected"{"CheckCircle"}else{"Circle"}},"actionIds":[id,back]}}));
                        }
                        sections.push(
                            json!({"type":"List.Section","props":{"title":title},"children":rows}),
                        );
                    }
                    json!({"type":"List","props":{"navigationTitle":"Add Timezones","searchBarPlaceholder":"Search Timezones","backActionId":back},"children":sections})
                }
                Screen::Order => {
                    let back = self.action(sid, "Done", Intent::Home, "pop", &mut actions);
                    let mut rows = vec![];
                    for (i, z) in self.selected.clone().into_iter().enumerate() {
                        let mut ids = vec![];
                        if !self.alpha && i > 0 {
                            ids.push(self.action(
                                sid,
                                "Move Up",
                                Intent::Move(z.clone(), -1),
                                "action",
                                &mut actions,
                            ));
                        }
                        if !self.alpha && i + 1 < self.selected.len() {
                            ids.push(self.action(
                                sid,
                                "Move Down",
                                Intent::Move(z.clone(), 1),
                                "action",
                                &mut actions,
                            ));
                        }
                        ids.push(self.action(
                            sid,
                            "Remove Timezone",
                            Intent::Ask(Some(z.clone())),
                            "push",
                            &mut actions,
                        ));
                        ids.push(back.clone());
                        rows.push(json!({"type":"List.Item","props":{"id":z,"title":z,"icon":{"source":"Clock"},"actionIds":ids}}));
                    }
                    json!({"type":"List","props":{"navigationTitle":"Reorder Timezones","searchBarPlaceholder":"Search Timezones","backActionId":back},"children":rows})
                }
                Screen::Custom => {
                    let submit =
                        self.action(sid, "Confirm", Intent::Submit, "submit-form", &mut actions);
                    let back = self.action(sid, "Cancel", Intent::Home, "pop", &mut actions);
                    let mut t = json!({"id":"customTime","title":"Custom time","info":format!("Enter local time in {} as YYYY-MM-DDTHH:mm:ss.SSS, or use Z / an explicit UTC offset to select either occurrence of a repeated hour",self.local)});
                    let mut z = json!({"id":"timezone","title":"Timezone","data":time::catalog().iter().map(|z|json!({"title":z,"value":z})).collect::<Vec<_>>()});
                    if self.initial_form {
                        t["value"] = json!(self.draft_time);
                        z["value"] = json!(self.draft_zone);
                        self.initial_form = false;
                    }
                    // Later responses omit defaults. Pinned reconcileFormValues then preserves client-owned drafts.
                    json!({"type":"Form","props":{"navigationTitle":"Custom Time","submitActionId":submit,"backActionId":back},"children":[{"type":"Form.TextField","props":t},{"type":"Form.Dropdown","props":z}]})
                }
                Screen::Confirm(z) => {
                    let title = if let Some(z) = &z {
                        format!("Are you sure you want to delete \"{z}\"?")
                    } else {
                        "Are you sure you want remove all timezones?".into()
                    };
                    let cancel_intent = if z.is_some() {
                        Intent::Order
                    } else {
                        Intent::Home
                    };
                    self.action(
                        sid,
                        if z.is_some() { "Delete" } else { "Remove All" },
                        Intent::Confirm(z),
                        "action",
                        &mut actions,
                    );
                    let back = self.action(sid, "Cancel", cancel_intent, "pop", &mut actions);
                    json!({"type":"Detail","props":{"markdown":format!("# {title}\nChoose the confirmation action to proceed."),"backActionId":back}})
                }
            }
        };
        for a in &mut actions {
            if [
                "Delete",
                "Remove All",
                "Remove Timezone",
                "Remove Added Timezones",
            ]
            .contains(&a["title"].as_str().unwrap_or(""))
            {
                a["style"] = json!("destructive");
            }
        }
        let mut result = view("Timezone Converter", root, actions);
        result["refreshMs"] = Value::Null;
        if let Some(e) = &self.error {
            result["message"] = json!(format!("Failed: {e}"));
        }
        Ok(result)
    }
}
impl Extension for Adapter {
    fn id(&self) -> &'static str {
        "com.tezbar.native.timezone-converter"
    }
    fn version(&self) -> &'static str {
        "2.0.0"
    }
    fn commands(&self) -> &'static [&'static str] {
        &["index"]
    }
    fn handle<'a>(
        &'a self,
        ctx: &'a ExtensionContext,
        command: &'a str,
        operation: &'a str,
        payload: &'a Value,
        _state: &'a mut Value,
    ) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            if command != "index" {
                return Err("Unknown command".into());
            }
            let sid = payload["sessionId"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 256)
                .ok_or("Session ID must contain 1..256 UTF-8 bytes")?;
            if operation == "extension:dispose-session" {
                self.sessions
                    .lock()
                    .map_err(|e| e.to_string())?
                    .items
                    .remove(sid);
                return Ok(unchanged());
            }
            if operation == "extension:refresh-session" || operation == "extension:load-more" {
                return Ok(unchanged());
            }
            if operation == "extension:run-command" {
                if payload
                    .get("searchText")
                    .is_some_and(|x| !x.is_null() && !x.is_string())
                {
                    return Err("searchText must be string or null".into());
                }
                let p = &payload["preferences"];
                let alpha = match p["sortTimezones"].as_str() {
                    Some("alphabetically") => true,
                    Some("manually") => false,
                    _ => {
                        return Err(
                            "Sort Options is required: choose alphabetically or manually".into(),
                        )
                    }
                };
                let hide_map = match p.get("hideWorldMap") {
                    None | Some(Value::Null) => false,
                    Some(Value::Bool(b)) => *b,
                    Some(Value::String(s)) if s == "true" => true,
                    Some(Value::String(s)) if s == "false" => false,
                    _ => return Err("Hide World Map must be a boolean".into()),
                };
                let local = self.clock.local_zone()?;
                time::zone(&local)?;
                let ms = self.clock.now_ms();
                time::instant(ms)?;
                let mut m = self.sessions.lock().map_err(|e| e.to_string())?;
                if m.items.len() >= 128 || m.items.contains_key(sid) {
                    return Err("Adapter session capacity reached/session already open".into());
                }
                m.next += 1;
                let life = m.next;
                let mut s = Session {
                    owner: self.owner.clone(),
                    life,
                    next: 0,
                    actions: HashMap::new(),
                    screen: Screen::Home,
                    local: local.clone(),
                    zone: local,
                    ms,
                    custom: false,
                    selected: vec![],
                    baseline: None,
                    ready: false,
                    alpha,
                    hide_map,
                    draft_time: String::new(),
                    draft_zone: String::new(),
                    initial_form: false,
                    error: None,
                };
                if let Err(e) = s.load(ctx) {
                    s.error = Some(e)
                }
                let result = s.render(sid, ms)?;
                m.items.insert(sid.into(), s);
                drop(m);
                let cleanup = Cleanup {
                    sessions: Arc::downgrade(&self.sessions),
                    sid: sid.into(),
                    life,
                };
                let c = ctx.clone();
                ctx.spawn(async move {
                    let _cleanup = cleanup;
                    c.cancelled().await;
                })?;
                return Ok(result);
            }
            if !["extension:invoke-action", "extension:search-text-changed"].contains(&operation) {
                return Err(format!("Unsupported operation: {operation}"));
            }
            let mut m = self.sessions.lock().map_err(|e| e.to_string())?;
            let s = m.items.get_mut(sid).ok_or("Adapter session is closed")?;
            if operation == "extension:search-text-changed" {
                if payload["searchText"]
                    .as_str()
                    .is_none_or(|s| s.len() > 1024)
                {
                    return Err("Search text must be a string of at most 1024 UTF-8 bytes".into());
                }
                // Entire catalog stays in the tree; pinned ListRuntime performs client filtering.
                return s.render(sid, self.clock.now_ms());
            }
            let Some(id) = payload["actionId"].as_str() else {
                return Ok(unchanged());
            };
            let Some(intent) = s.actions.remove(id) else {
                return Ok(unchanged());
            };
            // Consume the entire prior surface before any mutation/navigation. No await/rollback here.
            s.actions.clear();
            s.error = None;
            if let Err(e) = s.apply(ctx, intent, payload, self.clock.as_ref()) {
                s.error = Some(e)
            }
            s.render(sid, self.clock.now_ms())
        })
    }
    fn shutdown<'a>(&'a self, _ctx: &'a ExtensionContext) -> BoxFuture<'a, CoreResult> {
        Box::pin(async move {
            self.sessions
                .lock()
                .map_err(|e| e.to_string())?
                .items
                .clear();
            Ok(unchanged())
        })
    }
}
