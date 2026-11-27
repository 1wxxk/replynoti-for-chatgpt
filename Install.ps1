[CmdletBinding()]
param([switch]$AutoStart,[switch]$NoAutoStart,[switch]$NoStart,[switch]$Preview,[switch]$NoPreview,[switch]$Portable,[switch]$Upgrade,[string]$PortableRoot)
$ErrorActionPreference='Stop'
$root=Join-Path $env:LOCALAPPDATA 'ReplyNoti'
if($Portable){if($PortableRoot){$root=(Resolve-Path -LiteralPath $PortableRoot -ErrorAction Stop).Path}else{$root=$PSScriptRoot};if(!(Test-Path (Join-Path $root 'portable.flag') -PathType Leaf)){throw 'Portable package marker missing.'}}
$payload=if($Portable){$root}else{$PSScriptRoot}
if($Portable -and $Upgrade){throw 'Choose either portable registration or installed update.'}
if($AutoStart -and $NoAutoStart){throw 'Choose either -AutoStart or -NoAutoStart.'}
if($Preview -and $NoPreview){throw 'Choose either -Preview or -NoPreview.'}
$marker=Join-Path $root '.replynoti-install'
$appId='ReplyNoti.Notifier'
$appKey="HKCU:\Software\Classes\AppUserModelId\$appId"
$protocolKey='HKCU:\Software\Classes\replynoti-open'
$runKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runName='ReplyNoti'
$link=Join-Path ([Environment]::GetFolderPath('Programs')) 'ReplyNoti.lnk'
$dest=Join-Path $root 'replynoti.exe'
$src=Join-Path $payload 'replynoti.exe'
if(!(Test-Path -LiteralPath $src)){$src=Join-Path $PSScriptRoot 'target\release\replynoti.exe'}
if(!(Test-Path -LiteralPath $src)){throw 'Build or download replynoti.exe first.'}
if($env:PROCESSOR_ARCHITECTURE -ne 'AMD64'){throw 'This beta supports Windows x64 only.'}
if([Environment]::OSVersion.Version.Build -lt 22000){throw 'This beta targets Windows 11.'}
$arp='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ReplyNoti'
$command='"'+$dest+'" --activate "%1"'
$expectedRun='"'+$dest+'" --run'
$assets=@('replynoti.png','replynoti.ico','Uninstall.ps1','Uninstall.cmd','Uninstall.exe','LICENSE','THIRD_PARTY_NOTICES.txt','NSIS-LICENSE.txt')
$settingsSource=Join-Path $payload 'replynoti-settings.exe'
if(!(Test-Path $settingsSource)){$settingsSource=Join-Path $PSScriptRoot 'settings-ui\src-tauri\target\release\replynoti-settings.exe'}
if(!(Test-Path $settingsSource -PathType Leaf)){throw 'Settings binary missing.'}
function Assert-Registration {
 if(!(Test-Path -LiteralPath $marker -PathType Leaf) -or (Get-Content $marker -Raw).Trim() -ne 'ReplyNoti.Notifier'){throw 'Managed installation marker missing.'}
 $cmdKey=Join-Path $protocolKey 'shell\open\command'
 if(!(Test-Path $cmdKey) -or (Get-Item $cmdKey).GetValue('') -ne $command){throw 'Registration belongs to a different installation. Remove or unregister it first.'}
 if(!(Test-Path $appKey) -or !(Test-Path $link)){throw 'Incomplete registration. Repair or remove the existing installation first.'}
 $shortcut=(New-Object -ComObject WScript.Shell).CreateShortcut($link)
 if($shortcut.TargetPath -ne $dest){throw 'Existing shortcut belongs to a different installation.'}
 $run=(Get-ItemProperty $runKey -ErrorAction SilentlyContinue).ReplyNoti
 if($run -and $run -ne $expectedRun){throw 'Startup registration belongs to another installation.'}
 if(Test-Path $arp){
  $loc=(Get-ItemProperty $arp).InstallLocation
  if($Portable -or $loc -ne $root){throw 'Existing Windows app registration belongs to a different installation.'}
 }
}
function Register-Uninstall {
 if($Portable){return}
 $u=Join-Path $root 'Uninstall.exe'
 if(!(Test-Path $u)){return}
 New-Item -Path $arp -Force|Out-Null
 $properties=@{DisplayName='ReplyNoti';DisplayVersion='0.1.0-beta.9';Publisher='ReplyNoti contributors';InstallLocation=$root;DisplayIcon=(Join-Path $root 'replynoti.ico');UninstallString=('"'+$u+'"');QuietUninstallString=('"'+$u+'" /S');URLInfoAbout='https://github.com/1wxxk/replynoti-for-chatgpt'}
 foreach($k in $properties.Keys){New-ItemProperty -Path $arp -Name $k -Value $properties[$k] -PropertyType String -Force|Out-Null}
 foreach($k in @('NoModify','NoRepair')){New-ItemProperty -Path $arp -Name $k -Value 1 -PropertyType DWord -Force|Out-Null}
}
if(Test-Path $marker){
 Assert-Registration
 if($Portable){
  if(!$NoStart){Start-Process -FilePath $dest -ArgumentList '--run' -WorkingDirectory $root}
  Write-Output "Portable ReplyNoti registered: $root";return
 }
 if(!$Upgrade){throw 'Existing installation found; use the setup EXE to update.'}
 # Stage only managed files; configuration, custom assets and diagnostics remain untouched.
 $stage=Join-Path $root ('.update-'+[guid]::NewGuid().ToString('N'))
 $backup=Join-Path $stage 'backup';New-Item -ItemType Directory -Path $backup -Force|Out-Null
 $files=@{'replynoti.exe'=$src;'replynoti-settings.exe'=$settingsSource}
 foreach($n in $assets){$f=Join-Path $PSScriptRoot $n;if(Test-Path $f -PathType Leaf){$files[$n]=$f}}
 $swapped=@();$running=@(Get-Process -Name replynoti -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq $dest})
 try {
  foreach($n in $files.Keys){Copy-Item -LiteralPath $files[$n] -Destination (Join-Path $stage $n);if((Get-FileHash $files[$n]).Hash -ne (Get-FileHash (Join-Path $stage $n)).Hash){throw 'Staging checksum mismatch.'}}
  $se=Join-Path $root 'replynoti-settings.exe'
  Get-Process -Name replynoti-settings -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq $se}|Stop-Process -ErrorAction Stop
  if($running.Count){New-Item -ItemType File -Path (Join-Path $root 'stop.flag') -Force|Out-Null;foreach($process in $running){if(!$process.WaitForExit(15000)){throw 'Close ReplyNoti and retry the update.'}}}
  foreach($n in $files.Keys){$target=Join-Path $root $n;if(Test-Path $target){Move-Item -LiteralPath $target -Destination (Join-Path $backup $n)};$swapped+=$n;Move-Item -LiteralPath (Join-Path $stage $n) -Destination $target}
  Register-Uninstall
  if($AutoStart){New-ItemProperty -Path $runKey -Name $runName -Value $expectedRun -PropertyType String -Force|Out-Null}
  elseif($NoAutoStart){Remove-ItemProperty -LiteralPath $runKey -Name $runName -ErrorAction SilentlyContinue}
  Remove-Item -LiteralPath (Join-Path $root 'stop.flag') -ErrorAction SilentlyContinue
  if(!$NoStart){Start-Process -FilePath $dest -ArgumentList '--run' -WorkingDirectory $root}
 } catch {
  foreach($n in $swapped){$target=Join-Path $root $n;Remove-Item -LiteralPath $target -ErrorAction SilentlyContinue;if(Test-Path (Join-Path $backup $n)){Move-Item -LiteralPath (Join-Path $backup $n) -Destination $target}}
  Remove-Item -LiteralPath (Join-Path $root 'stop.flag') -ErrorAction SilentlyContinue
  if($running.Count -and !(Get-Process -Name replynoti -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq $dest})){Start-Process -FilePath $dest -ArgumentList '--run' -WorkingDirectory $root}
  throw
 } finally {Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue}
 Write-Output "Updated: $root";return
}
if(Test-Path $protocolKey){throw 'Another ReplyNoti installation is registered. Remove or unregister it first.'}
if(!$Portable -and (Test-Path $root)){throw 'Unmanaged ReplyNoti data directory already exists; move or back it up before installing.'}
if(Test-Path $arp){throw 'Existing Windows app registration; refusing to overwrite.'}
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
 if(!$Portable){New-Item -ItemType Directory -Path $root | Out-Null}
 Set-Content -LiteralPath $marker -Value 'ReplyNoti.Notifier' -Encoding ASCII
 if($Preview -and $NoPreview){throw 'Choose either -Preview or -NoPreview.'}
 Write-Warning "Reply previews are enabled by default. They may appear on the lock screen and in notification history. Filtering is best effort. Use -NoPreview or turn previews off in Settings."
 $previewValue=(!$NoPreview).ToString().ToLowerInvariant()
 if(!(Test-Path (Join-Path $root 'config.toml'))){@('display_name = "ReplyNoti"','language = "system"','theme = "system"',"enabled = true",'notification_mode = "smart"',"idle_seconds = 10","preview = $previewValue","preview_length = 100","open_conversation = true",'sound = "default"',"custom_icon = false") | Set-Content -LiteralPath (Join-Path $root 'config.toml') -Encoding ASCII}
 if(!$Portable){Copy-Item -LiteralPath $src -Destination $dest
 $settingsSource=Join-Path $payload 'replynoti-settings.exe'
 if(!(Test-Path -LiteralPath $settingsSource)){$settingsSource=Join-Path $PSScriptRoot 'settings-ui\src-tauri\target\release\replynoti-settings.exe'}
 if(Test-Path -LiteralPath $settingsSource){Copy-Item -LiteralPath $settingsSource -Destination (Join-Path $root 'replynoti-settings.exe');if((Get-FileHash -LiteralPath $settingsSource).Hash -ne (Get-FileHash -LiteralPath (Join-Path $root 'replynoti-settings.exe')).Hash){throw 'Settings binary copy verification failed'}}
 if((Get-FileHash -LiteralPath $src).Hash -ne (Get-FileHash -LiteralPath $dest).Hash){throw 'Binary copy verification failed'}
 foreach($asset in $assets){
  $file=Join-Path $PSScriptRoot $asset
  if(Test-Path -LiteralPath $file){Copy-Item -LiteralPath $file -Destination $root}
 }
 }
 New-Item -Path $appKey -Force|Out-Null
 $cfg=Get-Content (Join-Path $root 'config.toml') -Raw
 $display='ReplyNoti';if($cfg -match '(?m)^display_name\s*=\s*"([^"\r\n]{1,48})"'){$display=$Matches[1]}
 New-ItemProperty -LiteralPath $appKey -Name DisplayName -Value $display -PropertyType String -Force|Out-Null
 $icon=Join-Path $root 'replynoti.png'
 if($cfg -match '(?m)^custom_icon\s*=\s*true' -and (Test-Path (Join-Path $root 'custom-icon.png'))){$icon=Join-Path $root 'custom-icon.png'}
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
 Register-Uninstall
 if(!$NoStart){$p=Start-Process -FilePath $dest -ArgumentList '--run' -WorkingDirectory $root -PassThru;Start-Sleep -Milliseconds 800;$p.Refresh();if($p.HasExited){throw 'Watcher exited early; check supported session.'}}
 Write-Output "Installed: $root"
 Write-Output 'Run replynoti.exe --test-toast to test notifications.'
}catch{
 if($protocolCreated -and (Test-Path -LiteralPath $protocolKey)){Remove-Item -LiteralPath $protocolKey -Recurse -Force}
 if($p -and !$p.HasExited){Stop-Process -Id $p.Id -ErrorAction SilentlyContinue}
 Remove-ItemProperty -LiteralPath $runKey -Name $runName -ErrorAction SilentlyContinue
 if(Test-Path -LiteralPath $link){Remove-Item -LiteralPath $link -Force}
 if(Test-Path -LiteralPath $appKey){Remove-Item -LiteralPath $appKey -Recurse -Force}
 if(Test-Path $arp){Remove-Item -LiteralPath $arp -Recurse -Force}
 if(Test-Path -LiteralPath $marker){if($Portable){Remove-Item -LiteralPath $marker -Force}else{Remove-Item -LiteralPath $root -Recurse -Force}}
 throw
}
