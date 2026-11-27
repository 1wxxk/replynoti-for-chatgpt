#![windows_subsystem = "windows"]
mod clicks;
mod settings;
mod tray;
// ReplyNoti: status UI Automation; optional local preview. No input interception or network.
use std::{ffi::c_void, mem::ManuallyDrop};
use windows::{
    Win32::{
        Foundation::HWND,
        System::{
            Com::{
                CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize,
            },
            Ole::{
                SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetLBound, SafeArrayGetUBound,
                SafeArrayUnaccessData,
            },
            Variant::{VARIANT, VT_BSTR, VT_I4, VariantClear},
        },
        UI::Accessibility::*,
    },
    core::{BSTR, Result},
};

struct Variant(VARIANT);
impl Variant {
    fn int(n: i32) -> Self {
        let mut v = VARIANT::default();
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_I4;
            inner.Anonymous.lVal = n;
        }
        Self(v)
    }
    fn text(s: &str) -> Self {
        let mut v = VARIANT::default();
        unsafe {
            let inner = &mut *v.Anonymous.Anonymous;
            inner.vt = VT_BSTR;
            inner.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(s));
        }
        Self(v)
    }
}
impl Drop for Variant {
    fn drop(&mut self) {
        unsafe {
            let _ = VariantClear(&mut self.0);
        }
    }
}
struct Com;
impl Com {
    fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        }
        Ok(Self)
    }
}
impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
fn runtime_id(e: &IUIAutomationElement) -> Result<Vec<i32>> {
    unsafe {
        let a = e.GetRuntimeId()?;
        if a.is_null() {
            return Ok(vec![]);
        }
        let r = (|| {
            let lo = SafeArrayGetLBound(a, 1)?;
            let hi = SafeArrayGetUBound(a, 1)?;
            let mut ptr: *mut c_void = std::ptr::null_mut();
            SafeArrayAccessData(a, &mut ptr)?;
            let values = if !ptr.is_null() && hi >= lo {
                std::slice::from_raw_parts(ptr.cast::<i32>(), (hi - lo + 1) as usize).to_vec()
            } else {
                vec![]
            };
            SafeArrayUnaccessData(a)?;
            Ok(values)
        })();
        let _ = SafeArrayDestroy(a);
        r
    }
}
fn condition(
    u: &IUIAutomation,
    prop: UIA_PROPERTY_ID,
    v: Variant,
) -> Result<IUIAutomationCondition> {
    unsafe { u.CreatePropertyCondition(prop, &v.0) }
}
fn buttons(u: &IUIAutomation, names: &[&str]) -> Result<IUIAutomationCondition> {
    let mut c = condition(u, UIA_NamePropertyId, Variant::text(names[0]))?;
    for n in &names[1..] {
        unsafe {
            c = u.CreateOrCondition(&c, &condition(u, UIA_NamePropertyId, Variant::text(n))?)?;
        }
    }
    unsafe {
        u.CreateAndCondition(
            &condition(u, UIA_ControlTypePropertyId, Variant::int(50000))?,
            &c,
        )
    }
}
fn find(
    root: &IUIAutomationElement,
    c: &IUIAutomationCondition,
) -> Result<Option<IUIAutomationElement>> {
    match unsafe { root.FindFirst(TreeScope_Descendants, c) } {
        Ok(e) => Ok(Some(e)),
        Err(e) if matches!(e.code().0 as u32, 0 | 0x80004003) => Ok(None),
        Err(e) => Err(e),
    }
}

use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::DateTime,
    UI::Notifications::{NotificationSetting, ToastNotification, ToastNotificationManager},
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, LPARAM},
        System::{
            Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RRF_RT_REG_QWORD, RegGetValueW},
            RemoteDesktop::{ProcessIdToSessionId, WTSGetActiveConsoleSessionId},
            Threading::{
                CreateMutexW, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        },
    },
    core::{BOOL, HSTRING, PCWSTR, PWSTR, w},
};
const APP_ID: &str = "ReplyNoti.Notifier";
const NATIVE_IDS: [&str; 2] = [
    "OpenAI.Codex_2p2nqsd0c76g0!App",
    "OpenAI.ChatGPT-Desktop_2p2nqsd0c76g0!ChatGPT",
];
const REG_BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings";
const GRACE: Duration = Duration::from_millis(1250);
const IDLE_POLL: Duration = Duration::from_millis(800);
const BUSY_POLL: Duration = Duration::from_millis(200);

