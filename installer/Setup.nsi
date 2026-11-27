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
BrandingText "ReplyNoti  ·  v0.1.0-beta.9"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\ReplyNoti"
RequestExecutionLevel user
SetCompressor /SOLID zlib
ShowInstDetails nevershow
SetFont "Segoe UI" 10
ShowUninstDetails show
VIProductVersion "0.1.0.9"
VIAddVersionKey /LANG=1033 "ProductName" "ReplyNoti"
VIAddVersionKey /LANG=1033 "FileDescription" "ReplyNoti Windows Setup"
VIAddVersionKey /LANG=1033 "FileVersion" "0.1.0-beta.9"
VIAddVersionKey /LANG=1033 "LegalCopyright" "ReplyNoti contributors"
!define MUI_ICON "${PAYLOAD}\replynoti.ico"
!define MUI_UNICON "${PAYLOAD}\replynoti.ico"
!define MUI_ABORTWARNING
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${PAYLOAD}\setup-header.bmp"
!define MUI_HEADERIMAGE_RIGHT
!define MUI_BGCOLOR "FFFFFF"
Var OptionsDialog
Var StartupCheck
Var StartupChoice
Var StartupChanged
Var PreviewCheck
Var PreviewChoice
Var IsUpgrade
Var HeadingFont
Var OptionsLabel
Page custom OptionsCreate OptionsLeave "$(InstallButton)"
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$LOCALAPPDATA\ReplyNoti\replynoti.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "--run"
!define MUI_FINISHPAGE_RUN_TEXT "$(RunText)"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"
LangString InstallButton ${LANG_ENGLISH} "Install"
LangString InstallButton ${LANG_KOREAN} "설치"
LangString OptionsTitle ${LANG_ENGLISH} "Ready to install"
LangString OptionsTitle ${LANG_KOREAN} "설치 준비"
LangString OptionsSubtitle ${LANG_ENGLISH} "Choose your preferences. No administrator access needed."
LangString OptionsSubtitle ${LANG_KOREAN} "설치 옵션을 선택하세요. 관리자 권한이 필요하지 않습니다."
LangString OptionsTag ${LANG_ENGLISH} "Know when your answer is ready."
LangString OptionsTag ${LANG_KOREAN} "답변이 준비되면, 알려드릴게요."
LangString OptionsPath ${LANG_ENGLISH} "INSTALL LOCATION (CURRENT USER)"
LangString OptionsPath ${LANG_KOREAN} "설치 위치 (현재 사용자)"
LangString PreviewOption ${LANG_ENGLISH} "Show reply previews in notifications"
LangString PreviewOption ${LANG_KOREAN} "알림에 답변 미리보기 표시"
LangString PreviewWarning ${LANG_ENGLISH} "Previews may appear on your lock screen and in notification history."
LangString PreviewWarning ${LANG_KOREAN} "미리보기는 잠금 화면과 알림 기록에 표시될 수 있습니다."
LangString KeepPrefs ${LANG_ENGLISH} "Updating? Existing preferences and custom assets will be preserved."
LangString KeepPrefs ${LANG_KOREAN} "업데이트 시 기존 설정과 사용자 지정 리소스를 유지합니다."
LangString BetaNote ${LANG_ENGLISH} "v0.1.0-beta.9  ·  Windows 11 x64  ·  Unofficial"
LangString BetaNote ${LANG_KOREAN} "v0.1.0-beta.9  ·  Windows 11 x64  ·  비공식"
LangString RunText ${LANG_ENGLISH} "Start ReplyNoti"
LangString RunText ${LANG_KOREAN} "ReplyNoti 실행"
LangString StartupName ${LANG_ENGLISH} "Start with Windows"
LangString StartupName ${LANG_KOREAN} "Windows 시작 시 실행"
LangString Failed ${LANG_ENGLISH} "Setup failed. Review the details. A portable copy must be unregistered before installing."
LangString Failed ${LANG_KOREAN} "설치하지 못했습니다. 상세 정보를 확인하세요. 포터블 버전이 등록되어 있다면 먼저 등록을 해제하세요."
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
 StrCpy $StartupChoice ${BST_UNCHECKED}
 StrCpy $StartupChanged "0"
 StrCpy $PreviewChoice ${BST_CHECKED}
 StrCpy $IsUpgrade "0"
 IfFileExists "$INSTDIR\.replynoti-install" 0 +2
 StrCpy $IsUpgrade "1"
 ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "ReplyNoti"
 ${If} $0 == '$\"$INSTDIR\replynoti.exe$\" --run'
  StrCpy $StartupChoice ${BST_CHECKED}
 ${EndIf}
