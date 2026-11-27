[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$root=Join-Path $env:LOCALAPPDATA 'ReplyNoti'
$marker=Join-Path $root '.replynoti-install'
if(!(Test-Path -LiteralPath $marker) -or (Get-Content -LiteralPath $marker -Raw).Trim() -ne 'ReplyNoti.Notifier'){throw 'No managed ReplyNoti installation found; nothing changed.'}
$exe=Join-Path $root 'replynoti.exe'
$runKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$run=(Get-ItemProperty -LiteralPath $runKey -ErrorAction SilentlyContinue).ReplyNoti
if($run -and $run -ne ('"'+$exe+'" --run')){throw 'Unexpected startup entry; refusing to remove.'}
$protocolKey='HKCU:\Software\Classes\replynoti-open'
if(Test-Path -LiteralPath $protocolKey){
 $cmd=Join-Path $protocolKey 'shell\open\command'
 if(!(Test-Path -LiteralPath $cmd) -or (Get-Item -LiteralPath $cmd).GetValue('') -ne ('"'+$exe+'" --activate "%1"')){throw 'Unexpected protocol handler; refusing to remove.'}
}
$settingsExe=Join-Path $root 'replynoti-settings.exe'
Get-Process -Name replynoti-settings -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq $settingsExe}|Stop-Process -ErrorAction Stop
$flag=Join-Path $root 'stop.flag'
New-Item -ItemType File -Path $flag -Force | Out-Null
for($i=0;$i -lt 25;$i++){
 $running=@(Get-Process -Name replynoti -ErrorAction SilentlyContinue|Where-Object {$_.Path -eq $exe})
 if(!$running.Count){break}
 Start-Sleep -Milliseconds 200
}
if($running.Count){throw 'Watcher has not stopped; retry uninstall after it exits.'}
Remove-ItemProperty -LiteralPath $runKey -Name ReplyNoti -ErrorAction SilentlyContinue
$link=Join-Path ([Environment]::GetFolderPath('Programs')) 'ReplyNoti.lnk'
if(Test-Path -LiteralPath $link){
 $s=(New-Object -ComObject WScript.Shell).CreateShortcut($link)
 if($s.TargetPath -eq $exe){Remove-Item -LiteralPath $link -Force}
}
$key='HKCU:\Software\Classes\AppUserModelId\ReplyNoti.Notifier'
if(Test-Path -LiteralPath $key){Remove-Item -LiteralPath $key -Recurse -Force}
if(Test-Path -LiteralPath $protocolKey){Remove-Item -LiteralPath $protocolKey -Recurse -Force}
$ticketDir=Join-Path $root 'click-tickets'
if(Test-Path -LiteralPath $ticketDir -PathType Container){
 Get-ChildItem -LiteralPath $ticketDir -File|Where-Object {$_.Name -cmatch '^[0-9a-f]{32}\.(pending|clicked)$'}|Remove-Item -Force
 if(@(Get-ChildItem -LiteralPath $ticketDir -Force).Count -eq 0){Remove-Item -LiteralPath $ticketDir}
}
# Leave any unrelated files alone, including user-created files.
foreach($n in @('replynoti.exe','replynoti-settings.exe','replynoti.png','replynoti.ico','Uninstall.ps1','Uninstall.cmd','LICENSE','THIRD_PARTY_NOTICES.txt','replynoti.log','replynoti.log.1','replynoti.log.2','probe.txt','silent.flag','preview.on','config.toml','custom-icon.png','custom-icon.ico','custom-sound.wav','stop.flag','.replynoti-install')){
 $p=Join-Path $root $n
 if(Test-Path -LiteralPath $p -PathType Leaf){Remove-Item -LiteralPath $p -Force}
}
if(@(Get-ChildItem -LiteralPath $root -Force).Count -eq 0){Remove-Item -LiteralPath $root}
Write-Output 'ReplyNoti uninstalled. Other ChatGPT applications were not changed.'
