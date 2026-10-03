# P2-08 localization capacity review

The current implementation stays within every reviewed budget. P2-08 remains partly done because the remaining mandatory authentication copy cannot fit the per-catalog ceiling.

## Measured current build

- Largest catalog: Bengali (`bn-BD`), **124,788 / 125,000 bytes**; 212 bytes remain.
- Total locale data, including the shared key dictionary: **1,733,112 / 1,800,000 bytes**.
- Initial JavaScript: **504,726 / 513,000 bytes**; English (47,430 bytes) and the shared key dictionary (36,922 bytes) add **84,352 bytes** of separate startup data, not included in this JavaScript measure.
- Media feature: **249,129 / 250,000 bytes**.
- P2-11 ceilings were lowered after measured reductions: initial JavaScript 550,000 → 513,000; desktop route 310,000 → 302,000; Settings feature 160,000 → 159,000. No ceiling has been raised. UI literals have been ratcheted to the measured 362 findings; copied-English ceilings also reflect their current measured values.

## Concrete next scope

[The unapplied English/Bengali draft](localization-auth-capacity-draft.json) contains 13 required authentication setup/help strings: `qr_title`, `qr_description`, `advanced`, `continue_phone`, `continue_qr`, `qr_path`, `help_intro`, `portal_heading`, `portal_instruction`, `create_heading`, `create_instruction`, `copy_heading`, and `copy_instruction`, under the proposed `auth_copy` namespace. The API portal URL, externally named API development tools, numeric/string credential explanation and privacy meaning are preserved. This is a capacity proposal, not a shipping locale resource; the other 22 translations and UI extraction have not been implemented.

Encoding those values with the existing exact `td-locale-v1` format produces a **126,829-byte Bengali packet**, **1,829 bytes over** the current ceiling. This uses the same sorted union key table, SHA-256 identifier, canonical values and JSON serialization as the production encoder. The measurement matched actual builds for both preceding namespaces.

Before reaching this decision, 34 earlier dead keys (plus plural variants) and 12 further verified-unused keys were retired from all catalogs. Nine exact all-24 duplicate pairs were consolidated after caller checks, recovering 825 Bengali packet bytes with the additional retirements. Existing Android/native consumers were preserved; no local Java/Kotlin sources are present in this worktree.

All remaining groups whose values are identical across every locale offer a conservative upper bound of **568 additional Bengali packet bytes**, even assuming every dynamic caller can safely be consolidated. That theoretical maximum still leaves this next authentication scope **1,261 bytes over** the ceiling. The further help, shortcut, settings/privacy, share, archive, player and mobile prose has not been added to this estimate. A 126,829-byte ceiling would therefore only cover this next draft; it is not an estimate for finishing P2-08.

## Owner decision

`app/bundle-budget.json` states: “Further increases require a performance review.” The continuation brief forbids raising a ceiling without an owner decision. A reviewed per-catalog budget change or an approved product-copy/scope change is needed before continuing the remaining extraction. Safety and supporter statements have not been shortened, locale bytes have not been reclassified, and no new language fallback or compressed accounting has been introduced to make the check pass.

Native review is still required for every authored translated range listed in [LOCALIZATION_REVIEW.md](LOCALIZATION_REVIEW.md), including the unapplied Bengali `auth_copy` draft above. No translation is marked native-approved.
