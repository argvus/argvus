# Multilingual implementation audit — 2026-11-09

## Delivered architecture

- Ten locales, 29 application domains and locale metadata in the master package.
  Each locale covers all 3,286 canonical keys; extra keys are locale-specific
  plural forms. The optional argvus-ai package supplies its ai domain in all ten.
- Installed-catalog discovery replaces the Control Center's two-language list.
  Locale persistence and session environment remain based on user locale.conf.
- Script-aware normalization, regional resolution and English fallback are
  centralized. Traditional Chinese is not mislabeled as Simplified Chinese.
- Cardinal integer plural rules and validation cover Russian and the other
  supported languages. Interpolation does not expand user-supplied placeholders.
- The GTK calendar and removable-device menu no longer keep bilingual dictionaries.
  Their texts and calendar data are shared catalogs. Calendar overrides persist.
- Packaging installs every catalog directory; argvus-fonts requires Noto CJK.
  Font coverage was checked against actual installed font cmaps for sample text
  in all eight additional languages.

## Unicode audit and fixes

Character-by-character truncation could split combining sequences and joined
emoji. It now iterates extended grapheme clusters and measures terminal cells.
Status wrapping checked width before adding a character, allowing wide characters
to overflow. It now uses the shared grapheme-aware wrapper, including explicit
line breaks. About-page long words use the same safe splitting path.

Storage rows padded translated labels and mount paths by scalar count; they now
fit columns by terminal cells. The About key/value padding and snapshot diff
horizontal limits also use cell widths. Masked network keys use safe ellipsis.
Both text editors move/delete whole graphemes; the administration editor's
vertical movement uses visual columns instead of scalar offsets.

Tests exercise Latin accents, combining marks, Cyrillic, Han, Japanese, Hangul
and joined emoji. Settings pages render through Ratatui TestBackend at 40x12,
60x18, 80x24 and 140x40 in all installed languages, including narrow-terminal
fallbacks. Administrator warnings are tested in all installed languages.

## Translation quality and boundaries

Translations were created with machine translation assistance and technical
corrections. Initial batch generation produced empty translations and altered
command syntax; those results were replaced, and final catalogs must pass the
shared validator. Common actions have a maintained glossary. Month names, date
patterns, Russian plurals, calendar texts, removable-device actions and help
received explicit corrections. Placeholders and executable examples are checked
separately from natural-language text.

Independent native-speaker sign-off has not been obtained and is not claimed.
Structural tests cannot prove every translation's meaning. Administrator/PAM
messages, third-party programs and documentation outside the shared catalogs
retain their own translation/fallback policies. The pre-login greeter uses its
service account's language, not a not-yet-authenticated user's configuration.
CJK input methods are separate from font rendering. Terminal/font-specific emoji
and ambiguous-width behavior cannot be certified across all terminals by buffer
tests alone.

## Validation record

- `make validate` in de/argvus: passed; format, workspace Clippy with warnings
  denied, workspace tests and packaging/Shell checks. Tests: 2,309 passed,
  three existing host-dependent checks ignored (kernel inventory, disposable
  Btrfs mounts and findmnt inventory).
- `cargo check --workspace --locked`: passed.
- Catalog integration tests: 25 passed, including all ten locales, plural rules,
  regional/script resolution, placeholder safety and preservation of command
  options/code examples.
- Source catalogs and staged install from `arch_install_catalogs`: validation
  passed. The combined stage with argvus-ai also validated all ten locales.
- `make validate` in argvus-fonts, argvus-session and argvus-ai: passed.
  Session locale tests: seven passed. AI launcher/integration tests: 13 + 7 passed.
- `make build` in argvus-fonts: package produced, declaring noto-fonts-cjk.
- `qmllint` on the shared I18n singleton: passed. FontTools cmap checks verified
  sample glyph coverage in all eight additional languages using Fontconfig's
  selected fonts.
- Real-window visual inspection could not be completed because the session was
  locked. Captures were discarded; they are not evidence of application rendering.
  Ratatui TestBackend rendering and font coverage checks passed independently.
- Master `make build`: release binaries, the greeter profile and package checks
  completed successfully. The final package refresh includes the last calendar
  dropdown/data corrections and the command-preservation regression test.
- `make build` also produced the argvus-session and argvus-ai packages. Font,
  session and AI package linting emitted dependency-use warnings without errors.
- Final master package: `build/dist/argvus-0.0.0-1-x86_64.pkg.tar.zst`.
  Extracted all 300 JSON files and compared them byte-for-byte with source.
  The packaged argvus executable validated them and successfully detected all
  ten configured locales, returning the expected Control Center, calendar and
  removable-device translations for each locale.
- The final AI archive contains ten ai catalogs, all byte-identical to source.
  The session archive contains the updated locale helper byte-identical to source.
  The fonts archive's PKGINFO declares noto-fonts-cjk.
- Workspace Clippy with `--all-targets --all-features --locked -- -D warnings`:
  passed. Final whitespace/diff checks passed in all four modified repositories.
- Temporary translation scripts, profiles, screenshots and staging directories
  were removed after verification. Build archives remain in the projects' normal
  ignored build directories.
