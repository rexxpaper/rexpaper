import type { ComponentChildren } from "preact";
import { cn } from "../lib/cn";
import {
  applying,
  applyRandomWallpaper,
  chooseLiveWallpaperDir,
  chooseWallpaperDir,
  livePaused,
  openFolder,
  pausing,
  runScan,
  scanCounts,
  scanError,
  scanning,
  settings,
  settingsError,
  toggleLiveWallpaperPause,
  toggleMuteLiveWallpapers,
  togglePauseOnFullscreen,
  toggleRunOnStartup,
} from "../lib/store";

// --- Primitives --------------------------------------------------------------

const ghostButton =
  "rounded-md border border-border px-3 py-1.5 text-sm text-text transition-colors duration-150 ease-out hover:bg-panel focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:opacity-50";

const primaryButton =
  "rounded-md bg-primary px-4 py-2 text-sm font-medium text-bg transition-colors duration-150 ease-out hover:bg-primary/90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary disabled:opacity-50";

/** Switch: role=switch with a 150ms knob slide, per the DESIGN.md controls rule. */
function Toggle({
  checked,
  label,
  onChange,
}: {
  checked: boolean;
  label: string;
  onChange: (enabled: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      onClick={() => onChange(!checked)}
      class={cn(
        "relative h-5 w-9 shrink-0 rounded-full border transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
        checked ? "border-text bg-text" : "border-border bg-card",
      )}
    >
      <span
        class={cn(
          "absolute left-0.5 top-1/2 size-3.5 -translate-y-1/2 rounded-full transition-transform duration-150 ease-out",
          checked ? "translate-x-[18px] bg-bg" : "bg-secondary",
        )}
      />
    </button>
  );
}

/** One hairline-divided property row: label (+ optional hint) and a control. */
function SettingRow({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ComponentChildren;
}) {
  return (
    <div class="flex items-center justify-between gap-4 border-b border-border py-3 last:border-b-0">
      <div class="min-w-0">
        <p class="text-sm text-text text-pretty">{label}</p>
        {hint !== undefined && (
          <p class="mt-0.5 text-xs text-muted text-pretty">{hint}</p>
        )}
      </div>
      <div class="flex shrink-0 items-center gap-2">{children}</div>
    </div>
  );
}

/** Folder row: truncated path plus Change… and Open actions. */
function FolderRow({
  label,
  path,
  onPick,
}: {
  label: string;
  path: string | null;
  onPick: () => void;
}) {
  return (
    <SettingRow label={label}>
      <span
        class="max-w-56 truncate text-sm text-muted"
        title={path ?? undefined}
      >
        {path ?? "Not set"}
      </span>
      <button type="button" onClick={onPick} class={ghostButton}>
        Change&hellip;
      </button>
      {path !== null && (
        <button
          type="button"
          onClick={() => void openFolder(path)}
          class={ghostButton}
        >
          Open
        </button>
      )}
    </SettingRow>
  );
}

// --- Page --------------------------------------------------------------------

export function SettingsPage() {
  const s = settings.value;
  const counts = scanCounts.value;
  const applyingNow = applying.value !== null;
  const hasLive = s?.active_live_wallpaper != null;

  return (
    <div class="mx-auto w-full max-w-3xl space-y-6 p-6">
      {settingsError.value !== null && (
        <p
          role="alert"
          class="rounded-md border border-border bg-card px-3 py-2 text-sm text-text text-pretty"
        >
          {settingsError.value}
        </p>
      )}
      {settingsError.value === null && s === null && (
        <p class="text-sm text-muted">Loading&hellip;</p>
      )}

      {s !== null && (
        <>
          <section class="rounded-lg border border-border bg-card p-5">
            <h2 class="mb-1 text-sm font-medium text-secondary text-balance">
              Libraries
            </h2>
            <FolderRow
              label="Wallpaper folder"
              path={s.wallpaper_dir}
              onPick={() => void chooseWallpaperDir()}
            />
            <FolderRow
              label="Live wallpaper folder"
              path={s.live_wallpaper_dir}
              onPick={() => void chooseLiveWallpaperDir()}
            />
            <SettingRow label="Scan libraries" hint={counts !== null ? `${counts.staticCount} static \u00b7 ${counts.liveCount} live` : undefined}>
              <button
                type="button"
                onClick={() => void runScan()}
                disabled={scanning.value}
                class={primaryButton}
              >
                {scanning.value ? "Scanning\u2026" : "Scan libraries"}
              </button>
            </SettingRow>
            {scanError.value !== null && (
              <p class="mt-3 rounded-md border border-border bg-panel px-3 py-2 text-sm text-text text-pretty">
                {scanError.value}
              </p>
            )}
          </section>

          <section class="rounded-lg border border-border bg-card p-5">
            <h2 class="mb-1 text-sm font-medium text-secondary text-balance">
              Behavior
            </h2>
            <SettingRow
              label="Run on startup"
              hint="Starts RexPaper with Windows and restores your wallpaper."
            >
              <Toggle
                checked={s.run_on_startup}
                label="Run on startup"
                onChange={(next) => void toggleRunOnStartup(next)}
              />
            </SettingRow>
            <SettingRow
              label="Pause on fullscreen"
              hint="Live wallpapers pause while a game or video runs fullscreen."
            >
              <Toggle
                checked={s.pause_on_fullscreen}
                label="Pause on fullscreen"
                onChange={(next) => void togglePauseOnFullscreen(next)}
              />
            </SettingRow>
            <SettingRow
              label="Mute live wallpapers"
              hint="Previews are always muted; this affects the applied wallpaper. Applies next time a live wallpaper is set."
            >
              <Toggle
                checked={s.mute_live_wallpapers}
                label="Mute live wallpapers"
                onChange={(next) => void toggleMuteLiveWallpapers(next)}
              />
            </SettingRow>
          </section>

          <section class="rounded-lg border border-border bg-card p-5">
            <h2 class="mb-1 text-sm font-medium text-secondary text-balance">
              Status
            </h2>
            <SettingRow label="Mode">
              <span class="text-sm text-muted">
                {s.wallpaper_mode === "live"
                  ? livePaused.value
                    ? "Live \u2014 paused"
                    : "Live"
                  : s.wallpaper_mode === "static"
                    ? "Static"
                    : "\u2014"}
              </span>
            </SettingRow>
            {s.active_static_wallpaper !== null && (
              <SettingRow label="Active static">
                <span
                  class="max-w-56 truncate text-sm text-muted"
                  title={s.active_static_wallpaper}
                >
                  {s.active_static_wallpaper}
                </span>
                <button
                  type="button"
                  onClick={() => void openFolder(s.active_static_wallpaper ?? "")}
                  class={ghostButton}
                >
                  Open
                </button>
              </SettingRow>
            )}
            {hasLive && (
              <SettingRow label="Active live">
                <span
                  class="max-w-56 truncate text-sm text-muted"
                  title={s.active_live_wallpaper ?? undefined}
                >
                  {s.active_live_wallpaper}
                </span>
                <button
                  type="button"
                  onClick={() =>
                    void openFolder(s.active_live_wallpaper ?? "")
                  }
                  class={ghostButton}
                >
                  Open
                </button>
                <button
                  type="button"
                  onClick={() => void toggleLiveWallpaperPause()}
                  disabled={pausing.value}
                  class={ghostButton}
                >
                  {pausing.value
                    ? "\u2026"
                    : livePaused.value
                      ? "Resume"
                      : "Pause"}
                </button>
              </SettingRow>
            )}
            <SettingRow
              label="Next wallpaper"
              hint="Applies a random wallpaper from your library."
            >
              <button
                type="button"
                onClick={() => void applyRandomWallpaper()}
                disabled={applyingNow}
                class={ghostButton}
              >
                {applyingNow ? "Applying\u2026" : "Apply random"}
              </button>
            </SettingRow>
          </section>
        </>
      )}
    </div>
  );
}
