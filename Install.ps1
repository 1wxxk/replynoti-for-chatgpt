[CmdletBinding()]
param([switch]$AutoStart,[switch]$NoStart,[switch]$Preview,[switch]$NoPreview)
$ErrorActionPreference='Stop'
$root=Join-Path $env:LOCALAPPDATA 'ReplyNoti'
$marker=Join-Path $root '.replynoti-install'
$appId='ReplyNoti.Notifier'
$appKey="HKCU:\Software\Classes\AppUserModelId\$appId"
$protocolKey='HKCU:\Software\Classes\replynoti-open'
if(Test-Path -LiteralPath $protocolKey){throw 'ReplyNoti protocol already registered; refusing to overwrite.'}
$runKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runName='ReplyNoti'
$link=Join-Path ([Environment]::GetFolderPath('Programs')) 'ReplyNoti.lnk'
$dest=Join-Path $root 'replynoti.exe'
$src=Join-Path $PSScriptRoot 'replynoti.exe'
if(!(Test-Path -LiteralPath $src)){$src=Join-Path $PSScriptRoot 'target\release\replynoti.exe'}
if(!(Test-Path -LiteralPath $src)){throw 'Build or download replynoti.exe first.'}
if($env:PROCESSOR_ARCHITECTURE -ne 'AMD64'){throw 'This beta supports Windows x64 only.'}
if([Environment]::OSVersion.Version.Build -lt 22000){throw 'This beta targets Windows 11.'}
if(Test-Path -LiteralPath $root){throw 'ReplyNoti data directory already exists. Uninstall the previous version or move its data first.'}
if((Test-Path -LiteralPath $appKey) -or (Test-Path -LiteralPath $link) -or (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PSObject.Properties[$runName]){throw 'Existing ReplyNoti registration; refusing to overwrite.'}
# Use the Windows Shell Link property store to set AppUserModelID.
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
public static class ReplyNotiLink {
 [StructLayout(LayoutKind.Sequential, Pack=4)] struct Key {public Guid fmtid;public uint pid;}
 [StructLayout(LayoutKind.Explicit, Size=24)] struct Prop {
   [FieldOffset(0)] public ushort vt;
   [FieldOffset(8)] public IntPtr ptr;
 }
 [ComImport,Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99"),InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
 interface Store {
   void GetCount(out uint count);
   void GetAt(uint index,out Key key);
   void GetValue(ref Key key,out Prop value);
   void SetValue(ref Key key,ref Prop value);
   void Commit();
 }
 [ComImport,Guid("00021401-0000-0000-C000-000000000046")] class ShellLink {}
 public static void SetAppId(string path,string id) {
   object obj=new ShellLink();
   var file=(IPersistFile)obj;
   file.Load(path,2);
   var store=(Store)obj;
   Key key=new Key{fmtid=new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"),pid=5};
   Prop p=new Prop{vt=31,ptr=Marshal.StringToCoTaskMemUni(id)};
   try {store.SetValue(ref key,ref p);store.Commit();file.Save(path,true);}
   finally {Marshal.FreeCoTaskMem(p.ptr);Marshal.FinalReleaseComObject(obj);}
 }
}
'@
$p=$null
$protocolCreated=$false
try {
 New-Item -ItemType Directory -Path $root | Out-Null
 Set-Content -LiteralPath $marker -Value 'ReplyNoti.Notifier' -Encoding ASCII
 if($Preview -and $NoPreview){throw 'Choose either -Preview or -NoPreview.'}
 Write-Warning "Reply previews are enabled by default. They may appear on the lock screen and in notification history. Filtering is best effort. Use -NoPreview or turn previews off in Settings."
 $previewValue=(!$NoPreview).ToString().ToLowerInvariant()
 @('display_name = "ReplyNoti"','language = "system"','theme = "system"',"enabled = true","preview = $previewValue","preview_length = 100","open_conversation = true",'sound = "default"',"custom_icon = false") | Set-Content -LiteralPath (Join-Path $root 'config.toml') -Encoding ASCII
 Copy-Item -LiteralPath $src -Destination $dest
 $settingsSource=Join-Path $PSScriptRoot 'replynoti-settings.exe'
 if(!(Test-Path -LiteralPath $settingsSource)){$settingsSource=Join-Path $PSScriptRoot 'settings-ui\src-tauri\target\release\replynoti-settings.exe'}
 if(Test-Path -LiteralPath $settingsSource){Copy-Item -LiteralPath $settingsSource -Destination (Join-Path $root 'replynoti-settings.exe');if((Get-FileHash -LiteralPath $settingsSource).Hash -ne (Get-FileHash -LiteralPath (Join-Path $root 'replynoti-settings.exe')).Hash){throw 'Settings binary copy verification failed'}}
 if((Get-FileHash -LiteralPath $src).Hash -ne (Get-FileHash -LiteralPath $dest).Hash){throw 'Binary copy verification failed'}
 foreach($asset in @('replynoti.png','replynoti.ico','Uninstall.ps1','Uninstall.cmd','LICENSE','THIRD_PARTY_NOTICES.txt')){
  $file=Join-Path $PSScriptRoot $asset
  if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination $root}
 }
 New-Item -Path $appKey -Force|Out-Null
 New-ItemProperty -LiteralPath $appKey -Name DisplayName -Value 'ReplyNoti' -PropertyType String -Force|Out-Null
 $icon=Join-Path $root 'replynoti.png'
 if(Test-Path -LiteralPath $icon){New-ItemProperty -LiteralPath $appKey -Name IconUri -Value $icon -PropertyType String -Force|Out-Null}
 $shell=New-Object -ComObject WScript.Shell
 $s=$shell.CreateShortcut($link)
 $s.TargetPath=$dest;$s.Arguments='--settings';$s.WorkingDirectory=$root;$s.Description='ReplyNoti - unofficial notification helper'
 if(Test-Path (Join-Path $root 'replynoti.ico')){$s.IconLocation=(Join-Path $root 'replynoti.ico')+',0'}
 $s.Save()
 [ReplyNotiLink]::SetAppId($link,$appId)
 New-Item -Path $protocolKey -ErrorAction Stop|Out-Null
 $protocolCreated=$true
 Set-Item -LiteralPath $protocolKey -Value 'URL:ReplyNoti notification'
 New-ItemProperty -LiteralPath $protocolKey -Name 'URL Protocol' -PropertyType String -Value ''|Out-Null
 $commandKey=Join-Path $protocolKey 'shell\open\command'
 New-Item -Path $commandKey -Force|Out-Null
 $command='"'+$dest+'" --activate "%1"'
 Set-Item -LiteralPath $commandKey -Value $command
 if((Get-Item -LiteralPath $commandKey).GetValue('') -ne $command){throw 'Protocol verification failed'}
 if($AutoStart){New-ItemProperty -LiteralPath $runKey -Name $runName -Value ('"'+$dest+'" --run') -PropertyType String -Force|Out-Null}
 if(!$NoStart){$p=Start-Process -FilePath $dest -ArgumentList '--run' -WorkingDirectory $root -PassThru;Start-Sleep -Milliseconds 800;$p.Refresh();if($p.HasExited){throw 'Watcher exited early; check supported session.'}}
 Write-Output "Installed: $root"
 Write-Output 'Run replynoti.exe --test-toast to test notifications.'
}catch{
 if($protocolCreated -and (Test-Path -LiteralPath $protocolKey)){Remove-Item -LiteralPath $protocolKey -Recurse -Force}
 if($p -and !$p.HasExited){Stop-Process -Id $p.Id -ErrorAction SilentlyContinue}
 Remove-ItemProperty -LiteralPath $runKey -Name $runName -ErrorAction SilentlyContinue
 if(Test-Path -LiteralPath $link){Remove-Item -LiteralPath $link -Force}
 if(Test-Path -LiteralPath $appKey){Remove-Item -LiteralPath $appKey -Recurse -Force}
 if(Test-Path -LiteralPath $marker){Remove-Item -LiteralPath $root -Recurse -Force}
 throw
}
