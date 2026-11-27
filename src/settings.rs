use super::*;
use std::io::Cursor;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};
use windows::Win32::System::Registry::{
    HKEY, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegSetValueExW,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Config {
    pub display_name: String,
    pub language: String,
    pub theme: String,
    pub enabled: bool,
    pub notification_mode: String,
    pub idle_seconds: u32,
    pub preview: bool,
    pub length: usize,
    pub navigate: bool,
    pub sound: String,
    pub custom_icon: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            display_name: "ReplyNoti".into(),
            language: "system".into(),
            theme: "system".into(),
            enabled: true,
            notification_mode: "smart".into(),
            idle_seconds: 10,
            preview: true,
            length: 100,
            navigate: true,
            sound: "default".into(),
            custom_icon: false,
        }
    }
}
impl Config {
    fn parse(s: &str) -> Self {
        let mut c = Self::default();
        for line in s.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = v.trim();
            match k.trim() {
                "language" => {
                    if matches!(v, "\"system\"" | "\"ko\"" | "\"en\"") {
                        c.language = v.trim_matches('"').into();
                    }
                }
                "theme" => {
                    if matches!(v, "\"system\"" | "\"light\"" | "\"dark\"") {
                        c.theme = v.trim_matches('"').into();
                    }
                }
                "display_name" => {
                    if let Some(name) = v
                        .strip_prefix('"')
                        .and_then(|x| x.strip_suffix('"'))
                        .and_then(|x| valid_name(x).ok())
                    {
                        c.display_name = name;
                    }
                }
                "notification_mode" => {
                    if matches!(v, "\"always\"" | "\"background\"" | "\"smart\"") {
                        c.notification_mode = v.trim_matches('"').into();
                    }
                }
                "idle_seconds" => {
                    if let Ok(x @ (10 | 30 | 60 | 180)) = v.parse() {
                        c.idle_seconds = x;
                    }
                }
                "enabled" => {
                    if let Ok(x) = v.parse() {
                        c.enabled = x
                    }
                }
                "preview" => {
                    if let Ok(x) = v.parse() {
                        c.preview = x
                    }
                }
                "preview_length" => {
                    if let Ok(x @ (50 | 100 | 150)) = v.parse() {
                        c.length = x
                    }
                }
                "open_conversation" => {
                    if let Ok(x) = v.parse() {
                        c.navigate = x
                    }
                }
                "sound" => {
                    if matches!(v, "\"default\"" | "\"silent\"" | "\"custom\"") {
                        c.sound = v.trim_matches('"').into()
                    }
                }
                "custom_icon" => {
                    if let Ok(x) = v.parse() {
                        c.custom_icon = x
                    }
                }
                _ => {}
            }
        }
        c
    }
    pub fn load(root: &Path) -> Self {
        match fs::read_to_string(root.join("config.toml")) {
            Ok(s) => Self::parse(&s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let mut c = Self::default();
                if root.join("silent.flag").is_file() {
                    c.sound = "silent".into()
                }
                c
            }
            Err(_) => Self {
                preview: false,
                ..Self::default()
            },
        }
    }
    fn render(&self) -> String {
        format!(
            "# ReplyNoti settings\n# Changes take effect without restarting.\ndisplay_name = \"{}\"\nlanguage = \"{}\"\ntheme = \"{}\"\nenabled = {}\nnotification_mode = \"{}\"\nidle_seconds = {}\npreview = {}\npreview_length = {}\nopen_conversation = {}\nsound = \"{}\"\ncustom_icon = {}\n",
            self.display_name,
            self.language,
            self.theme,
            self.enabled,
            self.notification_mode,
            self.idle_seconds,
            self.preview,
            self.length,
            self.navigate,
            self.sound,
            self.custom_icon
        )
    }
    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        let tmp = root.join(format!("config.{}.tmp", std::process::id()));
        fs::write(&tmp, self.render())?;
        let r = fs::rename(&tmp, root.join("config.toml"));
        if r.is_err() {
            let _ = fs::remove_file(tmp);
        }
        r
    }
    pub fn icon(&self, root: &Path, ext: &str) -> PathBuf {
        let path = root.join(format!("custom-icon.{ext}"));
        if self.custom_icon && path.is_file() {
            path
        } else {
            root.join(format!("replynoti.{ext}"))
        }
    }
}
pub fn startup(root: &Path) -> bool {
    let exe = root.join("replynoti.exe");
    let expected = format!("\"{}\" --run", exe.display());
    startup_value().as_deref() == Some(expected.as_str())
}
fn startup_value() -> Option<String> {
    let sub = wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let n = wide("ReplyNoti");
    let mut buf = [0u16; 32768];
    let mut bytes = (buf.len() * 2) as u32;
    let ret = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            PCWSTR(n.as_ptr()),
            windows::Win32::System::Registry::RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    if ret.0 != 0 {
        return None;
    }
    Some(String::from_utf16_lossy(
        &buf[..(bytes as usize / 2).saturating_sub(1)],
    ))
}
pub fn set_startup(root: &Path, on: bool) -> std::result::Result<(), String> {
    let expected = format!("\"{}\" --run", root.join("replynoti.exe").display());
    if startup_value().is_some_and(|v| v != expected) {
        return Err("Unexpected existing startup registration.".into());
    }
    let sub = wide(r"Software\Microsoft\Windows\CurrentVersion\Run");
    let n = wide("ReplyNoti");
    let mut k = HKEY::default();
    unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            None,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            &mut k,
        )
        .ok()
        .map_err(|_| "Cannot open startup registry.")?;
        let code = if on {
            let value = wide(&expected);
            let bytes = std::slice::from_raw_parts(value.as_ptr().cast::<u8>(), value.len() * 2);
            RegSetValueExW(k, PCWSTR(n.as_ptr()), None, REG_SZ, Some(bytes))
        } else {
            RegDeleteValueW(k, PCWSTR(n.as_ptr()))
        };
        let _ = RegCloseKey(k);
        if code.0 != 0 && !(code.0 == 2 && !on) {
            return Err("Cannot change startup setting.".into());
        }
    }
    Ok(())
}
fn atomic_write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("import.tmp");
    fs::write(&tmp, data)?;
    let r = fs::rename(&tmp, path);
    if r.is_err() {
        let _ = fs::remove_file(tmp);
    }
    r
}
pub fn import_icon(root: &Path, source: &Path) -> std::result::Result<(), String> {
    use image::{ImageFormat, ImageReader, Limits, imageops::FilterType};
    if fs::metadata(source)
        .map_err(|_| "Cannot read image.")?
        .len()
        > 8 * 1024 * 1024
    {
        return Err("Image must be at most 8 MB.".into());
    }
    let format = match source
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => ImageFormat::Png,
        "ico" => ImageFormat::Ico,
        _ => return Err("Choose a PNG or ICO file.".into()),
    };
    let mut reader = ImageReader::with_format(
        std::io::BufReader::new(fs::File::open(source).map_err(|_| "Cannot open image.")?),
        format,
    );
    let mut limits = Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let im = reader
        .decode()
        .map_err(|_| "Invalid image or dimensions exceed 2048 px.")?
        .resize(256, 256, FilterType::Lanczos3)
        .to_rgba8();
    let mut canvas = image::RgbaImage::new(256, 256);
    image::imageops::overlay(
        &mut canvas,
        &im,
        ((256 - im.width()) / 2) as i64,
        ((256 - im.height()) / 2) as i64,
    );
    let image = image::DynamicImage::ImageRgba8(canvas);
    let mut png = Cursor::new(Vec::new());
    image
        .write_to(&mut png, ImageFormat::Png)
        .map_err(|_| "PNG conversion failed.")?;
    let mut ico = Cursor::new(Vec::new());
    image
        .write_to(&mut ico, ImageFormat::Ico)
        .map_err(|_| "ICO conversion failed.")?;
    atomic_write(&root.join("custom-icon.png"), png.get_ref()).map_err(|_| "Cannot save PNG.")?;
    atomic_write(&root.join("custom-icon.ico"), ico.get_ref()).map_err(|_| "Cannot save ICO.")?;
    Ok(())
}
pub fn import_sound(root: &Path, source: &Path) -> std::result::Result<(), String> {
    if fs::metadata(source).map_err(|_| "Cannot read WAV.")?.len() > 5 * 1024 * 1024 {
        return Err("WAV must be at most 5 MB.".into());
    }
    let b = fs::read(source).map_err(|_| "Cannot read WAV.")?;
    if b.len() < 44 || &b[..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return Err("Choose a valid WAV file.".into());
    }
    atomic_write(&root.join("custom-sound.wav"), &b).map_err(|_| "Cannot save WAV.".into())
}
pub fn play_custom(root: &Path) {
    if reg_value(APP_ID, "SoundEnabled", false) == Some(0) {
        return;
    }
    let path = root.join("custom-sound.wav");
    if !path.is_file() {
        return;
    }
    let p = wide(&path.to_string_lossy());
    unsafe {
        let _ = PlaySoundW(
            PCWSTR(p.as_ptr()),
            None,
            SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
        );
    }
}
pub fn shorten(s: &str, max: usize) -> String {
    let mut n = 0;
    let mut result = String::new();
    for c in s.chars() {
        if n + c.len_utf16() > max {
            break;
        }
        n += c.len_utf16();
        result.push(c)
    }
    if result.len() < s.len() {
        result = result.trim_end().into();
        result.push('…')
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults() {
        let c = Config::default();
        assert!(c.enabled && c.preview && c.navigate);
        assert_eq!(c.length, 100);
        assert_eq!(c.notification_mode, "smart");
        assert_eq!(c.idle_seconds, 10);
    }
    #[test]
    fn config_roundtrip() {
        let c = Config {
            display_name: "My replies".into(),
            language: "ko".into(),
            theme: "dark".into(),
            enabled: false,
            notification_mode: "background".into(),
            idle_seconds: 60,
            preview: false,
            length: 150,
            navigate: false,
            sound: "custom".into(),
            custom_icon: true,
        };
        assert_eq!(Config::parse(&c.render()), c);
    }
    #[test]
    fn invalid_options() {
        let c = Config::parse(
            "preview_length=10000\nsound=\"bad\"\npreview=no\nnotification_mode=\"bad\"\nidle_seconds=999",
        );
        assert_eq!(c, Config::default());
    }
    #[test]
    fn length_unicode() {
        assert_eq!(shorten(&"가".repeat(200), 50).encode_utf16().count(), 51);
        assert_eq!(shorten(&"🙂".repeat(100), 150).encode_utf16().count(), 151);
    }
}

// Resolve only the installed, publisher-identified desktop package. No profile
// access, downloads, user-provided commands, or bundled third-party assets.
pub fn installed_asset(icon: bool) -> std::result::Result<PathBuf, String> {
    use std::os::windows::process::CommandExt;
    let system = env::var_os("SystemRoot").ok_or("Windows directory unavailable.")?;
    let powershell = PathBuf::from(system).join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let output = std::process::Command::new(powershell)
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
          "$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); $p=Get-AppxPackage -Name OpenAI.Codex | Where-Object {$_.PackageFamilyName -eq 'OpenAI.Codex_2p2nqsd0c76g0'} | Sort-Object Version -Descending | Select-Object -First 1; if(!$p){exit 2}; [Console]::Write($p.InstallLocation)"])
        .creation_flags(0x08000000)
        .output().map_err(|_| "Could not query installed Windows packages.")?;
    if !output.status.success() {
        return Err(
            "Supported ChatGPT desktop package not found. Choose a local file instead.".into(),
        );
    }
    let location = String::from_utf8(output.stdout).map_err(|_| "Invalid package location.")?;
    let location = PathBuf::from(location.trim());
    if !location.is_absolute() || location.as_os_str().is_empty() {
        return Err("Invalid package location.".into());
    }
    let resource = location.join(r"app\resources");
    let names: &[&str] = if icon {
        &["chatgpt-app-dark.ico", "icon-chatgpt.ico"]
    } else {
        &["codex-notification.wav"]
    };
    names
        .iter()
        .map(|name| resource.join(name))
        .find(|p| p.is_file())
        .ok_or_else(|| {
            "Known local asset not found in this app version. Choose a local file instead.".into()
        })
}
pub fn import_installed(root: &Path, icon: bool) -> std::result::Result<(), String> {
    let path = installed_asset(icon)?;
    if icon {
        import_icon(root, &path)
    } else {
        import_sound(root, &path)
    }
}

