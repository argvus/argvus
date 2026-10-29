# Argvus Calendar — Wiki

Documentation of what each module and function does. The app is a native Wayland
popup calendar (GTK4 + Relm4 + `gtk4-layer-shell`) that opens exactly where the
user clicks in Waybar, stores events in SQLite and schedules desktop reminders.

## Entry point — `crates/main/src/main.rs`

Parses the CLI with `clap` and dispatches each command.

- `main()` — bootstraps logging, accepts the immutable `--x`/`--y` position
  reconstructed from Waybar's original button event and layer-surface origin
  (with a live-pointer fallback for direct CLI calls), resolves paths and loads
  `Settings`, purges ended events, then
  dispatches the subcommand. `toggle`/`show` pass that position to a running
  instance over IPC; when none is running they spawn the UI (`ui::run_ui`).

Subcommands: `Toggle`, `Show`, `Hide`, `Config`, `ConfigPath`, `Reload`,
`Import { file }`, `Export { output }`, `Sync`, `Status`, `Service`.

## CLI operations — `src/app.rs`

- `import_file(paths, file)` — opens the database, imports the ICS file into the
  default calendar and prints how many events were added.
- `export_file(paths, output)` — reads all events and writes them as ICS.
- `sync_once(paths, settings)` — reports the configured sync interval and how
  many remote calendars are enabled.
- `status(paths)` — prints the state of the system: database, config/style/theme
  paths, events/reminders state, calendar and event counts.

## Configuration — `src/config/settings.rs`

`AppConfig` (alias `Settings`) is a `serde` struct with `appearance`, `locale`,
`calendar`, `editor` and `terminal` sections. The system file under
`/etc/argvus/taskbar/calendar/config.toml` acts as the default base and the user file is
layered on top, so no elevated privileges are needed to persist changes.

- `Settings::load(paths)` — loads the system config, merges the user config over
  it (TOML deep merge) and validates the result.
- `Settings::save(paths)` — serializes the full config to the user-level file.
- `Settings::validate(config)` — clamps font size, and falls back to valid
  language/week-start values.
- `Settings::week_start()` — maps the week-start string to the `WeekStart` enum.
- `default_*()` helpers — the serde defaults for each key.
- `load_system_config(paths)` — reads the system config file (or the legacy
  `settings.toml` location when the modern file is missing); returns defaults
  when absent.
- `load_user_config(paths)` — reads the user config, if present.
- `merge_toml(base, override_)` — deep-merges the user TOML over the system
  values (recurses into tables).
- `effective_config_file(paths)` — the file `open_config` edits: the user file
  once it exists, otherwise the system file.
- `list_theme_names(paths)` — theme names found in the theme directory.
- `active_theme_name(paths)` — the ARGVUS theme recorded as active.
- `write_active_theme(paths, name)` — persists the active ARGVUS theme.
- `resolve_paths()` — builds `Paths` from XDG dirs (with `XDG_CONFIG_HOME` etc.
  and a user-local fallback).
- `load_events_enabled(paths, config)` — the effective events/reminders state:
  the state file wins over the legacy config key; missing state defaults to off.
- `service_events_enabled(paths)` — reads the persisted events/reminders state and
  defaults to off until the user explicitly enables events.
- `save_events_enabled(paths, enabled)` — persists the state toggle in
  `paths.state_dir` (`~/.local/state/argvus/taskbar/calendar/events-enabled`).
- `read_events_enabled(paths)` — raw state file read.
- `open_config(paths, config)` — opens the config file in the configured editor.
- `editor_terminal_command(config, path)` — builds the terminal command to run
  the editor with sudo (kitty/foot and fallbacks).
- `ensure(path)` / `xdg_home(var, fallback)` — directory helpers.
- `validate_export_path(path)` — rejects exporting over the config/theme files.

## IPC — `src/ipc.rs`

Single-instance control over a UNIX socket in the cache dir.

- `IpcCommand::as_str` / `parse` — serialize/deserialize commands (`toggle`,
  `show`, `hide`, `reload-theme`, `reload-config`).
- `IpcMessage` — a command plus the fixed click position supplied by the Waybar
  launcher, so the running UI does not re-read the cursor at present time.
