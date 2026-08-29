# Timezone Converter for Tezbar

Convert any time to any timezone in Tezbar on Windows or macOS.

## Command

- **Timezone Converter** — view local time and any added timezones. Add/remove zones via action submenu (selected vs others, with offset badge), toggle custom time via date picker, and sort alphabetically or manually (reorder command). Preferences: `Sort Options` (alphabetically / manually) and `Hide World Map` to show a compact list vs detail with world map.

World map assets are bundled under `assets/timezones/` (per-offset PNGs).

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform `@raycast/api`, `@raycast/utils`, and `luxon` (pure JS DateTime) and retains the upstream macOS-compatible design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
pnpm build
```

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `timezone-converter` extension (upstream package `raycast-timezone-converter`) at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author `joao.mainka` and contributors remain credited in `package.json`.
