# Timezone Converter — standalone Rust V2

Schema 3, executable process, protocol 1. This is an independent Rust implementation
of the single V1 `index` command, using pinned SDK 0.1.1 over HTTPS. No JavaScript,
Node, Bun, host source, sibling port, network provider or SDK modification is needed
at runtime. `Cargo.lock` is included for review. Source `artifacts` is
empty; macOS and Windows native acceptance remain `not-run`.

The clone is prepared for parent review, not publication. Branch
`feat/native-v2-rust`, base `784f7d1c1743ee6228f26a2384f925475cfb11e9`.
Temporary T3 project registration: `mcp:9aac3707-3d77-47db-aa87-8c7f05a394b1`.
The clone remains intact, with changes unstaged and uncommitted. No push or install was performed.

## Behavior

A new session detects the OS local IANA zone and snapshots the clock once. It does
not tick. Clear Custom Time takes a fresh local snapshot. Selected timezones persist
in the extension-private `dataDir/selected-timezones.json`, using the declared
`{"version":1,"selected":[...]}` format. V1 profile/cache data is never read or migrated.

Add Timezones is a List with complete Selected and Others sections. Every one of
the 419 choices (`UTC` plus the recorded original `Intl.supportedValuesOf` result)
is present; client filtering does not truncate the catalog. Names replace `/` with
` - ` and `_` with a space. Action labels use the current clock's GMT offset,
independently of the snapshot/custom instant used for conversions. Daylight saving,
quarter-hour, half-hour and historical second offsets use native IANA arithmetic.

The required Sort Options preference has no default, exactly as V1. Alphabetical
sorting mutates the in-memory selected order on load/render; the resulting order
is written only on the next deliberate persistence action, matching V1's in-place
`.sort()` behavior. Manual mode provides Move Up, Move Down and per-zone removal.
Single deletion and remove-all use separate confirmation screens with Cancel.
Reorder actions use current list bounds; see the specific V1 defect below.

Custom Time uses a supported TextField plus a full timezone Dropdown in place of
the original native DatePicker. Enter a local `YYYY-MM-DDTHH:mm[:ss[.SSS]]` value,
or a precise RFC3339 instant with `Z`/offset to select either occurrence of a DST
overlap. An unchanged initial local field retains its exact original instant,
including a later overlap occurrence. New ambiguous local input selects the
earlier occurrence; gaps advance by the actual transition width. Confirm then
performs the original Luxon `toLocal()` components → `set()` in the selected zone
→ `setZone()` chain. It does **not** simply reinterpret an ISO string as the target
zone. The original TSX is executed in the oracle to verify this surprising behavior.
Invalid input remains on the Form with its client-owned draft and a fresh Confirm
action. Later Form responses omit defaults, so the pinned reconciliation function
preserves drafts, including A → B invalid → A.

Hide World Map selects List rows with converted times. Otherwise Detail shows the
local/custom snapshot, exact original map and converted metadata tag lists. PNG
bytes are embedded and emitted as `data:image/png;base64,...`; the pinned actual
Detail image resolver accepts this scheme. No clone-dependent asset path or remote
placeholder is used. V1 ships maps only for UTC and integral offsets ±1..12. For
fractional offsets, +13/+14, or historical second offsets, V2 explicitly reports
that the original map is unavailable. It never regenerates or substitutes a map.

## Data, bounds and intentional differences

Native offsets use `tz-rs 0.7.0`, `tzdb 0.7.2`, and `tzdb_data 0.2.4` (IANA **2026a**),
including POSIX recurrence beyond the last TZif transition. IANA 2026a `backward`
links reproduce ICU's non-backzone historical model; full-backzone offsets would
otherwise differ before 1970. English short names and historical metazone changes
use Unicode CLDR **48**, commit `acd6d88ae493633240e19a87a721076a8a75c310`.

The original oracle runs with installed Node 26.7.0 / ICU 78.3 / CLDR 48.0 /
tz 2026a and Luxon **3.3.0** from the unchanged V1 lock. Catalog and `ff` / `ZZZZ`
formatting are deliberately frozen to that en-US environment. Other locales and
future changes to an OS's Intl catalog are not claimed. Conversion supports the
actual chrono civil range (roughly years -262143..+262142), with checked local
boundary arithmetic and explicit errors; JavaScript's small additional extreme
date range is excluded. There is no invented 1900–2099 cutoff. Leap seconds and
sub-millisecond input are rejected explicitly.

The original captured OrderTimezones props keep the initial length after removal.
Its stale Move Down bound can introduce `undefined` into the list. The executed
TSX oracle reproduces that defect. V2 deliberately uses the current length and
keeps only valid, unique catalog zones, rather than persisting that invalid state.
This is a narrow behavior difference, not a claim of bug-for-bug parity.

Other resource bounds: 128 active adapter/SDK sessions; 1..256-byte session IDs;
1024-byte search text; 40-byte dates; 128-byte timezone input; at most 419 saved
zones; 65536-byte storage reads; fixed, at most 8-step IANA link resolution; no
unbounded zone scan or guessed recurring clock task. The SDK additionally bounds
JSONL frames at 16 MiB and queues/handlers as documented in its pinned contract.

## Persistence and lifecycle

