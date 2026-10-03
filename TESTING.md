# End-to-end testing

Behavioral regression testing uses E2E suites. New tests must exercise an application boundary and a meaningful user or service journey. Isolated function/component assertions, inline Rust unit modules, and Android JVM unit suites are retired. Historical review reports describe the checks available at the time; this document defines the current workflow.

## Host verification

Install JavaScript dependencies in `app/` and `supporter-service/`, the normal native build prerequisites, and Playwright Chromium:

```sh
npm ci --prefix app
npm ci --prefix supporter-service
cd app
npx playwright install chromium
cd ..
node scripts/run-e2e.cjs
```

On Linux CI, install the browser system dependencies with `npx playwright install --with-deps chromium`. The top-level runner stops on the first failing suite. It never deploys the Worker, uses a real PayPal purchase, or signs in to a personal Telegram account.

Individual suites:

| Boundary | Command from the repository root | What it exercises |
| --- | --- | --- |
| Browser | `npm run test:e2e --prefix app` | User journeys in Chromium, including existing visual/accessibility checks and full application flows; the Tauri/Telegram boundary is controlled by fixtures. |
| Native | `cargo test --locked --manifest-path app/src-tauri/Cargo.toml --features native-e2e --test native_e2e` | Native child processes, real local storage, vault/envelope persistence and recovery drills, per-account folder layouts, sync planning and recorded results, resumable uploads, transfer retry policy, the redacted log file, private staging, download publication, and the loopback media, REST and WebDAV servers over real HTTP, using synthetic accounts and private temporary directories. |
| Worker and application | `node --test scripts/e2e/supporter-token.e2e.mjs` | A supporter entitlement issued by the Worker in local workerd, verified by the application's own verifier in the native driver process. Uses a key pair generated for the run; needs both the Rust toolchain and `supporter-service` dependencies. |
| Supporter service | `npm run test:e2e --prefix supporter-service` | Real HTTP requests to the production Worker in a local workerd runtime with migrated D1; only the external PayPal boundary is simulated. |
| Release tools | `node scripts/e2e/assurance.e2e.cjs` | Shipped SBOM/checksum CLI processes, generated artifacts, independent checksum verification, invalid-input rejection, the dependency-baseline expiry check, the Android publication guard against real Git repositories, and mandatory route/chunk budget enforcement against generated bundle artifacts. |

`npm test` is an alias for `test:e2e` in both JavaScript projects. `visual:test` and `visual:update` remain browser tooling aliases. The native E2E driver is feature-gated and is excluded from normal application builds.

## Static checks and builds

E2E-only describes behavioral tests. Keep the independent checks that catch invalid configuration, type errors, dependency advisories, licensing problems, formatting issues, translation regressions, and oversized bundles:

```sh
node scripts/check-rust-formatting.cjs
node scripts/check-test-policy.cjs
node scripts/check-app-security.cjs
node scripts/check-android-publication.cjs
node scripts/check-baseline-expiry.cjs
npm run build:verify --prefix app
npm run i18n:check --prefix app
npm run check --prefix supporter-service
cargo fmt --manifest-path app/src-tauri/Cargo.toml --all -- --check
cargo clippy --locked --manifest-path app/src-tauri/Cargo.toml --features native-e2e --lib --all-targets -- -D warnings
```

The dependency-assurance workflow remains mandatory for desktop publication. Release verification requires browser, native, and service E2E suites before creating a release. CI also builds the normal application on Windows, Linux, and macOS. Unit-test line-coverage floors have been retired; a green E2E run must not be presented as 100% code or feature coverage.

## Device and external-service acceptance

`check-app-security.cjs` also fails when the REST routes and the published OpenAPI contract disagree, or when an updater endpoint is not HTTPS. `check-android-publication.cjs` inspects tracked files and every outgoing commit; run it before any push.

### Workspace paging and archive boundaries

The native suite includes connection reuse across operations, transaction commit/rollback and isolation across account/root paths; rejection of a newer database version through a cached connection; shares WAL readers alongside another process's writer; consistent 256-file pages during indexing and account changes; and a 10,000-file library with 1,000 pending removals whose page reply must arrive within five seconds. The browser suite checks access beyond page one after organization and folder indexing.

