# Separate host prerequisite — NOT APPLIED

Base: `ea100cee3f69711de12485ba061950c5fc65258f`.
The port never modifies the host checkout or SDK. Review `host.patch` separately.

The exact pinned native producer annotates `unchanged` unchanged. The renderer's
action handler then treats it as a view, replacing the current root with undefined.
The next render throws when reading `state.root.type`. Old/foreign IDs must remain
inert at the extension, so pretending they produced a fresh view would obscure the
contract. The patch makes successful unchanged search/action responses inert in
the renderer as well (load-more already handles them).

Pinned ListRuntime already filters Lists client-side when `__hasServerSearch` is
absent, but still emits debounced server-search calls. Those unnecessary responses
can arrive across navigation and replace a newer surface. The patch sends a search
only for explicitly server-owned Lists. This port intentionally never echoes
`searchText` and supplies all 419 catalog choices, so local drafts stay client-owned.
No extension-specific branch, asset resolver or reverse RPC is added to the host.

Form fields are submit-only. Initial values are supplied once; later responses omit
values, activating the existing `reconcileFormValues` draft-preservation path.
There is no server field revision pretending to own a newer client draft.

Run `python3 v2/scripts/owned.py node scripts/host-tests.cjs` after bootstrap and
production process tests. It applies this patch **only** to ignored fixture copies
under `v2/qa/patched-host/`, executes the original and patched actual components,
reproduces the original unchanged failure, and tests input drafts, filtering,
metadata, exact PNG data URLs and schema-3 positive/negative cases. The Rust host
fixture tests execute original launch/annotation/validation bodies; hashes of full
immutable inputs are in `host-pins.json`. No substring-only success assertion and
no hand-written replacement renderer is used. Native host acceptance on macOS and
Windows remains pending.