- `socket_path(paths)` — the socket location.
- `notify(paths, message)` — connects to a running instance, writes the message
  and returns true when it acknowledged.
- `listen(paths)` — binds the socket; handles a stale socket file left by a dead
  instance and returns `None` when another instance is alive.
- `serve(listener, on_command)` — background thread dispatching messages to the
  UI and replying `ok`.

## Calendar domain — `src/calendar/`

### `date.rs`

- `WeekStart::offset_for(date)` — weekday offset for Monday- or Sunday-start.
- `today()` — the current local date.
- `first_of_month(date)` / `next_month(date)` / `previous_month(date)` — month
  navigation.
- `shift_year_clamped(date, years)` — shifts the year, clamping a Feb 29 to
  Feb 28 on non-leap years.
- `month_grid(anchor, week_start)` — the calendar grid of `NaiveDate`s for the
  month, padded with days from adjacent months.
- `day_bounds(tz, date)` — the half-open UTC range `[local midnight, next local
  midnight)` of a day; every day query goes through it, so the local day (not
  the UTC day) decides where an event appears.

### `event.rs`

- `CalendarSource` — `Local` / `CalDav` / `ImportedIcs`, with string conversion
  for storage.
- `CalendarEvent::new_local(...)` — builds an event in the local calendar.
- `CalendarEvent::local_days(tz)` — the local days an event occupies, with an
  exclusive end (an all-day event covers one day; an overnight event two).
- `CalendarEvent::validate()` — checks title non-empty and start before end.

### `recurrence.rs`

- `Recurrence::is_empty()` — whether the recurrence is disabled.

## Storage — `src/storage/database.rs`

SQLite persistence (rusqlite). `Database::open(path)` runs migrations on open.

- `open(path)` — opens the database and migrates the schema.
- `migrate()` — applies pending migrations.
- `default_calendar_id()` — the Local calendar id.
- `calendars()` — all calendars.
- `events_for_day(tz, date)` — events for one local day (all-day included).
- `events_between(start, end)` — events overlapping the half-open UTC range
  `[start, end)`; an event ending exactly at `start` is left out.
- `all_events()` — every event.
- `event_by_id(id)` — one event with its reminders.
- `upsert_event(&mut event)` — inserts or updates an event plus its reminders,
  returning the id.
- `mark_deleted(id)` — soft-deletes an event.
- `purge_expired_local_single_events(now)` — removes local events deleted in the
  popup and local non-recurring events 10 minutes after their end time,
  preserving CalDAV/ICS and recurring events.
- `due_reminders(now)` — reminders whose trigger time has passed and have not
  fired, joined with their events, including recently missed reminders inside a
  short grace window.
- `ensure_start_reminders_for_unscheduled_due_events(now)` — creates the start
  reminder (`minutes_before = 0`) for every event that started a moment ago or
  is still in progress and lacks one, so each event notifies at least once
  (imported events, rows saved before the start reminder existed).
- `mark_reminder_fired(id, when)` — records that a reminder fired.
- `event_from_row` / `event_from_offset_row` / `reminders_for_event` — row
  mapping helpers.
- `parse_datetime` / `parse_optional_datetime` — SQLite text to `DateTime<Utc>`.

## ICS import/export — `src/ical/`

- `import_ics_file(path, calendar_id)` — reads a file and parses it.
- `parse_ics(contents, calendar_id)` — parses `DTSTART`/`DTEND`, all-day flags,
  reminder `TRIGGER`s and `RRULE` recurrence into `CalendarEvent`s. `RRULE`
  is stored and exported but not expanded: only the first occurrence is shown
  and reminded.
- `to_utc(value)` — `DATE` values start at the local midnight, floating times
  are local, `TZID` times use that zone (local fallback for unknown ids).
- `trigger_to_minutes(trigger)` — relative ISO-8601 `TRIGGER` (`W`, `D`, `H`,
  `M`, `S`) to minutes before the start.
- `export_ics_file(path, events)` — writes events as an ICS file.
- `export_ics(events)` — serializes events to an ICS string; all-day events go
  out as `DATE` values so they import back unchanged.