fn root_path() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("portable.flag").is_file() {
                return dir.to_path_buf();
            }
        }
    }
    PathBuf::from(env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA")).join("ReplyNoti")
}
fn log(root: &Path, msg: &str) {
    let path = root.join("replynoti.log");
    if fs::metadata(&path)
        .map(|m| m.len() > 262144)
        .unwrap_or(false)
    {
        let _ = fs::remove_file(root.join("replynoti.log.2"));
        let _ = fs::rename(root.join("replynoti.log.1"), root.join("replynoti.log.2"));
        let _ = fs::rename(&path, root.join("replynoti.log.1"));
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(f, "{stamp} {msg}");
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
fn reg_value(app: &str, name: &str, qword: bool) -> Option<u64> {
    let path = wide(&format!("{REG_BASE}\\{app}"));
    let name = wide(name);
    let mut val: u64 = 0;
    let mut bytes = if qword { 8 } else { 4 };
    let code = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(path.as_ptr()),
            PCWSTR(name.as_ptr()),
            if qword {
                RRF_RT_REG_QWORD
            } else {
                RRF_RT_REG_DWORD
            },
            None,
            Some((&mut val as *mut u64).cast()),
            Some(&mut bytes),
        )
    };
    if code.0 == 0 { Some(val) } else { None }
}
fn last_toast(ids: &[&str]) -> u64 {
    ids.iter()
        .filter_map(|id| reg_value(id, "LastNotificationAddedTime", true))
        .max()
        .unwrap_or(0)
}
fn native_stamp() -> u64 {
    last_toast(&NATIVE_IDS)
}
fn console_session() -> bool {
    let mut sid = 0u32;
    unsafe {
        ProcessIdToSessionId(std::process::id(), &mut sid).is_ok()
            && sid != 0
            && sid != u32::MAX
            && sid == WTSGetActiveConsoleSessionId()
    }
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn mutex() -> Option<Handle> {
    let h = unsafe { CreateMutexW(None, false, w!("Local\\ReplyNoti.Notifier")).ok()? };
    let exists = unsafe { GetLastError() == ERROR_ALREADY_EXISTS };
    let h = Handle(h);
    if exists { None } else { Some(h) }
}
fn window_pid(h: HWND) -> Option<u32> {
    if h.0.is_null() {
        return None;
    }
    let mut pid = 0;
    let tid = unsafe { GetWindowThreadProcessId(h, Some(&mut pid)) };
    (tid != 0 && pid != 0).then_some(pid)
}
fn official_chatgpt_process(pid: u32) -> Option<bool> {
    unsafe {
        let p = Handle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
        let mut buf = vec![0u16; 32768];
        let mut len = buf.len() as u32;
        QueryFullProcessImageNameW(p.0, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
            .ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
        Some(path.contains("\\windowsapps\\openai.codex_") && path.ends_with("\\app\\chatgpt.exe"))
    }
}
fn valid_window(h: HWND) -> bool {
    unsafe {
        if !IsWindowVisible(h).as_bool() {
            return false;
        }
        let mut title = [0u16; 128];
        let n = GetWindowTextW(h, &mut title);
        if n <= 0 || String::from_utf16_lossy(&title[..n as usize]) != "ChatGPT" {
            return false;
        }
    }
    window_pid(h).and_then(official_chatgpt_process) == Some(true)
}
fn windows() -> Vec<usize> {
    unsafe extern "system" fn collect(h: HWND, p: LPARAM) -> BOOL {
        if valid_window(h) {
            unsafe { (&mut *(p.0 as *mut Vec<usize>)).push(h.0 as usize) }
        }
        BOOL(1)
    }
    let mut result = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect),
            LPARAM((&mut result as *mut Vec<usize>) as isize),
        );
    }
    result
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Identity {
    transcript: Vec<i32>,
    user: Vec<i32>,
}
#[derive(Clone, Debug)]
struct Context {
    id: Identity,
    copies: HashSet<Vec<i32>>,
}
struct Anchor {
    meta: Context,
    element: IUIAutomationElement,
}
#[derive(Clone, Debug)]
struct Snapshot {
    composer: Vec<i32>,
    busy: bool,
}
struct Observer {
    u: IUIAutomation,
    mode: IUIAutomationCondition,
    voice: IUIAutomationCondition,
    attach: IUIAutomationCondition,
    stop: IUIAutomationCondition,
    user_copy: IUIAutomationCondition,
    text: IUIAutomationCondition,
}
fn class(e: &IUIAutomationElement) -> String {
    unsafe {
        e.CurrentClassName()
            .map(|x| x.to_string())
            .unwrap_or_default()
    }
}
fn name(e: &IUIAutomationElement) -> String {
    unsafe { e.CurrentName().map(|x| x.to_string()).unwrap_or_default() }
}
fn kind(e: &IUIAutomationElement) -> i32 {
    unsafe { e.CurrentControlType().map(|x| x.0).unwrap_or(0) }
}
impl Observer {
    fn new() -> Result<Self> {
        let u: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)? };
        Ok(Self {
            mode: buttons(
                &u,
                &[
                    "모드 전환, 현재 모드: ChatGPT",
                    "Mode switch, current mode: ChatGPT",
                    "Switch mode, current mode: ChatGPT",
                ],
            )?,
            voice: buttons(&u, &["음성 입력", "Voice input", "Dictate"])?,
            attach: buttons(&u, &["파일 등 추가", "Add files and more", "Add files"])?,
            stop: buttons(&u, &["중지", "Stop", "Stop generating"])?,
            user_copy: buttons(&u, &["메시지 복사", "Copy message"])?,
            text: condition(&u, UIA_ControlTypePropertyId, Variant::int(50020))?,
            u,
        })
    }
    fn root(&self, h: usize) -> Result<IUIAutomationElement> {
        unsafe { self.u.ElementFromHandle(HWND(h as *mut c_void)) }
    }
    fn observe(&self, h: usize) -> Result<Option<Snapshot>> {
        unsafe {
            let root = self.root(h)?;
            if find(&root, &self.mode)?.is_none() {
                return Ok(None);
            }
            let Some(v) = find(&root, &self.voice)? else {
                return Ok(None);
            };
            let Some(a) = find(&root, &self.attach)? else {
                return Ok(None);
            };
            let w = self.u.RawViewWalker()?;
            let p = w.GetParentElement(&v)?;
            let q = w.GetParentElement(&a)?;
            let composer = runtime_id(&p)?;
            if composer.is_empty() || composer != runtime_id(&q)? {
                return Ok(None);
            }
            Ok(Some(Snapshot {
                composer,
                busy: find(&p, &self.stop)?.is_some(),
            }))
        }
    }
    fn anchors(&self, h: usize) -> Result<Vec<Anchor>> {
        unsafe {
            let root = self.root(h)?;
            let w = self.u.RawViewWalker()?;
            let arr = root.FindAll(TreeScope_Descendants, &self.user_copy)?;
            let len = arr.Length()?;
            let mut entries = Vec::new();
            for i in (len - 24).max(0)..len {
                let e = arr.GetElement(i)?;
                let Ok(prev) = w.GetPreviousSiblingElement(&e) else {
                    continue;
                };
                if !class(&prev).contains("bg-user-message") {
                    continue;
                }
                let user = runtime_id(&e)?;
                if user.is_empty() {
                    continue;
                }
                let mut p = w.GetParentElement(&e).ok();
                let mut transcript = None;
                for _ in 0..7 {
                    let Some(x) = p else { break };
                    if class(&x).contains("transcriptContent-") {
                        transcript = Some(runtime_id(&x)?);
                        break;
                    }
                    p = w.GetParentElement(&x).ok();
                }
                if let Some(transcript) = transcript {
                    if !transcript.is_empty() {
                        entries.push((Identity { transcript, user }, e));
                    }
                }
            }
            let mut result = Vec::new();
            for (id, e) in &entries {
                let copies = entries
                    .iter()
                    .filter(|(other, _)| other.transcript == id.transcript)
                    .map(|(other, _)| other.user.clone())
                    .collect();
                result.push(Anchor {
                    meta: Context {
                        id: id.clone(),
                        copies,
                    },
                    element: e.clone(),
                });
            }
            Ok(result)
        }
    }
    fn context(&self, h: usize) -> Result<Option<Anchor>> {
        Ok(self.anchors(h)?.pop())
    }
    // Status-only scans never read reply content.
    fn answer_ready(&self, a: &Anchor) -> Result<bool> {
        Ok(self.answer(a, false)?.0)
    }
    fn answer(&self, a: &Anchor, preview: bool) -> Result<(bool, String)> {
        unsafe {
            let w = self.u.RawViewWalker()?;
            let mut scan = AnswerScan {
                header: false,
                finished: false,
                renderer: false,
                renderers: Vec::new(),
                collect_renderers: preview,
                budget: 160,
            };
            let mut next = w.GetNextSiblingElement(&a.element).ok();
            while let Some(node) = next {
                if scan.budget == 0 || class(&node).contains("bg-user-message") {
                    break;
                }
                scan.visit(&w, &node, 0);
                next = w.GetNextSiblingElement(&node).ok();
            }
            let ready = scan.header && scan.finished && scan.renderer;
            if !ready || !preview {
                return Ok((ready, String::new()));
            }
            let mut parts = Vec::new();
            let mut count = 0;
            for renderer in scan.renderers.iter().take(5) {
                let nodes = renderer.FindAll(TreeScope_Descendants, &self.text)?;
                for i in 0..nodes.Length()?.min(12) {
                    let el = nodes.GetElement(i)?;
                    // Skip screen-reader headers; extract at most a bounded prefix.
                    if class(&el).contains("sr-only") {
                        continue;
                    }
                    let value = el.CurrentName()?.to_string();
                    let value = value.trim();
                    if !value.is_empty()
                        && !matches!(
                            value,
                            "ChatGPT 답변:" | "ChatGPT said:" | "ChatGPT response:"
                        )
                    {
                        let part: String = value.chars().take(256).collect();
                        count += part.chars().count();
                        parts.push(part);
                    }
                    if count >= 200 {
                        break;
                    }
                }
                if count >= 200 {
                    break;
                }
            }
            Ok((true, safe_preview(&parts.join(" "))))
        }
    }
}

