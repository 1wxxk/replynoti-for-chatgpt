# Privacy and scope

ReplyNoti is a local, experimental accessibility-based status observer. It reads window identity, accessibility control types, known UI labels, class names, and runtime IDs to infer generation state. Reply previews are enabled by default and can be disabled during installation or in Settings. While previews are enabled, it reads a bounded prefix of a verified completed answer through local accessibility controls. The excerpt is filtered and passed to Windows notifications, never written to application logs or sent over a network by ReplyNoti. It does not record keystrokes, read the clipboard, access account tokens, or send telemetry. It does not change the official app or bypass its controls.

It checks native Windows notification timestamps as a best-effort duplicate suppression mechanism. This is not an official or conversation-specific completion signal.

Local logs contain events, error codes, timestamps and window IDs, not prompts or replies. Windows may retain notification text, including enabled previews, or show it on a lock screen. Sensitive-text filtering is best effort and cannot detect all private data. Disable previews when handling sensitive conversations. Turning previews off does not delete previously displayed notifications. Review diagnostics before sharing them. No official ChatGPT icon or sound is redistributed. The original project artwork is included under the project license.

The implementation is unofficial. Users should review the applicable service terms; status observation and optional automated excerpt extraction are not official integrations or assurances of policy compliance. Review applicable terms before using previews. Avoid using preview features where automated extraction is prohibited.

## Smart notification timing

The watcher checks the foreground window handle and the Windows last-input timestamp locally. It does not record individual keyboard or mouse events, input contents, or input history. Deferred alerts retain only the tracked turn identity and timestamps in memory, expire after five minutes, and are not persisted.

## Notification activation

To reopen a conversation, the watcher reads the selected sidebar title and its accessibility runtime ID, retaining them in memory for up to five minutes. Temporary one-time ticket files contain only the window handle and process ID; they contain no conversation title, reply, or durable conversation ID. Clicking a notification invokes the per-user replynoti-open handler. Invalid or expired tokens are rejected. Draft checks inspect composer values locally only when attempting navigation, without storing or transmitting them. Navigation is skipped if the current conversation cannot be safely changed. Windows foreground activation is best effort.

Settings and imported PNG/ICO/WAV assets are stored in the local installation directory; no asset upload is performed. The app does not read reply text when previews are disabled.

Installed-app import is explicitly user initiated. It queries installed package metadata and reads only known icon/WAV resources, not application profiles, conversations, account information, or settings. Copies remain under the local installation directory and are removed by uninstall; never include imported assets in source or release archives.
