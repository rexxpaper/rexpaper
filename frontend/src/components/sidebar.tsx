import type { JSX } from "preact";
import { route, setRoute, type Route } from "../lib/store";
import { cn } from "../lib/cn";

/** Image glyph: framed photo with a sun and a mountain ridge. */
function StaticGlyph() {
  return (
    <>
      <rect x="2" y="3" width="12" height="10" rx="1.5" />
      <circle cx="5.5" cy="6.4" r="1.1" />
      <path d="M2.6 11.4 5.8 8.2l2.1 2.1 3-3.1 2.5 2.6" />
    </>
  );
}

/** Play glyph: a media button, for live (video) wallpapers. */
function LiveGlyph() {
  return (
    <>
      <circle cx="8" cy="8" r="6" />
      <path d="M6.7 5.9 10.4 8l-3.7 2.1Z" />
    </>
  );
}

/** Sliders glyph: two tracks with knobs, for settings/preferences. */
function SettingsGlyph() {
  return (
    <>
      <path d="M2.5 5.5h11" />
      <circle cx="10.5" cy="5.5" r="1.6" fill="currentColor" stroke="none" />
      <path d="M2.5 10.5h11" />
      <circle cx="5.5" cy="10.5" r="1.6" fill="currentColor" stroke="none" />
    </>
  );
}

const items: { id: Route; label: string; Glyph: () => JSX.Element }[] = [
  { id: "static", label: "Static Wallpapers", Glyph: StaticGlyph },
  { id: "live", label: "Live Wallpapers", Glyph: LiveGlyph },
  { id: "settings", label: "Settings", Glyph: SettingsGlyph },
];

export function Sidebar() {
  return (
    <nav class="flex w-48 shrink-0 flex-col gap-1 border-r border-border bg-panel p-3">
      {items.map(({ id, label, Glyph }) => {
        const active = route.value === id;
        return (
          <button
            key={id}
            type="button"
            onClick={() => setRoute(id)}
            aria-current={active ? "page" : undefined}
            class={cn(
              "flex items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm transition-colors",
              id === "settings" && "mt-auto",
              active
                ? "bg-card font-medium text-text"
                : "text-secondary hover:bg-card hover:text-text",
            )}
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
              class="shrink-0"
            >
              <Glyph />
            </svg>
            {label}
          </button>
        );
      })}
    </nav>
  );
}
