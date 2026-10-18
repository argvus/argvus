pub const CURRENT_SCHEMA_VERSION: i64 = 1;

pub const V1: &[&str] = &[
  r#"
    CREATE TABLE IF NOT EXISTS calendars (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        source TEXT NOT NULL,
        remote_url TEXT,
        color TEXT,
        enabled INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
    "#,
  r#"
    CREATE TABLE IF NOT EXISTS events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        uid TEXT NOT NULL,
        calendar_id INTEGER NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
        title TEXT NOT NULL,
        description TEXT,
        location TEXT,
        start_utc TEXT NOT NULL,
        end_utc TEXT NOT NULL,
        all_day INTEGER NOT NULL DEFAULT 0,
        timezone TEXT,
        recurrence_json TEXT NOT NULL DEFAULT '{}',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        source TEXT NOT NULL,
        external_etag TEXT,
        remote_url TEXT,
        dirty INTEGER NOT NULL DEFAULT 0,
        deleted INTEGER NOT NULL DEFAULT 0,
        UNIQUE(calendar_id, uid)
    )
    "#,
  r#"
    CREATE TABLE IF NOT EXISTS reminders (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
        minutes_before INTEGER NOT NULL,
        fired_at TEXT,
        UNIQUE(event_id, minutes_before)
    )
    "#,
  r#"
    CREATE TABLE IF NOT EXISTS caldav_accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        server_url TEXT NOT NULL,
        username TEXT NOT NULL,
        secret_lookup TEXT NOT NULL,
        sync_enabled INTEGER NOT NULL DEFAULT 1,
        last_sync_at TEXT,
        last_error TEXT
    )
    "#,
  r#"
    CREATE TABLE IF NOT EXISTS sync_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_id INTEGER,
        calendar_id INTEGER,
        started_at TEXT NOT NULL,
        finished_at TEXT,
        status TEXT NOT NULL,
        message TEXT
    )
    "#,
  r#"
    CREATE TABLE IF NOT EXISTS app_metadata (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    )
    "#,
  r#"
    INSERT OR IGNORE INTO calendars (id, name, source, color, enabled)
    VALUES (1, 'Local', 'local', '#62a0ea', 1)
    "#,
];