A process-owned mutex serializes bounded storage transactions and session state.
A temporary file in the data directory is flushed and atomically replaced; failed
reads, validation, temp-file writes or replacement preserve the previous file.
Corrupt/unsupported storage produces a visible error and explicit Retry Loading
Saved Timezones action. No corrupt-state reset and no automatic mutation retry.
Stale sessions compare the exact loaded bytes before writing; another session's
successful update produces an error instead of overwriting it. Reload Saved
Timezones is explicit and never repeats a rejected mutation.

Action IDs include a per-adapter UUID so another process/adapter cannot reuse them
even when a session name is reused. Navigation, drafts and consumed tokens live outside the SDK staged
`Value`. All prior surface IDs are consumed before mutation/navigation; duplicate,
stale and foreign IDs return `unchanged` without storage or UI changes. Scoped
cleanup guards handle disposal, canceled opens after adapter return, EOF and
shutdown. The test suite cancels 140 opens in that exact window, then opens 128
live sessions. It also cancels a storage action after the handler's write and
proves the old ID cannot toggle it back. Cancellation may occur after a write;
there is no exactly-once or crash-durability claim. Close/reopen a session if its
response was canceled after its action tokens were consumed.

## Required host integration (unapplied)

Read [integration/README.md](integration/README.md). The small
[integration/host.patch](integration/host.patch) targets pinned host
`ea100cee3f69711de12485ba061950c5fc65258f`. It preserves the current screen on
`unchanged` responses and avoids sending server-search requests for client-owned
Lists. Original and patched actual components are executed in the tests, including
the reproduced baseline failure. No host or SDK working file was changed. Native
host acceptance requires parent review/application of that separate patch.

## Reproduce verification

From the selected clone root, using existing installed toolchains only:

```sh
python3 v2/scripts/bootstrap.py
python3 v2/scripts/verify.py
python3 v2/scripts/owned.py --lane fresh-review cargo +stable build --release --locked
python3 v2/scripts/owned.py python3 scripts/package.py qa/lanes/fresh-review/target/release/timezone-converter
python3 v2/scripts/owned.py python3 scripts/process-test.py --packaged
python3 v2/scripts/owned.py python3 scripts/audit.py --independent
```

Use a never-used lane name to prove an empty Cargo home/target build. The wrapper
binds subprocess HOME/TMPDIR, Cargo homes/targets, npm cache and XDG paths beneath
`v2/qa/`, while reusing installed Rust toolchains. It never installs a toolchain or
changes global settings. Bootstrap copies exact pinned V1 through `git archive`,
then runs `npm ci --ignore-scripts` on that disposable copy and exact QA tooling
lock. `legal.py` needs existing Python ≥3.11 (here `/opt/homebrew/bin/python3`);
other scripts also work with the installed Python 3.9. No V1 lock is regenerated.

`oracle.cjs` loads original TSX/helper and locked Luxon into a closed VM, exposing
only synthetic React hooks, cached state, navigation, confirmation, fixed Date and
Intl metadata. No filesystem/network/OS callback is exposed to evaluated source.
`host-tests.cjs` executes pinned actual Form/List/Detail/metadata/RuntimeView and
Form reconciliation functions with inert hooks and host APIs. Rust host-contract
tests execute original extracted validator, launch JSON producer and annotation
function bodies, with test-only outer types. Only two narrow Clippy allowances
preserve copied host code unchanged. These are component/function tests, not a
full host build, browser test or native UI acceptance.

`package.py` runs without any host checkout. It makes two byte-identical ZIPs,
checks closed hashes and modes, extracts into owned paths, and leaves only binary,
manifest, resources and authentic notices in the archive. No QA helper, source,
SDK checkout, node_modules, cache or test endpoint is packaged. The extracted
production executable is then tested through real SDK stdin/stdout pipes.
Development ZIPs are ignored under `qa/package/`, never release artifacts.

See [VERIFICATION.md](VERIFICATION.md), [legal/source-pins.json](legal/source-pins.json),
[legal/dependencies.json](legal/dependencies.json), and
[REVIEW-INVENTORY.json](REVIEW-INVENTORY.json). The inventory covers every
Git-visible README/V2 review candidate except itself; generated QA paths remain
ignored. Parent must rehash staged blobs after review.

## Original artwork and licenses

Icon SHA256: `71a3e94c18b7465d05ac0d7a897a14bb49f4c1d92797b3911813aca15f8b6dd5`.
The icon and all 25 PNG maps match own pinned V1 **and** independently fetched
Raycast upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a` byte-for-byte.
[legal/upstream-assets.json](legal/upstream-assets.json) records individual URLs,
lengths and hashes. The upstream extension declares MIT and has no per-extension
LICENSE; the repository's genuine 2021 Raycast MIT grant is reproduced separately.
No asset-specific authorship or additional grant is invented. SDK Apache notices,
Unicode and IANA licenses, own original license bytes and authentic locked Rust
archive notices are included. Both locked `r-efi` archives retain their authentic `AUTHORS` files, which include
the MIT grant and copyright notices. The target-only `wasip2` 1.0.1+wasi-0.2.4
archive has no grant file; its declaration and evidence gap are recorded
explicitly in `legal/dependencies.json`.
