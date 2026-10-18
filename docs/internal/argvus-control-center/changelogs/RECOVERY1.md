# Recovery 1 — UX/UI and navigation

This recovery pass does not claim completion of the functional phases. It
stabilizes the shell used by the already-present domain crates.

`argvus-tui::page` owns the shared Control Center page shell and selectable
list primitive. Domain crates provide breadcrumb text and rows; they do not
draw a competing full-screen block. The shell uses the existing semantic
`Theme`, `argvus-i18n`, and `chrome` header/footer implementation.

Domain handlers return `true` only when Back is requested from their home
page. The application router consumes that result and changes from the
domain route to global Home. `Left` is normalized to the same Back action for
domain pages. Esc remains layer-local for settings editors, confirmations and
search fields.

Selection is normalized against the current row count after navigation and
background refresh. Home, End, PageUp and PageDown are supported by the shared
selection primitive; domain pages retain their existing action-specific keys.

The recovery tests use `TestBackend` and key-event simulation for domain
backtracking, Boot home selection, and shared chrome markers. They do not
execute system actions or alter system configuration. Functional limitations
documented by PHASE2–PHASE5 remain limitations; this pass changes their
presentation and routing only.
