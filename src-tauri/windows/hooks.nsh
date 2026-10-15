; RexPaper NSIS installer hooks.
; Referenced from tauri.conf.json > bundle.windows.nsis.installerHooks.
;
; Kept intentionally small. Tauri's generated installer already detects a
; running RexPaper and prompts the user to close it before an upgrade, so we
; only need to clean up machine state that the app itself creates.

!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove the per-user autostart entry the app writes when
  ; "Run on startup" is enabled.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "RexPaper"

  ; Remove leftover wallpaper bitmap conversions from %TEMP%.
  Delete "$TEMP\rexpaper_wallpaper_*.bmp"
!macroend
