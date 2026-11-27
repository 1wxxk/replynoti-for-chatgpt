Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "Sections.nsh"
!include "nsDialogs.nsh"
!include "WinMessages.nsh"
!ifndef PAYLOAD
 !error "Specify PAYLOAD"
!endif
!ifndef OUTPUT
 !error "Specify OUTPUT"
!endif
Name "ReplyNoti"
Caption "ReplyNoti"
BrandingText "ReplyNoti  ·  v0.1.0-beta.10"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\ReplyNoti"
RequestExecutionLevel user
SetCompressor /SOLID zlib
ShowInstDetails nevershow
SilentInstall normal
SetFont "Segoe UI" 10
ShowUninstDetails show
VIProductVersion "0.1.0.10"
VIAddVersionKey /LANG=1033 "ProductName" "ReplyNoti"
VIAddVersionKey /LANG=1033 "FileDescription" "ReplyNoti Windows Setup"
VIAddVersionKey /LANG=1033 "FileVersion" "0.1.0-beta.10"
VIAddVersionKey /LANG=1033 "LegalCopyright" "ReplyNoti contributors"
!define MUI_ICON "${PAYLOAD}\replynoti.ico"
!define MUI_UNICON "${PAYLOAD}\replynoti.ico"
!define MUI_ABORTWARNING
!include "FileFunc.nsh"
Var Interactive
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"
LangString Failed ${LANG_ENGLISH} "Setup failed. Review the details."
LangString Failed ${LANG_KOREAN} "설치하지 못했습니다. 상세 정보를 확인하세요."
LangString RemoveFailed ${LANG_ENGLISH} "Removal failed. Review the details and try again."
LangString RemoveFailed ${LANG_KOREAN} "삭제하지 못했습니다. 상세 정보를 확인한 후 다시 시도하세요."
Function .onInit
 SetShellVarContext current
 ${IfNot} ${RunningX64}
  MessageBox MB_ICONSTOP "Windows 11 x64 is required." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 SetRegView 64
 StrCpy $INSTDIR "$LOCALAPPDATA\ReplyNoti"
 StrCpy $Interactive "1"
 IfSilent 0 +2
 StrCpy $Interactive "0"
 SetSilent silent
FunctionEnd
Section "-Install"
 InitPluginsDir
 SetOutPath "$PLUGINSDIR\payload"
 File "${PAYLOAD}\replynoti.exe"
 File "${PAYLOAD}\replynoti-settings.exe"
 File "${PAYLOAD}\replynoti.ico"
 File "${PAYLOAD}\replynoti.png"
 File "${PAYLOAD}\Install.ps1"
 File "${PAYLOAD}\Uninstall.ps1"
 File "${PAYLOAD}\Uninstall.cmd"
 File "${PAYLOAD}\LICENSE"
 File "${PAYLOAD}\THIRD_PARTY_NOTICES.txt"
 File "${PAYLOAD}\NSIS-LICENSE.txt"
 WriteUninstaller "$PLUGINSDIR\payload\Uninstall.exe"
 File "${PAYLOAD}\SetupUI.ps1"
 File "${PAYLOAD}\setup-icon.png"
 ${If} $Interactive == "1"
  nsExec::ExecToLog '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -STA -ExecutionPolicy RemoteSigned -File "$PLUGINSDIR\payload\SetupUI.ps1"'
 ${Else}
  nsExec::ExecToLog '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File "$PLUGINSDIR\payload\Install.ps1" -Upgrade -NoStart'
 ${EndIf}
 Pop $0
 ${If} $0 == 2
  SetErrorLevel 2
  Quit
 ${EndIf}
 ${If} $0 != 0
  MessageBox MB_ICONSTOP "$(Failed)" /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 SetOutPath "$INSTDIR"
SectionEnd
Function un.onInit
 SetShellVarContext current
 SetRegView 64
 ${If} $INSTDIR != "$LOCALAPPDATA\ReplyNoti"
  MessageBox MB_ICONSTOP "Unexpected installation path." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
FunctionEnd
Section "Uninstall"
 nsExec::ExecToLog '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File "$INSTDIR\Uninstall.ps1"'
 Pop $0
 ${If} $0 != 0
  MessageBox MB_ICONSTOP "$(RemoveFailed)" /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 SetOutPath "$TEMP"
 Delete "$INSTDIR\Uninstall.exe"
 RMDir "$INSTDIR"
SectionEnd
