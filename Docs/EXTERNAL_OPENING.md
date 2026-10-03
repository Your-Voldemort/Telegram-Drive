# External opening inventory and acceptance

Caller inventory reviewed before restrictions (2026-10-02):

| Boundary | Callers and produced destinations |
|---|---|
| Shell `open` | `AuthWizard` (Telegram API setup, existing Trust Wallet LTC/BTC links, auth gateway); `HelpCenterDialog` (GitHub issues); `SettingsTabs` (author site/repository); `SupporterSettingsSection` (PayPal approval, terms, help); `FfmpegInstallNotice` (FFmpeg download); `services/sponsorLinks.ts` (existing campaign) |
| Opener `openUrl` | `MobileSupporterCard` (PayPal, terms, help); `MobileDashboard` (privacy/author/repository); `useUpdateCheck` (release page) |
| Opener `revealItemInDir` | `useFileDownload` (newly saved files and durable queued-job paths) |
| Native `cmd_open_file_externally` | `DesktopDashboard` (offline/downloaded asset); `PdfViewer` (app preview or provided offline file); `MobileMediaPlayer` (existing Android JNI path) |

Desktop shell URLs permit only HTTP, HTTPS and mailto. Opener URL scopes use the same schemes, and reveal-in-directory remains permitted; generic opener open-path is not granted. Sponsor destination validation and its `window.open(..., 'noopener,noreferrer')` fallback are untouched. The Android native file-opening branch is untouched.

Desktop native opening requires a successfully produced file, canonical identity and unchanged content. Downloads are registered only after publication; cached previews/offline assets are registered after their existing account/protection checks. A symlink substituted in a registered path or an unregistered arbitrary file is rejected. The registration is per account and persisted in the existing workspace record store, requiring no schema migration. New production and each opening hash the produced file (one bounded-buffer sequential read); that disk cost prevents same-size substitution from inheriting authorization. Hashing and database work use at most two blocking workers and 64 KiB buffers. Verified cache reuse preserves its digest when touching the LRU timestamp; a changed registered cache entry is refused without deleting it. Legacy unregistered caches keep their existing internal preview/offline availability but gain no external-opening permission. Explicitly recreate/download the file to open it externally. This makes opening large files more expensive and intentionally rejects externally edited/replaced copies; download or produce the file again to authorize the new copy. A normal app restart preserves registration.

Local native journeys use an opening sink instead of launching another OS application. Browser journeys prove sponsor, terms, fixture PayPal approval and donation links reach the permitted opener; they do not exercise OS browser dispatch or real payment. Installed Tauri/OS checks remain UNRUN: open a plain PDF preview, an offline file and a completed download; reveal the download in Finder/Explorer; verify sponsor/terms/approval links open. Arbitrary files and substituted symlinks must be refused. Record results in [release acceptance](RELEASE_ACCEPTANCE.md).

Final file-capability regressions replace a file after successful hashing and before lease return, through separate native processes and real filesystem replacement. Opening and cache timestamp renewal retain the exact authenticated identity; neither can authorize a replacement by capturing a later identity. The controlled sink does not launch an OS application.