## Reminders — `src/reminders/`

### `notification.rs`

- `notify_event(event, minutes_before, i18n)` — shows a desktop notification
  (notify-rust) with a translated message for all-day/in progress/now/later
  (`reminder_body`) and requests a bell sound, with desktop audio command
  fallbacks.

### `scheduler.rs`

- `ReminderScheduler::new(paths)` — scheduler polling every 30s.
- `run()` — loop calling `tick` and sleeping. When `/proc/self/exe` shows the
  binary was replaced (package update) it returns an error, so systemd
  (`Restart=on-failure`) restarts the service on the new version.
- `tick()` — purges expired local non-recurring events, creates start-time
  reminders for due imported events without alarms, fetches due reminders,
  skips stale repeat triggers and fires notifications in the language of the
  calendar settings (re-read on every tick).
- `is_stale_repeat(start, minutes_before, now)` — avoids re-notifying old
  repetitions of an in-progress all-day event.

### `service/mod.rs`

- `run_service(paths)` — the `argvus-calendar service` entry point; exits
  immediately when events/reminders are disabled, otherwise runs the scheduler.

## CalDAV foundation — `src/caldav/`

- `client.rs::CalDavClient::new(base_url, username, password)` — HTTP client with
  Basic auth.
- `client.rs::discover_collections()` — `PROPFIND` to list calendar collections.
- `client.rs::list_objects(collection_url)` — lists remote calendar objects.
- `client.rs::put_object(...)` / `delete_object(...)` — push and remove remote
  objects (with ETag).
- `client.rs::dav(...)` — low-level WebDAV HTTP helper.
- `client.rs::parse_collections(xml)` / `parse_remote_objects(xml)` /
  `local_name(name)` — XML parsing helpers (quick-xml).
- `discovery.rs::discover(client)` — convenience wrapper around collection
  discovery.
- `credentials.rs::store_password(...)` / `load_password(lookup)` — keyring-backed; needs a Secret Service provider on the session bus (see `argvus-session` docs)
  credential helpers.
- `sync.rs::CalDavSyncService::new(client)` and
  `initial_sync(collection_url)` — first-pass one-way pull producing a
  `SyncReport`.
- `types.rs` — the `CalDavAccount`, `CalendarCollection`, `RemoteObject` and
  `SyncReport` types.

## UI — `src/ui/`

### `mod.rs`

- Re-exports the popup command used by `main.rs`.

### `app.rs` and `app/`

The Relm4 component: the whole popup is one layer-shell surface. `app.rs` holds
the model, the messages and their handling; the submodules of `app/` hold the
widget construction and the helpers.

`app.rs`:

- `run_ui(paths, settings, command, fixed_position)` — entry point. Pins the
  popup to the click position supplied by the Waybar launcher, with an
  in-process fallback read for direct CLI calls, before building the GTK app.
- `AppModel::init` — creates the model, starts the theme watcher and the IPC
  listener, assembles the calendar, editor and settings views from `app/build.rs`
  and presents when the command asks for it.
- `AppModel::update` — handles navigation, day selection, event CRUD, settings
  editing and popup show/hide; reloads events when `AppMsg::reloads_events` says
  the state may have changed. Calendar navigation and "new event" messages
  (`AppMsg::applies_to`) are ignored outside the calendar page, so a shortcut
  cannot discard an editor or settings draft. A failed save logs a warning and
  keeps the editor open. Saving, deleting, toggling and showing the popup
  are the `AppModel` methods `save_editor`, `delete_edited_event`,
  `save_settings`, `toggle_popup` and `show_popup`.
- `reapply_runtime` — reapplies locale, CSS and layer-shell settings after a
  config save.

`app/build.rs` — the widget tree and its signals:

- `build_calendar_view`, `build_editor_view`, `build_settings_view` — each
  builds one page of the stack and connects the signals of its own widgets.
- `connect_focus_dismiss` — outside-click dismissal on focus loss.
- `install_keyboard_shortcuts` — Escape, `t`, `n`, arrows and Page Up/Down.
- `watch_theme` — re-reads the theme fingerprint every 700 ms.
- `serve_ipc` — forwards the IPC commands of later launches.

