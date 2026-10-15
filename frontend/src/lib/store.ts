import { computed, signal } from "@preact/signals";
import { listen } from "@tauri-apps/api/event";
import {
  applyLive,
  applyStatic,
  getSettings,
  nextWallpaper,
  openInExplorer,
  pickLiveWallpaperDir,
  pickWallpaperDir,
  resolveThumbnails,
  scanLive,
  scanStatic,
  setMuteLiveWallpapers,
  setPauseOnFullscreen,
  setRunOnStartup,
  stopLive,
  toggleLivePause,
  type LiveWallpaperItem,
  type Settings,
  type ThumbnailBatch,
  type ThumbnailKind,
  type WallpaperItem,
} from "./api";

export type Route = "static" | "live" | "settings";

export type ThumbState =
  | { status: "pending" }
  | { status: "failed" }
  | { status: "done"; url: string };

export const route = signal<Route>("static");
export const setRoute = (r: Route): void => {
  route.value = r;
};

export const settings = signal<Settings | null>(null);
export const settingsError = signal<string | null>(null);

export const scanning = signal(false);
export const scanCounts = signal<{
  staticCount: number;
  liveCount: number;
} | null>(null);
export const scanError = signal<string | null>(null);

export const showAbout = signal(false);

// Gallery state ----------------------------------------------------------------

export const staticItems = signal<WallpaperItem[]>([]);
export const liveItems = signal<LiveWallpaperItem[]>([]);
export const staticCategories = signal<string[]>(["All"]);
export const liveCategories = signal<string[]>(["All"]);
export const staticCategory = signal("All");
export const staticSearch = signal("");
export const liveCategory = signal("All");
export const liveSearch = signal("");
export const applying = signal<string | null>(null);
export const stopping = signal(false);
export const galleryError = signal<string | null>(null);
export const livePaused = signal(false);
export const pausing = signal(false);

export const thumbStates = signal<Record<string, ThumbState>>({});
export const thumbPendingCount = computed(
  () =>
    Object.values(thumbStates.value).filter((t) => t.status === "pending")
      .length,
);

export const errorMessage = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

const uniqueCategories = (items: { category: string }[]): string[] => [
  ...new Set(["All", ...items.map((i) => i.category)]),
];

export async function loadSettings(): Promise<void> {
  try {
    settings.value = await getSettings();
    settingsError.value = null;
  } catch (err) {
    settingsError.value = errorMessage(err);
  }
}

export async function runScan(): Promise<void> {
  scanning.value = true;
  scanError.value = null;
  try {
    const statics = await scanStatic();
    const lives = await scanLive();
    staticItems.value = statics;
    liveItems.value = lives;
    staticCategories.value = uniqueCategories(statics);
    liveCategories.value = uniqueCategories(lives);
    scanCounts.value = {
      staticCount: statics.length,
      liveCount: lives.length,
    };
  } catch (err) {
    scanError.value = errorMessage(err);
  } finally {
    scanning.value = false;
  }
}

/** Opens the native folder picker, stores the choice, and rescans. */
export async function chooseWallpaperDir(): Promise<void> {
  galleryError.value = null;
  try {
    const updated = await pickWallpaperDir();
    if (updated === null) return;
    settings.value = updated;
    staticItems.value = [];
    staticCategory.value = "All";
    // The mount effect notices the new directory and triggers the scan.
  } catch (err) {
    galleryError.value = errorMessage(err);
  }
}

/** Same as [`chooseWallpaperDir`] for the live video library. */
export async function chooseLiveWallpaperDir(): Promise<void> {
  galleryError.value = null;
  try {
    const updated = await pickLiveWallpaperDir();
    if (updated === null) return;
    settings.value = updated;
    liveItems.value = [];
    liveCategory.value = "All";
  } catch (err) {
    galleryError.value = errorMessage(err);
  }
}

export async function applyStaticWallpaper(path: string): Promise<void> {
  if (applying.value !== null) return;
  applying.value = path;
  galleryError.value = null;
  try {
    settings.value = await applyStatic(path);
  } catch (err) {
    galleryError.value = errorMessage(err);
  } finally {
    applying.value = null;
  }
}

export async function applyLiveWallpaper(path: string): Promise<void> {
  if (applying.value !== null || stopping.value) return;
  applying.value = path;
  galleryError.value = null;
  try {
    settings.value = await applyLive(path);
    livePaused.value = false;
  } catch (err) {
    galleryError.value = errorMessage(err);
  } finally {
    applying.value = null;
  }
}