Archive journeys stage a real file through the production stream, inject a transport failure after a complete prefix, and exercise disk-backed ZIP listing/extraction with a 2.1 MB entry, basename sanitization, private permissions, registered deletion, CRC corruption, truncated headers, compression-ratio refusal, download cancellation and account-switch cleanup. The Telegram source is replaced by a controlled file stream. Real Telegram archive transport, RAR/7z inputs, cursor expiry/eviction, database file replacement and cancellation during blocking extraction are not established by these journeys.

### Shared preview cache and video boundaries

Native child-process journeys stream local source files through the production asset service and exercise source/account/protection identity, cache hits without indexing arbitrary messages, failed completion, lookup cancellation, pinned eviction/clearing/deletion, shared native/async byte reservations and clear generations, explicit roots under similarly named parent directories, offline/status compatibility, and competing thumbnail/preview or canceled SQLite metadata publication. A gated legacy pin crosses shared admission. Real JPEG/PNG decoding checks the three-job ceiling and output dimensions. REST journeys use the real HTTP server with only the remote source replaced by a controlled fixture.

Video journeys prefer a provided JPEG, exercise a controlled extraction child that succeeds/fails/stalls, count a live JPEG partial once at the reservation ceiling, and verify clear/deadline kill-and-wait cleanup. That helper does not establish codec compatibility. A separate local macOS acceptance used the installed FFmpeg with a synthetic MP4 and produced a 480×360 JPEG; other codecs, operating systems and packaged FFmpeg discovery remain unrun. Sources above 16 MiB and unsupported formats gracefully lack frame fallback. Browser journeys cover video cards, delayed old-account responses, same-name/size source replacement and closing an already displayed preview on an account switch; complete visual snapshots were unchanged.

### What the native journeys do not reach

The native driver has no Telegram connection and no Tauri window. These limits apply to the journeys above:

- **Folder scanning.** A process journey pauses a controlled dialog source while reading the production peer-cache publication path, then covers successful replacement, interrupted enumeration and an account switch before publication. Real Telegram dialogs and legacy About lookups still require installed-app acceptance.
- **Streaming.** Disk-backed download fixtures exercise the production HTTP builders over real loopback HTTP: exact suffix/open-ended/CDN-boundary ranges, empty plain files, `416` resource sizes, UTF-8 and hostile download names, source errors, premature EOF, account revocation, and full/seeked protected reads beyond 4 MiB. Protected journeys reject corrupted or missing data/final records, stop on an in-flight vault lock, and retain independently authenticated range access when the final record is unavailable. The production resolution-cache path covers concurrent seeks, retry after a failed lookup, account/session invalidation, expiry, capacity eviction and a lookup completing after an account switch. These sources replace Telegram downloads and peer/message lookup; real Telegram CDN behaviour, file-reference expiry and playback in WebView2, WKWebView, WebKitGTK and Android WebView remain unverified. Empty protected media retains its existing full-read refusal.
- **REST listings.** The journeys seed what walking an account's folders would return, then exercise authentication, account scoping, filtering, ordering, paging, timestamps, reports and the error contract over HTTP. The walk itself, and every route that reads or changes Telegram (download, upload, copy, rename, move, delete, the bulk archive), needs a signed-in account. Thumbnail HTTP/cache journeys replace the remote source and do not establish Telegram thumbnail delivery.
- **Shared inventory.** The native child process reads controlled remote snapshots from real files and persists identifier cursors in real SQLite. Journeys exercise external rename/delete/add, bootstrap races, restart hydration without history rewalks, failed and timed-out audits without partial publication, capacity replacement, concurrent consumers, abandoned waiters, completion between waiters, account cancellation, cross-consumer notifications, timed background polling and idle expiry. Publication journeys cross local mutation and vault-lock boundaries. The browser fixture exercises matching-owner change events, stale failures, heartbeat renewal, cache-eviction recovery and continuing-build loading. These fixtures do not exercise Telegram history ordering, known-ID queries, flood waits or real service latency. Ordinary inventory audits cover existing media IDs and newer messages; converting an older text-only message into media is outside this known-ID design. Frontend consumers still assemble the complete listing in memory.
- **WebDAV.** Token/account refusal, the root listing, read-only write refusal, and permission/missing/transport status mapping are exercised through the actual HTTP server. Remote errors are injected at the disconnected Telegram boundary. The shared upload retry path reads and reopens real staging files with controlled transport failures; real Telegram part upload, message publication, ambiguous lost replies, replacement cleanup and Finder/Explorer integration remain unrun.
- **Folder Sync.** Planning, deletion guards, adoption, interrupted-upload recovery, incremental scanning, and placing a protected file by the path inside its envelope are covered through the engine's own stages. The engine loop, bounded shutdown, fetching envelope headers from Telegram, and real uploads and downloads through the transfer queue need the running application.
- **Transfers.** The stored retry and expiry policy, what a supervising part of the application is told about a transfer, and the removal of supervised transfers at start-up are covered. Running a transfer in the queue, and resuming a real download from its last chunk, are not.
- **Large uploads.** The uploader, its per-account record, and the protected envelope are covered with a stand-in for Telegram's part storage: interruption, resumption in a new process, refusal of changed content, and decryption of an envelope assembled across attempts. Sending parts to Telegram, publishing the message, and Telegram discarding old parts are not.
- **Start-up failure.** The message and log are covered; the native dialog is not.

