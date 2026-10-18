# UI Architecture

`argvus-calendar` keeps GTK4 and Relm4 as the implementation toolkit while presenting the popup as a Argvus shell component.

The current UI strategy is:

- Relm4 owns state transitions for month/day selection.
- GTK widgets remain accessible and keyboard focusable.
- `gtk4-layer-shell` creates a transparent overlay; Argvus Waybar supplies immutable button-event `{x_root}`/`{y_root}` coordinates relative to its surface and the launcher adds the stable layer origin before supplying desktop `--x`/`--y` coordinates. The popup opens at that X coordinate and 12 pixels below the click.
- CSS removes stock GTK chrome and applies the Argvus visual language: IBM Plex Mono typography, dark Waybar-derived colors, thin borders, flat controls and compact spacing.
- Styles load from external CSS: `/etc/argvus/taskbar/calendar/style.css`, `/etc/argvus/taskbar/calendar/theme.css`, the active ARGVUS theme in `/etc/argvus/taskbar/calendar/themes/`, the generated cache theme at `$XDG_CACHE_HOME/argvus-taskbar-calendar/theme.css`, then runtime font CSS from `config.toml`. The appearance switcher writes the cache as a standalone stylesheet so Float themes do not depend on relative imports.
- Calendar, agenda and event editing are states of one layer-shell surface. The event editor replaces the calendar view inside a `gtk::Stack`; it no longer opens a secondary Hyprland window for normal event creation/editing. The agenda/reminder feature is toggled from the UI and persisted in the user state directory (`~/.local/state/argvus/taskbar/calendar/events-enabled`).

Argvus references used for the default palette:

- background: `#111316`
- foreground: `#DFE5EA`
- muted foreground: `#B0BFCB`
- accent/decorate: `#3590bd`

## Toolkit Evaluation

Moving to `iced` or Slint is not justified for the current goal. The desired behavior depends on native GTK widgets, keyboard/accessibility semantics, and mature `gtk4-layer-shell` support. The visual issue was GTK's default presentation, not an architectural blocker.

`iced` or Slint could be reconsidered later if Argvus wants a broader shared shell-widget renderer across multiple official apps. That would be a product-level toolkit decision, not a calendar-specific fix.
