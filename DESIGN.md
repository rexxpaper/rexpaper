---
version: alpha
name: RexPaper
description: Monochrome, flat design language for the RexPaper Windows wallpaper manager.
colors:
  bg: "#0a0a0a"
  panel: "#111111"
  card: "#171717"
  border: "#262626"
  text: "#fafafa"
  primary: "#fafafa"
  secondary: "#a3a3a3"
  muted: "#737373"
typography:
  sans:
    fontFamily: Segoe UI, ui-sans-serif, system-ui, sans-serif
---

## Overview

RexPaper is a wallpaper manager for Windows: a desktop utility that stays out of
the way. The interface is dark and strictly monochrome — a grayscale ramp from
near-black surfaces to near-white text, with no saturated accent color.
Interactive emphasis comes from contrast (light on dark, dark on light), not
hue. Surfaces are flat and regions are separated by hairline borders.

## Colors

The ramp is grayscale only. `{bg}` anchors the window; `{panel}` carries chrome
(titlebar, sidebar, menus); `{card}` lifts contained content (cards, rows,
hover fills); `{border}` draws every separator at 1px. Text descends
`{text}` → `{secondary}` → `{muted}` by importance.

Primary actions invert the ramp: fill with `{primary}`, label with `{bg}`.
Reserve this inversion for the single most important action in a view;
everything else is a bordered ghost button over `{panel}` or `{card}` hover
fills. Use `{muted}` only for placeholder and disabled content, never for
actionable labels.

## Typography

One family, Segoe UI with system fallbacks. Body copy is the base size;
`{secondary}` and `{muted}` carry the hierarchy alongside size, not weight
variations beyond medium/semibold for headings and active labels.

## Components

Containers are flat: `{card}` fill, 1px `{border}` edge, no shadow. Elevation
exists only for transient overlays (menus, dialogs, popovers) that must read
as floating above the layout.

Gallery cards are visual-only: the thumbnail fills the frame edge to edge
inside a 1px `{border}` edge — no filename captions under it, and no overlay
badges. The active wallpaper is marked by a high-contrast `{text}` border on
the card instead.

Lists and property rows are divided by hairline bottom borders, never by
alternating fills or gaps.

Controls (buttons, toggles, inputs, selects) share one silhouette: a small
uniform radius, a 1px `{border}` edge when unfilled, and a neutral fill on
hover. Selects stay native elements with the system arrow hidden
(`appearance-none`) and a hairline chevron drawn in — native keyboard
behavior, and the OS popup scrolls long option lists for free. Keyboard
focus is a visible 2px outline in `{text}` with offset — contrast
again, not color.

Interaction feedback is short and uniform: 150ms ease-out transitions on
`transform`, `opacity`, and small local color changes (text, border, hover
fills) — never on layout properties or large blur surfaces.

Empty states are centered `{muted}` text in the view with one primary action
to get started.

## Do's and Don'ts

- Do separate regions with flat fills and 1px hairline borders.
- Don't use gradients, neon, glow, or decorative filler.
- Don't introduce saturated color anywhere, including status and highlight
  states; communicate state with contrast, weight, and border.