FunctionEnd
Function OptionsCreate
 !insertmacro MUI_HEADER_TEXT "$(OptionsTitle)" "$(OptionsSubtitle)"
 nsDialogs::Create 1018
 Pop $OptionsDialog
 ${If} $OptionsDialog == error
  Abort
 ${EndIf}
 SetCtlColors $OptionsDialog 222222 F0F0F0
 ${NSD_CreateLabel} 0 0 100% 20u "ReplyNoti"
 Pop $OptionsLabel
 CreateFont $HeadingFont "Segoe UI" "17" "650"
 SendMessage $OptionsLabel ${WM_SETFONT} $HeadingFont 1
 SetCtlColors $OptionsLabel 2456D6 transparent
 ${NSD_CreateLabel} 0 22u 100% 13u "$(OptionsTag)"
 Pop $0
 SetCtlColors $0 444444 transparent
 ${NSD_CreateLabel} 0 40u 100% 10u "$(BetaNote)"
 Pop $0
 SetCtlColors $0 666666 transparent
 ${NSD_CreateLabel} 0 58u 100% 10u "$(OptionsPath)"
 Pop $0
 SetCtlColors $0 555555 transparent
 ${NSD_CreateText} 0 70u 100% 14u "%LOCALAPPDATA%\ReplyNoti"
 Pop $0
 SendMessage $0 ${EM_SETREADONLY} 1 0
 ${NSD_CreateCheckbox} 0 91u 100% 13u "$(StartupName)"
 Pop $StartupCheck
 ${NSD_SetState} $StartupCheck $StartupChoice
 ${NSD_CreateCheckbox} 0 106u 100% 13u "$(PreviewOption)"
 Pop $PreviewCheck
 ${NSD_SetState} $PreviewCheck $PreviewChoice
 ${If} $IsUpgrade == "1"
  ShowWindow $PreviewCheck ${SW_HIDE}
  ${NSD_CreateLabel} 0 107u 100% 26u "$(KeepPrefs)"
  Pop $0
  SetCtlColors $0 555555 transparent
 ${Else}
  ${NSD_CreateLabel} 0 121u 100% 20u "$(PreviewWarning)"
  Pop $0
  SetCtlColors $0 666666 transparent
 ${EndIf}
 nsDialogs::Show
FunctionEnd
Function OptionsLeave
 ${NSD_GetState} $StartupCheck $StartupChoice
 StrCpy $StartupChanged "1"
 ${NSD_GetState} $PreviewCheck $PreviewChoice
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
 StrCpy $1 ""
 ${If} $StartupChanged == "1"
  ${If} $StartupChoice == ${BST_CHECKED}
   StrCpy $1 "-AutoStart"
  ${Else}
   StrCpy $1 "-NoAutoStart"
  ${EndIf}
 ${EndIf}
 ${If} $IsUpgrade == "0"
 ${AndIf} $PreviewChoice == ${BST_UNCHECKED}
  StrCpy $1 "$1 -NoPreview"
 ${EndIf}
 nsExec::ExecToLog '"$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy RemoteSigned -File "$PLUGINSDIR\payload\Install.ps1" -Upgrade -NoStart $1'
 Pop $0
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
