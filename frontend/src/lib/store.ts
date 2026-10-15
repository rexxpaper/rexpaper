import { computed, signal } from "@preact/signals";
import { listen } from "@tauri-apps/api/event";
import {
  applyStatic,
  getSettings,
  pickWallpaperDir,
  resolveThumbnails,
  scanLive,
  scanStatic,
  type LiveWallpaperItem,
  type Settings,
  type ThumbnailBatch,
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
export const applying = signal<string | null>(null);
export const galleryError = signal<string | null>(null);

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

/**
 * Queues thumbnail generation for items without state. Paths are marked
 * pending up front; `resolve_thumbnails` returns the cached ones and the
 * worker drains the rest through progress events. Existing states are never
 * downgraded, so a late response cannot mask an already-arrived event.
 */
export async function resolveStaticThumbnails(
  items: WallpaperItem[],
): Promise<void> {
  const paths = items
    .filter((i) => thumbStates.value[i.path] === undefined)
    .map((i) => i.path);
  if (paths.length === 0) return;

  const marked = { ...thumbStates.value };
  for (const p of paths) marked[p] = { status: "pending" };
  thumbStates.value = marked;

  try {
    const result = await resolveThumbnails(paths, "static");
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
