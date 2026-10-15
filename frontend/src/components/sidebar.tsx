import { route, setRoute, type Route } from "../lib/store";
import { cn } from "../lib/cn";

const items: { id: Route; label: string }[] = [
  { id: "static", label: "Static Wallpapers" },
  { id: "live", label: "Live Wallpapers" },
  { id: "settings", label: "Settings" },
];

export function Sidebar() {
  return (
    <nav class="flex w-48 shrink-0 flex-col gap-1 border-r border-border bg-panel p-3">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          onClick={() => setRoute(item.id)}
          class={cn(
            "rounded-md px-3 py-2 text-left text-sm transition-colors",
            item.id === "settings" && "mt-auto",
            route.value === item.id
              ? "bg-card font-medium text-text"
              : "text-secondary hover:bg-card hover:text-text",
          )}
        >
          {item.label}
        </button>
      ))}
    </nav>
  );
}