/** Stops the live wallpaper; the static background beneath is revealed. */
export async function stopLiveWallpaper(): Promise<void> {
  if (stopping.value) return;
  stopping.value = true;
  galleryError.value = null;
  try {
    settings.value = await stopLive();
    livePaused.value = false;
  } catch (err) {
    galleryError.value = errorMessage(err);
  } finally {
    stopping.value = false;
  }
}

// Settings actions -----------------------------------------------------------

export async function toggleRunOnStartup(enabled: boolean): Promise<void> {
  settingsError.value = null;
  try {
    settings.value = await setRunOnStartup(enabled);
  } catch (err) {
    settingsError.value = errorMessage(err);
  }
}

export async function togglePauseOnFullscreen(enabled: boolean): Promise<void> {
  settingsError.value = null;
  try {
    settings.value = await setPauseOnFullscreen(enabled);
  } catch (err) {
    settingsError.value = errorMessage(err);
  }
}

export async function toggleMuteLiveWallpapers(enabled: boolean): Promise<void> {
  settingsError.value = null;
  try {
    settings.value = await setMuteLiveWallpapers(enabled);
  } catch (err) {
    settingsError.value = errorMessage(err);
  }
}

/** Applies a random wallpaper from the library (Settings → Status). */
export async function applyRandomWallpaper(): Promise<void> {
  if (applying.value !== null) return;
  applying.value = "next";
  settingsError.value = null;
  try {
    settings.value = await nextWallpaper();
  } catch (err) {
    settingsError.value = errorMessage(err);
  } finally {
    applying.value = null;
  }
}

/** Reveals a path in Explorer; errors surface on the Settings page. */
export async function openFolder(path: string): Promise<void> {
  settingsError.value = null;
  try {
    await openInExplorer(path);
  } catch (err) {
    settingsError.value = errorMessage(err);
  }
}

/** Pauses/resumes the active live wallpaper; follows up via `livePaused`. */
export async function toggleLiveWallpaperPause(): Promise<void> {
  if (pausing.value) return;
  pausing.value = true;
  settingsError.value = null;
  try {
    livePaused.value = await toggleLivePause();
  } catch (err) {
    settingsError.value = errorMessage(err);
  } finally {
    pausing.value = false;
  }
}

export async function initThumbnailListener(): Promise<void> {
  await listen<ThumbnailBatch>("thumbnail-progress", (event) => {
    const next = { ...thumbStates.value };
    for (const item of event.payload.items) {
      next[item.path] = item.thumb
        ? { status: "done", url: item.thumb }
        : { status: "failed" };
    }
    thumbStates.value = next;
  });
}

/** Fullscreen auto-pause and the tray toggle share the `live-paused` event. */
export async function initLivePausedListener(): Promise<void> {
  await listen<boolean>("live-paused", (event) => {
    livePaused.value = event.payload;
  });
}

/** Tray actions that change settings (Next Wallpaper) push the new snapshot. */
export async function initSettingsListener(): Promise<void> {
  await listen<Settings>("settings-changed", (event) => {
    settings.value = event.payload;
  });
}

/** Tray page actions ask the UI to navigate before the window is shown. */
export async function initRouteListener(): Promise<void> {
  await listen<Route>("route", (event) => {
    route.value = event.payload;
  });
}

/**
 * Queues thumbnail generation for items without state. Paths are marked
 * pending up front; `resolve_thumbnails` returns the cached ones and the
 * worker drains the rest through progress events. Existing states are never
 * downgraded, so a late response cannot mask an already-arrived event.
 */
export async function queueThumbnails(
  items: { path: string }[],
  kind: ThumbnailKind,
): Promise<void> {
  const paths = items
    .filter((i) => thumbStates.value[i.path] === undefined)
    .map((i) => i.path);
  if (paths.length === 0) return;

  const marked = { ...thumbStates.value };
  for (const p of paths) marked[p] = { status: "pending" };
  thumbStates.value = marked;

  try {
    const result = await resolveThumbnails(paths, kind);
    const next = { ...thumbStates.value };
    for (const entry of result.ready) {
      if (entry.thumb !== null) {
        next[entry.path] = { status: "done", url: entry.thumb };
      }
    }
    thumbStates.value = next;
  } catch {
    const next = { ...thumbStates.value };
    for (const p of paths) {
      if (next[p]?.status === "pending") next[p] = { status: "failed" };
    }
    thumbStates.value = next;
  }
}
