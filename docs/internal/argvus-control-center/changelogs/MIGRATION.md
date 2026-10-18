# Migration map

## Existing applications

`argvus-settings-term` supplied the route stack, list selection and scrolling,
search mode, default-application backend, fontconfig discovery, font persistence,
feedback popups, and the ARGVUS settings breadcrumbs.

`argvus-about-term` supplied the five-tab document interface, cached system
information, link actions, per-document scrolling, scrollbar, responsive system
layout, and the raster/graphics/half-block/ASCII ARGVUS logo renderer.

## Shared code

- `argvus-theme`: global ARGVUS CSS palette loading, imports, semantic resolution,
  and centralized fallback.
- `argvus-i18n`: locale detection and Portuguese/English selection.
- `argvus-tui`: terminal lifecycle, panic restoration, common header, footer,
  minimum-size view, and contextual help popup.
- `argvus-control-center`: top-level routes, global event routing, Home, and CLI.

## Domain code

- `argvus-control-center-settings`: default apps, fonts, list/search UI, and the
  existing ARGVUS persistence backends.
- `argvus-control-center-about`: About tabs, documents, system probes, links,
  scroll state, and logo rendering.

The migration intentionally leaves both source projects intact. The new workspace
contains one terminal/event loop and one loaded theme/language context, while each
domain retains its specialized body layout and state.