Include these in installed-app acceptance and report them as unrun when they were not exercised.

Browser fixtures do not prove the Tauri IPC bridge, OS credential stores, native codecs, real Telegram network behavior, or installed-app updates. Native loopback/process tests do not replace GUI or OS integration checks. Their encrypted-file fixtures verify vault/envelope persistence and recovery; they do not verify Telegram encrypted-download staging or publication. Worker tests with a simulated PayPal transport do not prove the live provider integration.

Before a supported release, perform the relevant installed-app and device acceptance: sign in/out, switch accounts, upload/download/stream, restart/update without losing settings or activation, verify encrypted-file recovery, and exercise Android device flows. Keep the PayPal sandbox purchase/recovery/refund acceptance required by [the supporter contract](SUPPORTER_LICENSE_INVARIANTS.md) before payment-path or schema changes reach production. Never use production purchases, personal sessions, production D1, or real keychain entries as automated test fixtures.

Android device/emulator verification stays local; Android source and test projects must not be published to GitHub. With the SDK/NDK configured and the required system images installed, create the universal debug JNI inputs and run a private emulator from `app/`:

```sh
npm run tauri -- android build --debug --target aarch64 armv7 i686 x86_64 --apk true --ci
bash scripts/run-android-emulator-tests.sh phone
```

The runner uses 4096 MB of emulator RAM by default (`ANDROID_EMULATOR_RAM_MB` overrides it). This is a verified test-environment setting, not an application memory requirement. API 35+ phone runs include real PIN cancellation/reopen, camera-journal process restart, and session-recovery journeys after device instrumentation. The runner reinstalls the instrumentation APK for the separate ADB-driven journeys because Gradle removes it after its own tests. Each journey restores its synthetic state; the emulator is shut down on exit. Report device or platform checks that were unavailable as unrun, never as passing.

When fixing a regression, add a failing journey at the affected application boundary, make the smallest implementation change, and run that suite plus relevant static/build checks. Use explicit readiness and observable state changes rather than sleeps that assume a busy runner's scheduling speed.

## Desktop package contents

After the frontend build and native E2E build, run `node scripts/e2e/desktop-package.e2e.cjs` on macOS or Linux. This invokes the real Tauri bundler against the compiled debug application and checks the app/Debian package contents for accidental E2E executables. It does not sign or publish artifacts, launch the GUI, or replace Windows installer verification. The release workflow runs this journey on Linux before creating the draft release.

## Local full-text search

