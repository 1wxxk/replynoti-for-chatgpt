#![windows_subsystem = "windows"]
use std::{env,fs,path::{Path,PathBuf}};
use windows::{core::PCWSTR,Win32::System::Registry::{HKEY_CURRENT_USER,RRF_RT_REG_DWORD,RRF_RT_REG_QWORD,RegGetValueW}};
use serde::{Serialize,Deserialize};
use base64::{Engine,engine::general_purpose::STANDARD};
#[path="../../../src/settings.rs"]
mod settings;
const APP_ID:&str="ReplyNoti.Notifier";
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(std::iter::once(0)).collect()}
fn reg_value(app:&str,name:&str,qword:bool)->Option<u64>{
 let path=wide(&format!(r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\{app}"));
 let name=wide(name);let mut val=0u64;let mut bytes=if qword{8}else{4};
 let code=unsafe{RegGetValueW(HKEY_CURRENT_USER,PCWSTR(path.as_ptr()),PCWSTR(name.as_ptr()),if qword{RRF_RT_REG_QWORD}else{RRF_RT_REG_DWORD},None,Some((&mut val as *mut u64).cast()),Some(&mut bytes))};
 if code.0==0{Some(val)}else{None}
}
fn root()->Result<PathBuf,String>{
 let root=PathBuf::from(env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?).join("ReplyNoti");
 if !root.join(".replynoti-install").is_file(){return Err("ReplyNoti must be installed first.".into())}Ok(root)
}
#[derive(Serialize,Deserialize)]
struct Preferences{
 display_name:String,enabled:bool,notification_mode:String,idle_seconds:u32,preview:bool,length:usize,navigate:bool,sound:String,custom_icon:bool,startup:bool,language:String,theme:String,
}
#[derive(Serialize)]
struct State{preferences:Preferences,icon_data:String,has_sound:bool,has_icon:bool}
fn state(r:&Path)->State{
 let c=settings::Config::load(r);
 let icon=fs::read(c.icon(r,"png")).ok().filter(|b|b.len()<1024*1024).map(|b|format!("data:image/png;base64,{}",STANDARD.encode(b))).unwrap_or_default();
 State{preferences:Preferences{display_name:c.display_name,enabled:c.enabled,notification_mode:c.notification_mode,idle_seconds:c.idle_seconds,preview:c.preview,length:c.length,navigate:c.navigate,sound:c.sound,custom_icon:c.custom_icon,startup:settings::startup(r),language:c.language,theme:c.theme},icon_data:icon,has_sound:r.join("custom-sound.wav").is_file(),has_icon:r.join("custom-icon.png").is_file()&&r.join("custom-icon.ico").is_file()}
}
fn set_icon_registration(r:&Path,c:&settings::Config)->Result<(),String>{
 use windows::Win32::System::Registry::{HKEY,KEY_SET_VALUE,REG_SZ,RegOpenKeyExW,RegSetValueExW,RegCloseKey};
 let sub=wide(r"Software\Classes\AppUserModelId\ReplyNoti.Notifier");
 let name=wide("IconUri");let val=wide(&c.icon(r,"png").to_string_lossy());let mut k=HKEY::default();
 unsafe{
 RegOpenKeyExW(HKEY_CURRENT_USER,PCWSTR(sub.as_ptr()),None,KEY_SET_VALUE,&mut k).ok().map_err(|_|"Cannot open notification registration")?;
 let b=std::slice::from_raw_parts(val.as_ptr().cast::<u8>(),val.len()*2);
 let ret=RegSetValueExW(k,PCWSTR(name.as_ptr()),None,REG_SZ,Some(b));let _=RegCloseKey(k);ret.ok().map_err(|_|"Cannot update notification icon".into())
 }
}
#[tauri::command]
fn load_settings()->Result<State,String>{Ok(state(&root()?))}
#[tauri::command]
fn save_settings(preferences:Preferences)->Result<State,String>{
 let r=root()?;let mut c=settings::Config::load(&r);
 let name=settings::valid_name(&preferences.display_name)?;
 if !matches!(preferences.notification_mode.as_str(),"smart"|"always"|"background")||!matches!(preferences.idle_seconds,10|30|60|180)||!matches!(preferences.length,50|100|150)||!matches!(preferences.sound.as_str(),"default"|"silent"|"custom")||!matches!(preferences.language.as_str(),"system"|"ko"|"en")||!matches!(preferences.theme.as_str(),"system"|"light"|"dark"){return Err("Invalid setting value.".into())}
 if preferences.sound=="custom"&&!r.join("custom-sound.wav").is_file(){return Err("Choose a WAV file first.".into())}
 if preferences.custom_icon&&!(r.join("custom-icon.png").is_file()&&r.join("custom-icon.ico").is_file()){return Err("Choose an icon first.".into())}
 c.display_name=name;c.enabled=preferences.enabled;c.notification_mode=preferences.notification_mode;c.idle_seconds=preferences.idle_seconds;c.preview=preferences.preview;c.length=preferences.length;c.navigate=preferences.navigate;c.sound=preferences.sound;c.custom_icon=preferences.custom_icon;c.language=preferences.language;c.theme=preferences.theme;
 settings::set_startup(&r,preferences.startup)?;settings::set_display_name(&c.display_name)?;set_icon_registration(&r,&c)?;c.save(&r).map_err(|_|"Cannot save local preferences.")?;Ok(state(&r))
}
#[tauri::command]
async fn import_asset(kind:String,source:Option<String>)->Result<State,String>{
 tauri::async_runtime::spawn_blocking(move||{
 let r=root()?;let icon=match kind.as_str(){"icon"=>true,"sound"=>false,_=>return Err("Unknown asset type".into())};
 match source {Some(path)=>{let p=Path::new(&path);if !p.is_absolute(){return Err("Choose an absolute local path.".into())}if icon{settings::import_icon(&r,p)?}else{settings::import_sound(&r,p)?}},None=>settings::import_installed(&r,icon)?}
 let mut c=settings::Config::load(&r);
 if icon{c.custom_icon=true;set_icon_registration(&r,&c)?}else{c.sound="custom".into()}
 c.save(&r).map_err(|_|"Could not save imported asset.")?;Ok(state(&r))
 }).await.map_err(|_|"Asset import failed".to_string())?
}
#[tauri::command]
async fn test_notification()->Result<(),String>{
 tauri::async_runtime::spawn_blocking(||{
 let r=root()?;let status=std::process::Command::new(r.join("replynoti.exe")).arg("--test-toast").status().map_err(|_|"Could not run test notification.")?;
 if status.success(){Ok(())}else{Err("Windows notification test failed.".into())}
 }).await.map_err(|_|"Notification test failed".to_string())?
}
fn main(){
 tauri::Builder::default().plugin(tauri_plugin_dialog::init()).invoke_handler(tauri::generate_handler![load_settings,save_settings,import_asset,test_notification]).run(tauri::generate_context!()).expect("settings window");
}
