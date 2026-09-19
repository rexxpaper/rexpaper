## Summary

This PR fixes the Windows 11 24H2+ ("raised desktop") live wallpaper issue where:
1. **Extra floating mpv window** - mpv was rendering in a misplaced window instead of behind desktop icons
2. **Console/terminal flash** - a console window appeared when using live wallpapers

## Changes

### `src/platform/windows.rs`
- **Raised-desktop detection**: Checks `GetWindowLongPtrW(progman, GWL_EXSTYLE) & WS_EX_NOREDIRECTIONBITMAP` to identify Win11 24H2+ layout
- **Layered host window**: Creates a `WS_EX_LAYERED` opaque child window of Progman, z-ordered between `SHELLDLL_DefView` (desktop icons) and the system wallpaper `WorkerW` - mpv renders into this via `--wid`
- **Classic fallback**: Preserves the existing top-level `WorkerW`-behind-`DefView` path for Windows 10 / non-raised Win11
- **Cleanup**: Destroys the host window on `stop_live_wallpaper()`
- **Console suppression**: Added `--no-terminal` to mpv args; `CREATE_NO_WINDOW` already present
- **mpv.exe-only resolution**: `find_mpv_executable()` skips `mpv.com` (console subsystem) and resolves `mpv.exe` sibling

### `src/thumbnail.rs`
- `find_mpv_for_thumbnail()` mirrors the same mpv.exe-only PATH fallback

### `.github/workflows/release.yml`
- Job gate `if: github.repository == 'rexxpaper/rexpaper'` - fork pushes skip; builds run only on merges/tags to main org repo
- Removed `mpv.com` from release staging and portable zip

### `wix/main.wxs`
- Dropped `MpvCom` component - only `mpv.exe`, `libmpv-2.dll`, `mpv.dll` ship in MSI

## Verification
- `cargo check --release` ✅
- `cargo build --release` ✅
- Single build works on both Windows 10 and Windows 11 24H2+
- No `mpv.com` ships -> no console flash on Windows 11 (default terminal = Windows Terminal)
- Live wallpaper renders behind desktop icons on raised-desktop layouts

## Requirements
- None for end users - MSI bundles all mpv runtime binaries
- Works out of the box after install