Bundled SQLite is compiled with FTS5 via `.cargo/config.toml`; the application build requires that environment setting and verifies the linked host library using `app/src-tauri/build/search-support.rs`. The Cargo-build process journey proves that a missing setting fails and that changing it without rebuilding the cached SQLite dependency also fails. Rebuilding `sqlite3-src` with the required setting passes. Cross-target packaged search is additionally an unrun release smoke/acceptance check. Search creates only private `:memory:` databases with `temp_store=MEMORY`; workspace schema version1 and persistent tables are unchanged. The native suite crosses the application child-process/SQLite/shared-inventory/asset boundaries: account and Drive-folder isolation, accent/CJK/punctuation matching, bounded metadata, filters before generation-bound pages, saved rules, transaction fencing, cancellation-retained capacity, external inventory and vault invalidation, date cutoffs fixed across pages,20-folder bootstrap beyond the inventory cache, live-only organization, embedded flag compatibility and unindexed media previews. File sources replace Telegram; these journeys do not validate live SDK dialog discovery, protected-envelope delivery, or CDN behavior.

Browser journeys use the actual application with a controlled IPC boundary to check paged results, coverage/offline labels, visible failures, continuing-build retry, lock/unlock invalidation, facet-only search, duplicate message IDs with cross-folder bulk selection disabled, saved-rule editing, live-result selection/slideshows and the selected-folder scan before offline preparation. Existing visual snapshots are compared without regeneration. A partial/offline index is not evidence that all remote files or protected names are searchable. Live Telegram, real WebViews, Android devices, installed-app acceptance and CI remain separate unrun release checks.


## Bandwidth quota and scheduling boundaries

Native child-process/storage/HTTP journeys exercise saved quota changes, owned holds and crash restart, week rollover and late cancellation, shrinking after lowering the limit, backward clocks, corrupt-file repair, and failure to persist committed usage. HTTP ranges charge completed response bytes; failures and abandoned bodies release their own reservation. An abortive TCP reset explicitly confirms server waits are dropped before checking that fresh transfers inherit no canceled pacing delay. A TCP half-close can legitimately keep an HTTP response active; that transport behavior is not reported as an application cancellation.

Content adapters use one shared upload lane and one shared download lane, including ordinary bounded readers and resumable part sinks. Disk-copy and synthetic part-storage journeys verify aggregation, skipped confirmed prefixes, pause/cancellation/staging cleanup and durable quota. Media HTTP journeys cover download limits outside VPN, flag-off behavior, half-open local weekday windows, overnight start-day ownership, overlapping limits, resume without a burst, and account/vault revocation while paused. Browser journeys exercise localized controls, saved windows and allowance after remount, and a visible quota-save failure.

Bounded envelope/header discovery and first-2-MiB video-duration probes remain metadata work and are exempt from payload pacing/quota; scheduled pauses do not disable account/catalog/metadata requests. This is a deliberate coverage boundary, not an all-network traffic limit.

Quota counts successful declared application payload, not exact wire traffic: retries, SDK/CDN prefetch, transport overhead and failed whole operations do not add separate charges. Protected file transfers reserve ciphertext length; HTTP ranges reserve delivered response length. Archive source downloads and transcode originals count once; rereading local staged/cache bytes adds no charge. Existing historical counters are retained; unfinished reservations persisted by older versions cannot be identified retroactively.

A failed accounting write retains the completed charge in memory and blocks admission until persistence succeeds. A process crash before repair can lose that unsaved charge. Proxy secret rollback is tested through fixture-owned storage; it never writes the real credential store, and OS credentials plus JSON do not share a crash-atomic transaction. Real credential prompts and each OS backend remain acceptance work. Local civil-time scheduling follows skipped/repeated DST minutes as observed locally; elapsed pacing uses a monotonic clock. Actual OS clock/DST transitions, live Telegram upload/download timing, real WebViews, Android devices, installed builds and CI remain unrun. The socket-reset journey ran on this host; its Windows branch has not run. Scheduling/upload enablement and windows are device-local and are not added to Telegram Settings Sync; the existing upload/download rate preferences retain their sync compatibility. No persistent database schema changes are involved.

## Sponsor reliability boundaries

