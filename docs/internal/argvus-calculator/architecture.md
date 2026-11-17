# Calculator

The `argvus-calculator` workspace crate provides the private GTK4 executable reached through `argvus calculator`. The main package builds and installs it at `/usr/lib/argvus/commands/calculator`; its desktop entry uses the public command.

`engine.rs` owns bounded parsing, evaluation and formatting without GTK or external execution. Expressions use explicit multiplication, right-associative exponentiation, postfix percent and f64 values. Limits are 4096 bytes and 64 recursive parser calls. Domain failures and non-finite results are rejected.

`main.rs` owns GTK application lifetime, keyboard editing, separate calculator/menu/history stack pages, mode visibility and a persisted history limited to 100 results. GTK widgets are accessed through Relm4's GTK re-export. There is no daemon or layer-shell surface. The `calculator` translation domain uses the shared i18n API and existing locales.

The existing `argvus-time-ui::theme::ThemeStyle` loads and watches the `time` consumer; calculator CSS uses its semantic colors with compact flat controls and IBM Plex Mono. No duplicate theme state or new theme registry entry is needed.

Validation: `cargo +1.95.0 test -p argvus-calculator`, `cargo +1.95.0 clippy -p argvus-calculator --all-targets -- -D warnings`, `cargo +1.95.0 build --release -p argvus-calculator`, and the CLI dispatcher integration tests. Documentation lives in the existing locale directories under `components/argvus-calculator`.

The stack is nonhomogeneous so hidden scientific controls and history do not constrain the calculator page. Mode resizing runs after GTK layout updates. The window opens at 370 × 420 (basic), with 440 × 640 as the scientific default. Mode changes briefly constrain the surface until the compositor applies the new allocation, then restore manual resizing. The expanding keypad fills available height; sizes are not persisted. Secondary pages use a top back-arrow with a translated accessible label. Menu contains mode/angle controls, copy and history navigation; Escape navigates back before clearing an expression. `SUPER + C` dispatches directly to `argvus calculator`; the launcher no longer requires rofi-calc.

The calculator uses a keyboard-first footer (`s Settings`, localized) instead of a hamburger button. Clicking the footer or pressing plain `s` on the calculator page opens the settings page; Escape or the back arrow returns. The shortcut is captured before the expression editor consumes the key.

Mode defaults are also the GTK minimum-size requests (370 × 420 basic, 440 × 640 scientific), including secondary pages. Numeric keys use a dedicated CSS class with 16px text.

Successful evaluation replaces the expression in the editable entry and positions the caret at the end; there is no separate result label. Copy reads this display. Errors preserve the expression and expose a translated error tooltip through the entry icon; editing clears the error. History retains the original expression and result.

Entry focus uses `grab_focus_without_selecting`; evaluation and history reuse explicitly collapse selection and place the caret after the result. This allows continued arithmetic without replacing the result.

History is JSON in `$XDG_DATA_HOME/argvus/calculator/history.json`, with the HOME `.local/share` fallback. Writes use a private 0600 temporary file and atomic rename after every calculation and clear action. Load/save failures appear on the entry icon and stderr. Settings use GTK ListBox rows with single-click activation, native Up/Down and Enter navigation, checkbox-like text states, and a selected-row highlight.

The basic/scientific preference is saved on each mode change to `$XDG_CONFIG_HOME/argvus/calculator/settings.json` (HOME `.config` fallback) using the shared atomic JSON writer. Missing preferences default to basic; saved scientific mode restores its controls, minimum size and default geometry. Manual window dimensions are not persisted.