Popup placement and dismissal now live in `crates/argvus-taskbar-popup` and are
shared with ARGVUS Focus and ARGVUS Clock. `PopupController` applies the
layer-shell overlay, selects the monitor and horizontal bar from the click's
desktop coordinates, and positions the popup at the bar's right edge with a
4-pixel gap (below a top bar or above a bottom bar). Without click coordinates,
it uses the pointer position. After 800 ms, focus loss with the pointer outside
dismisses the popup; a 400 ms guard prevents the same outside click from
immediately reopening it. The calendar retains its own namespace and dimensions.

`app/styling.rs` — CSS:

- `load_css(paths)` — loads the CSS cascade.
- `stylesheet_paths(paths)` — style.css, theme.css, the active ARGVUS theme and
  the user cache theme, in that order.
- `runtime_css(paths)` — font CSS generated from config.
- `css_string(value)` — CSS-escaping helper.
- `file_fingerprint`, `theme_fingerprint` — change detection of the theme files.

`app/render.rs`:

- `render_model` — pushes the current month/selection/agenda into the widgets.
- `load_day_and_month(paths, selected, month)` — events + event-day set.

`app/drafts.rs`:

- `cancel_editor`, `update_draft`, `update_settings_draft`,
  `apply_editor_field`, `apply_settings_field` — view-state and draft helpers.
- `can_create_event` — events can only be created from today on.
- `control_reminder_service(enabled)` — enables/disables the user systemd unit;
  on disable it also signals a manually started `taskbar-calendar service`
  process of the current user.

`app/form_widgets.rs` — GTK builders: `editor_row`, `editor_label`,
`editor_entry`, `editor_number_spin`, `editor_control_has_focus` (any
`gtk::Editable`, so spin buttons count as text fields),
`settings_label`, `settings_section_label`, `settings_number_spin`.

### `calendar.rs`

- `day_cells(month, week_start)` — the grid of (date, in-month) cells.
- `weekday_labels(week_start)` — the header labels.

### `agenda.rs`

- `rebuild_agenda(list, events, i18n, on_event)` — repopulates the day agenda
  list from events.

### `event_editor.rs`

- `EditorDraft::new(date, defaults)` / `from_event(event)` — build a draft for a
  new event (09:00 plus the default length, default reminder) or an existing one.
- `EditorDraft::set_start_time(hour, minute)` — moves the start and keeps the
  length; `can_save()` — false for past dates (Save is disabled, Delete stays).
- `build_editor()` — the editor form widgets.
- `populate_editor(widgets, draft, i18n)` — fills the form (`fill_editor`) with
  the change handlers listed by `handler_bindings` blocked, so the values set
  here do not come back as edit messages.
- `save_editor(db_path, settings, draft, i18n)` — writes the event to the
  database. A blank title becomes the translated "New event" text; an end time
  before the start time means the event ends on the next day (`timed_range`).
- `build_reminders(draft, previous, start, end, now)` — always includes the start
  reminder, plus the optional earlier reminder and the all-day repetitions; when
  an edit keeps the start, reminders already fired keep `fired_at` and are not
  shown again.
- `all_day_range(draft)` — an all-day event is stored from the typed time
  ("NOTIFY AT", the time its start reminder fires) to the next local midnight;
  `from_event` reads that time back into the form.
- `local_datetime(date, hour, minute)` — local components to `DateTime<Utc>`.
- `empty_to_none`, `set_entry_text`, `set_number` — widget helpers.

### `settings.rs`

- `SettingsDraft::from_config(...)` / `into_config()` — snapshot the current
  settings for the settings screen and rebuild a `Settings` on save (with
  clamping and fallbacks).
- `build_settings()` — the settings form container.
- `set_dropdown(dropdown, model, value)` / `dropdown_value(dropdown)` — dropdown
  helpers.
- `populate_settings(widgets, draft, i18n)` — fill the form (`fill_settings`)
  with the change handlers listed by `handler_bindings` blocked.

## Errors — `src/error.rs`

`ArgvusError` enum and the `Result<T>` alias used everywhere.
