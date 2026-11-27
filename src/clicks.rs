use super::*;
use windows::Win32::{
    System::Com::CoCreateGuid,
    UI::WindowsAndMessaging::{
        GetForegroundWindow, IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindowAsync,
    },
};
const PREFIX: &str = "replynoti-open://open/";
const LIFETIME: Duration = Duration::from_secs(300);

#[derive(Clone)]
struct Row {
    element: IUIAutomationElement,
    title: String,
    rid: Vec<i32>,
    selected: bool,
}
#[derive(Clone)]
struct Target {
    title: String,
    rid: Vec<i32>,
}
fn rows(o: &Observer, h: usize) -> Result<Vec<Row>> {
    let root = o.root(h)?;
    let cond = condition(&o.u, UIA_ControlTypePropertyId, Variant::int(50000))?;
    let arr = unsafe { root.FindAll(TreeScope_Descendants, &cond)? };
    let mut result = Vec::new();
    for i in 0..unsafe { arr.Length()? } {
        let e = unsafe { arr.GetElement(i)? };
        let cl = unsafe { e.CurrentClassName()?.to_string() };
        if !cl.contains("sidebar-item")
            || !cl.contains("data-[app-action-sidebar-thread-selected=true]")
        {
            continue;
        }
        let title = unsafe { e.CurrentName()?.to_string() };
        let rid = runtime_id(&e)?;
        if title.is_empty() || rid.is_empty() {
            continue;
        }
        result.push(Row {
            element: e,
            title,
            rid,
            selected: cl.contains("[--color-text:var(--color-text-emphasis)]"),
        });
    }
    Ok(result)
}
fn retry_rows(o: &Observer, h: usize) -> Result<Vec<Row>> {
    for _ in 0..3 {
        match rows(o, h) {
            Err(e) if e.code().0 as u32 == 0x80040201 => thread::sleep(Duration::from_millis(150)),
            other => return other,
        }
    }
    rows(o, h)
}
fn target(o: &Observer, h: usize) -> Option<Target> {
    let rows = retry_rows(o, h).ok()?;
    let mut selected = rows.iter().filter(|r| r.selected);
    let r = selected.next()?;
    if selected.next().is_some() || rows.iter().filter(|x| x.title == r.title).count() != 1 {
        return None;
    }
    Some(Target {
        title: r.title.clone(),
        rid: r.rid.clone(),
    })
}
fn matching<'a>(rows: &'a [Row], t: &Target) -> Option<&'a Row> {
    let mut candidates = rows.iter().filter(|r| r.title == t.title);
    let r = candidates.next()?;
    if candidates.next().is_some() || r.rid != t.rid {
        return None;
    }
    Some(r)
}
fn composer_empty(o: &Observer, h: usize) -> Result<bool> {
    let root = o.root(h)?;
    let cond = condition(&o.u, UIA_ControlTypePropertyId, Variant::int(50004))?;
    let arr = unsafe { root.FindAll(TreeScope_Descendants, &cond)? };
    let mut found = 0;
    for i in 0..unsafe { arr.Length()? } {
        let e = unsafe { arr.GetElement(i)? };
        if !unsafe { e.CurrentClassName()?.to_string() }.contains("ProseMirror") {
            continue;
        }
        found += 1;
        let p: IUIAutomationValuePattern = unsafe { e.GetCurrentPatternAs(UIA_ValuePatternId)? };
        let value = unsafe { p.CurrentValue()?.to_string() };
        if !matches!(
            value.trim(),
            "" | "ChatGPT에게 물어보세요" | "Ask ChatGPT" | "Ask anything"
        ) {
            return Ok(false);
        }
    }
    Ok(found == 1)
}
fn window_pid(h: usize) -> u32 {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(HWND(h as *mut c_void), Some(&mut pid));
    }
    pid
}
fn focus(h: usize, pid: u32) -> bool {
    let hwnd = HWND(h as *mut c_void);
    if pid == 0 || window_pid(h) != pid || !valid_window(hwnd) {
        return false;
    }
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindowAsync(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
        GetForegroundWindow() == hwnd
    }
}
fn navigate(o: &Observer, h: usize, pid: u32, t: &Target) -> &'static str {
    if window_pid(h) != pid || !valid_window(HWND(h as *mut c_void)) {
        return "WINDOW_CHANGED";
    }
    if !o.observe(h).ok().flatten().is_some_and(|s| !s.busy)
        || !composer_empty(o, h).unwrap_or(false)
    {
        return "BUSY_OR_DRAFT";
    }
    let Ok(rows) = retry_rows(o, h) else {
        return "SIDEBAR_UNAVAILABLE";
    };
    let Some(row) = matching(&rows, t) else {
        return "IDENTITY_NOT_MATCHED";
    };
    if row.selected {
        return "ALREADY_OPEN";
    }
    // Reacquire before Invoke, and never use a title alone as identity.
    let Ok(rows) = retry_rows(o, h) else {
        return "SIDEBAR_UNAVAILABLE";
    };
    let Some(row) = matching(&rows, t) else {
        return "IDENTITY_NOT_MATCHED";
    };
    if !o.observe(h).ok().flatten().is_some_and(|s| !s.busy)
        || !composer_empty(o, h).unwrap_or(false)
    {
        return "BUSY_OR_DRAFT";
    }
    if row.selected {
        return "ALREADY_OPEN";
    }
    if !unsafe { row.element.CurrentIsEnabled() }.is_ok_and(|b| b.as_bool())
        || !unsafe { row.element.CurrentIsOffscreen() }.is_ok_and(|b| !b.as_bool())
    {
        return "NOT_VISIBLE_OR_ENABLED";
    }
    let Ok(p): Result<IUIAutomationInvokePattern> =
        (unsafe { row.element.GetCurrentPatternAs(UIA_InvokePatternId) })
    else {
        return "INVOKE_UNAVAILABLE";
    };
    let invoked = unsafe { p.Invoke() }.is_ok();
    // The provider can become unavailable during navigation even after a successful Invoke.
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        thread::sleep(Duration::from_millis(250));
        if let Ok(rows) = retry_rows(o, h) {
            if matching(&rows, t).is_some_and(|r| r.selected)
                && rows.iter().filter(|r| r.selected).count() == 1
            {
                return "CHAT_OPENED";
            }
        }
    }
    if invoked {
        "NAVIGATION_UNCONFIRMED"
    } else {
        "INVOKE_FAILED"
    }
}
struct Ticket {
    h: usize,
    pid: u32,
    target: Option<Target>,
    created: Instant,
}
pub(super) struct Clicks {
    dir: PathBuf,
    tickets: HashMap<String, Ticket>,
}
fn token_ok(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn token_from_uri(uri: &str) -> Option<&str> {
    uri.strip_prefix(PREFIX).filter(|s| token_ok(s))
}
fn token() -> Option<String> {
    let g = unsafe { CoCreateGuid().ok()? };
    Some(format!(
        "{:08x}{:04x}{:04x}{}",
        g.data1,
        g.data2,
        g.data3,
        g.data4
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}
impl Clicks {
    pub(super) fn new(root: &Path) -> Self {
        let dir = root.join("click-tickets");
        let _ = fs::create_dir_all(&dir);
        // No ticket survives a watcher restart. Delete only our exact marker names.
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                let n = e.file_name();
                let n = n.to_string_lossy();
                if let Some((t, ext)) = n.rsplit_once('.') {
                    if token_ok(t) && matches!(ext, "pending" | "clicked") {
                        let _ = fs::remove_file(e.path());
                    }
                }
            }
        }
        Self {
            dir,
            tickets: HashMap::new(),
        }
    }
    pub(super) fn issue(
        &mut self,
        root: &Path,
        o: &Observer,
        h: usize,
        exact: bool,
    ) -> Option<String> {
        let pid = window_pid(h);
        if pid == 0 {
            return None;
        }
        let id = token()?;
        let target = if exact { target(o, h) } else { None };
        let has_target = target.is_some();
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(self.dir.join(format!("{id}.pending")))
            .ok()?;
        if writeln!(f, "{h} {pid}").is_err() {
            let _ = fs::remove_file(self.dir.join(format!("{id}.pending")));
            return None;
        }
        self.tickets.insert(
            id.clone(),
            Ticket {
                h,
                pid,
                target,
                created: Instant::now(),
            },
        );
        log(
            root,
            if has_target {
                "CLICK_TARGET_CAPTURED"
            } else {
                "CLICK_FOCUS_ONLY"
            },
        );
        Some(format!("{PREFIX}{id}"))
    }
    pub(super) fn drain(&mut self, root: &Path, o: &Observer) {
        let mut done = Vec::new();
        for (id, t) in &self.tickets {
            let pending = self.dir.join(format!("{id}.pending"));
            let clicked = self.dir.join(format!("{id}.clicked"));
            if t.created.elapsed() >= LIFETIME {
                let _ = fs::remove_file(pending);
                let _ = fs::remove_file(clicked);
                done.push(id.clone());
                continue;
            }
            if clicked.is_file() {
                let _ = fs::remove_file(clicked);
                // Only the matching watcher owns the in-memory title/ID.
                let outcome = t
                    .target
                    .as_ref()
                    .filter(|_| crate::settings::Config::load(root).navigate)
                    .map(|target| navigate(o, t.h, t.pid, target))
                    .unwrap_or("FOCUS_ONLY");
                log(root, &format!("CLICK_RESULT {outcome}"));
                done.push(id.clone());
            }
        }
        for id in done {
            self.tickets.remove(&id);
        }
    }
}
pub(super) fn activate(root: &Path, uri: &str) {
    if !root.join(".replynoti-install").is_file() {
        return;
    }
    let Some(id) = token_from_uri(uri) else {
        log(root, "CLICK_REJECT_INVALID_URI");
        return;
    };
    let dir = root.join("click-tickets");
    let pending = dir.join(format!("{id}.pending"));
    let clicked = dir.join(format!("{id}.clicked"));
    let Ok(md) = fs::metadata(&pending) else {
        log(root, "CLICK_REJECT_UNKNOWN");
        return;
    };
    if md
        .modified()
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_none_or(|d| d >= LIFETIME)
    {
        log(root, "CLICK_REJECT_EXPIRED");
        return;
    }
    let Ok(value) = fs::read_to_string(&pending) else {
        return;
    };
    let mut parts = value.split_whitespace();
    let (Some(h), Some(pid)) = (
        parts.next().and_then(|v| v.parse::<usize>().ok()),
        parts.next().and_then(|v| v.parse::<u32>().ok()),
    ) else {
        return;
    };
    if parts.next().is_some() {
        return;
    }
    // Consume one-time ticket atomically; do not accept arbitrary HWND or title from URI.
    if fs::rename(&pending, &clicked).is_err() {
        return;
    }
    log(
        root,
        if focus(h, pid) {
            "CLICK_WINDOW_FOCUSED"
        } else {
            "CLICK_FOCUS_NOT_CONFIRMED"
        },
    );
}
pub(super) fn probe(root: &Path, o: &Observer) {
    for h in windows() {
        let ok = target(o, h).is_some();
        log(
            root,
            &format!(
                "CLICK_PROBE target={ok} empty={} hwnd={h}",
                composer_empty(o, h).unwrap_or(false)
            ),
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_token_only() {
        assert!(token_ok("0123456789abcdef0123456789abcdef"));
        for s in [
            "",
            "../foo",
            "0123456789ABCDEF0123456789abcdef",
            "0123456789abcdef0123456789abcdeg",
        ] {
            assert!(!token_ok(s));
        }
    }
    #[test]
    fn uri_strict() {
        let good = "1234567890abcdef1234567890abcdef";
        assert_eq!(token_from_uri(&format!("{PREFIX}{good}")), Some(good));
        for s in [
            "replynoti-open://open/../../bad",
            "replynoti-open://open/123",
            "replynoti-open://open/1234567890abcdef1234567890abcdef?x=1",
            "https://example.com",
        ] {
            assert!(token_from_uri(s).is_none())
        }
    }
}
