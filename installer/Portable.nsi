Unicode true
!include "LogicLib.nsh"
!include "x64.nsh"
!include "FileFunc.nsh"
!ifndef PAYLOAD
 !error "Specify PAYLOAD"
!endif
!ifndef OUTPUT
 !error "Specify OUTPUT"
!endif
Name "ReplyNoti Portable"
OutFile "${OUTPUT}"
RequestExecutionLevel user
SilentInstall silent
SetCompressor /SOLID zlib
Icon "${PAYLOAD}\replynoti.ico"
VIProductVersion "0.1.0.11"
VIAddVersionKey /LANG=1033 "ProductName" "ReplyNoti"
VIAddVersionKey /LANG=1033 "FileDescription" "ReplyNoti Portable launcher"
VIAddVersionKey /LANG=1033 "FileVersion" "0.1.0-beta.11"
VIAddVersionKey /LANG=1033 "LegalCopyright" "ReplyNoti contributors"
Section
 ${IfNot} ${RunningX64}
  MessageBox MB_ICONSTOP "Windows 11 x64 is required." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 InitPluginsDir
 ; Keep the PowerShell working directory separate from NSIS plug-in DLLs.
 SetOutPath "$PLUGINSDIR\payload"
 !ifdef UNREGISTER
  File "${PAYLOAD}\Uninstall.ps1"
  nsExec::Exec '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File "$PLUGINSDIR\payload\Uninstall.ps1" -Portable -PortableRoot "$EXEDIR"'
 !else
  File "${PAYLOAD}\Install.ps1"
  StrCpy $1 ""
  ${GetParameters} $2
  ClearErrors
  ; /NOSTART is a documented smoke-test switch; normal launches start the watcher.
  ${If} $2 == "/NOSTART"
  ${OrIf} $2 == "/S /NOSTART"
  ${OrIf} $2 == "/NOSTART /S"
   StrCpy $1 "-NoStart"
  ${EndIf}
  nsExec::Exec '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File "$PLUGINSDIR\payload\Install.ps1" -Portable -PortableRoot "$EXEDIR" $1'
 !endif
 Pop $0
 ${If} $0 != 0
  MessageBox MB_ICONSTOP "ReplyNoti could not register or unregister. Close ReplyNoti, check the extracted folder, and remove any other registered copy first. / 등록 또는 등록 해제 실패. 기존 버전을 종료하거나 먼저 제거하세요." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 !ifdef UNREGISTER
  MessageBox MB_ICONINFORMATION "Registration removed. Your files are preserved. / 등록이 해제되었습니다. 설정과 파일은 유지됩니다." /SD IDOK
 !endif
SectionEnd
