Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "Sections.nsh"
!ifndef PAYLOAD
 !error "Specify PAYLOAD"
!endif
!ifndef OUTPUT
 !error "Specify OUTPUT"
!endif
Name "ReplyNoti"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\ReplyNoti"
RequestExecutionLevel user
SetCompressor /SOLID zlib
ShowInstDetails show
ShowUninstDetails show
VIProductVersion "0.1.0.8"
VIAddVersionKey /LANG=1033 "ProductName" "ReplyNoti"
VIAddVersionKey /LANG=1033 "FileDescription" "ReplyNoti Windows Setup"
VIAddVersionKey /LANG=1033 "FileVersion" "0.1.0-beta.8"
VIAddVersionKey /LANG=1033 "LegalCopyright" "ReplyNoti contributors"
!define MUI_ICON "${PAYLOAD}\replynoti.ico"
!define MUI_UNICON "${PAYLOAD}\replynoti.ico"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "$(WelcomeTitle)"
!define MUI_WELCOMEPAGE_TEXT "$(WelcomeText)"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$LOCALAPPDATA\ReplyNoti\replynoti.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "--run"
!define MUI_FINISHPAGE_RUN_TEXT "$(RunText)"
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"
LangString WelcomeTitle ${LANG_ENGLISH} "Install ReplyNoti"
LangString WelcomeTitle ${LANG_KOREAN} "ReplyNoti 설치"
LangString WelcomeText ${LANG_ENGLISH} "Get notified when ChatGPT finishes generating.$\r$\n$\r$\nThis experimental, unofficial app is installed for your Windows account. Reply previews are enabled by default and may appear on the lock screen or in notification history. Change them in Settings.$\r$\n$\r$\nExisting ReplyNoti settings are preserved."
LangString WelcomeText ${LANG_KOREAN} "ChatGPT 답변이 준비되면 알려드립니다.$\r$\n$\r$\n비공식 베타 앱으로 현재 Windows 계정에 설치합니다. 기본으로 켜진 답변 미리보기는 잠금 화면과 알림 기록에 표시될 수 있으며 설정에서 끌 수 있습니다.$\r$\n$\r$\n업데이트 시 기존 ReplyNoti 설정을 유지합니다."
LangString RunText ${LANG_ENGLISH} "Start ReplyNoti"
LangString RunText ${LANG_KOREAN} "ReplyNoti 실행"
LangString MainName ${LANG_ENGLISH} "ReplyNoti (required)"
LangString MainName ${LANG_KOREAN} "ReplyNoti (필수)"
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
FunctionEnd
Section "$(MainName)" Main
 SectionIn RO
SectionEnd
Section /o "$(StartupName)" Startup
SectionEnd
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
 SectionGetFlags ${Startup} $0
 IntOp $0 $0 & ${SF_SELECTED}
 ${If} $0 != 0
  StrCpy $1 "-AutoStart"
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