struct AnswerScan {
    header: bool,
    finished: bool,
    renderer: bool,
    renderers: Vec<IUIAutomationElement>,
    collect_renderers: bool,
    budget: usize,
}
impl AnswerScan {
    fn visit(&mut self, w: &IUIAutomationTreeWalker, node: &IUIAutomationElement, depth: usize) {
        if self.budget == 0 {
            return;
        }
        self.budget -= 1;
        let cl = class(node);
        let typ = kind(node);
        if typ == 50020 && cl.contains("sr-only") {
            if matches!(
                name(node).as_str(),
                "ChatGPT 답변:" | "ChatGPT said:" | "ChatGPT response:"
            ) {
                self.header = true
            }
            return;
        }
        if cl.contains("Renderer-") {
            if self.header {
                self.renderer = true;
                if self.collect_renderers && self.renderers.len() < 5 {
                    self.renderers.push(node.clone());
                }
            }
            return;
        }
        if typ == 50000 {
            if self.header && matches!(name(node).as_str(), "소리 내어 읽기" | "Read aloud") {
                self.finished = true
            }
            return;
        }
        if depth < 5
            && typ == 50026
            && (cl.contains("text-size-chat")
                || cl.contains("relative shrink-0")
                || cl.contains("absolute inset-x-0"))
        {
            let mut child = unsafe { w.GetFirstChildElement(node).ok() };
            while let Some(q) = child {
                if self.budget == 0 {
                    break;
                }
                self.visit(w, &q, depth + 1);
                child = unsafe { w.GetNextSiblingElement(&q).ok() };
            }
        }
    }
}
fn preview_enabled(root: &Path) -> bool {
    settings::Config::load(root).preview
}
fn safe_preview(s: &str) -> String {
    static FILTER: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() < 8 || s.chars().any(char::is_control) {
        return String::new();
    }
    let filter = FILTER.get_or_init(|| regex::Regex::new(r"(?i)(password|passwd|api.?key|access.?token|secret|authorization|bearer|비밀번호|인증키|토큰|주민등록|개인정보|카드번호|계좌번호|[\w.+-]+@[\w.-]+\.[a-z]{2,}|\b\d{6}-[1-4]\d{6}\b|\b(?:\d[ -]?){12,19}\b|\b(?:sk-|ghp_|gho_|github_pat_|eyJ)[a-z0-9_-]{8,}|https?://\S+|\b[A-Za-z0-9_-]{32,}\b)").unwrap());
    if filter.is_match(&s) {
        return String::new();
    }
    let mut result = String::new();
    let mut units = 0;
    for ch in s.chars() {
        if units + ch.len_utf16() > 150 {
            break;
        }
        result.push(ch);
        units += ch.len_utf16();
    }
    if result.len() < s.len() {
        result = result.trim_end().to_owned();
        result.push('…');
    }
    result
}
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn file_uri(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file:///");
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            out.push(b as char)
        } else {
            out.push_str(&format!("%{b:02X}"))
        }
    }
    out
}
struct WinRt;
impl WinRt {
    fn new() -> Result<Self> {
        unsafe {
            RoInitialize(RO_INIT_MULTITHREADED)?;
        }
        Ok(Self)
    }
}
impl Drop for WinRt {
    fn drop(&mut self) {
        unsafe {
            RoUninitialize();
        }
    }
}
fn toast(
    root: &Path,
    test: bool,
    verified: bool,
    preview: &str,
    launch: Option<&str>,
) -> std::result::Result<(), String> {
    toast_guarded(root, test, verified, preview, launch, || Delivery::Send).map(|_| ())
}
fn toast_guarded(
    root: &Path,
    test: bool,
    verified: bool,
    preview: &str,
    launch: Option<&str>,
    mut guard: impl FnMut() -> Delivery,
) -> std::result::Result<Delivery, String> {
    use windows::{
        Foundation::{IReference, PropertyValue},
        core::Interface,
    };
    if !console_session() {
        return Err("NOT_CONSOLE".into());
    }
    if reg_value(APP_ID, "Enabled", false) == Some(0) {
        return Err("NOTIFICATIONS_DISABLED".into());
    }
    let config = settings::Config::load(root);
    if !test && !config.enabled {
        return Ok(Delivery::Drop);
    }
    let _rt = WinRt::new().map_err(|e| format!("WINRT_{:08x}", e.code().0 as u32))?;
    let before = last_toast(&[APP_ID]);
    let mut send = || -> Result<Delivery> {
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_ID))?;
        if let Ok(setting) = notifier.Setting() {
            if setting != NotificationSetting::Enabled {
                return Err(windows::core::Error::from_hresult(windows::core::HRESULT(
                    0x80070005u32 as i32,
                )));
            }
        }
        let (_title, body) = if test {
            (
                "ReplyNoti test",
                "Windows notification is working. / 알림 테스트입니다.",
            )
        } else if verified {
            (
                "ReplyNoti",
                "A ChatGPT reply is ready. / 답변이 준비되었습니다.",
            )
        } else {
            (
                "ReplyNoti",
                "ChatGPT generation ended. / 답변 생성이 종료되었습니다.",
            )
        };
        let title = if test {
            format!("{} test", config.display_name)
        } else {
            config.display_name.clone()
        };
        // Recheck consent immediately before submitting the toast.
        let body = if verified && config.preview && !preview.is_empty() {
            settings::shorten(preview, config.length)
        } else {
            body.to_owned()
        };
        let icon = config.icon(root, "png");
        let image = if icon.is_file() {
            format!(
                "<image placement=\"appLogoOverride\" src=\"{}\"/>",
                escape_xml(&file_uri(&icon))
            )
        } else {
            String::new()
        };
        let audio = if config.sound != "default" {
            "<audio silent=\"true\"/>"
        } else {
            "<audio src=\"ms-winsoundevent:Notification.Default\"/>"
        };
        let activation = launch
            .map(|uri| {
                format!(
                    " activationType=\"protocol\" launch=\"{}\"",
                    escape_xml(uri)
                )
            })
            .unwrap_or_default();
        let xml = format!(
            "<toast{activation}><visual><binding template=\"ToastGeneric\">{}<text>{}</text><text>{}</text></binding></visual>{}</toast>",
            image,
            escape_xml(&title),
            escape_xml(&body),
            audio
        );
        let doc = XmlDocument::new()?;
        doc.LoadXml(&HSTRING::from(xml))?;
        let t = ToastNotification::CreateToastNotification(&doc)?;
        let sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let exp = PropertyValue::CreateDateTime(DateTime {
            UniversalTime: (sec + 11644473600 + 300) * 10_000_000,
        })?;
        let exp: IReference<DateTime> = exp.cast()?;
        t.SetExpirationTime(&exp)?;
        let decision = guard();
        if decision != Delivery::Send {
            return Ok(decision);
        }
        notifier.Show(&t)?;
        Ok(Delivery::Send)
    };
    let decision = send().map_err(|e| format!("TOAST_{:08x}", e.code().0 as u32))?;
    if decision != Delivery::Send {
        return Ok(decision);
    }
    log(root, "TOAST_SUBMITTED");
    for _ in 0..12 {
        thread::sleep(Duration::from_millis(150));
        if last_toast(&[APP_ID]) > before {
            log(root, "TOAST_REGISTERED");
            if config.sound == "custom" {
                if guard() == Delivery::Send {
                    settings::play_custom(root);
                } else {
                    log(root, "CUSTOM_SOUND_FOCUSED_SKIP");
                }
            }
            return Ok(Delivery::Send);
        }
    }
    // Registry timing is an undocumented diagnostic, not a delivery guarantee.
    log(root, "TOAST_REGISTRATION_UNCONFIRMED");
    Ok(Delivery::Send)
}
#[derive(Clone, Debug)]
struct Active {
    composer: Vec<i32>,
    anchor: Option<Identity>,
    baseline: u64,
    seen_unfinished: bool,
    idle: u8,
    pending: Option<Instant>,
}
#[derive(Default)]
struct Machine {
    active: Option<Active>,
    prior: Option<Identity>,
    prior_stamp: u64,
    notified: Option<Identity>,
}
#[derive(Debug, PartialEq, Eq)]
enum Event {
    None,
    Busy,
    Pending,
    Short,
    Reset,
    Native,
    Complete,
}
impl Machine {
    fn sample(
        &mut self,
        snap: Option<&Snapshot>,
        anchor: Option<&Context>,
        stamp: u64,
        now: Instant,
        ready: Option<bool>,
    ) -> Event {
        let Some(s) = snap else {
            self.prior = None;
            return if self.active.take().is_some() {
                Event::Reset
            } else {
                Event::None
            };
        };
        let id = anchor.map(|a| a.id.clone());
        if s.busy {
            let fresh = self.active.as_ref().is_none_or(|a| {
                a.composer != s.composer
                    || a.anchor
                        .as_ref()
                        .zip(id.as_ref())
                        .is_some_and(|(x, y)| x.transcript != y.transcript)
            });
            if fresh {
                self.active = Some(Active {
                    composer: s.composer.clone(),
                    anchor: id.clone(),
                    baseline: stamp,
                    seen_unfinished: id.is_some() && ready == Some(false),
                    idle: 0,
                    pending: None,
                });
                return Event::Busy;
            }
            let a = self.active.as_mut().unwrap();
            if id.is_some() {
                if a.anchor != id {
                    a.seen_unfinished = false;
                }
                a.anchor = id;
                if ready == Some(false) {
                    a.seen_unfinished = true;
                }
            }
            a.idle = 0;
            a.pending = None;
            return Event::None;
        }
        if let Some(a) = &mut self.active {
            if a.composer != s.composer
                || a.anchor
                    .as_ref()
                    .zip(id.as_ref())
                    .is_some_and(|(x, y)| x != y)
            {
                self.active = None;
                self.prior = id;
                self.prior_stamp = stamp;
                return Event::Reset;
            }
            // Never trust a missing conversation anchor when a known anchor was previously observed.
            if a.anchor.is_some() && id.is_none() {
                a.idle = 0;
                a.pending = None;
                return Event::None;
            }
            if a.anchor.is_none() && id.is_some() {
                a.anchor = id.clone()
            }
            if id.is_some() && ready == Some(false) {
                a.seen_unfinished = true;
            }
            let new_turn = anchor.is_some_and(|n| {
                self.prior.as_ref().is_some_and(|p| {
                    p.transcript == n.id.transcript
                        && p.user != n.id.user
                        && n.copies.contains(&p.user)
                })
            });
            if id.is_none() || ready != Some(true) || (!a.seen_unfinished && !new_turn) {
                a.idle = 0;
                a.pending = None;
                return Event::None;
            }
            a.idle = a.idle.saturating_add(1);
            if a.idle < 2 {
                return Event::None;
            }
            match a.pending {
                None => {
                    a.pending = Some(now + GRACE);
                    return Event::Pending;
                }
                Some(t) if now < t => return Event::None,
                _ => {}
            }
            let a = self.active.take().unwrap();
            self.prior = id.clone();
            self.prior_stamp = stamp;
            if let Some(x) = id {
                self.notified = Some(x)
            }
            return if stamp > a.baseline {
                Event::Native
            } else {
                Event::Complete
            };
        }
        // Short response: a new user anchor in the same transcript, after a previously
        // observed idle turn that is still present. The actual reply actions must exist.
        let short = ready == Some(true)
            && anchor.is_some_and(|n| {
                self.prior.as_ref().is_some_and(|p| {
                    p.transcript == n.id.transcript
                        && p.user != n.id.user
                        && n.copies.contains(&p.user)
                }) && self.notified.as_ref() != Some(&n.id)
            });
        if short {
            self.active = Some(Active {
                composer: s.composer.clone(),
                anchor: id,
                baseline: self.prior_stamp,
                seen_unfinished: false,
                idle: 1,
                pending: None,
            });
            return Event::Short;
        }
        self.prior = id;
        self.prior_stamp = stamp;
        Event::None
    }
    fn short_candidate(&self, a: Option<&Context>) -> bool {
        self.active.is_none()
            && a.is_some_and(|n| {
                self.prior.as_ref().is_some_and(|p| {
                    p.transcript == n.id.transcript
                        && p.user != n.id.user
                        && n.copies.contains(&p.user)
                }) && self.notified.as_ref() != Some(&n.id)
            })
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Delivery {
    Send,
    Hold,
    Drop,
}
fn delivery(mode: &str, focused: Option<bool>, idle_ms: Option<u32>, threshold: u32) -> Delivery {
    match (mode, focused) {
        ("always", _) | (_, Some(false)) => Delivery::Send,
        ("background", Some(true)) => Delivery::Drop,
        ("smart", Some(true)) if idle_ms.is_some_and(|ms| ms >= threshold * 1000) => Delivery::Send,
        _ => Delivery::Hold,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusReason {
    SameWindow,
    OwnedWindow,
    SameProcess,
    OfficialApp,
    OtherApp,
    Unknown,
}
impl FocusReason {
    fn focused(self) -> Option<bool> {
        match self {
            Self::SameWindow | Self::OwnedWindow | Self::SameProcess | Self::OfficialApp => {
                Some(true)
            }
            Self::OtherApp => Some(false),
            Self::Unknown => None,
        }
    }
}
#[derive(Clone, Copy)]
struct Activity {
    foreground: usize,
    reason: FocusReason,
    idle: Option<u32>,
}
fn classify_focus(
    same_window: bool,
    same_owner: bool,
    same_process: bool,
    official: Option<bool>,
) -> FocusReason {
    if same_window {
        FocusReason::SameWindow
    } else if same_owner {
        FocusReason::OwnedWindow
    } else if same_process {
        FocusReason::SameProcess
    } else {
        match official {
            Some(true) => FocusReason::OfficialApp,
            Some(false) => FocusReason::OtherApp,
            None => FocusReason::Unknown,
        }
    }
}
fn user_activity(h: usize) -> Activity {
    use windows::Win32::{
        System::SystemInformation::GetTickCount,
        UI::{
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            WindowsAndMessaging::{GA_ROOTOWNER, GetAncestor, GetForegroundWindow},
        },
    };
    unsafe {
        let watched = HWND(h as *mut c_void);
        let foreground = GetForegroundWindow();
        let reason = if foreground.0.is_null() || window_pid(watched).is_none() {
            FocusReason::Unknown
        } else {
            let owner = GetAncestor(foreground, GA_ROOTOWNER);
            let watched_owner = GetAncestor(watched, GA_ROOTOWNER);
            let fg_pid = window_pid(foreground);
            let watched_pid = window_pid(watched);
            let owner_pid = window_pid(owner);
            let same_window = foreground == watched;
            let same_owner =
                !owner.0.is_null() && !watched_owner.0.is_null() && owner == watched_owner;
            let same_process =
                watched_pid.is_some_and(|pid| fg_pid == Some(pid) || owner_pid == Some(pid));
            let official = if same_window || same_owner || same_process {
                None
            } else {
                let fg = fg_pid.and_then(official_chatgpt_process);
                let root_owner = if owner == foreground {
                    fg
                } else {
                    owner_pid.and_then(official_chatgpt_process)
                };
                match (fg, root_owner) {
                    (Some(true), _) | (_, Some(true)) => Some(true),
                    (Some(false), Some(false)) => Some(false),
                    _ => None,
                }
            };
            classify_focus(same_window, same_owner, same_process, official)
        };
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let idle = GetLastInputInfo(&mut info)
            .as_bool()
            .then(|| GetTickCount().wrapping_sub(info.dwTime));
        // If foreground changed during the process/owner query, this sample is not reliable.
        let reason = if GetForegroundWindow() == foreground {
            reason
        } else {
            FocusReason::Unknown
        };
        Activity {
            foreground: foreground.0 as usize,
            reason,
            idle,
        }
    }
}
const BACKGROUND_STABLE: Duration = Duration::from_millis(600);
#[derive(Default)]
struct FocusGate {
    background: Option<(usize, Instant)>,
}
impl FocusGate {
    fn decide(&mut self, config: &settings::Config, activity: Activity, now: Instant) -> Delivery {
        let focused = activity.reason.focused();
        if focused == Some(false) {
            if !self
                .background
                .is_some_and(|(h, _)| h == activity.foreground)
            {
                self.background = Some((activity.foreground, now));
            }
        } else {
            self.background = None;
        }
        let result = delivery(
            &config.notification_mode,
            focused,
            activity.idle,
            config.idle_seconds,
        );
        if config.notification_mode != "always"
            && focused == Some(false)
            && result == Delivery::Send
            && !self
                .background
                .is_some_and(|(_, since)| now.saturating_duration_since(since) >= BACKGROUND_STABLE)
        {
            Delivery::Hold
        } else {
            result
        }
    }
}
fn focus_decision(
    root: &Path,
    h: usize,
    gate: &mut FocusGate,
    config: &settings::Config,
    stage: &str,
    record: bool,
) -> Delivery {
    let activity = user_activity(h);
    let result = gate.decide(config, activity, Instant::now());
    if record || result != Delivery::Hold {
        log(
            root,
            &format!(
                "FOCUS stage={stage} mode={} reason={:?} decision={result:?}",
                config.notification_mode, activity.reason
            ),
        );
    }
    result
}
struct Deferred {
    id: Identity,
    stamp: u64,
    at: Instant,
    focus: FocusGate,
}
struct Watch {
    machine: Machine,
    anchor: Option<Anchor>,
    checked: Instant,
    deferred: Option<Deferred>,
}
fn monitor(root: &Path, o: &Observer, shadow: bool, limit: Option<Duration>) -> Result<()> {
    let start = Instant::now();
    let mut states: HashMap<usize, Watch> = HashMap::new();
    let mut click_state = if shadow {
        None
    } else {
        Some(clicks::Clicks::new(root))
    };
    let mut last_error = Instant::now() - Duration::from_secs(31);
    log(
        root,
        &format!(
            "START mode={} pid={} v0.1.0-beta.10",
            if shadow { "shadow" } else { "production" },
            std::process::id()
        ),
    );
    while console_session()
        && !root.join("stop.flag").exists()
        && limit.is_none_or(|d| start.elapsed() < d)
    {
        let t = Instant::now();
        let config = settings::Config::load(root);
        if let Some(clicks) = click_state.as_mut() {
            clicks.drain(root, o);
        }
        if !config.enabled && !shadow {
            states.clear();
            thread::sleep(IDLE_POLL);
            continue;
        }
        let handles = windows();
        states.retain(|h, _| handles.contains(h));
        let stamp = native_stamp();
        for h in handles {
            let state = states.entry(h).or_insert_with(|| Watch {
                machine: Machine::default(),
                anchor: None,
                checked: t - Duration::from_secs(2),
                deferred: None,
            });
            let snap = match o.observe(h) {
                Ok(x) => x,
                Err(e) => {
                    if last_error.elapsed() > Duration::from_secs(30) {
                        log(root, &format!("UIA_ERROR {:08x}", e.code().0 as u32));
                        last_error = Instant::now()
                    }
                    None
                }
            };
            if let Some(ref s) = snap {
                let active = state.machine.active.as_ref();
                let refresh = state.checked.elapsed() >= Duration::from_secs(1)
                    || (s.busy && active.is_none())
                    || (!s.busy && active.is_some());
                if refresh {
                    state.anchor = o.context(h).unwrap_or(None);
                    state.checked = Instant::now();
                }
            } else {
                state.anchor = None
            }
            let track = snap.as_ref().is_some_and(|s| s.busy)
                || state.machine.active.is_some()
                || state.deferred.is_some()
                || state
                    .machine
                    .short_candidate(state.anchor.as_ref().map(|a| &a.meta));
            let ready = if track {
                state.anchor.as_ref().and_then(|a| o.answer_ready(a).ok())
            } else {
                None
            };
            let event = state.machine.sample(
                snap.as_ref(),
                state.anchor.as_ref().map(|a| &a.meta),
                stamp,
                Instant::now(),
                ready,
            );
            if event != Event::None {
                log(root, &format!("STATE {:?} hwnd={h}", event))
            }
            if event == Event::Complete {
                log(root, &format!("GENERATION_ENDED verified=true hwnd={h}"));
                if let Some(a) = state.anchor.as_ref() {
                    state.deferred = Some(Deferred {
                        id: a.meta.id.clone(),
                        stamp,
                        at: Instant::now(),
                        focus: FocusGate::default(),
                    });
                }
            }
            if let Some(d) = &mut state.deferred {
                let still_same = state.anchor.as_ref().is_some_and(|a| a.meta.id == d.id);
                let resumed = snap.as_ref().is_some_and(|s| s.busy);
                if !still_same
                    || resumed
                    || d.at.elapsed() >= Duration::from_secs(300)
                    || stamp > d.stamp
                {
                    state.deferred = None;
                    log(root, "DEFERRED_DISCARDED");
                    continue;
                }
                if ready != Some(true) || snap.is_none() {
                    continue;
                }
                match focus_decision(
                    root,
                    h,
                    &mut d.focus,
                    &config,
                    "candidate",
                    event == Event::Complete,
                ) {
                    Delivery::Hold => {
                        if event == Event::Complete {
                            log(root, "NOTIFICATION_DEFERRED");
                        }
                        continue;
                    }
                    Delivery::Drop => {
                        state.deferred = None;
                        log(root, "NOTIFICATION_FOCUSED_SKIP");
                        continue;
                    }
                    Delivery::Send => {}
                }
                let mut d = state.deferred.take().unwrap();
                let stamp = d.stamp;
                let verified = true;
                if !shadow {
                    // Recheck native stamp to reduce races between completion and display.
                    if native_stamp() > stamp {
                        log(root, "NATIVE_LATE_SKIP")
                    } else {
                        let mut preview = String::new();
                        if verified && preview_enabled(root) {
                            // Reacquire the current anchor and require it to match the completed turn.
                            if let (Some(previous), Ok(Some(current))) =
                                (state.anchor.as_ref(), o.context(h))
                            {
                                if previous.meta.id == current.meta.id
                                    && o.observe(h).ok().flatten().is_some_and(|s| !s.busy)
                                {
                                    if let Ok((true, text)) = o.answer(&current, true) {
                                        preview = text;
                                    }
                                }
                            }
                        }
                        log(
                            root,
                            if preview.is_empty() {
                                "PREVIEW generic"
                            } else {
                                "PREVIEW included"
                            },
                        );
                        if native_stamp() > stamp {
                            log(root, "NATIVE_LATE_SKIP");
                        } else {
                            let exact = config.navigate
                                && verified
                                && state.anchor.as_ref().is_some_and(|a| {
                                    o.context(h)
                                        .ok()
                                        .flatten()
                                        .is_some_and(|fresh| fresh.meta.id == a.meta.id)
                                })
                                && o.observe(h).ok().flatten().is_some_and(|s| !s.busy);
                            let latest = settings::Config::load(root);
                            match focus_decision(root, h, &mut d.focus, &latest, "prepare", true) {
                                Delivery::Hold => {
                                    state.deferred = Some(d);
                                    continue;
                                }
                                Delivery::Drop => {
                                    log(root, "NOTIFICATION_FOCUSED_SKIP");
                                    continue;
                                }
                                Delivery::Send => {}
                            }
                            if !o.observe(h).ok().flatten().is_some_and(|s| !s.busy)
                                || !o.context(h).ok().flatten().is_some_and(|a| {
                                    a.meta.id == d.id && o.answer_ready(&a).unwrap_or(false)
                                })
                            {
                                state.deferred = Some(d);
                                continue;
                            }
                            let launch = click_state
                                .as_mut()
                                .and_then(|c| c.issue(root, o, h, exact));
                            let result = toast_guarded(
                                root,
                                false,
                                verified,
                                &preview,
                                launch.as_deref(),
                                || {
                                    let latest = settings::Config::load(root);
                                    if !latest.enabled
                                        || root.join("stop.flag").exists()
                                        || native_stamp() > stamp
                                    {
                                        log(root, "NOTIFICATION_FINAL_CANCEL");
                                        return Delivery::Drop;
                                    }
                                    focus_decision(root, h, &mut d.focus, &latest, "show", true)
                                },
                            );
                            match result {
                                Ok(Delivery::Send) => {}
                                other => {
                                    if let (Some(c), Some(uri)) =
                                        (click_state.as_mut(), launch.as_deref())
                                    {
                                        c.revoke(uri);
                                    }
                                    match other {
                                        Ok(Delivery::Hold) => state.deferred = Some(d),
                                        Ok(Delivery::Drop) => log(root, "NOTIFICATION_FINAL_SKIP"),
                                        Err(e) => log(root, &e),
                                        Ok(Delivery::Send) => unreachable!(),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let interval = if states
            .values()
            .any(|s| s.machine.active.is_some() || s.deferred.is_some())
        {
            BUSY_POLL
        } else {
            IDLE_POLL
        };
        thread::sleep(
            interval
                .saturating_sub(t.elapsed())
                .max(Duration::from_millis(50)),
        );
    }
    log(root, "STOP");
    Ok(())
}
fn run() -> std::result::Result<(), String> {
    let root = root_path();
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("--run");
    if !console_session() {
        return Ok(());
    }
    if mode == "--activate" {
        if args.len() == 3 && root.join(".replynoti-install").is_file() {
            clicks::activate(&root, &args[2]);
        }
        return Ok(());
    }
    fs::create_dir_all(&root).map_err(|_| "DATA_DIR_FAILED".to_string())?;
    if mode == "--enable-preview" || mode == "--disable-preview" {
        if !root.join(".replynoti-install").is_file() {
            return Err("INSTALL_REQUIRED".into());
        }
        let mut c = settings::Config::load(&root);
        c.preview = mode == "--enable-preview";
        c.save(&root)
            .map_err(|_| "PREVIEW_SETTING_FAILED".to_string())?;
        log(
            &root,
            if mode == "--enable-preview" {
                "PREVIEW_ENABLED"
            } else {
                "PREVIEW_DISABLED"
            },
        );
        return Ok(());
    }
    if mode == "--ui-self-test" {
        let Some(_lock) = mutex() else {
            return Err("STOP_WATCHER_FOR_UI_TEST".into());
        };
        return tray::run(false, true).map_err(|e| format!("GUI_{:08x}", e.code().0 as u32));
    }
    if mode == "--import-icon" || mode == "--import-sound" {
        if args.len() != 3 || !root.join(".replynoti-install").is_file() {
            return Err("INSTALL_AND_LOCAL_FILE_REQUIRED".into());
        }
        let mut c = settings::Config::load(&root);
        if mode == "--import-icon" {
            settings::import_icon(&root, Path::new(&args[2]))?;
            c.custom_icon = true;
        } else {
            settings::import_sound(&root, Path::new(&args[2]))?;
            c.sound = "custom".into();
        }
        c.save(&root)
            .map_err(|_| "CONFIG_WRITE_FAILED".to_string())?;
        return Ok(());
    }
    if mode == "--import-installed-icon" || mode == "--import-installed-sound" {
        if !root.join(".replynoti-install").is_file() {
            return Err("INSTALL_REQUIRED".into());
        }
        let icon = mode == "--import-installed-icon";
        settings::import_installed(&root, icon)?;
        let mut c = settings::Config::load(&root);
        if icon {
            c.custom_icon = true
        } else {
            c.sound = "custom".into()
        }
        c.save(&root)
            .map_err(|_| "CONFIG_WRITE_FAILED".to_string())?;
        return Ok(());
    }
    if mode == "--set-display-name" {
        if args.len() != 3 || !root.join(".replynoti-install").is_file() {
            return Err("INSTALL_AND_NAME_REQUIRED".into());
        }
        let mut c = settings::Config::load(&root);
        let name = settings::valid_name(&args[2])?;
        settings::set_display_name(&name)?;
        c.display_name = name;
        c.save(&root)
            .map_err(|_| "CONFIG_WRITE_FAILED".to_string())?;
        return Ok(());
    }
    if mode == "--test-toast" {
        toast(
            &root,
            true,
            true,
            if preview_enabled(&root) {
                "Preview test: a sample reply is ready. / 미리보기 테스트입니다."
            } else {
                ""
            },
            None,
        )?;
        if settings::Config::load(&root).sound == "custom" {
            thread::sleep(Duration::from_secs(2));
        }
        return Ok(());
    }
    let _com = Com::new().map_err(|e| format!("COM_{:08x}", e.code().0 as u32))?;
    let o = Observer::new().map_err(|e| format!("OBSERVER_{:08x}", e.code().0 as u32))?;
    if mode == "--click-probe" {
        clicks::probe(&root, &o);
        return Ok(());
    }
    // Bounded synthetic click test; does not monitor conversation completion.
    if mode == "--test-click-toast" {
        if !root.join(".replynoti-install").is_file() {
            return Err("INSTALL_REQUIRED".into());
        }
        let Some(_lock) = mutex() else {
            return Err("STOP_WATCHER_FOR_CLICK_TEST".into());
        };
        let handles = windows();
        if handles.len() != 1 {
            return Err("CLICK_TEST_NO_UNIQUE_WINDOW".into());
        }
        let mut clicks = clicks::Clicks::new(&root);
        let launch = clicks
            .issue(&root, &o, handles[0], true)
            .ok_or("CLICK_TICKET_FAILED")?;
        toast(
            &root,
            true,
            true,
            "Click to open ChatGPT. / 클릭하면 ChatGPT가 열립니다.",
            Some(&launch),
        )?;
        let end = Instant::now() + Duration::from_secs(15);
        while Instant::now() < end && !root.join("stop.flag").exists() {
            clicks.drain(&root, &o);
            thread::sleep(Duration::from_millis(200));
        }
        return Ok(());
    }
    if mode == "--probe" || mode == "--preview-probe" {
        let handles = windows();
        let mut out = format!("MATCHED_WINDOWS {}\n", handles.len());
        for h in handles {
            let s = o
                .observe(h)
                .map_err(|e| format!("PROBE_{:08x}", e.code().0 as u32))?;
            out.push_str(&format!(
                "WINDOW {h} STATE {}\n",
                s.as_ref()
                    .map(|s| if s.busy { "BUSY" } else { "IDLE" })
                    .unwrap_or("UNKNOWN")
            ));
            let anchors = o
                .anchors(h)
                .map_err(|e| format!("ANCHOR_{:08x}", e.code().0 as u32))?;
            out.push_str(&format!("ANCHORS {}\n", anchors.len()));
            for (i, a) in anchors.iter().enumerate().rev().take(4) {
                let preview = mode == "--preview-probe" && preview_enabled(&root);
                let (done, text) = o.answer(a, preview).unwrap_or((false, String::new()));
                out.push_str(&format!(
                    "TURN {i} FINISHED {done} LATEST {} PREVIEW_CHARS {}\n",
                    i + 1 == anchors.len(),
                    text.chars().count()
                ));
            }
        }
        fs::write(root.join("probe.txt"), out).map_err(|_| "PROBE_WRITE_FAILED".to_string())?;
        return Ok(());
    }
    if mode == "--shadow" {
        return monitor(&root, &o, true, Some(Duration::from_secs(12)))
            .map_err(|e| format!("MONITOR_{:08x}", e.code().0 as u32));
    }
    if mode != "--run" && mode != "--settings" {
        return Err("UNKNOWN_MODE".into());
    }
    if !root.join(".replynoti-install").is_file() {
        return Err("INSTALL_REQUIRED".into());
    }
    let Some(_lock) = mutex() else {
        if mode == "--settings" {
            tray::show_existing();
        }
        return Ok(());
    };
    if root.join("stop.flag").exists() {
        fs::remove_file(root.join("stop.flag"))
            .map_err(|_| "STOP_FLAG_REMOVE_FAILED".to_string())?;
    }
    let show = mode == "--settings";
    let ui = thread::spawn(move || {
        if let Err(e) = tray::run(show, false) {
            log(
                &root_path(),
                &format!("GUI_ERROR {:08x}", e.code().0 as u32),
            );
        }
    });
    let result =
        monitor(&root, &o, false, None).map_err(|e| format!("MONITOR_{:08x}", e.code().0 as u32));
    let _ = fs::write(root.join("stop.flag"), b"stop");
    let _ = ui.join();
    result
}
fn main() {
    if let Err(e) = run() {
        log(&root_path(), &format!("FATAL {e}"));
        std::process::exit(1)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn snap(busy: bool) -> Snapshot {
        Snapshot {
            composer: vec![1],
            busy,
        }
    }
    fn tick(m: &mut Machine, busy: bool, stamp: u64, t: Instant) -> Event {
        m.sample(
            Some(&snap(busy)),
            Some(&ctx(1, 1, &[1])),
            stamp,
            t,
            Some(!busy),
        )
    }
    #[test]
    fn complete_once() {
        let mut m = Machine::default();
        let t = Instant::now();
        assert_eq!(tick(&mut m, false, 10, t), Event::None);
        assert_eq!(tick(&mut m, true, 10, t), Event::Busy);
        assert_eq!(tick(&mut m, false, 10, t), Event::None);
        assert_eq!(tick(&mut m, false, 10, t), Event::Pending);
        assert_eq!(tick(&mut m, false, 10, t + GRACE), Event::Complete);
        assert_eq!(tick(&mut m, false, 10, t + GRACE), Event::None);
    }
    #[test]
    fn native_duplicate() {
        let mut m = Machine::default();
        let t = Instant::now();
        tick(&mut m, true, 10, t);
        tick(&mut m, false, 10, t);
        tick(&mut m, false, 10, t);
        assert_eq!(tick(&mut m, false, 11, t + GRACE), Event::Native);
    }
    #[test]
    fn busy_resumes() {
        let mut m = Machine::default();
        let t = Instant::now();
        tick(&mut m, true, 10, t);
        tick(&mut m, false, 10, t);
        tick(&mut m, false, 10, t);
        assert_eq!(tick(&mut m, true, 10, t + GRACE), Event::None);
        assert!(m.active.as_ref().unwrap().pending.is_none());
        assert_eq!(tick(&mut m, false, 10, t + GRACE), Event::None);
    }
    #[test]
    fn unavailable_and_changed_composer() {
        let mut m = Machine::default();
        let t = Instant::now();
        tick(&mut m, true, 10, t);
        assert_eq!(m.sample(None, None, 10, t, None), Event::Reset);
        tick(&mut m, true, 10, t);
        assert_eq!(
            m.sample(
                Some(&Snapshot {
                    composer: vec![2],
                    busy: false
                }),
                None,
                10,
                t,
                Some(true)
            ),
            Event::Reset
        );
    }
    fn ctx(transcript: i32, user: i32, copies: &[i32]) -> Context {
        Context {
            id: Identity {
                transcript: vec![transcript],
                user: vec![user],
            },
            copies: copies.iter().map(|n| vec![*n]).collect(),
        }
    }
    #[test]
    fn changed_chat_never_completes() {
        let mut m = Machine::default();
        let t = Instant::now();
        let a = ctx(1, 1, &[1]);
        let b = ctx(2, 2, &[2]);
        assert_eq!(
            m.sample(Some(&snap(true)), Some(&a), 10, t, Some(false)),
            Event::Busy
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t, Some(true)),
            Event::Reset
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t + GRACE, Some(true)),
            Event::None
        );
    }
    #[test]
    fn busy_anchor_refresh_and_missing_anchor() {
        let mut m = Machine::default();
        let t = Instant::now();
        let a = ctx(1, 1, &[1]);
        let b = ctx(1, 2, &[1, 2]);
        m.sample(Some(&snap(true)), Some(&a), 10, t, Some(false));
        m.sample(Some(&snap(true)), Some(&b), 10, t, Some(false));
        assert_eq!(
            m.sample(Some(&snap(false)), None, 10, t, Some(true)),
            Event::None
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t, Some(true)),
            Event::None
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t, Some(true)),
            Event::Pending
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t + GRACE, Some(true)),
            Event::Complete
        );
    }
    #[test]
    fn short_turn_with_completed_reply() {
        let mut m = Machine::default();
        let t = Instant::now();
        let a = ctx(1, 1, &[1]);
        let b = ctx(1, 2, &[1, 2]);
        m.sample(Some(&snap(false)), Some(&a), 10, t, Some(true));
        assert!(m.short_candidate(Some(&b)));
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t, Some(true)),
            Event::Short
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t, Some(true)),
            Event::Pending
        );
        assert_eq!(
            m.sample(Some(&snap(false)), Some(&b), 10, t + GRACE, Some(true)),
            Event::Complete
        );
        assert!(!m.short_candidate(Some(&b)));
    }
    #[test]
    fn typing_does_not_complete_unfinished_reply() {
        let mut m = Machine::default();
        let t = Instant::now();
        let a = ctx(1, 1, &[1]);
        assert_eq!(
            m.sample(Some(&snap(true)), Some(&a), 0, t, Some(false)),
            Event::Busy
        );
        for sec in 1..=20 {
            assert_eq!(
                m.sample(
                    Some(&snap(false)),
                    Some(&a),
                    0,
                    t + Duration::from_secs(sec),
                    Some(false)
                ),
                Event::None
            );
        }
        assert_eq!(
            m.sample(
                Some(&snap(false)),
                Some(&a),
                0,
                t + Duration::from_secs(21),
                Some(true)
            ),
            Event::None
        );
        assert_eq!(
            m.sample(
                Some(&snap(false)),
                Some(&a),
                0,
                t + Duration::from_secs(22),
                Some(true)
            ),
            Event::Pending
        );
        assert_eq!(
            m.sample(
                Some(&snap(false)),
                Some(&a),
                0,
                t + Duration::from_secs(24),
                Some(true)
            ),
            Event::Complete
        );
    }
    #[test]
    fn previous_completed_turn_is_not_new_completion() {
        let mut m = Machine::default();
        let t = Instant::now();
        let a = ctx(1, 1, &[1]);
        m.sample(Some(&snap(false)), Some(&a), 0, t, Some(true));
        m.sample(Some(&snap(true)), Some(&a), 0, t, Some(true));
        for sec in 1..=5 {
            assert_eq!(
                m.sample(
                    Some(&snap(false)),
                    Some(&a),
                    0,
                    t + Duration::from_secs(sec),
                    Some(true)
                ),
                Event::None
            );
        }
    }
    #[test]
    fn smart_delivery_policy() {
        assert_eq!(
            delivery("smart", Some(true), Some(9999), 10),
            Delivery::Hold
        );
        assert_eq!(
            delivery("smart", Some(true), Some(10000), 10),
            Delivery::Send
        );
        assert_eq!(delivery("smart", Some(false), Some(0), 10), Delivery::Send);
        assert_eq!(delivery("smart", Some(true), None, 10), Delivery::Hold);
        assert_eq!(
            delivery("background", Some(true), Some(60000), 10),
            Delivery::Drop
        );
        assert_eq!(
            delivery("background", Some(false), None, 10),
            Delivery::Send
        );
        assert_eq!(delivery("always", None, None, 10), Delivery::Send);
    }
    #[test]
    fn focus_recognizes_owned_and_other_official_windows() {
        assert_eq!(
            classify_focus(true, false, false, None),
            FocusReason::SameWindow
        );
        assert_eq!(
            classify_focus(false, true, false, None),
            FocusReason::OwnedWindow
        );
        assert_eq!(
            classify_focus(false, false, true, None),
            FocusReason::SameProcess
        );
        assert_eq!(
            classify_focus(false, false, false, Some(true)).focused(),
            Some(true)
        );
        assert_eq!(
            classify_focus(false, false, false, Some(false)).focused(),
            Some(false)
        );
        assert_eq!(classify_focus(false, false, false, None).focused(), None);
    }
    #[test]
    fn background_gate_rejects_transient_switches_and_refocus() {
        let mut c = settings::Config::default();
        c.notification_mode = "background".into();
        let t = Instant::now();
        let mut gate = FocusGate::default();
        let sample = |foreground, reason| Activity {
            foreground,
            reason,
            idle: Some(60_000),
        };
        assert_eq!(
            gate.decide(&c, sample(2, FocusReason::OtherApp), t),
            Delivery::Hold
        );
        assert_eq!(
            gate.decide(
                &c,
                sample(2, FocusReason::OtherApp),
                t + BACKGROUND_STABLE - Duration::from_millis(1)
            ),
            Delivery::Hold
        );
        assert_eq!(
            gate.decide(&c, sample(2, FocusReason::OtherApp), t + BACKGROUND_STABLE),
            Delivery::Send
        );
        // Recheck just before submitting: a ChatGPT-owned popup must suppress.
        assert_eq!(
            gate.decide(
                &c,
                sample(3, FocusReason::OwnedWindow),
                t + BACKGROUND_STABLE
            ),
            Delivery::Drop
        );
        assert_eq!(
            gate.decide(&c, sample(2, FocusReason::OtherApp), t + BACKGROUND_STABLE),
            Delivery::Hold
        );
        assert_eq!(
            gate.decide(
                &c,
                sample(4, FocusReason::OtherApp),
                t + BACKGROUND_STABLE * 2
            ),
            Delivery::Hold
        );
        assert_eq!(
            gate.decide(
                &c,
                sample(0, FocusReason::Unknown),
                t + BACKGROUND_STABLE * 3
            ),
            Delivery::Hold
        );
        assert_eq!(
            gate.decide(
                &c,
                sample(4, FocusReason::OtherApp),
                t + BACKGROUND_STABLE * 4
            ),
            Delivery::Hold
        );
    }
    #[test]
    fn smart_and_always_keep_their_intended_policy() {
        let mut c = settings::Config::default();
        let t = Instant::now();
        let mut gate = FocusGate::default();
        let mut a = Activity {
            foreground: 1,
            reason: FocusReason::SameProcess,
            idle: Some(9999),
        };
        assert_eq!(gate.decide(&c, a, t), Delivery::Hold);
        a.idle = Some(10000);
        assert_eq!(gate.decide(&c, a, t), Delivery::Send);
        c.notification_mode = "always".into();
        a.reason = FocusReason::Unknown;
        assert_eq!(gate.decide(&c, a, t), Delivery::Send);
    }
    #[test]
    fn preview_filter_and_unicode() {
        assert_eq!(
            safe_preview("정상적인 테스트 답변입니다."),
            "정상적인 테스트 답변입니다."
        );
        for s in [
            "api key: sk-test123456789",
            "email abc@example.com",
            "비밀번호는 테스트입니다",
            "https://example.com",
            "123456-1234567",
            "1234 5678 1234 5678",
        ] {
            assert!(safe_preview(s).is_empty());
        }
        assert_eq!(safe_preview(&"가".repeat(200)).encode_utf16().count(), 151);
        assert_eq!(safe_preview(&"🙂".repeat(100)).encode_utf16().count(), 151);
    }
    #[test]
    fn escaped_payload() {
        assert_eq!(escape_xml("&<>'\""), "&amp;&lt;&gt;&apos;&quot;");
        assert_eq!(
            file_uri(Path::new(r"C:\Test Folder\icon.png")),
            "file:///C:/Test%20Folder/icon.png"
        );
    }
}
