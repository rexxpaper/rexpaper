import { useEffect, useMemo, useRef } from "preact/hooks";
import { convertFileSrc } from "@tauri-apps/api/core";
import { cn } from "../lib/cn";
import {
  applying,
  applyStaticWallpaper,
  chooseWallpaperDir,
  galleryError,
  resolveStaticThumbnails,
  runScan,
  scanError,
  scanning,
  settings,
  staticCategory,
  staticCategories,
  staticItems,
  staticSearch,
  thumbPendingCount,
  thumbStates,
} from "../lib/store";

const fileName = (path: string): string =>
  path.split(/[\\/]/).pop() ?? path;

const matchesFilter = (path: string, category: string, query: string): boolean => {
  const q = query.trim().toLowerCase();
  if (q === "") return true;
  const name = path.split(/[\\/]/).pop()?.toLowerCase() ?? "";
  return name.includes(q) || category.toLowerCase().includes(q);
};

/** Cards sharing the first card's offsetTop form the first grid row. */
const columnCount = (cards: HTMLElement[]): number => {
  if (cards.length === 0) return 1;
  const top = cards[0].offsetTop;
  let count = 0;
  while (count < cards.length && cards[count].offsetTop === top) count += 1;
  return Math.max(1, count);
};

function ErrorBanner({ message }: { message: string }) {
  return (
    <p
      role="alert"
      class="mx-6 mt-4 rounded-md border border-border bg-card px-3 py-2 text-sm text-text text-pretty"
    >
      {message}
    </p>
  );
}

function EmptyState({
  title,
  hint,
  actionLabel,
  onAction,
}: {
  title: string;
  hint: string;
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <div class="flex flex-col items-center justify-center gap-3 px-6 py-24 text-center">
      <p class="text-sm text-muted text-balance">{title}</p>
      <p class="max-w-md text-sm text-muted text-pretty">{hint}</p>
      <button
        type="button"
        onClick={onAction}
        class="mt-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-bg transition-colors hover:bg-primary/90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        {actionLabel}
      </button>
    </div>
  );
}

function SkeletonGrid() {
  return (
    <div class="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-4 p-6">
      {Array.from({ length: 12 }, (_, i) => (
        <div
          // eslint-disable-next-line react/no-array-index-key
          key={i}
          class="overflow-hidden rounded-lg border border-border bg-card"
        >
          <div class="aspect-video bg-panel" />
          <div class="h-7 border-t border-border" />
        </div>
      ))}
    </div>
  );
}

