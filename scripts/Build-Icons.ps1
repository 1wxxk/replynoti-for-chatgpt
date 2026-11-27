[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$source=Join-Path $root 'replynoti.svg'
$small=Join-Path $root 'replynoti-small.svg'
$svgo=Get-Command svgo.cmd -ErrorAction SilentlyContinue
if(!$svgo){throw 'Install SVGO and add it to PATH.'}
$resvg=Get-Command resvg.exe -ErrorAction SilentlyContinue
if(!$resvg){
 $fallback=Join-Path $env:LOCALAPPDATA 'AI-Tools\resvg'
 $resvg=Get-ChildItem -LiteralPath $fallback -Filter resvg.exe -Recurse -File -ErrorAction SilentlyContinue|Select-Object -First 1
}
if(!$resvg){throw 'Install resvg and add it to PATH.'}
$resvgExe=if($resvg.Source){$resvg.Source}else{$resvg.FullName}
$tmp=Join-Path $env:TEMP ('replynoti-icons-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp|Out-Null
try{
 $optimized=Join-Path $tmp 'optimized.svg'
 & $svgo.Source --multipass -i $source -o $optimized
 if($LASTEXITCODE -ne 0){throw 'SVGO failed'}
 [xml]$doc=Get-Content -LiteralPath $optimized -Raw
 if(!$doc.DocumentElement.GetAttribute('viewBox')){throw 'Missing viewBox'}
 if($doc.SelectNodes('//*[local-name()="image" or local-name()="script" or local-name()="foreignObject"]').Count){throw 'SVG must be pure vector artwork'}
 $smallOptimized=Join-Path $tmp 'small.svg'
 & $svgo.Source --multipass -i $small -o $smallOptimized
 if($LASTEXITCODE -ne 0){throw 'SVGO small icon failed'}
 [xml]$smallDoc=Get-Content -LiteralPath $smallOptimized -Raw
 if(!$smallDoc.DocumentElement.GetAttribute('viewBox') -or $smallDoc.SelectNodes('//*[local-name()="image" or local-name()="script" or local-name()="foreignObject"]').Count){throw 'Invalid small icon SVG'}
 $sizes=@(16,20,24,32,40,48,64,128,256)
 $images=@()
 foreach($size in $sizes){
  $png=Join-Path $tmp "$size.png"
  $drawing=if($size -le 32){$smallOptimized}else{$optimized}
  & $resvgExe -w $size -h $size $drawing $png
  if($LASTEXITCODE -ne 0){throw "resvg failed: $size"}
  $images+=,[IO.File]::ReadAllBytes($png)
 }
 # Standard multi-resolution ICO container, each frame an exact-size PNG.
 $ico=Join-Path $tmp 'replynoti.ico'
 $stream=[IO.File]::Create($ico);$writer=New-Object IO.BinaryWriter($stream)
 try{
  $writer.Write([uint16]0);$writer.Write([uint16]1);$writer.Write([uint16]$sizes.Count)
  $offset=6+16*$sizes.Count
  for($i=0;$i -lt $sizes.Count;$i++){
   $dimension=if($sizes[$i] -eq 256){0}else{$sizes[$i]}
   $writer.Write([byte]$dimension);$writer.Write([byte]$dimension)
   $writer.Write([byte]0);$writer.Write([byte]0);$writer.Write([uint16]1);$writer.Write([uint16]32)
   $writer.Write([uint32]$images[$i].Length);$writer.Write([uint32]$offset)
   $offset+=$images[$i].Length
  }
  foreach($image in $images){$writer.Write([byte[]]$image)}
 }finally{$writer.Dispose()}
 Copy-Item -LiteralPath (Join-Path $tmp '256.png') -Destination (Join-Path $root 'replynoti.png') -Force
 Copy-Item -LiteralPath $ico -Destination (Join-Path $root 'replynoti.ico') -Force
 & $resvgExe -w 512 -h 512 $optimized (Join-Path $root 'installer\setup-icon.png')
 if($LASTEXITCODE -ne 0){throw 'Setup icon rendering failed'}
 Copy-Item -LiteralPath (Join-Path $root 'replynoti.png') -Destination (Join-Path $root 'settings-ui\public\replynoti.png') -Force
 Copy-Item -LiteralPath (Join-Path $root 'replynoti.ico') -Destination (Join-Path $root 'settings-ui\src-tauri\icons\icon.ico') -Force
 Write-Output ('Built icon sizes: '+($sizes -join ', '))
}finally{Remove-Item -LiteralPath $tmp -Recurse -Force}