`Docs/SPONSOR_ACCEPTANCE.md` records the unrun WebView2/WKWebView/WebKitGTK/Android acceptance matrix and the owner decision before switching the default origin. Desktop `TELEGRAM_DRIVE_AD_ORIGIN=separate` selects an ad-only loopback listener; the established media origin remains the default. Native journeys drive actual listener/CSP/HTTP and verified signed fixture access, including active/grace204, expiry, an invalid verification, and suppression during fresh/stale relay responses. The fixture transport is compiled only with `native-e2e`, replaces provider HTTP/DNS, and reuses the loader request and validator. It proves no real provider delivery or installed OS credential behavior. Browser journeys cover both selected origins, unchanged sandbox/countdown/15-minute interval, translated status and the existing fallback destination through a controlled opener. Real WebViews, live provider/DNS, Android/TV, installed update/recovery acceptance and CI remain unrun.

For hosts running many synthetic native processes and Actix worker pools, `RUST_TEST_THREADS=4 cargo test --locked --manifest-path app/src-tauri/Cargo.toml --features native-e2e --test native_e2e` runs the complete suite with four independent fixtures at a time. It retains explicit concurrency inside each journey. An earlier P2-20 full run at default concurrency had 97 passes and two failures (SQLite busy in the media-range journey and a 5-second FFmpeg fixture readiness deadline); retain that result alongside isolated diagnostics and the bounded complete run. This invocation is not a substitute for real platform/installed-app acceptance.

## Accessibility journeys and device boundaries

The application browser suite audits all twelve Settings tabs plus expanded REST API controls, a populated Transfer Center, and vault creation, passphrase visibility, recovery drills, lock/unlock and nested encryption dialogs. Transfer actions cross the controlled native IPC/event boundary; vault commands use controlled crypto responses. These browser journeys do not certify real encrypted storage, which remains covered separately by native journeys. No axe rules are disabled. Audits wait for finite UI transitions to settle before measuring contrast.

A development-only TV fixture uses the production spatial-navigation hook with real DOM inputs, a native select, a scroll region and Settings. Keyboard journeys check below-fold focus/scrolling, dialog containment/restoration and native caret, number and selection behavior against an identical control with the hook disabled. Physical Google TV remotes and Android devices remain unrun.

The Arabic main dashboard has a new, inspected RTL snapshot, overflow checks, a full-document axe audit and keyboard focus coverage. Existing visual snapshots were not regenerated. The two new catalog keys `settings.show_passphrase` and `settings.hide_passphrase` were translated in all 24 locales; each needs owner-arranged native-language review. This accessibility work does not complete the broader localization debt.

The TV native-field exit journey enters controls using arrows, edits text/number/select values, uses Escape to leave editing, then navigates to another control without Tab. Enter resumes editing; the first Escape inside an editing field keeps Settings open, held-key repeats do not close it, and a new Escape after leaving the field closes it. Blur resets the field mode and Enter resumes editing without submitting a surrounding form. This is browser keyboard evidence; actual remote Back event delivery remains part of the unrun device acceptance.

## Built language resources

The production-catalog browser journey builds real Vite assets and serves them over local HTTP, then starts the actual application in all 24 languages and opens Settings. It compares every decoded catalog value with the canonical JSON source and exercises malformed-resource rejection followed by a language retry through Settings. Native IPC remains controlled by the desktop fixture. Artifact paths are anchored to the test file, so root and app-directory invocations exercise the same build.

The release-tool suite also runs the locale-validation CLI against temporary catalogs, proving missing required plurals, changed extra-form interpolation and copied-English extra forms are rejected. The bundle CLI counts the shared dictionary, rejects unpacked production resources, missing tables and changed packed values, and retains all route/chunk/locale ceilings. Source catalogs remain ordinary JSON; the distributed arrays are a lossless representation. Every added translation/plural range requiring native review is listed in `Docs/LOCALIZATION_REVIEW.md`.

Mobile supporter browser journeys in Spanish, Arabic and Japanese cover the translated promise, narrow layout, terms-gated recovery input, a restored entitlement and its localized toast. They verify one recovery command and zero new checkout commands for an existing purchaser. The boundary uses controlled native responses; real Android Keystore, devices and purchases remain unrun.

All 24 catalogs, including English, are now local data assets. The bootstrap awaits complete resources before React mounts, using a small canonical-derived set of translated loading/error/retry labels if resources fail. The attempt has an eight-second deadline; timed-out responses cannot mount the application later. Reload clears dictionary/catalog requests. Saved-language lookup waits at most 700 ms before using the system language; normal Settings loading can apply the persisted preference later. This lookup does not write preferences or credentials.

