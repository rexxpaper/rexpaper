import type { RefObject } from "preact";

/** Case-insensitive match of a wallpaper's file name or category. */
export const matchesFilter = (
  path: string,
  category: string,
  query: string,
): boolean => {
  const q = query.trim().toLowerCase();
  if (q === "") return true;
  const name = path.split(/[\\/]/).pop()?.toLowerCase() ?? "";
  return name.includes(q) || category.toLowerCase().includes(q);
};

/** Cards sharing the first card's offsetTop form the first grid row. */
export const columnCount = (cards: HTMLElement[]): number => {
  if (cards.length === 0) return 1;
  const top = cards[0].offsetTop;
  let count = 0;
  while (count < cards.length && cards[count].offsetTop === top) count += 1;
  return Math.max(1, count);
};

/**
 * Arrow-key navigation for a card grid: left/right walk one card at a time,
 * up/down jump a row, focus stays clamped to the grid. Enter is native —
 * cards are buttons.
 */
export const createGridKeyDown =
  (gridRef: RefObject<HTMLDivElement | null>) =>
  (e: KeyboardEvent): void => {
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
