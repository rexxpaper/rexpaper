import { useEffect, useState } from "preact/hooks";
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = () => getCurrentWindow();

const captionBase =
  "flex h-8 w-[38px] items-center justify-center rounded-md text-secondary transition-colors duration-150 ease-out hover:text-text focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary";

function Titlebar() {
  const [maximized, setMaximized] = useState(false);

  // Track maximize state for the restore/maximize glyph.
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    void win()
      .isMaximized()
      .then((m) => {
        if (active) setMaximized(m);
      });
    void win()
      .onResized(async () => {
        if (active) setMaximized(await win().isMaximized());
      })
      .then((f) => {
        if (active) unlisten = f;
        else f();
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return (
    <header
      data-tauri-drag-region
      class="relative z-40 flex h-10 shrink-0 select-none items-stretch border-b border-border bg-panel"
    >
      <div
        data-tauri-drag-region
        class="flex items-center pl-4 text-sm font-semibold tracking-tight text-text"
      >
        RexPaper
      </div>

      <div
        data-tauri-drag-region
        class="flex-1"
        onDblClick={() => void win().toggleMaximize()}
      />

      <div class="flex h-full items-center gap-1 pr-2">
        <button
          type="button"
          aria-label="Minimize"
          onClick={() => void win().minimize()}
          class={`${captionBase} hover:bg-card`}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path
              d="M1 5h8"
              stroke="currentColor"
              stroke-width="1"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <button
          type="button"
          aria-label={maximized ? "Restore" : "Maximize"}
          onClick={() => void win().toggleMaximize()}
          class={`${captionBase} hover:bg-card`}
        >
          {maximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <rect
                x="0.5"
                y="3.5"
                width="6"
                height="6"
                rx="1"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
              />
              <path
                d="M3.5 3.5v-3h6v6h-3"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
                stroke-linecap="round"
              />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <rect
                x="0.5"
                y="0.5"
                width="9"
                height="9"
                rx="1"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
              />
            </svg>
          )}
        </button>
        <button
          type="button"
          aria-label="Close"
          onClick={() => void win().close()}
          class={`${captionBase} hover:bg-[#e81123]`}
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <path
              d="M1.5 1.5l7 7M8.5 1.5l-7 7"
              stroke="currentColor"
              stroke-width="1"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </header>
  );
}

export default Titlebar;
