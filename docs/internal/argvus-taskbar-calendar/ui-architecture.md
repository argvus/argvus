# UI Architecture

`argvus-calendar` keeps GTK4 and Relm4 as the implementation toolkit while presenting the popup as a Argvus shell component.

The current UI strategy is:

- Relm4 owns state transitions for month/day selection.
- GTK widgets remain accessible and keyboard focusable.
- `gtk4-layer-shell` creates a transparent overlay. The Taskbar launcher converts immutable root click coordinates to desktop `--x`/`--y`; the common `argvus-taskbar-popup` crate (`crates/argvus-taskbar-popup`) selects the monitor and bar from `hyprctl` layer geometry, aligns the popup to the bar's right edge, and places it 4 pixels below a top bar or above a bottom bar. The calendar keeps its size estimate (398 × 520) and namespace `argvus-taskbar-calendar`.
- The common crate also owns layer-shell setup, the pointer-position fallback, exact popup bounds, focus-loss dismissal and the 400 ms same-click reopen guard. Its `PopupController` is shared by the calendar, ARGVUS Focus and ARGVUS Clock; each consumer supplies its namespace and popup size.
- CSS removes stock GTK chrome and applies the Argvus visual language: the GTK Applications Font, dark Taskbar-derived colors, thin borders, flat controls and compact spacing.
- Styles load from external CSS: `/etc/argvus/taskbar/calendar/style.css`, `/etc/argvus/taskbar/calendar/theme.css`, the cache theme at `$XDG_CACHE_HOME/argvus-taskbar-calendar/theme.css` (copied from the active theme build by every theme or accent change, and loaded only when its first line names the active theme), then runtime font CSS from `config.toml`. The settings screen has no theme picker: the calendar follows the ARGVUS theme.
- Calendar, agenda and event editing are states of one layer-shell surface. The event editor replaces the calendar view inside a `gtk::Stack`; it no longer opens a secondary Hyprland window for normal event creation/editing. The agenda/reminder feature is toggled from the UI and persisted in the user state directory (`~/.local/state/argvus/taskbar/calendar/events-enabled`).

Argvus references used for the default palette:

- background: `#111316`
- foreground: `#DFE5EA`
- muted foreground: `#B0BFCB`
- accent/decorate: `#3590bd`

## Toolkit Evaluation

Moving to `iced` or Slint is not justified for the current goal. The desired behavior depends on native GTK widgets, keyboard/accessibility semantics, and mature `gtk4-layer-shell` support. The visual issue was GTK's default presentation, not an architectural blocker.

`iced` or Slint could be reconsidered later if Argvus wants a broader shared shell-widget renderer across multiple official apps. That would be a product-level toolkit decision, not a calendar-specific fix.

Session GTK applications inherit the Control Center Applications Font via gtk-font-name. Calculator, calendar, time popups and theme/layout selectors must not pin a font family. Calendar appearance.font_family is retained for config compatibility but no longer exposed or applied; appearance.font_size remains a local sizing control. Terminal, panel, taskbar and pre-login greeter retain their separate font targets.

## Embedded Taskbar panels

The embedded Taskbar preview uses `taskbar-calendar-morphing.sock` in the ARGVUS runtime directory. Its session owns the legacy popup socket while connected and reuses the GTK calendar’s independent day and month reads, editor save operation and reminder settings. The snapshot includes selected-day event and reminder counts for the summary cards. The user unit continues scheduling reminders without a popup. The Calendar provides ICS actions and the same CLI status and sync reports.

The embedded panel launches the native GTK file chooser in an auxiliary process. The Taskbar releases keyboard focus while it is open and takes focus again on its response. The chooser returns a local file URI. Import and export use the same Rust operations as the CLI. Timed exports use the recorded IANA time zone when it can represent the original instant. Calendar Morphing presents a responsive TUI-inspired grid and agenda using Taskbar theme tokens and the projected element radius. Sticky uses square control corners unless the Control Center global corner setting requests rounding. In preview mode, hovering over the date opens the panel; clicking toggles it, and pointer-opened panels dismiss after five seconds outside the anchor and content.
