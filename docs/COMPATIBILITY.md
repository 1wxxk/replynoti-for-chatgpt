# Compatibility and beta test plan

## Target

- Windows 11 x64, standard interactive console session.
- Packaged unified ChatGPT/Codex app (OpenAI.Codex), Chat mode.
- Known Korean and English accessibility labels.

## Release gate

Before claiming broad compatibility, test a clean Windows user or VM and at least one independent physical PC.

| Case | Expected |
|---|---|
| First install, no Python/Rust | Install, test toast |
| Long reply while another app is active | One notification after generation ends |
| Short reply | Best-effort notification |
| Preview disabled | Generic status, no reply extraction |
| Preview explicitly enabled | Bounded local excerpt or generic fallback |
| Sensitive preview or changed chat | Generic status |
| Manual stop | Generic generation-ended status; no claim of verified success |
| Native completion notification | Best-effort skip |
| Switch chat mid-reply | Avoid reporting the previous chat as completed |
| ChatGPT closed / Work mode | No false notifications |
| English and Korean UI | Correct detection with known labels |
| Minimized window and sleep/resume | Document actual behavior |
| Auto-start after sign-in, uninstall | One watcher, then clean removal |

Only the current Windows 11 Korean x64 host is available for direct testing during preparation. Other environments remain unverified. A clean data directory on the same host is not equivalent to a clean OS or separate computer.

## Report format

Windows build, app version, UI language, window state, action, expected behavior and actual behavior. Include only reviewed diagnostic logs; no conversation content.

## Click-to-open beta.3

| Case | Expected |
|---|---|
| Notification click with a valid ticket | Bring the original ChatGPT window forward; open the uniquely matched conversation if safe |
| Duplicate title or changed runtime ID | No conversation switch |
| Active generation or draft | No conversation switch |
| Invalid, expired or reused ticket | Reject |
| Watcher restart | Previous activation tickets invalid |
| UIA element invalidation | Reacquire elements, verify selection |

The private predecessor passed a manual A → B → A test on the current host. The public package uses an independent protocol and settings; end-to-end clicking and other-PC navigation still require beta testing.

## Settings beta.4

- Native Windows tray and settings window; no separate GUI runtime.
- Default previews on, installer -NoPreview option, persistent local configuration.
- Custom PNG/ICO import, icon reset, custom WAV or silent mode, start-at-sign-in toggle.
- Verify tray and settings behavior on different DPI, Windows languages, Explorer restarts, and clean PCs before declaring general compatibility.

## Installed asset import beta.5

Known resources in the publisher-identified unified Windows package are supported. Missing package or changed resource filenames return an error and retain manual file selection. No guarantee of compatibility with future app updates or of matching the currently selected native sound.