The production journey delays English and dictionary responses, rejects missing and malformed versions of either, waits through an actual timeout, checks that a late response cannot mount the app, and reloads to recover. It also retains supporter loading suppression and checks every canonical value in every language. Standalone development entrypoints explicitly await English before mounting. An earlier foundation full run passed 90 browser journeys and failed 12 standalone journeys because those entrypoints skipped resource readiness; the fixtures were corrected without changing assertions, clocks, axe rules or snapshots. Real installed Tauri/WebView asset loading remains unrun. Startup English and dictionary data bytes must be reported alongside JavaScript reductions.

Localization runtime journeys hold the real entry point at its simulated native startup-health boundary and verify the saved-language progress label, detail and percent. Production update/recovery components are also exercised through Spanish download/verification/install phases and release notes, then an Arabic language change while the error boundary is showing, technical details and reload. The existing update-install retry/package-manager journeys remain required. The browser controls native state and render failures; OS presentation and installed update acceptance remain unrun.

The localization preview journey opens a two-file folder and verifies translated controls, a changed zoom percentage, next/previous file navigation and closing. A saved-language deletion journey opens the real confirmation dialog, confirms deletion, checks the translated outcome and records exactly one native deletion call. Catalog templates for action/name and action/count remain subject to interpolation checks; the CLI distinguishes non-language structure from copied prose.

Localization journeys also exercise the Spanish recovery drill through bundle creation and native verification, asserting no import, and the Spanish favorites menu through the account-scoped file flag command. These browser fixtures do not prove Android reauthentication or installed OS behavior.

