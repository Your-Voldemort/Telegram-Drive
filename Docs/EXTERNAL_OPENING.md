# External opening inventory and acceptance

Caller inventory reviewed before restrictions (2026-10-02):

| Boundary | Callers and produced destinations |
|---|---|
| Shell `open` | `AuthWizard` (Telegram API setup, existing Trust Wallet LTC/BTC links, auth gateway); `HelpCenterDialog` (GitHub issues); `SettingsTabs` (author site/repository); `SupporterSettingsSection` (PayPal approval, terms, help); `FfmpegInstallNotice` (FFmpeg download); `services/sponsorLinks.ts` (existing campaign) |
| Opener `openUrl` | `MobileSupporterCard` (PayPal, terms, help); `MobileDashboard` (privacy/author/repository); `useUpdateCheck` (release page) |
| Opener `revealItemInDir` | `useFileDownload` (newly saved files and durable queued-job paths) |
| Native `cmd_open_file_externally` | `DesktopDashboard` (offline/downloaded asset); `PdfViewer` (app preview or provided offline file); `MobileMediaPlayer` (existing Android JNI path) |

Desktop shell URLs permit only HTTP, HTTPS and mailto. Opener URL scopes use the same schemes, and reveal-in-directory remains permitted; generic opener open-path is not granted. Sponsor destination validation and its `window.open(..., 'noopener,noreferrer')` fallback are untouched. The Android native file-opening branch is untouched.

Desktop native opening requires a successfully produced file, canonical identity and unchanged content. Downloads are registered only after publication; cached previews/offline assets are registered after their existing account/protection checks. A symlink substituted in a registered path or an unregistered arbitrary file is rejected. The registration is per account and persisted in the existing workspace record store, requiring no schema migration. New production and each opening hash the produced file (one bounded-buffer sequential read); that disk cost prevents same-size substitution from inheriting authorization. Hashing and database work use at most two blocking workers and 64 KiB buffers. Verified cache reuse preserves its digest when touching the LRU timestamp; a changed registered cache entry is refused without deleting it. An unregistered 3.9.x cache file can be registered lazily when an exact account record proves its private preview, thumbnail or ready offline-pack destination. Both historical workspace filenames and the desktop command filenames are supported. Registration and a per-file migration marker are committed together in the existing account store. Arbitrary siblings, symlink files or directories, removed records, protected images, other accounts and user-chosen download directories are excluded. Missing previously migrated registrations cannot be silently recreated. First opening reads the file twice (registration and validation); later openings retain one full hash. This makes opening large files more expensive and intentionally rejects externally edited/replaced copies; download or produce the file again to authorize the new copy. A normal app restart preserves registration.

Local native journeys use an opening sink instead of launching another OS application. Browser journeys prove sponsor, terms, fixture PayPal approval and donation links reach the permitted opener; they do not exercise OS browser dispatch or real payment. Installed Tauri/OS checks remain UNRUN: open a plain PDF preview, an offline file and a completed download; reveal the download in Finder/Explorer; verify sponsor/terms/approval links open. Arbitrary files and substituted symlinks must be refused. Record results in [release acceptance](RELEASE_ACCEPTANCE.md).

Final file-capability regressions replace a file after successful hashing and before lease return, through separate native processes and real filesystem replacement. Opening and cache timestamp renewal retain the exact authenticated identity; neither can authorize a replacement by capturing a later identity. The controlled sink does not launch an OS application.

The opt-in measurement on this Mac wrote and synced 2,000,000,000 synthetic bytes, then opened a recorded ready offline copy through the native driver. With no other verification gate running, the debug driver took 213.8 seconds for first registration/opening (4 GB hashed) and 108.8 seconds for the subsequent open (2 GB hashed). This is warm local-storage and debug-code evidence, not release-build, cold-disk or installed-OS dispatch performance. No rehash was skipped. The measurement fixture was removed. Migration performs no Telegram requests: workspace caches scan account metadata one row at a time; an offline copy reads its pack record. Hash buffers remain 64 KiB with at most two workers; registration adds two small per-file records in the existing account database.


## Release-build opening measurements

On macOS 15.7.9 (24G830), the x86_64-apple-darwin driver built with Rust 1.92.0 and `cargo build --release --locked --features native-e2e --bin native-e2e-driver -j 4` completed under the optimized release profile confirmed by the build output. Binary SHA-256: `d86cc4288a30b280a0a2a36d44c186ede2ccbe02cf8b197c77759d6b864ff220`. These are separate single-run warm measurements, not installed application/OS dispatch benchmarks. No other verification gate ran during timed opening. The native-e2e build includes hash-byte instrumentation and a controlled opening sink.

| Allocated synthetic bytes | First legacy open seconds | Reuse seconds | First/reuse hashed bytes |
| ---: | ---: | ---: | ---: |
| 2,000,000,000 | 16.223549 | 7.858260 | 4,000,000,000 / 2,000,000,000 |
| 200,000,000 | 1.579457 | 0.789655 | 400,000,000 / 200,000,000 |

The synthetic file writes, fixture record/hard-link setup and driver startup precede timing. Fixture seeding does not hash contents. First opening includes migration registration and validation; reuse retains a complete hash. Hash-byte assertions pass for both files. The private fixtures were removed after a clean driver exit. No Telegram request or real OS application launch occurs.

Cold-cache values are UNMEASURED. No purge was performed: this host offers a system-wide purge utility, but no verified, file-scoped purge was used. Synced writes or restarting the driver are not treated as cold. Release/cold behavior on other hardware, Windows/Linux and installed OS dispatch remain unmeasured.

The 2 GB release cost remains well above about two seconds. The following choices require an owner decision before any implementation or new locale text. The present full-hash security checks are unchanged.
## Options requiring a separate owner decision

Neither option below is implemented. These are design trade-offs, not measured speedups. Full hashing and the existing refusal checks remain the baseline.

- **Progress and cancellation while hashing.** Keep both first-open reads and the full subsequent hash, report completed bytes across the verification stages, and stop before OS dispatch when canceled. This preserves the same-size substitution check if cancellation fails closed and account/cache guards remain held until the worker exits. It improves responsiveness rather than reducing disk work or elapsed verification time. It needs cooperative cancellation in the bounded worker plus reviewed translated progress text; the current locale ceilings have no headroom.
- **Verified-digest reuse.** Reuse an account-scoped previously verified digest only when device/volume, stable file ID, size, modification time and change time still match, with regular-file/nonsymlink checks and handle identity checks around dispatch. This could eliminate later full reads on unchanged files. It changes the guarantee: a metadata tuple is not proof of current contents. A same-size mutation with a restored or insufficiently precise tuple, file-ID reuse, rollback or a filesystem without trustworthy metadata could evade reuse invalidation. Such filesystems would need a full-hash fallback; account switches, deletion/protection changes, cache clearing and storage changes must invalidate entries. A persistent cache adds private records and recovery/invalidation work; an in-memory cache loses acceleration at restart.

Platform semantics need review rather than assuming creation time is change time. POSIX identifies `st_ctim` as file-status change time ([Open Group](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_stat.h.html)); Windows distinguishes `ChangeTime` from both `CreationTime` and `LastWriteTime` ([FILE_BASIC_INFO](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_basic_info)). The cache's security consequences above are an inference from relying on metadata instead of reading current contents, not a claim of platform/device acceptance.