pub fn valid_name(value: &str) -> std::result::Result<String, String> {
    let value = value.trim();
    if value.is_empty()||value.encode_utf16().count()>48||value.chars().any(|c|c.is_control()||matches!(c,'"'|'\\'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'|'\u{200e}'|'\u{200f}')){
  return Err("Enter a display name of 1-48 characters without quotes, backslashes or control characters.".into())
 }
    Ok(value.to_owned())
}
pub fn set_display_name(value: &str) -> std::result::Result<(), String> {
    let value = valid_name(value)?;
    let sub = wide(r"Software\Classes\AppUserModelId\ReplyNoti.Notifier");
    let name = wide("DisplayName");
    let v = wide(&value);
    let mut k = HKEY::default();
    unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(sub.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut k,
        )
        .ok()
        .map_err(|_| "Cannot open ReplyNoti notification registration.")?;
        let bytes = std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2);
        let result = RegSetValueExW(k, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes));
        let _ = RegCloseKey(k);
        result
            .ok()
            .map_err(|_| "Cannot update notification display name.".to_string())
    }
}
#[cfg(test)]
#[test]
fn display_name_validation() {
    assert_eq!(valid_name("  My Chat  ").unwrap(), "My Chat");
    assert_eq!(valid_name("나의 알림").unwrap(), "나의 알림");
    for x in ["", "   ", "a\nb", "a\"b", "a\\b", "\u{202e}a"] {
        assert!(valid_name(x).is_err())
    }
    assert!(valid_name(&"x".repeat(49)).is_err());
}
