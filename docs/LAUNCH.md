# Beta release and launch checklist

The project is an experimental, unofficial desktop status notifier, not a ChatGPT client.

## Release gates

- [x] Native x64 Release build and state-machine tests on the development host.
- [x] Per-user clean-directory installation, native test toast, startup registration, single instance and removal on the development host.
- [ ] Preview opt-out, filtered fallback, and Windows notification privacy on another PC.
- [ ] Independent Windows 11 x64 PC: no installed Rust or Python.
- [ ] English UI and Korean UI: normal completion while another application is active.
- [ ] End-to-end long/short response and cancellation tests; document misses.
- [ ] Restart/sign-in, minimize/restore, sleep/resume tests.
- [x] Owner confirms ReplyNoti name and requests source publication. Preserve original MIT copyright notice.
- [ ] Push source-only repository, run Windows CI, check result.
- [ ] Create prerelease tag and publish checked ZIP + SHA256SUMS.txt.
- [ ] Check the README and any screenshot or recording for personal information.

## Positioning

A tiny native Windows helper that notifies you when the official ChatGPT desktop app stops generating. No replacement client, browser extension, API keys, or telemetry; configurable local previews (enabled by default). Best-effort detection; no affiliation with OpenAI.

## Draft GitHub description

Unofficial Rust notification helper for ChatGPT desktop on Windows. Local-only, optional reply previews.

## Draft beta announcement (English)

I built ReplyNoti, a small Rust-based Windows helper for the official ChatGPT desktop app. It detects changes in local accessibility controls and posts a native Windows notification when generation appears to end. It needs no API keys and sends no telemetry. A configurable local reply excerpt is on by default.

I'm looking for a few Windows 11 x64 beta testers, especially people who miss completion notifications while working in other apps. The current beta has been tested on one Korean-language Windows host; English UI and other PCs are not yet verified. Please share your OS/app version and whether notifications arrive for normal and short replies. Don't share conversation contents.

## Draft beta announcement (Korean)

ChatGPT Windows 앱에서 답변 완료 알림이 누락되는 불편함을 해결하려고 Rust로 ReplyNoti을 만들었습니다. 공식 앱을 수정하거나 교체하지 않고, Windows 접근성 컨트롤의 상태를 감지해 기본 알림을 띄우는 작은 보조 프로그램입니다. API 키와 텔레메트리는 없으며, 답변 미리보기는 기본 활성화되며 설정에서 끌 수 있습니다.

현재 Windows 11 x64 베타를 준비 중입니다. 한국어 Windows 환경 한 대에서 테스트했으며, 영어 UI와 다른 PC의 호환성을 확인할 베타 테스터를 모집하려고 합니다.

## Distribution

The initial GitHub repository is source-only. Before distributing binaries, complete the license inventory for the Tauri Rust dependency tree and verify all third-party notices are included. CI intentionally builds without uploading packaged binaries until these release gates are met.

Keep the initial release marked prerelease. Use GitHub Releases with SHA-256 checksums and report the binary is unsigned. Do not suggest disabling SmartScreen or organization security policies. Announce in relevant communities only where self-promotion is permitted; avoid unsolicited or repetitive posts.

- [ ] Verify click-to-open on a separate Windows PC and confirm safe fallback for draft, busy, duplicate-title and changed-ID cases.
