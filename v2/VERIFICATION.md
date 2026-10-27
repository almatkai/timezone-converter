# Verification — parent review handoff

All checks below used the selected clone only. The branch is
`feat/native-v2-rust`; HEAD remains
`784f7d1c1743ee6228f26a2384f925475cfb11e9`. The index matches HEAD with no
staged changes. Expected short status is ` M README.md` and `?? v2/`.
The clone and temporary project registration
`mcp:9aac3707-3d77-47db-aa87-8c7f05a394b1` remain intact for parent cleanup.
No commit, push, PR, release, install, host/SDK edit or profile migration occurred.

## Executed checks

| Check | Result |
| --- | --- |
| Rust stable 1.98.0 fmt / locked tests / strict all-target Clippy | Passed, 10 tests |
| Rust 1.85.0 fmt / locked tests / strict all-target Clippy | Passed, 10 tests |
| Windows GNU 1.85 all-target check / strict Clippy | Passed |
| Windows GNU link build | Blocked by missing `x86_64-w64-mingw32-dlltool`; no installer used |
| Original V1 disposable-copy tests / typecheck | Passed, 9 tests; exact original lock retained |
| Original TSX/helper + locked Luxon 3.3.0 oracle | 419 catalog choices in original order; 2,933 formatting/offset cases and 288 actual CustomTime component cases matched |
| Source component flows | Snapshot, fresh reset, selected/others toggles, alphabetical mutation, up/down, single/remove-all confirmations, List and Detail executed |
| SDK lifecycle | 140 opens canceled after adapter handle return, then 128 live opens; canceled storage action cannot replay; EOF cleanup passed |
| Real production SDK JSONL pipes | 303 responses per production smoke; every UI action family, invalid→valid drafts, foreign/old IDs, stale-session conflict/reload, storage corruption/failure, 128-session capacity/dispose, EOF/shutdown, 4 invalid frame classes passed |
| Actual pinned renderer components | Form/List/Detail/metadata/RuntimeView and draft reconciliation executed; A/B/A retained; original host unchanged-result failure reproduced and patched fixture passed |
| Exact schema-3 / pinned producer | Schema positive + 9 negatives; original Rust validator and launch/annotation bodies executed with positive/negative cases; absent launch searchText is JSON null |
| Empty Cargo home + target release build | Passed in `qa/lanes/parent-review-20261027/`; SDK fetched from pinned HTTPS Git revision, no sibling dependency |
| Deterministic ARM64 development ZIP | Two byte-identical archives; 185 closed files including manifest, exact executable/icon/maps, 0755 executable mode; extracted production smoke passed |
| Independent immutable-object audit | All 38 original V1 files other than README remain byte-identical; README only adds the discovery link; all 26 original artwork files also match pinned upstream; SDK license/notice and 8 host fixtures match pinned Git objects |

`python3 v2/scripts/verify.py` reproduces the normal suite after bootstrap. Its
machine-readable per-command exit results are in ignored
`qa/verification-results.json`; all 14 checks returned zero. Build/package/smoke
commands and the owned subprocess environment are documented in README.
The integration patch is **unapplied** and must be reviewed separately. No full
host build or real renderer DOM/native acceptance was claimed.

## Development package evidence

- Archive: `qa/package/timezone-converter-aarch64-apple-darwin-dev.zip`.
- Extracted root: `qa/package/extracted-44296ada94c2c473`.
- ZIP SHA256: `44296ada94c2c473b29b3ae3432fcbfc03e4714fc0c3b6f1d6e0160e0039172a`.
- Executable SHA256: `dc1ad8304140951fb06ef4357f9bf8138349d4613b0ec96b02ba9ea2a7af16cb`.
- Original icon SHA256: `71a3e94c18b7465d05ac0d7a897a14bb49f4c1d92797b3911813aca15f8b6dd5`.
- Closed per-file hashes: ignored `qa/package/report.json` and extracted manifest.
- Source `artifacts: {}`; both OS native verification entries remain `not-run`.

The archive contains no test runner, QA endpoint, fixtures, SDK checkout, Node/Bun
runtime, dependency cache, target directory or dev server. Its executable performs
the real conversion, persistence and protocol work. The deterministic claim is
about ZIP assembly from the same inputs, not cross-toolchain binary reproducibility.

## Explicit remaining gaps and reviewed adaptations

Native UI acceptance on macOS and Windows is pending, as is a real Windows MSVC
executable. The Windows GNU link attempt hit a missing tool rather than a Rust
check/Clippy failure. The separate host patch is required for safe unchanged
responses and client-owned List search. It was tested only against owned pinned
fixture copies, never applied to the user's host checkout.

Formatting/catalog are frozen to the recorded en-US Node/ICU/CLDR environment,
not every possible OS locale/catalog revision. Native date bounds are chrono's
actual representable civil range; sub-millisecond and leap-second input is rejected.
Missing original fractional/+13/+14/historical-offset map images have explicit
unavailable text, preserving the exact shipped resources. The original captured
OrderTimezones stale-length defect is reproduced in the TSX oracle; V2 deliberately
prevents its undefined-zone corruption by using current bounds. Full details are
in README. Cancellation after a write does not imply rollback; no exactly-once or
crash-durability assertion is made.

Exact license inputs and all locked crate declarations/checksums are retained.
Both locked `r-efi` archives retain their authentic `AUTHORS` files with MIT grants
and copyright notices. The `wasip2` 1.0.1+wasi-0.2.4 locked archive has no grant
file; that evidence gap is explicitly recorded, without synthesizing a grant.
The upstream extension directory has no LICENSE; the genuine repository-root MIT
license and extension MIT declaration are both included.

## Final review inventory

`REVIEW-INVENTORY.json` is generated last by
`python3 v2/scripts/owned.py python3 scripts/audit.py --independent`. It hashes
every Git-visible root README/V2 candidate excluding itself, checks that all eight
immutable host fixtures are visible, and rejects visible QA/cache/target/ZIP paths.
Ignored `qa/fixtures`, `qa/patched-host`, dependencies, build outputs and development
archives remain local. Parent should rehash staged blobs after review.

Parent review repeated all 14 checks and the empty-Cargo-home production build.
The extracted executable passed 303 production JSONL responses. The parent also
restored the authentic r-efi AUTHORS grant files and simplified the date field hint.

Final XERJ refresh for `ax-9244e91eebbd` was incomplete: the server refused a
keyword-to-text mapping change and then returned HTTP 429 for its memtable budget.
Only this indexing client was stopped; no server/schema/settings changes were made.
Current source reads, Git-object comparisons and exhaustive hash checks supplied
final coverage instead of treating missing index matches as absent code.
