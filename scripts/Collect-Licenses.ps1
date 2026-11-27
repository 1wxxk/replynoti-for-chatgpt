$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
Push-Location $root
try{
 $m=cargo metadata --format-version 1 --locked | ConvertFrom-Json
 if($LASTEXITCODE -ne 0){throw 'cargo metadata failed'}
 $lines=New-Object 'System.Collections.Generic.List[string]'
 $lines.Add('Third-party notices for ReplyNoti. Includes direct and transitive Cargo dependencies.')
 foreach($p in ($m.packages | Where-Object {$_.source -like 'registry+*'}|Sort-Object name,version)){
  $lines.Add('');$lines.Add(('='*72));$lines.Add("$($p.name) $($p.version) | $($p.license)")
  $lines.Add("https://crates.io/crates/$($p.name)/$($p.version)")
  $folder=Split-Path $p.manifest_path -Parent
  $licenseFiles=@(Get-ChildItem -LiteralPath $folder -File | Where-Object {$_.Name -match '^(LICENSE|COPYING|NOTICE|COPYRIGHT)'})
  if(!$licenseFiles.Count){throw "Missing bundled license text for $($p.name)"}
  foreach($f in $licenseFiles){$lines.Add("--- $($f.Name) ---");$lines.Add((Get-Content -LiteralPath $f.FullName -Raw -Encoding UTF8))}
 }
 [IO.File]::WriteAllLines((Join-Path $root 'THIRD_PARTY_NOTICES.txt'),$lines,(New-Object Text.UTF8Encoding($false)))
 Write-Output "Dependency licenses collected: $(@($m.packages|Where-Object {$_.source -like 'registry+*'}).Count)"
}finally{Pop-Location}
