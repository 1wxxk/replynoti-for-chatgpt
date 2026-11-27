use super::*;
use crate::settings::{self, Config};
use windows::Win32::{
    Foundation::{HINSTANCE, LRESULT, POINT, WPARAM},
    Graphics::Gdi::{COLOR_WINDOW, DEFAULT_GUI_FONT, GetStockObject, HBRUSH},
    System::LibraryLoader::GetModuleHandleW,
    UI::{Controls::Dialogs::*, Shell::*, WindowsAndMessaging::*},
};
const CLASS: PCWSTR = w!("ReplyNotiSettingsV4");
const TRAY_MSG: u32 = WM_APP + 1;
const SHOW_MSG: u32 = WM_APP + 2;
const ENABLE: i32 = 101;
const PREVIEW: i32 = 102;
const NAV: i32 = 103;
const START: i32 = 104;
const LEN: i32 = 105;
const SOUND: i32 = 106;
const ICON: i32 = 107;
const RESET: i32 = 108;
const WAV: i32 = 109;
const INSTALLED_ICON: i32 = 114;
const INSTALLED_SOUND: i32 = 115;
const TEST: i32 = 110;
const SAVE: i32 = 111;
const CLOSE: i32 = 112;
const STATUS: i32 = 113;
const DISPLAY_NAME: i32 = 116;
const MENU_SETTINGS: usize = 201;
const MENU_QUIT: usize = 202;
const MENU_ENABLE: usize = 203;
const MENU_PREVIEW: usize = 204;
const MENU_TEST: usize = 205;
thread_local! {static TRAY:std::cell::RefCell<Option<NOTIFYICONDATAW>>=const{std::cell::RefCell::new(None)};}
fn root() -> PathBuf {
    root_path()
}
fn text(s: &str) -> Vec<u16> {
    wide(s)
}
unsafe fn child(
    h: HWND,
    id: i32,
    cl: &str,
    label: &str,
    style: WINDOW_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<HWND> {
    let c = text(cl);
    let t = text(label);
    let v = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(c.as_ptr()),
            PCWSTR(t.as_ptr()),
            WS_CHILD | WS_VISIBLE | style,
            x,
            y,
            width,
            height,
            Some(h),
            Some(HMENU(id as usize as *mut c_void)),
            None,
            None,
        )?
    };
    unsafe {
        let font = GetStockObject(DEFAULT_GUI_FONT);
        SendMessageW(
            v,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(LPARAM(1)),
        );
    }
    Ok(v)
}
fn ctl(h: HWND, id: i32) -> Option<HWND> {
    unsafe { GetDlgItem(Some(h), id).ok() }
}
fn label(h: HWND, id: i32, s: &str) {
    if let Some(c) = ctl(h, id) {
        let t = text(s);
        unsafe {
            let _ = SetWindowTextW(c, PCWSTR(t.as_ptr()));
        }
    }
}
fn set_title(h: HWND, name: &str) {
    let t = wide(&format!("{name} - ReplyNoti Settings"));
    unsafe {
        let _ = SetWindowTextW(h, PCWSTR(t.as_ptr()));
    }
}
fn check(h: HWND, id: i32, b: bool) {
    if let Some(c) = ctl(h, id) {
        unsafe {
            SendMessageW(c, BM_SETCHECK, Some(WPARAM(b as usize)), None);
        }
    }
}
fn checked(h: HWND, id: i32) -> bool {
    ctl(h, id).is_some_and(|c| unsafe { SendMessageW(c, BM_GETCHECK, None, None).0 == 1 })
}
fn select(h: HWND, id: i32, n: usize) {
    if let Some(c) = ctl(h, id) {
        unsafe {
            SendMessageW(c, CB_SETCURSEL, Some(WPARAM(n)), None);
        }
    }
}
fn selected(h: HWND, id: i32) -> isize {
    ctl(h, id)
        .map(|c| unsafe { SendMessageW(c, CB_GETCURSEL, None, None).0 })
        .unwrap_or(-1)
}
fn items(h: HWND, id: i32, values: &[&str]) {
    if let Some(c) = ctl(h, id) {
        for s in values {
            let t = text(s);
            unsafe {
                SendMessageW(c, CB_ADDSTRING, None, Some(LPARAM(t.as_ptr() as isize)));
            }
        }
    }
}
fn refresh(h: HWND) {
    let r = root();
    let c = Config::load(&r);
    label(h, DISPLAY_NAME, &c.display_name);
    set_title(h, &c.display_name);
    check(h, ENABLE, c.enabled);
    check(h, PREVIEW, c.preview);
    check(h, NAV, c.navigate);
    check(h, START, settings::startup(&r));
    select(
        h,
        LEN,
        match c.length {
            50 => 0,
            150 => 2,
            _ => 1,
        },
    );
    select(
        h,
        SOUND,
        match c.sound.as_str() {
            "silent" => 1,
            "custom" => 2,
            _ => 0,
        },
    );
    label(
        h,
        STATUS,
        if c.custom_icon {
            "Icon: custom image"
        } else {
            "Icon: ReplyNoti default"
        },
    );
}
fn save(h: HWND) -> std::result::Result<(), String> {
    let r = root();
    let mut c = Config::load(&r);
    let mut value = [0u16; 128];
    let field = ctl(h, DISPLAY_NAME).ok_or("Display name field unavailable.")?;
    let n = unsafe { GetWindowTextW(field, &mut value) }.max(0) as usize;
    c.display_name = settings::valid_name(&String::from_utf16_lossy(&value[..n]))?;
    c.enabled = checked(h, ENABLE);
    c.preview = checked(h, PREVIEW);
    c.navigate = checked(h, NAV);
    c.length = match selected(h, LEN) {
        0 => 50,
        2 => 150,
        _ => 100,
    };
    c.sound = match selected(h, SOUND) {
        1 => "silent",
        2 => "custom",
        _ => "default",
    }
    .into();
    if c.sound == "custom" && !r.join("custom-sound.wav").is_file() {
        return Err("Choose a WAV file first.".into());
    }
    settings::set_startup(&r, checked(h, START))?;
    c.save(&r)
        .map_err(|_| "Could not save settings.".to_string())?;
    settings::set_display_name(&c.display_name)?;
    set_title(h, &c.display_name);
    update_tray(h);
    log(&r, "SETTINGS_SAVED");
    Ok(())
}
fn alert(h: HWND, s: &str) {
    let t = text(s);
    unsafe {
        MessageBoxW(
            Some(h),
            PCWSTR(t.as_ptr()),
            w!("ReplyNoti"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
fn pick(h: HWND, icon: bool) -> Option<PathBuf> {
    let mut file = vec![0u16; 32768];
    let filter = if icon {
        "Image (*.png;*.ico)\0*.png;*.ico\0\0"
    } else {
        "WAV (*.wav)\0*.wav\0\0"
    };
    let f: Vec<u16> = filter.encode_utf16().collect();
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: h,
        lpstrFilter: PCWSTR(f.as_ptr()),
        lpstrFile: PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR | OFN_HIDEREADONLY,
        ..Default::default()
    };
    if unsafe { GetOpenFileNameW(&mut ofn) }.as_bool() {
        let n = file.iter().position(|x| *x == 0)?;
        Some(PathBuf::from(String::from_utf16_lossy(&file[..n])))
    } else {
        None
    }
}
fn import(h: HWND, icon: bool) {
    if let Some(p) = pick(h, icon) {
        let r = root();
        let result = if icon {
            settings::import_icon(&r, &p)
        } else {
            settings::import_sound(&r, &p)
        };
        match result {
            Ok(()) => {
                if icon {
                    let mut c = Config::load(&r);
                    c.custom_icon = true;
                    if c.save(&r).is_err() {
                        alert(h, "Could not save icon setting.");
                        return;
                    }
                    label(h, STATUS, "Icon: custom image");
                    update_tray(h)
                } else {
                    select(h, SOUND, 2);
                }
            }
            Err(e) => alert(h, &e),
        }
    }
}
fn import_installed(h: HWND, icon: bool) {
    let msg = if icon {
        "Import the installed ChatGPT app icon for local use? ReplyNoti remains unofficial. Use only assets you have permission to use."
    } else {
        "Import the bundled completion WAV from your installed ChatGPT app? The sound is copied locally, not distributed. It may differ from the sound selected in ChatGPT settings."
    };
    let t = wide(msg);
    if unsafe {
        MessageBoxW(
            Some(h),
            PCWSTR(t.as_ptr()),
            w!("Local asset import"),
            MB_YESNO | MB_ICONINFORMATION,
        )
    } != IDYES
    {
        return;
    }
    let r = root();
    match settings::import_installed(&r, icon) {
        Ok(()) => {
            let mut c = Config::load(&r);
            if icon {
                c.custom_icon = true;
            } else {
                c.sound = "custom".into();
            }
            if c.save(&r).is_err() {
                alert(h, "Could not save settings.");
                return;
            }
            if icon {
                label(h, STATUS, "Icon: imported from installed app");
                update_tray(h);
            } else {
                select(h, SOUND, 2);
            }
            alert(
                h,
                "Local asset imported. No official app files were modified.",
            );
        }
        Err(e) => alert(h, &e),
    }
}
fn icon() -> Option<HICON> {
    let r = root();
    let p = Config::load(&r).icon(&r, "ico");
    let p = wide(&p.to_string_lossy());
    unsafe {
        LoadImageW(
            None,
            PCWSTR(p.as_ptr()),
            IMAGE_ICON,
            32,
            32,
            LR_LOADFROMFILE,
        )
        .ok()
        .map(|h| HICON(h.0))
    }
}
fn korean_language(language: &str, system_lang: u16) -> bool {
    match language {
        "ko" => true,
        "en" => false,
        _ => system_lang & 0x03ff == 0x12,
    }
}
fn korean(c: &Config) -> bool {
    korean_language(&c.language, unsafe {
        windows::Win32::Globalization::GetUserDefaultUILanguage()
    })
}
fn localized(ko: bool, en: &'static str, kr: &'static str) -> &'static str {
    if ko { kr } else { en }
}
#[cfg(test)]
mod locale_tests {
    use super::*;
    #[test]
    fn explicit_language_overrides_system() {
        assert!(korean_language("ko", 0x0409));
        assert!(!korean_language("en", 0x0412));
    }
    #[test]
    fn system_language_uses_windows_ui_language() {
        assert!(korean_language("system", 0x0412));
        assert!(!korean_language("system", 0x0409));
        assert!(!korean_language("system", 0x0411));
    }
}
fn tray_data(h: HWND) -> Option<NOTIFYICONDATAW> {
    let r = root();
    let c = Config::load(&r);
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: h,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: TRAY_MSG,
        hIcon: icon()?,
        ..Default::default()
    };
    let tip = wide(&format!(
        "{} - {}",
        c.display_name,
        if c.enabled {
            localized(korean(&c), "watching", "알림 감지 중")
        } else {
            localized(korean(&c), "paused", "감지 일시 중지")
        }
    ));
    data.szTip[..tip.len()].copy_from_slice(&tip);
    Some(data)
}
fn update_tray(h: HWND) {
    if let Some(d) = tray_data(h) {
        TRAY.with(|cell| {
            let mut state = cell.borrow_mut();
            let action = if state.is_some() { NIM_MODIFY } else { NIM_ADD };
            if unsafe { Shell_NotifyIconW(action, &d) }.as_bool() {
                if let Some(old) = state.replace(d) {
                    unsafe {
                        let _ = DestroyIcon(old.hIcon);
                    }
                }
            } else {
                unsafe {
                    let _ = DestroyIcon(d.hIcon);
                }
            }
        })
    }
}
fn remove_tray() {
    TRAY.with(|c| {
        if let Some(d) = c.borrow_mut().take() {
            unsafe {
                let _ = Shell_NotifyIconW(NIM_DELETE, &d);
                let _ = DestroyIcon(d.hIcon);
            }
        }
    })
}
thread_local! {static MODERN: std::cell::RefCell<Option<std::process::Child>>=const{std::cell::RefCell::new(None)};}
fn modern_settings() -> bool {
    let path = root().join("replynoti-settings.exe");
    if !path.is_file() {
        return false;
    }
    MODERN.with(|cell| {
        let mut child = cell.borrow_mut();
        if let Some(p) = child.as_mut() {
            if p.try_wait().ok() == Some(None) {
                unsafe {
                    if let Ok(h) = FindWindowW(PCWSTR::null(), w!("ReplyNoti")) {
                        let _ = ShowWindow(h, SW_RESTORE);
                        let _ = SetForegroundWindow(h);
                    }
                }
                return true;
            }
        }
        match std::process::Command::new(path).spawn() {
            Ok(p) => {
                *child = Some(p);
                true
            }
            Err(_) => false,
        }
    })
}
fn show(h: HWND) {
    if modern_settings() {
        return;
    }
    refresh(h);
    unsafe {
        let _ = ShowWindow(h, SW_SHOWNORMAL);
        let _ = SetForegroundWindow(h);
    }
}
fn popup(h: HWND) {
    let r = root();
    let c = Config::load(&r);
    unsafe {
        let Ok(m) = CreatePopupMenu() else { return };
        let ko = korean(&c);
        for (id, s, flags) in [
            (
                MENU_ENABLE,
                localized(ko, "Completion notifications", "응답 완료 알림"),
                if c.enabled { MF_CHECKED } else { MF_UNCHECKED },
            ),
            (
                MENU_PREVIEW,
                localized(ko, "Reply preview", "응답 미리보기"),
                if c.preview { MF_CHECKED } else { MF_UNCHECKED },
            ),
            (
                MENU_SETTINGS,
                localized(ko, "Settings...", "설정..."),
                MF_UNCHECKED,
            ),
            (
                MENU_TEST,
                localized(ko, "Test notification", "테스트 알림"),
                MF_UNCHECKED,
            ),
            (
                MENU_QUIT,
                localized(ko, "Exit ReplyNoti", "ReplyNoti 종료"),
                MF_UNCHECKED,
            ),
        ] {
            let t = wide(s);
            let _ = AppendMenuW(m, MF_STRING | flags, id, PCWSTR(t.as_ptr()));
        }
        let mut p = POINT::default();
        let _ = GetCursorPos(&mut p);
        let _ = SetForegroundWindow(h);
        let cmd =
            TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON, p.x, p.y, None, h, None).0 as usize;
        let _ = DestroyMenu(m);
        let _ = PostMessageW(Some(h), WM_NULL, WPARAM(0), LPARAM(0));
        match cmd {
            MENU_SETTINGS => show(h),
            MENU_QUIT => {
                let _ = fs::write(r.join("stop.flag"), b"stop");
                let _ = DestroyWindow(h);
            }
            MENU_ENABLE | MENU_PREVIEW => {
                let mut c = Config::load(&r);
                if cmd == MENU_ENABLE {
                    c.enabled = !c.enabled
                } else {
                    c.preview = !c.preview
                }
                if c.save(&r).is_ok() {
                    update_tray(h);
                    if IsWindowVisible(h).as_bool() {
                        refresh(h)
                    }
                } else {
                    alert(
                        h,
                        localized(
                            ko,
                            "Could not save settings.",
                            "설정을 저장하지 못했습니다.",
                        ),
                    )
                }
            }
            MENU_TEST => test_toast(),
            _ => {}
        }
    }
}
fn test_toast() {
    thread::spawn(|| {
        let r = root();
        let _ = toast(
            &r,
            true,
            true,
            "ReplyNoti preview test / 미리보기 테스트",
            None,
        );
        thread::sleep(Duration::from_secs(2));
    });
}
fn build(h: HWND) -> Result<()> {
    unsafe {
        let s = WINDOW_STYLE(0);
        let btn = WINDOW_STYLE(BS_PUSHBUTTON as u32) | WS_TABSTOP;
        let chk = WINDOW_STYLE(BS_AUTOCHECKBOX as u32) | WS_TABSTOP;
        child(
            h,
            0,
            "STATIC",
            "ReplyNoti | Notifications for ChatGPT",
            s,
            20,
            16,
            420,
            24,
        )?;
        child(
            h,
            ENABLE,
            "BUTTON",
            "Completion notifications",
            chk,
            20,
            52,
            420,
            24,
        )?;
        child(
            h,
            PREVIEW,
            "BUTTON",
            "Show reply preview",
            chk,
            20,
            82,
            420,
            24,
        )?;
        child(
            h,
            0,
            "STATIC",
            "Preview can appear on your lock screen or in notification history.",
            s,
            20,
            112,
            424,
            36,
        )?;
        child(h, 0, "STATIC", "Preview length:", s, 20, 152, 155, 24)?;
        child(
            h,
            LEN,
            "COMBOBOX",
            "",
            WINDOW_STYLE(CBS_DROPDOWNLIST as u32) | WS_TABSTOP,
            186,
            148,
            226,
            120,
        )?;
        child(
            h,
            NAV,
            "BUTTON",
            "Open the matching conversation when clicked",
            chk,
            20,
            188,
            430,
            24,
        )?;
        child(
            h,
            START,
            "BUTTON",
            "Start with Windows",
            chk,
            20,
            218,
            420,
            24,
        )?;
        child(h, 0, "STATIC", "Notification sound:", s, 20, 264, 155, 24)?;
        child(
            h,
            SOUND,
            "COMBOBOX",
            "",
            WINDOW_STYLE(CBS_DROPDOWNLIST as u32) | WS_TABSTOP,
            186,
            260,
            226,
            120,
        )?;
        child(h, WAV, "BUTTON", "Choose WAV...", btn, 20, 296, 202, 30)?;
        child(
            h,
            INSTALLED_SOUND,
            "BUTTON",
            "Import installed app sound",
            btn,
            230,
            296,
            202,
            30,
        )?;
        child(
            h,
            STATUS,
            "STATIC",
            "Icon: ReplyNoti default",
            s,
            20,
            350,
            420,
            24,
        )?;
        child(
            h,
            ICON,
            "BUTTON",
            "Choose PNG / ICO...",
            btn,
            20,
            378,
            202,
            32,
        )?;
        child(
            h,
            RESET,
            "BUTTON",
            "Restore default icon",
            btn,
            230,
            378,
            202,
            32,
        )?;
        child(
            h,
            INSTALLED_ICON,
            "BUTTON",
            "Import installed app icon",
            btn,
            20,
            414,
            412,
            30,
        )?;
        child(
            h,
            TEST,
            "BUTTON",
            "Test notification",
            btn,
            20,
            522,
            144,
            34,
        )?;
        child(h, 0, "STATIC", "Display name:", s, 20, 460, 125, 24)?;
        let edit = child(
            h,
            DISPLAY_NAME,
            "EDIT",
            "ReplyNoti",
            WS_BORDER | WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            150,
            456,
            282,
            27,
        )?;
        SendMessageW(edit, 0x00C5, Some(WPARAM(48)), None);
        child(
            h,
            0,
            "STATIC",
            "Local display only. ReplyNoti remains an unofficial helper.",
            s,
            20,
            488,
            420,
            24,
        )?;
        child(h, SAVE, "BUTTON", "Save", btn, 254, 522, 84, 34)?;
        child(h, CLOSE, "BUTTON", "Close", btn, 346, 522, 84, 34)?;
    }
    items(h, LEN, &["50", "100", "150"]);
    items(h, SOUND, &["Windows default", "Silent", "Custom WAV"]);
    refresh(h);
    Ok(())
}
unsafe extern "system" fn wnd(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        TRAY_MSG => {
            match lp.0 as u32 {
                WM_RBUTTONUP => popup(h),
                WM_LBUTTONDBLCLK => show(h),
                _ => {}
            }
            return LRESULT(0);
        }
        SHOW_MSG => {
            show(h);
            return LRESULT(0);
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xffff) as i32;
            if (wp.0 >> 16) == 0 {
                match id {
                    ICON => import(h, true),
                    WAV => import(h, false),
                    INSTALLED_ICON => import_installed(h, true),
                    INSTALLED_SOUND => import_installed(h, false),
                    RESET => {
                        let r = root();
                        let mut c = Config::load(&r);
                        c.custom_icon = false;
                        if c.save(&r).is_ok() {
                            label(h, STATUS, "Icon: ReplyNoti default");
                            update_tray(h)
                        } else {
                            alert(h, "Could not save settings.")
                        }
                    }
                    SAVE => match save(h) {
                        Ok(()) => {
                            alert(h, "Settings saved.");
                        }
                        Err(e) => alert(h, &e),
                    },
                    TEST => match save(h) {
                        Ok(()) => test_toast(),
                        Err(e) => alert(h, &e),
                    },
                    CLOSE => unsafe {
                        let _ = ShowWindow(h, SW_HIDE);
                    },
                    _ => {}
                }
            }
            return LRESULT(0);
        }
        WM_CLOSE => {
            unsafe {
                let _ = ShowWindow(h, SW_HIDE);
            }
            return LRESULT(0);
        }
        WM_TIMER => {
            thread_local! {static LAST:std::cell::RefCell<Option<(Config,Option<SystemTime>)>>=const{std::cell::RefCell::new(None)};}
            let r = root();
            let c = Config::load(&r);
            let modified = fs::metadata(c.icon(&r, "ico"))
                .and_then(|m| m.modified())
                .ok();
            LAST.with(|state| {
                let mut prev = state.borrow_mut();
                if prev.as_ref() != Some(&(c.clone(), modified)) {
                    *prev = Some((c, modified));
                    update_tray(h);
                }
            });
            if root().join("stop.flag").exists() || !console_session() {
                unsafe {
                    let _ = DestroyWindow(h);
                }
            }
            return LRESULT(0);
        }
        WM_DESTROY => {
            remove_tray();
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        _ => {}
    }
    unsafe { DefWindowProcW(h, msg, wp, lp) }
}
pub fn show_existing() {
    unsafe {
        if let Ok(h) = FindWindowW(CLASS, PCWSTR::null()) {
            let _ = PostMessageW(Some(h), SHOW_MSG, WPARAM(0), LPARAM(0));
        }
    }
}
pub fn run(show_now: bool, self_test: bool) -> Result<()> {
    unsafe {
        let module = GetModuleHandleW(PCWSTR::null())?;
        let instance = HINSTANCE(module.0);
        let wc = WNDCLASSW {
            style: CS_DBLCLKS,
            lpfnWndProc: Some(wnd),
            hInstance: instance,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as usize as *mut c_void),
            lpszClassName: CLASS,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err(windows::core::Error::from_thread());
        }
        let h = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS,
            w!("ReplyNoti Settings"),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            480,
            612,
            None,
            None,
            Some(instance),
            None,
        )?;
        build(h)?;
        if self_test {
            let c = Config::load(&root());
            if checked(h, PREVIEW) != c.preview
                || selected(h, LEN)
                    != match c.length {
                        50 => 0,
                        150 => 2,
                        _ => 1,
                    }
            {
                let _ = DestroyWindow(h);
                return Err(windows::core::Error::from_thread());
            }
            update_tray(h);
            let added = TRAY.with(|x| x.borrow().is_some());
            let _ = DestroyWindow(h);
            log(
                &root(),
                if added {
                    "GUI_TRAY_SELF_TEST_OK"
                } else {
                    "GUI_TRAY_ADD_FAILED"
                },
            );
            if !added {
                return Err(windows::core::Error::from_thread());
            }
            return Ok(());
        }
        update_tray(h);
        if !TRAY.with(|x| x.borrow().is_some()) {
            log(&root(), "TRAY_ADD_FAILED")
        }
        if show_now {
            show(h)
        }
        SetTimer(Some(h), 1, 1000, None);
        let mut m = MSG::default();
        while GetMessageW(&mut m, None, 0, 0).0 > 0 {
            if !IsDialogMessageW(h, &m).as_bool() {
                let _ = TranslateMessage(&m);
                DispatchMessageW(&m);
            }
        }
        Ok(())
    }
}
