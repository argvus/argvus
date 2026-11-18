# Universal launcher

The `argvus-launcher` component owns the Quickshell/QML surface and its theme
consumer. `argvus-launcher-core` in the master workspace owns typed preferences,
ranking, provider orchestration, the file-name cache and action activation.
`argvus launcher-engine serve` runs the engine in-process in the public dispatcher.
The engine checks whether its executable was replaced during the session.
Once the replacement exists, the engine exits and Quickshell starts the new
binary automatically, then resends the current search.
No additional Rust executable is installed.

## Sources of truth

- `argvus config`: `/launcher`, `/projects`, keyboard shortcuts and default apps.
- Control Center: `--search-catalog` exports destinations from the compiled search
  registry and enabled shortcut descriptions. `--search-id ID` opens a registry
  destination through the same route dispatcher as internal search. A running
  window receives the requested destination through the user's runtime directory.
- Quickshell: desktop entries and Hyprland toplevels. The frontend executes only
  application/window activation targets that still exist in its live models.
- Existing commands: `argvus projects list --resolved`, `argvus projects open`,
  `argvus ssh list` and `argvus ssh open`.
- Audio: the Control Center audio backend; device IDs are revalidated before use.

## Protocol and workers

The QML process keeps the engine's stdin open and exchanges one JSON object per
line. Requests contain `op` and, for queries, a monotonic `sequence`, `query` and
optional `category`. `clients` supplies application/window metadata, including
whether a window is active. `activate` accepts a visible result ID; it cannot
execute arbitrary shell text. `favorite`, `refresh`, `back`, `configure`,
`rebuild` and `clear_index` are control operations.

Events are `ready` (effective preferences), `results` (sequence and entries),
`submenu`, `activate` (frontend action), `index_status`, `error` and `close`.
QML discards stale sequences and preserves the selected stable ID on refresh.
Result actions are serde-tagged enums. Provider failures are logged separately
and leave other sources usable.

Input and provider collection run on separate workers. Provider snapshots arrive
progressively; one refresh runs at a time. Name traversal checks cancellation.
The engine checks preferences every two seconds and reconciles provider/file
snapshots every thirty seconds, including while hidden. It writes the cached
name list only when names change or a rebuild is requested. This is periodic
name reconciliation, not a content index or filesystem event watcher.

The cache is `$XDG_CACHE_HOME/argvus/launcher/index.json` (falling back to
`~/.cache`). It is atomically replaced with mode 0600 and can be deleted safely.
Cache writes and deletion share an advisory file lock. A worker rechecks the
canonical preferences under that lock before writing, so a paused or removed
index cannot be recreated by an older scan. Preferences are saved as field
patches, preserving independent edits from the other interface.

Indexing is opt-in; explicit absolute roots and exclusions are required. Hidden
children are omitted unless enabled. Symlink traversal is disabled, removed
roots are filtered on refresh, and missing files disappear from results.
XDG `recently-used.xbel` supplies local recent-file paths; applications that do
not register their usage will not appear in that source.

## Entry points and compatibility

`argvus launcher` toggles one Quickshell instance by IPC. Specialized Rofi menus
remain unchanged. The default launcher setting is `argvus`; the historical `rofi`
value still selects the ARGVUS launcher in Hyprland. Other custom launchers retain
`<program> --show drun`. The main launcher rejects Rasi `--config` and other
legacy Rofi arguments with exit status 2.

Control Center lists Launcher and Search under Preferences. The page supports
source toggles and Ctrl+Up/Down ordering, favorite removal, explicit root and
exclusion entry, hidden files, recent files, reduced motion, pause, cache removal
and rebuild. Settings I/O runs off the render loop; F5 refreshes index status.
The new consumer renders `launcher/Theme.qml`; Rofi continues using
`launcher/theme.rasi`.

Application activations launched or selected through the launcher are recorded
by desktop-entry ID in `$XDG_STATE_HOME/argvus/launcher/application-usage.json`
(falling back to `~/.local/state/argvus/launcher/application-usage.json`). The
file keeps the existing activation counts and a most-recent-first ID list. Writes
are locked and atomic; no window title or timestamp is stored. Empty Applications
results use the configured A–Z or Recently opened order (Recently opened by default),
with unknown IDs falling back alphabetically. For typed searches, relevance remains
primary and the selected order breaks ties. The Control Center can clear both counts
and recency history.

The `show_open_now` preference defaults to `false`. When enabled, empty-query
results include the currently open windows before other results regardless of
the saved category; the active window is first and the remaining windows follow
the source order. The Windows source toggle remains authoritative. Typed queries
continue to respect the selected category.
