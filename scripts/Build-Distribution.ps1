param(
 [Parameter(Mandatory=$true)][string]$Makensis,
 [string]$EnginePath,
 [string]$SettingsPath,
 [string]$OutputDirectory
)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
if(!$EnginePath){$EnginePath=Join-Path $root 'target\release\replynoti.exe'}
if(!$SettingsPath){$SettingsPath=Join-Path $root 'settings-ui\src-tauri\target\release\replynoti-settings.exe'}
if(!$OutputDirectory){$OutputDirectory=Join-Path $root 'dist\0.1.0-beta.9'}
if(Test-Path $OutputDirectory){throw 'Output directory already exists; choose a fresh output directory.'}
New-Item -ItemType Directory -Path $OutputDirectory|Out-Null
$out=(Resolve-Path $OutputDirectory).Path
$payload=Join-Path $out 'payload'
$portable=Join-Path $out 'ReplyNoti-Portable-x64'
New-Item -ItemType Directory -Path $payload,$portable|Out-Null
Copy-Item -LiteralPath $EnginePath -Destination (Join-Path $payload 'replynoti.exe')
Copy-Item -LiteralPath $SettingsPath -Destination (Join-Path $payload 'replynoti-settings.exe')
foreach($n in @('replynoti.png','replynoti.ico','Install.ps1','Uninstall.ps1','Uninstall.cmd','LICENSE','THIRD_PARTY_NOTICES.txt')){
 Copy-Item -LiteralPath (Join-Path $root $n) -Destination $payload
}
Copy-Item -LiteralPath (Join-Path $root 'installer\setup-header.bmp') -Destination $payload
$nsisRoot=Split-Path $Makensis -Parent
Copy-Item -LiteralPath (Join-Path $nsisRoot 'COPYING') -Destination (Join-Path $payload 'NSIS-LICENSE.txt')
function Build-NSIS($script,$destination,[switch]$Unregister){
 $args=@('/V2',"/DPAYLOAD=$payload","/DOUTPUT=$destination")
 if($Unregister){$args+='/DUNREGISTER'}
 $args+=Join-Path $root "installer\$script"
 & $Makensis @args
 if($LASTEXITCODE -ne 0 -or !(Test-Path $destination)){throw "NSIS build failed: $script"}
}
$setup=Join-Path $out 'ReplyNoti-Setup-x64.exe'
Build-NSIS 'Setup.nsi' $setup
foreach($n in @('replynoti.exe','replynoti-settings.exe','replynoti.png','replynoti.ico','LICENSE','THIRD_PARTY_NOTICES.txt','NSIS-LICENSE.txt')){
 Copy-Item -LiteralPath (Join-Path $payload $n) -Destination $portable
}
Set-Content -LiteralPath (Join-Path $portable 'portable.flag') -Value 'ReplyNoti portable mode' -Encoding ASCII
Copy-Item -LiteralPath (Join-Path $root 'installer\PORTABLE-README.txt') -Destination (Join-Path $portable 'README.txt')
Build-NSIS 'Portable.nsi' (Join-Path $portable 'Start-Portable.exe')
Build-NSIS 'Portable.nsi' (Join-Path $portable 'Unregister-Portable.exe') -Unregister
$zip=Join-Path $out 'ReplyNoti-Portable-x64.zip'
Compress-Archive -Path $portable -DestinationPath $zip
$hashes=foreach($f in @($setup,$zip)){(Get-FileHash -Algorithm SHA256 -LiteralPath $f).Hash.ToLowerInvariant()+'  '+(Split-Path $f -Leaf)}
Set-Content -LiteralPath (Join-Path $out 'SHA256SUMS.txt') -Value $hashes -Encoding ASCII
Get-Item $setup,$zip,(Join-Path $out 'SHA256SUMS.txt')|Select-Object Name,Length
Get-Content (Join-Path $out 'SHA256SUMS.txt')
