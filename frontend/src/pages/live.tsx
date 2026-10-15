import { useEffect, useMemo, useRef } from "preact/hooks";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  GalleryCard,
  GalleryEmptyState,
  GalleryErrorBanner,
  GallerySkeletonGrid,
  GalleryToolbar,
  useEscapeClearsSearch,
  useGalleryScan,
} from "../components/gallery";
import { createGridKeyDown, matchesFilter } from "../lib/gallery";
import {
  applying,
  applyLiveWallpaper,
  chooseLiveWallpaperDir,
  galleryError,
  liveCategory,
  liveCategories,
  liveItems,
  liveSearch,
  queueThumbnails,
  scanError,
  scanning,
  settings,
  stopLiveWallpaper,
  stopping,
} from "../lib/store";

/**
 * Hover preview: muted, looping video streamed over the asset protocol's
 * range support. Always silent — the *applied* wallpaper honors the mute
 * setting instead.
 */
function startPreview(video: HTMLVideoElement | null): void {
  if (video === null) return;
  video.style.opacity = "1";
  const played = video.play();
  if (played !== undefined) played.catch(() => {});
}

function stopPreview(video: HTMLVideoElement | null): void {
  if (video === null) return;
  video.style.opacity = "0";
  video.pause();
  try {
    video.currentTime = 0;
  } catch {
    // Not seekable yet (never started); nothing to reset.
  }
}

export function LivePage() {
  const gridRef = useRef<HTMLDivElement>(null);
  const onGridKeyDown = useMemo(() => createGridKeyDown(gridRef), [gridRef]);
  /** Videos currently previewing — paused explicitly on unmount. */
  const previewing = useRef(new Set<HTMLVideoElement>());

  const currentSettings = settings.value;
  const items = liveItems.value;
  const categories = liveCategories.value;
  const category = liveCategory.value;
  const search = liveSearch.value;

  useGalleryScan(currentSettings?.live_wallpaper_dir ?? null, items.length > 0);
  useEscapeClearsSearch(liveSearch);

  // Queue video thumbnail generation whenever the item list changes.
  useEffect(() => {
    if (items.length > 0) void queueThumbnails(items, "live");
  }, [items]);

  // Leaving the page stops any hover preview.
  useEffect(
    () => () => {
      for (const video of previewing.current) {
        video.pause();
        video.style.opacity = "0";
      }
      previewing.current.clear();
    },
    [],
  );

  const filtered = useMemo(
    () =>
      items.filter(
        (item) =>
          (category === "All" || item.category === category) &&
          matchesFilter(item.path, item.category, search),
      ),
    [items, category, search],
  );

  const error = galleryError.value ?? scanError.value;

  if (currentSettings === null) {
    return (
      <div class="flex h-full items-center justify-center text-sm text-muted">
        Loading&hellip;
      </div>
    );
  }

  if (currentSettings.live_wallpaper_dir === null) {
    return (
      <GalleryEmptyState
        title="No video folder selected"
        hint="Pick a folder of video files (mp4, webm, mkv, …). Subfolders become categories you can filter by."
        actionLabel="Choose folder"
        onAction={() => void chooseLiveWallpaperDir()}
      />
    );
  }

  if (scanning.value && items.length === 0) {
    return <GallerySkeletonGrid />;
  }

  if (items.length === 0) {
    return (
      <div>
        {error !== null && <GalleryErrorBanner message={error} />}
        <GalleryEmptyState
          title="No videos found"
          hint={`RexPaper found no video files in ${currentSettings.live_wallpaper_dir}.`}
          actionLabel="Choose another folder"
          onAction={() => void chooseLiveWallpaperDir()}
        />
      </div>
    );
  }

  const isLiveActive =
    currentSettings.wallpaper_mode === "live" &&
    currentSettings.active_live_wallpaper !== null;

  return (
    <div class="flex h-full min-h-0 flex-col">
      <GalleryToolbar
        count={filtered.length}
        total={items.length}
        search={liveSearch}
        categories={categories}
        category={liveCategory}
        placeholder="Search videos…"
        ariaLabel="Search videos"
      >
        {isLiveActive && (
          <button
            type="button"
            onClick={() => void stopLiveWallpaper()}
            disabled={stopping.value}
            class="flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm text-text transition-colors duration-150 ease-out hover:bg-card disabled:opacity-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
          >
            <svg
              width="9"
              height="9"
              viewBox="0 0 9 9"
              aria-hidden="true"
              class="shrink-0"
            >
              <rect width="9" height="9" rx="1" fill="currentColor" />
            </svg>
            {stopping.value ? "Stopping…" : "Stop live"}
          </button>
        )}
      </GalleryToolbar>

      {error !== null && <GalleryErrorBanner message={error} />}

      {filtered.length === 0 ? (
        <GalleryEmptyState
          title={`No matches for “${search.trim() || category}”`}
          hint="No video in this view matches the current search and category."
          actionLabel="Clear filters"
          onAction={() => {
            liveSearch.value = "";
            liveCategory.value = "All";
          }}
        />
      ) : (
        <div
          ref={gridRef}
          onKeyDown={onGridKeyDown}
          class="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-4 p-6"
        >
          {filtered.map((item) => {
            const isActive =
              currentSettings.wallpaper_mode === "live" &&
              currentSettings.active_live_wallpaper === item.path;
            return (
              <GalleryCard
                key={item.path}
                path={item.path}
                isActive={isActive}
                isApplying={applying.value === item.path}
                onClick={() => void applyLiveWallpaper(item.path)}
                media={
                  <div
                    class="absolute inset-0"
                    onMouseEnter={(e) => {
                      const video = e.currentTarget.querySelector("video");
                      if (video !== null) previewing.current.add(video);
                      startPreview(video);
                    }}
                    onMouseLeave={(e) => {
                      const video = e.currentTarget.querySelector("video");
                      if (video !== null) previewing.current.delete(video);
                      stopPreview(video);
                    }}
                  >
                    <video
                      src={convertFileSrc(item.path)}
                      muted
                      playsinline
                      loop
                      preload="none"
                      aria-hidden="true"
                      class="pointer-events-none absolute inset-0 size-full object-cover opacity-0 transition-opacity duration-150 ease-out"
                    />
                  </div>
                }
              />
            );
          })}
        </div>
      )}
    </div>
  );
}
