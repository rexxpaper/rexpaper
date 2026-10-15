import { useEffect, useMemo, useRef } from "preact/hooks";
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
  applyStaticWallpaper,
  chooseWallpaperDir,
  galleryError,
  queueThumbnails,
  scanError,
  scanning,
  settings,
  staticCategory,
  staticCategories,
  staticItems,
  staticSearch,
} from "../lib/store";

export function StaticPage() {
  const gridRef = useRef<HTMLDivElement>(null);
  const onGridKeyDown = useMemo(() => createGridKeyDown(gridRef), [gridRef]);

  const currentSettings = settings.value;
  const items = staticItems.value;
  const categories = staticCategories.value;
  const category = staticCategory.value;
  const search = staticSearch.value;

  useGalleryScan(currentSettings?.wallpaper_dir ?? null, items.length > 0);
  useEscapeClearsSearch(staticSearch);

  // Queue thumbnail generation whenever the item list changes.
  useEffect(() => {
    if (items.length > 0) void queueThumbnails(items, "static");
  }, [items]);

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

  if (currentSettings.wallpaper_dir === null) {
    return (
      <GalleryEmptyState
        title="No wallpaper folder selected"
        hint="Pick a folder to build your library. Subfolders become categories you can filter by."
        actionLabel="Choose folder"
        onAction={() => void chooseWallpaperDir()}
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
          title="No wallpapers found"
          hint={`RexPaper found no images in ${currentSettings.wallpaper_dir}.`}
          actionLabel="Choose another folder"
          onAction={() => void chooseWallpaperDir()}
        />
      </div>
    );
  }

  return (
    <div class="flex min-h-full flex-col">
      <GalleryToolbar
        count={filtered.length}
        total={items.length}
        search={staticSearch}
        categories={categories}
        category={staticCategory}
        placeholder="Search wallpapers…"
        ariaLabel="Search wallpapers"
      />

      {error !== null && <GalleryErrorBanner message={error} />}

      {filtered.length === 0 ? (
        <GalleryEmptyState
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
          {filtered.map((item) => (
            <GalleryCard
              key={item.path}
              path={item.path}
              isActive={currentSettings.active_static_wallpaper === item.path}
              onClick={() => void applyStaticWallpaper(item.path)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