Share-password native HTTP journeys cover all 24 canonical catalogs, translated HTML escaping, weighted Accept-Language values, exclusions, ties, aliases and exact decoded language query fields. Each language survives a wrong-password form submission and successful cookie/redirect verification; account switching still revokes access. Header behavior follows [HTTP language negotiation](https://www.rfc-editor.org/rfc/rfc9110.html#section-12.5.4), with a non-excluded default when possible and a default response when no offered language is acceptable. Real WebViews and live Telegram remain unrun.

Desktop notification journeys drive real stored transfer transitions into the same coordinator used by the Tauri event listener, with controlled preference/foreground, tray and delivery boundaries. They cover translated delivery, private filename defaults, persisted receipt/restart deduplication, arrivals during aggregation handoff and tray language changes without a transfer event. The full TransferEngine → Tauri event → native window/OS delivery remains installed-platform acceptance, unrun here. Production catalog browser coverage also rejects native language publication after an invalid resource and permits it only after a successful retry.

P2-08 localization continuation covers translated protection/access safety explanations, the real file-picker upload choice cancellation, and readonly native tray/notification language publication. The local-access journey checks child Escape restoration, resumed parent Tab trapping and final restoration to the outside Settings opener; no server is enabled. Held durable-receipt journeys change notification/filename privacy after the claim is written and before delivery. This is controlled child-process/storage/delivery coverage, not actual OS notification or tray rendering. The media fixture seeds a private session database before atomic replacement and checks both old-account stream revocation and a fresh-account successful read; Windows replacement behavior remains unrun. Locale-capacity review and unapplied next authentication draft are in `Docs/LOCALIZATION_BUDGET_REVIEW.md`; unimplemented surfaces are not covered by passing localized journeys.

The native formatting gate also runs `node scripts/check-rust-formatting.cjs`. A dense expression in the reviewed driver, journeys, inventory, or search paths fails even if rustfmt silently leaves it untouched.

Release correction journeys exercise 50,001-file truncation, real REST and WebDAV partial publication (older known DAV entries are retained), forced refresh while an unforced job is pending, meaningful raw Telegram message changes, cheap cold/restarted reads, scope-specific verified publication and mutations, actual 31-second global-search reuse, and partial search above the row/text bounds. Audit timing can be accelerated in the native driver; the rolling request timestamps are taken at its Telegram-lookup boundary. The 600 ms fixture represents ten nominal minutes at a 1:1000 clock ratio, not ten minutes against Telegram. Disk no-write evidence uses a second SQLite connection's `data_version`. No fixture result establishes a live Telegram rate or latency guarantee.

Online search indexes retain monitored inventory-generation pins; each successful cached query renews their ten-minute consumer lease. Capacity or scope changes can evict the cached index; alternating global and folder scopes can therefore rebuild. An offline fallback has a 30-second lease so it can detect reconnection. The per-index row/text limits are unchanged (100,000 rows and 32 MiB); larger libraries return a bounded `complete: false` answer.


### Release continuation: inventory correction coverage

The native child-process suite exercises two queued old-session peer discoveries across cache clear, discovery-order changes without cursor writes or index rebuild, recovered audit failures, and fitting-prefix search results at the existing text cap. The search fixture shares the production folder/overlay assembly. A native-only clock advance drives the production 30-second poll, four-minute audit eligibility and one-minute rolling lookup budget across a 100,000-row retained set. Its per-ID measurements include admission-to-first lookup, consecutive successful lookups and trailing overdue age; this is healthy controlled-source evidence, not a live Telegram latency or FLOOD-wait guarantee. The release build-input security check requires `.cargo/config.toml` and the FTS5 build probe; packaged target-runtime search is still required on every OS.

TDENC2 known-answer native journeys use immutable public vectors from the v3.9.8 source encoders and this tree. They compare exact bytes, continue saved headers, restore a historical vault/recovery bundle across restart, and reject corrupted envelopes without publishing plaintext. They do not exercise an installed historical app or live Telegram. See [format provenance](Docs/TDENC2_FORMAT.md) and [real-device recovery drill](Docs/ENCRYPTION_RECOVERY_DRILL.md).

Release input preflight: `node scripts/release-preflight.cjs --service-url "$SUPPORTER_SERVICE_URL" --public-key "$SUPPORTER_PUBLIC_KEY"`. Before the owner finalizes the Unreleased heading, add `--allow-unreleased` for local input readiness only. `--skip-network` explicitly returns exit 2 / SKIPPED and cannot pass release readiness. Health verification makes one read-only GET; local HTTP stand-ins are allowed only on loopback. Artifact provenance can be checked with `--verify-attestation FILE --attestation-repo OWNER/REPO`. This tree has not verified a production health response or published attestation.

Storage insights reuse the generation-backed search inventory and report `complete: false` for bounded/offline inventory. The native journey scans 1,200 files across two fixture folders, retains the 1,000-result display limit, and proves subsequent views need no additional source requests. Insights now exclude locally hidden files consistently with workspace search. Proxy transport journeys accept a valid RPC rejection but refuse malformed/disconnected controlled HTTP; the real MTProto/proxy handshake remains UNRUN. Desktop external-file journeys register published downloads, restart, and refuse arbitrary/modified/symlink-substituted paths using an opening sink; installed OS external opening remains UNRUN. Six new hook IDs use UUIDs; persisted legacy transfer IDs remain opaque strings in the durable engine.

File-capability review added a same-size modified-cache journey and a held 32 MiB file-worker journey. Modified registered cache reuse is rejected without deleting bytes. Legacy kept caches remain internally available offline or above the disposable cap, without gaining external-opening permission. Two hashing jobs hold their blocking permits while the native runtime remains responsive; 32 MiB registration plus two validations are expected to read 96 MiB total in 64 KiB buffers. This is controlled scheduling evidence, not a real multi-GB OS launch benchmark. Storage insight aggregation temporarily clones bounded FileMetadata values from the retained index (up to its 100,000-row/32 MiB serialized-text limits), in addition to the already retained index; it does not clone all raw Telegram messages or rebuild the index per view.

Release pipeline process journeys cover plain/RC/malformed tags, first-block changelog extraction, base-version agreement, paginated draft reuse, awaited publication with prerelease/latest policy, refusal to redraft published tags, isolated readiness/early-exit/deadline handling, and RC-to-plain Debian/Arch recipe mapping. These use local API/application fixtures; every GitHub workflow and actual packaged OS startup remains UNRUN. `scripts/check-workflow-pins.cjs` preserves full commit pins, scheduled baseline expiry and conditional signing. YAML was parsed locally; that does not validate GitHub runtime behavior.

Native CI sets `RUST_TEST_THREADS=4` to match the accepted local baseline and bound simultaneous heavy Argon2/search/HTTP journeys. Video fixture helpers now use `.exe` on Windows. Readiness waits allow 15 seconds; deliberate responsiveness/performance assertions retain their own tighter limits. First-run risks: platform file-replacement/permissions and Windows symlink cases have narrower coverage under `cfg(unix)`; WebKitGTK/system libraries, WebView2/COM, FFmpeg helper process termination, macOS Intel execution/Rosetta on the selected runner, GH CLI attestation flags and actual package layouts/signing remain unexecuted. Device/OS acceptance is centralized in [RELEASE_ACCEPTANCE.md](Docs/RELEASE_ACCEPTANCE.md).

Final file-capability regressions replace a file after successful hashing and before lease return, through separate native processes and real filesystem replacement. Opening and cache timestamp renewal retain the exact authenticated identity; neither can authorize a replacement by capturing a later identity. The controlled sink does not launch an OS application.

## Legacy external-opening upgrade

Native child-process journeys seed historical workspace, flat desktop preview/thumbnail and offline-pack layouts with account records and no produced-file registrations. They exercise lazy adoption across restart, exact filenames, planted siblings, protected records, account isolation, symlink directories, concurrent first opens, registration loss and account/protection changes during hashing. The opening sink records the original path; it does not launch an OS application. Installed 3.9.x upgrades remain a release acceptance check.

After building the native driver, run `node scripts/measure-external-opening.cjs` for an opt-in 2,000,000,000-byte synthetic-file measurement. It writes and syncs real bytes, measures first and subsequent opening through the driver, reports hashed byte counters, and removes its private fixture. Results describe warm local storage, not network downloads or cold disk performance.

The maximum-inventory detection journey compresses 184 polls (92 virtual minutes) over 100,000 retained rows into one driver request. An isolated observation took 53.6 seconds; repeated runs exceeded the ordinary 60-second IPC deadline. This request alone has a named 120-second observation deadline. Dataset, polling horizon, production timeouts, four test threads and every request-cost/detection assertion are unchanged.

## Desktop HEIC/HEIF renditions

The native E2E suite exercises real account stores, source registrations, retained cache leases, output publication and an out-of-process controlled decoder. The helper copies known synthetic JPEG pixels; it never proves HEVC decoding. Journeys cover tiled dimensions, original opening, corruption/oversized metadata, missing/old decoders, first-tile rejection, output caps, Telegram-thumbnail precedence, protected sources, offline account records, source mutation, pin/unpin/clear, exact cancellation, caller abort, account change, deadlines and a helper that touches allocated memory until containment terminates it. Linux uses a 2 GiB address-space limit, Windows a 1 GiB committed-memory Job Object before execution, and macOS a 1 GiB sampled RSS termination threshold. The macOS threshold is reactive and can overshoot; it is not a hard address-space ceiling.

Browser journeys show a `.HEIC` photo as its JPEG rendition, retain zoom/original actions, and show `workspace.preview_failed` when decoding is unavailable. Their native and Telegram boundaries are controlled.

After building the native driver, `node scripts/measure-heic-preview.cjs /absolute/path/to/synthetic-48mp.heic` exercises real `/usr/bin/sips` and `/usr/local/bin/ffmpeg` on this Mac. Generate the 48MP fixture with FFmpeg `testsrc2=size=8064x6048`, then `sips -s format heic`; never supply photographs. The script reports per-child OS peak RSS and sampled RSS separately from complete pipeline time, verifies full 12MP pixels, rotation and mirroring, and removes each private account/cache fixture. Real decoder versions, measurements and their limitations are recorded in [HEIC_PREVIEW.md](Docs/HEIC_PREVIEW.md). Real iPhone photos, installed WebViews and Windows/Linux helper behavior remain owner-run acceptance rows.

The controlled HEIC helper is the 103 MB native fixture executable. Its first platform signature/startup check is primed with a bounded 20-second fixture-only version call; the actual production version probe still has its 2-second deadline. This prewarmed helper cannot establish cold FFmpeg startup performance. Real host FFmpeg measurements use the installed executable and the production deadline.
