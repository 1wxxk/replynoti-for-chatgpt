$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$dist=Join-Path $root 'dist'
$stage=Join-Path $dist 'replynoti-windows-x64'
New-Item -ItemType Directory -Path $dist -Force | Out-Null
if(Test-Path $stage){throw 'Existing staging folder: remove it manually before packaging.'}
New-Item -ItemType Directory -Path $stage|Out-Null
Copy-Item -LiteralPath (Join-Path $root 'target\release\replynoti.exe') -Destination $stage
Copy-Item -LiteralPath (Join-Path $root 'settings-ui\src-tauri\target\release\replynoti-settings.exe') -Destination $stage
foreach($name in @('Install.ps1','Install.cmd','Uninstall.ps1','Uninstall.cmd','replynoti.svg','replynoti.ico','replynoti.png','README.md','README.ko.md','LICENSE','THIRD_PARTY_NOTICES.txt')){
 Copy-Item -LiteralPath (Join-Path $root $name) -Destination $stage
}
Copy-Item -LiteralPath (Join-Path $root 'docs') -Destination $stage -Recurse
$zip=Join-Path $dist 'replynoti-windows-x64.zip'
if(Test-Path $zip){throw 'Existing archive; remove it manually before packaging.'}
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
$hash=(Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath (Join-Path $dist 'SHA256SUMS.txt') -Value ($hash+'  replynoti-windows-x64.zip') -Encoding ASCII
Write-Output "Package: $zip"
Write-Output $hash
