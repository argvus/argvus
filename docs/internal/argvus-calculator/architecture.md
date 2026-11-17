# Calculator

`argvus calculator` launches the private `argvus-calculator` GTK4 executable. The
master package installs it as `/usr/lib/argvus/commands/calculator` and installs
`argvus-calculator.desktop`. No additional package, daemon or layer surface is
needed. `main.rs` retains the application instance and presents an existing window
on activation. Closing and launching again constructs a fresh UI from saved state.

## Engines and profiles

- `engine.rs`: basic/scientific bounded f64 expression parser and 15-significant-digit
  output; explicit multiplication, right-associative powers, postfix percent.
- `programming.rs`: exact modular integers, 8/16/32/64-bit words, binary/octal/decimal/
  hexadecimal input and output. Default: signed decimal 64-bit. Division truncates
  toward zero; signed right shifts extend the sign. Arithmetic overflow wraps modulo
  the word width. Shift counts must be in `[0, bits)`. Programming percent is remainder;
  `^` is XOR, not exponentiation. No intermediate float conversions are used.
- `financial.rs`: simple/compound accumulation, end-of-period fixed payment,
  present/future value with periodic payments, and percentage change. Rates are
  percentages per period; counts and rates must use the same period. Zero rates
  use exact limiting formulas. f64 results are rejected if non-finite.
- `conversion.rs`: static SI factors and temperature offsets; categories own their
  unit lists so incompatible units cannot be selected. Data units distinguish bit,
  byte, SI multiples and binary multiples. Absolute temperatures below zero Kelvin
  are rejected. This profile has no network or currency dependency.

Expression input is bounded to 4096 bytes and 64 recursive parser calls. All engines
return the common syntax/domain/limit error enum. Financial fields accept numeric
expressions via the existing f64 parser. Empty/invalid visible fields are errors;
unused fields do not affect evaluation.

## UI and persistence

`ui.rs` builds GTK widgets through Relm4's GTK re-export. The settings, calculation
and history implementations live in `ui/`. Settings and profile selection are
single-click ListBox rows with native arrow/Enter navigation. A nonhomogeneous Stack
separates the calculator, settings, profiles and history. Profile changes keep
independent in-memory expression drafts. Required financial fields are shown only
for the selected operation. Other profile options remain in Settings, with a summary
on the calculator page. Plain `s` and the clickable footer open Settings. Scientific functions beginning
with `s` can be inserted with the scientific keypad or pasted into the entry. Escape navigates history/profiles to
Settings, Settings to the calculator, then clears the expression on the main page.

Successful evaluation replaces the editable expression and collapses selection at
its end; the result can be reused directly. Entry focus never selects the entire
result. Repeated Enter on an unchanged conversion result is a no-op. Reusing a
conversion history result swaps the unit direction so the displayed value is treated
as the previously produced target unit. Other history results restore their saved
profile/options and financial fields before inserting the result.

`preferences.rs` saves profile, radians, programming base/width/sign, financial
operation and conversion category/unit indices to
`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (HOME `.config` fallback).
The old `scientific` boolean migrates to Basic/Scientific if `profile` is absent.
Missing files default to Basic; invalid loaded indices and word widths normalize to
valid defaults. Preferences are saved immediately on each change. There is no
persistent geometry. Default/minimum sizes are Basic 370×420, Scientific/Financial
440×640, Programming 520×640 and Conversion 440×520. Mode changes briefly constrain
the surface, wait for allocation, then restore manual resizing. A generation counter
prevents stale resize callbacks from overriding later selections.

`history.rs` retains up to 100 records in
`$XDG_DATA_HOME/argvus/calculator/history.json` (HOME `.local/share` fallback).
Each new record contains the original expression, result and an optional context:
full Preferences plus three financial input strings. Older records without context
remain valid and default to Basic when reused. History is saved after successful
calculations; clearing also writes an empty saved list. The shared JSON writer uses
a private 0600 temporary file and atomic replacement. Failures appear as translated
entry-icon tooltips and technical stderr messages.

The existing `argvus-time-ui::theme::ThemeStyle` watches the `time` theme consumer.
Calculator CSS uses semantic colors, the GTK Applications Font, 18px expression text and 20px
numeric keys and 22px mathematical symbols (18px function labels). Expanding grids use available window space; all descendant focus
outlines are suppressed while selected settings rows retain their color highlight.

## Validation

Run `cargo +1.95.0 test -p argvus-calculator`, Clippy with `--all-targets -- -D warnings`,
format checking and the release build. The ignored GTK integration test additionally
needs a real display, `ARGVUS_I18N_DIR` pointed at the source catalogs, and a floating
Hyprland rule matching `org.argvus.CalculatorTest`; run with
`-- --include-ignored --test-threads=1`. It covers all profiles, default/minimum sizes,
resizable keys, inline results, history contexts and restoration after reopening.
Localized user pages are under each existing docs locale's
`components/argvus-calculator/` directory. Validate source and staged catalogs with
`argvus i18n validate` and validate the desktop entry with `desktop-file-validate`.

Session GTK applications inherit the Control Center Applications Font via gtk-font-name. Calculator, calendar, time popups and theme/layout selectors must not pin a font family. Calendar appearance.font_family is retained for config compatibility but no longer exposed or applied; appearance.font_size remains a local sizing control. Terminal, panel, taskbar and pre-login greeter retain their separate font targets.

The persistent shell has an accent ARGVUS badge and localized application heading above the page stack, matching the live monitor header hierarchy. A full-width time_surface footer band contains the clickable localized s Settings hint, with the key in the accent color. Both regions inherit the Applications Font and theme palette without hardcoded colors.