export function StaticPage() {
  const gridRef = useRef<HTMLDivElement>(null);
  const scannedFor = useRef<string | null>(null);

  const currentSettings = settings.value;
  const items = staticItems.value;
  const categories = staticCategories.value;
  const category = staticCategory.value;
  const search = staticSearch.value;

  const filtered = useMemo(
    () =>
      items.filter(
        (item) =>
          (category === "All" || item.category === category) &&
          matchesFilter(item.path, item.category, search),
      ),
    [items, category, search],
  );

  // Scan once per configured directory (first mount or after a folder change).
  useEffect(() => {
    const dir = currentSettings?.wallpaper_dir ?? null;
    if (dir !== null && scannedFor.current !== dir && !scanning.value) {
      scannedFor.current = dir;
      void runScan();
    }
  }, [currentSettings?.wallpaper_dir, scanning.value, items.length]);

  // Queue thumbnail generation whenever the item list changes.
  useEffect(() => {
    if (items.length > 0) void resolveStaticThumbnails(items);
  }, [items]);

  // Escape clears the active search filter.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && staticSearch.value !== "") {
        staticSearch.value = "";
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  const onGridKeyDown = (e: KeyboardEvent): void => {
    if (!["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp"].includes(e.key)) {
      return;
    }
    const target = e.target as HTMLElement | null;
    if (target === null || !target.matches("[data-card]")) return;
    e.preventDefault();

    const cards = Array.from(
      gridRef.current?.querySelectorAll<HTMLElement>("[data-card]") ?? [],
    );
    const index = cards.indexOf(target);
    if (index < 0) return;

    let next = index;
    if (e.key === "ArrowRight") next = index + 1;
    if (e.key === "ArrowLeft") next = index - 1;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      const columns = columnCount(cards);
      next = e.key === "ArrowDown" ? index + columns : index - columns;
    }
    cards[Math.max(0, Math.min(cards.length - 1, next))]?.focus();
  };

  const error = galleryError.value ?? scanError.value;

  if (currentSettings === null) {
    return (
      <div class="flex h-full items-center justify-center text-sm text-muted">
        Loading&hellip;
      </div>
    );
  }

  if (currentSettings.wallpaper_dir === null) {
    return (
      <EmptyState
        title="No wallpaper folder selected"
        hint="Pick a folder to build your library. Subfolders become categories you can filter by."
        actionLabel="Choose folder"
        onAction={() => void chooseWallpaperDir()}
      />
    );
  }

  if (scanning.value && items.length === 0) {
    return <SkeletonGrid />;
  }

  if (items.length === 0) {
    return (
      <div>
        {error !== null && <ErrorBanner message={error} />}
        <EmptyState
          title="No wallpapers found"
          hint={`RexPaper found no images in ${currentSettings.wallpaper_dir}.`}
          actionLabel="Choose another folder"
          onAction={() => void chooseWallpaperDir()}
        />
      </div>
    );
  }

  const pending = thumbPendingCount.value;

  return (
    <div class="flex h-full min-h-0 flex-col">
      <div class="shrink-0 border-b border-border bg-bg">
        <div class="flex items-center gap-3 px-6 py-3">
          <span class="text-xs text-muted tabular-nums">
            {filtered.length} of {items.length}
          </span>
          <div class="flex-1" />
          {pending > 0 && (
            <span class="text-xs text-muted tabular-nums">
              Generating thumbnails &mdash; {pending} left
            </span>
          )}
          {applying.value !== null && (
            <span class="text-xs text-muted">Applying&hellip;</span>
          )}
          <div class="relative">
            <input
              type="text"
              value={search}
              onInput={(e) => {
                staticSearch.value = (e.currentTarget as HTMLInputElement).value;
              }}
              placeholder="Search wallpapers…"
              aria-label="Search wallpapers"
              class="h-8 w-56 rounded-md border border-border bg-card px-2.5 pr-8 text-sm text-text placeholder:text-muted focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary"
            />
            {search !== "" && (
              <button
                type="button"
                aria-label="Clear search"
                onClick={() => {
                  staticSearch.value = "";
                }}
                class="absolute inset-y-0 right-0 flex w-8 items-center justify-center text-muted transition-colors hover:text-text"
              >
                <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                  <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.5" />
                </svg>
              </button>
            )}
          </div>
        </div>
        {categories.length > 1 && (
          <div class="flex gap-1.5 overflow-x-auto px-6 pb-3">
            {categories.map((cat) => (
              <button
                key={cat}
                type="button"
                aria-pressed={category === cat}
                onClick={() => {
                  staticCategory.value = cat;
                }}
                class={cn(
                  "shrink-0 rounded-md border border-border px-2.5 py-1 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
                  category === cat
                    ? "bg-card font-medium text-text"
                    : "text-secondary hover:bg-card hover:text-text",
                )}
              >
                {cat}
              </button>
            ))}
          </div>
        )}
      </div>

      {error !== null && <ErrorBanner message={error} />}

      {filtered.length === 0 ? (
        <EmptyState
          title={`No matches for “${search.trim() || category}”`}
          hint="No wallpaper in this view matches the current search and category."
          actionLabel="Clear filters"
          onAction={() => {
            staticSearch.value = "";
            staticCategory.value = "All";
          }}
        />
      ) : (
        <div
          ref={gridRef}
          onKeyDown={onGridKeyDown}
          class="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-4 p-6"
        >
          {filtered.map((item) => {
            const state = thumbStates.value[item.path];
            const isActive =
              currentSettings.active_static_wallpaper === item.path;
            const isApplying = applying.value === item.path;
            return (
              <button
                key={item.path}
                type="button"
                data-card
                aria-current={isActive ? "true" : undefined}
                title={item.path}
                onClick={() => void applyStaticWallpaper(item.path)}
                class={cn(
                  "group relative flex flex-col overflow-hidden rounded-lg border bg-card text-left transition-colors focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
                  isActive ? "border-text" : "border-border hover:border-secondary",
                )}
              >
                <div class="relative aspect-video bg-panel">
                  {state?.status === "done" ? (
                    <img
                      src={convertFileSrc(state.url)}
                      alt=""
                      loading="lazy"
                      decoding="async"
                      class="size-full object-cover"
                    />
                  ) : state?.status === "failed" ? (
                    <div class="flex size-full items-center justify-center">
                      <span class="text-xs text-muted">No preview</span>
                    </div>
                  ) : null}
                  {isActive && (
                    <span class="absolute right-1.5 bottom-1.5 rounded bg-bg/85 px-1.5 py-0.5 text-[10px] font-medium text-text">
                      Active
                    </span>
                  )}
                  {isApplying && (
                    <span class="absolute inset-x-0 bottom-0 bg-bg/85 px-2 py-1 text-center text-[10px] text-text">
                      Applying&hellip;
                    </span>
                  )}
                </div>
                <div class="border-t border-border px-2.5 py-1.5">
                  <span class="block truncate text-xs text-secondary">
                    {fileName(item.path)}
                  </span>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
