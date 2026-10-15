import type { ComponentChildren } from "preact";
import type { Signal } from "@preact/signals";
import { useEffect, useMemo, useRef } from "preact/hooks";
import { convertFileSrc } from "@tauri-apps/api/core";
import { cn } from "../lib/cn";
import {
  applying,
  runScan,
  scanning,
  thumbPendingCount,
  thumbStates,
} from "../lib/store";

// --- Hooks ------------------------------------------------------------------

/**
 * Scans once per configured directory (first visit or after a folder
 * change); skips the walk when the item list is already populated.
 */
export function useGalleryScan(dir: string | null, hasItems: boolean): void {
  const scannedFor = useRef<string | null>(null);
  useEffect(() => {
    if (
      dir !== null &&
      !hasItems &&
      scannedFor.current !== dir &&
      !scanning.value
    ) {
      scannedFor.current = dir;
      void runScan();
    }
  }, [dir, hasItems, scanning.value]);
}

/** Escape clears the given search filter while its page is mounted. */
export function useEscapeClearsSearch(search: Signal<string>): void {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && search.value !== "") {
        search.value = "";
      }
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [search]);
}

// --- Toolbar ----------------------------------------------------------------

/**
 * Shared gallery toolbar: counts, subfolder dropdown, generation/applying
 * status, optional extra controls (before the status), and search field.
 */
export function GalleryToolbar({
  count,
  total,
  search,
  categories,
  category,
  placeholder,
  ariaLabel,
  children,
}: {
  count: number;
  total: number;
  search: Signal<string>;
  categories: string[];
  category: Signal<string>;
  placeholder: string;
  ariaLabel: string;
  children?: ComponentChildren;
}) {
  const searchValue = search.value;
  const activeCategory = category.value;
  const pending = thumbPendingCount.value;

  return (
    <div class="shrink-0 border-b border-border bg-bg">
      <div class="flex items-center gap-3 px-6 py-3">
        <span class="text-xs text-muted tabular-nums">
          {count} of {total}
        </span>
        {categories.length > 1 && (
          <div class="relative">
            <select
              aria-label="Subfolder"
              value={activeCategory}
              onChange={(e) => {
                category.value = (e.currentTarget as HTMLSelectElement).value;
              }}
              class="h-8 appearance-none rounded-md border border-border bg-card py-1 pl-2.5 pr-7 text-sm text-text transition-colors duration-150 ease-out hover:border-secondary focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary"
            >
              {categories.map((cat) => (
                <option key={cat} value={cat}>
                  {cat === "All" ? "All subfolders" : cat}
                </option>
              ))}
            </select>
            <svg
              class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-muted"
              width="9"
              height="5"
              viewBox="0 0 9 5"
              aria-hidden="true"
            >
              <path
                d="M0.5 0.5l4 4 4-4"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
              />
            </svg>
          </div>
        )}
        <div class="flex-1" />
        {children}
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
            value={searchValue}
            onInput={(e) => {
              search.value = (e.currentTarget as HTMLInputElement).value;
            }}
            placeholder={placeholder}
            aria-label={ariaLabel}
            class="h-8 w-56 rounded-md border border-border bg-card px-2.5 pr-8 text-sm text-text placeholder:text-muted focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary"
          />
          {searchValue !== "" && (
            <button
              type="button"
              aria-label="Clear search"
              onClick={() => {
                search.value = "";
              }}
              class="absolute inset-y-0 right-0 flex w-8 items-center justify-center text-muted transition-colors duration-150 ease-out hover:text-text"
            >
              <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
                <path
                  d="M1 1l8 8M9 1l-8 8"
                  stroke="currentColor"
                  stroke-width="1.5"
                />
              </svg>
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// --- Grid card --------------------------------------------------------------

function GalleryThumb({ path }: { path: string }) {
  const state = thumbStates.value[path];
  if (state?.status === "done") {
    return (
      <img
        src={convertFileSrc(state.url)}
        alt=""
        loading="lazy"
        decoding="async"
        ref={(el) => {
          if (el != null && el.complete && el.naturalWidth > 0) {
            el.style.opacity = "1";
          }
        }}
        onLoad={(e) => {
          e.currentTarget.style.opacity = "1";
        }}
        class="size-full object-cover opacity-0 transition-opacity duration-150 ease-out"
      />
    );
  }
  if (state?.status === "failed") {
    return (
      <div class="flex size-full items-center justify-center">
        <span class="text-xs text-muted">No preview</span>
      </div>
    );
  }
  return null;
}

/**
 * Visual-only gallery tile: full-bleed thumbnail in a hairline frame with a
 * subtle hover lift. `media` renders above the thumbnail (hover video
 * preview); state badges overlay both.
 */
export function GalleryCard({
  path,
  isActive,
  isApplying,
  onClick,
  media,
}: {
  path: string;
  isActive: boolean;
  isApplying: boolean;
  onClick: () => void;
  media?: ComponentChildren;
}) {
  return (
    <button
      type="button"
      data-card
      aria-current={isActive ? "true" : undefined}
      title={path}
      onClick={onClick}
      class={cn(
        "group relative overflow-hidden rounded-lg border bg-card text-left transition duration-150 ease-out hover:-translate-y-0.5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
        isActive ? "border-text" : "border-border hover:border-secondary",
      )}
    >
      <div class="relative aspect-video bg-panel">
        <GalleryThumb path={path} />
        {media}
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
    </button>
  );
}

// --- States -----------------------------------------------------------------

/** Static structural placeholder grid shown while a scan runs. */
export function GallerySkeletonGrid() {
  const slots = useMemo(() => Array.from({ length: 12 }, (_, i) => i), []);
  return (
    <div class="grid grid-cols-[repeat(auto-fill,minmax(176px,1fr))] gap-4 p-6">
      {slots.map((i) => (
        <div
          key={i}
          class="overflow-hidden rounded-lg border border-border bg-card"
        >
          <div class="aspect-video bg-panel" />
        </div>
      ))}
    </div>
  );
}

/** Centered muted empty state with exactly one primary action. */
export function GalleryEmptyState({
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
        class="mt-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-bg transition-colors duration-150 ease-out hover:bg-primary/90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        {actionLabel}
      </button>
    </div>
  );
}

/** Flat inline error callout, shown next to the action that failed. */
export function GalleryErrorBanner({ message }: { message: string }) {
  return (
    <p
      role="alert"
      class="mx-6 mt-4 rounded-md border border-border bg-card px-3 py-2 text-sm text-text text-pretty"
    >
      {message}
    </p>
  );
}
