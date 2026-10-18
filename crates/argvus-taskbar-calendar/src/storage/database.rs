use std::path::Path;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use rusqlite::{Connection, params};

use crate::calendar::{Calendar, CalendarEvent, CalendarSource, Reminder};
use crate::error::{ArgvusError, Result};

use super::migrations::{CURRENT_SCHEMA_VERSION, V1};

pub struct Database {
  conn: Connection,
}

pub const EXPIRED_LOCAL_EVENT_GRACE_MINUTES: i64 = 10;
const MISSED_REMINDER_GRACE_MINUTES: i64 = 5;

impl Database {
  pub fn open(path: &Path) -> Result<Self> {
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let db = Self { conn };
    db.migrate()?;
    Ok(db)
  }

  pub fn migrate(&self) -> Result<()> {
    let version: i64 = self
      .conn
      .query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
      let tx = self.conn.unchecked_transaction()?;
      for statement in V1 {
        tx.execute_batch(statement)?;
      }
      tx.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)?;
      tx.commit()?;
    }
    Ok(())
  }

  pub fn default_calendar_id(&self) -> Result<i64> {
    Ok(self.conn.query_row(
      "SELECT id FROM calendars WHERE source = 'local' ORDER BY id LIMIT 1",
      [],
      |row| row.get(0),
    )?)
  }

  pub fn calendars(&self) -> Result<Vec<Calendar>> {
    let mut stmt = self
      .conn
      .prepare("SELECT id, name, source, remote_url, color, enabled FROM calendars ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
      Ok(Calendar {
        id: row.get(0)?,
        name: row.get(1)?,
        source: CalendarSource::from(row.get::<_, String>(2)?.as_str()),
        remote_url: row.get(3)?,
        color: row.get(4)?,
        enabled: row.get::<_, i64>(5)? != 0,
      })
    })?;
    rows
      .collect::<std::result::Result<Vec<_>, _>>()
      .map_err(Into::into)
  }

  pub fn events_for_day(&self, date: NaiveDate) -> Result<Vec<CalendarEvent>> {
    let start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end = date.and_hms_opt(23, 59, 59).unwrap().and_utc();
    self.events_between(start, end)
  }

  pub fn events_between(
    &self,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
  ) -> Result<Vec<CalendarEvent>> {
    let mut stmt = self.conn.prepare(
      r#"
            SELECT id, uid, calendar_id, title, description, location, start_utc, end_utc,
                   all_day, timezone, recurrence_json, created_at, updated_at, source,
                   external_etag, remote_url, dirty, deleted
            FROM events
            WHERE deleted = 0
              AND datetime(start_utc) <= datetime(?2)
              AND datetime(end_utc) >= datetime(?1)
            ORDER BY all_day DESC, start_utc ASC, title ASC
            "#,
    )?;
    let rows = stmt.query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| {
      self.event_from_row(row)
    })?;
    rows
      .collect::<std::result::Result<Vec<_>, _>>()
      .map_err(Into::into)
  }

  pub fn all_events(&self) -> Result<Vec<CalendarEvent>> {
    let mut stmt = self.conn.prepare(
      r#"
            SELECT id, uid, calendar_id, title, description, location, start_utc, end_utc,
                   all_day, timezone, recurrence_json, created_at, updated_at, source,
                   external_etag, remote_url, dirty, deleted
            FROM events
            WHERE deleted = 0
            ORDER BY start_utc ASC
            "#,
    )?;
    let rows = stmt.query_map([], |row| self.event_from_row(row))?;
    rows
      .collect::<std::result::Result<Vec<_>, _>>()
      .map_err(Into::into)
  }

  pub fn event_by_id(&self, event_id: i64) -> Result<Option<CalendarEvent>> {
    let mut stmt = self.conn.prepare(
      r#"
            SELECT id, uid, calendar_id, title, description, location, start_utc, end_utc,
                   all_day, timezone, recurrence_json, created_at, updated_at, source,
                   external_etag, remote_url, dirty, deleted
            FROM events
            WHERE id = ?1 AND deleted = 0
            "#,
    )?;
    match stmt.query_row(params![event_id], |row| self.event_from_row(row)) {
      Ok(event) => Ok(Some(event)),
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  pub fn upsert_event(&mut self, event: &mut CalendarEvent) -> Result<i64> {
    event.validate()?;
    let tx = self.conn.transaction()?;
    tx.execute(
      r#"
            INSERT INTO events (
                uid, calendar_id, title, description, location, start_utc, end_utc, all_day,
                timezone, recurrence_json, created_at, updated_at, source, external_etag,
                remote_url, dirty, deleted
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(calendar_id, uid) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                location = excluded.location,
                start_utc = excluded.start_utc,
                end_utc = excluded.end_utc,
                all_day = excluded.all_day,
                timezone = excluded.timezone,
                recurrence_json = excluded.recurrence_json,
                updated_at = excluded.updated_at,
                source = excluded.source,
                external_etag = excluded.external_etag,
                remote_url = excluded.remote_url,
                dirty = excluded.dirty,
                deleted = excluded.deleted
            "#,
      params![
        event.uid,
        event.calendar_id,
        event.title,
        event.description,
        event.location,
        event.start.to_rfc3339(),
        event.end.to_rfc3339(),
        event.all_day as i64,
        event.timezone,
        serde_json::to_string(&event.recurrence)
          .map_err(|err| ArgvusError::Serialization(err.to_string()))?,
        event.created_at.to_rfc3339(),
        Utc::now().to_rfc3339(),
        event.source.as_str(),
        event.external_etag,
        event.remote_url,
        event.dirty as i64,
        event.deleted as i64,
      ],
    )?;
    let id = tx.query_row(
      "SELECT id FROM events WHERE calendar_id = ?1 AND uid = ?2",
      params![event.calendar_id, event.uid],
      |row| row.get(0),
    )?;
    tx.execute("DELETE FROM reminders WHERE event_id = ?1", params![id])?;
    for reminder in &event.reminders {
      tx.execute(
        "INSERT OR IGNORE INTO reminders (event_id, minutes_before, fired_at) VALUES (?1, ?2, ?3)",
        params![
          id,
          reminder.minutes_before,
          reminder.fired_at.map(|dt| dt.to_rfc3339())
        ],
      )?;
    }
    tx.commit()?;
    event.id = Some(id);
    Ok(id)
  }

  #[allow(dead_code)]
  pub fn mark_deleted(&self, event_id: i64) -> Result<()> {
    self.conn.execute(
      "UPDATE events SET deleted = 1, dirty = 1, updated_at = ?2 WHERE id = ?1",
      params![event_id, Utc::now().to_rfc3339()],
    )?;
    Ok(())
  }

  #[allow(dead_code)]
  pub fn purge_events_ended_before(&self, cutoff: DateTime<Utc>) -> Result<usize> {
    Ok(self.conn.execute(
      "DELETE FROM events WHERE datetime(end_utc) <= datetime(?1)",
      params![cutoff.to_rfc3339()],
    )?)
  }

  pub fn purge_expired_local_single_events(&self, now: DateTime<Utc>) -> Result<usize> {
    let cutoff = now - Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES);
    let mut stmt = self.conn.prepare(
      r#"
            SELECT id, recurrence_json
            FROM events
            WHERE deleted = 0
              AND source = 'local'
              AND datetime(end_utc) <= datetime(?1)
            "#,
    )?;
    let rows = stmt.query_map(params![cutoff.to_rfc3339()], |row| {
      Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut deleted = 0;
    for row in rows {
      let (id, recurrence_json) = row?;
      let recurrence =
        serde_json::from_str::<crate::calendar::Recurrence>(&recurrence_json).unwrap_or_default();
      if recurrence.is_empty() {
        self
          .conn
          .execute("DELETE FROM events WHERE id = ?1", params![id])?;
        deleted += 1;
      }
    }
    Ok(deleted)
  }

  pub fn due_reminders(&self, now: DateTime<Utc>) -> Result<Vec<(Reminder, CalendarEvent)>> {
    let missed_since = now - Duration::minutes(MISSED_REMINDER_GRACE_MINUTES);
    let mut stmt = self.conn.prepare(
            r#"
            SELECT r.id, r.event_id, r.minutes_before, r.fired_at,
                   e.id, e.uid, e.calendar_id, e.title, e.description, e.location,
                   e.start_utc, e.end_utc, e.all_day, e.timezone, e.recurrence_json,
                   e.created_at, e.updated_at, e.source, e.external_etag, e.remote_url,
                   e.dirty, e.deleted
            FROM reminders r
            JOIN events e ON e.id = r.event_id
            WHERE e.deleted = 0
              AND r.fired_at IS NULL
              AND datetime(e.start_utc, printf('%+d minutes', -r.minutes_before)) <= datetime(?1)
              AND (
                    datetime(e.end_utc) >= datetime(?1)
                    OR datetime(e.start_utc, printf('%+d minutes', -r.minutes_before)) >= datetime(?2)
                  )
            ORDER BY e.start_utc ASC
            "#,
        )?;
    let rows = stmt.query_map(
      params![now.to_rfc3339(), missed_since.to_rfc3339()],
      |row| {
        let reminder = Reminder {
          id: row.get(0)?,
          event_id: row.get(1)?,
          minutes_before: row.get(2)?,
          fired_at: parse_optional_datetime(row.get::<_, Option<String>>(3)?)?,
        };
        let event = self.event_from_offset_row(row, 4)?;
        Ok((reminder, event))
      },
    )?;
    rows
      .collect::<std::result::Result<Vec<_>, _>>()
      .map_err(Into::into)
  }

  pub fn ensure_start_reminders_for_unscheduled_due_events(
    &self,
    now: DateTime<Utc>,
  ) -> Result<usize> {
    let missed_since = now - Duration::minutes(MISSED_REMINDER_GRACE_MINUTES);
    Ok(self.conn.execute(
      r#"
            INSERT OR IGNORE INTO reminders (event_id, minutes_before, fired_at)
            SELECT e.id, 0, NULL
            FROM events e
            WHERE e.deleted = 0
              AND datetime(e.start_utc) <= datetime(?1)
              AND datetime(e.start_utc) >= datetime(?2)
              AND NOT EXISTS (
                    SELECT 1 FROM reminders r WHERE r.event_id = e.id
                  )
            "#,
      params![now.to_rfc3339(), missed_since.to_rfc3339()],
    )?)
  }

  pub fn mark_reminder_fired(&self, reminder_id: i64, when: DateTime<Utc>) -> Result<()> {
    self.conn.execute(
      "UPDATE reminders SET fired_at = ?2 WHERE id = ?1",
      params![reminder_id, when.to_rfc3339()],
    )?;
    Ok(())
  }

  fn event_from_row(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarEvent> {
    let event = self.event_from_offset_row(row, 0)?;
    Ok(CalendarEvent {
      reminders: self.reminders_for_event(event.id.unwrap_or_default())?,
      ..event
    })
  }

  fn event_from_offset_row(
    &self,
    row: &rusqlite::Row<'_>,
    offset: usize,
  ) -> rusqlite::Result<CalendarEvent> {
    let recurrence_json: String = row.get(offset + 10)?;
    let id: i64 = row.get(offset)?;
    Ok(CalendarEvent {
      id: Some(id),
      uid: row.get(offset + 1)?,
      calendar_id: row.get(offset + 2)?,
      title: row.get(offset + 3)?,
      description: row.get(offset + 4)?,
      location: row.get(offset + 5)?,
      start: parse_datetime(row.get(offset + 6)?)?,
      end: parse_datetime(row.get(offset + 7)?)?,
      all_day: row.get::<_, i64>(offset + 8)? != 0,
      timezone: row.get(offset + 9)?,
      recurrence: serde_json::from_str(&recurrence_json).unwrap_or_default(),
      reminders: Vec::new(),
      created_at: parse_datetime(row.get(offset + 11)?)?,
      updated_at: parse_datetime(row.get(offset + 12)?)?,
      source: CalendarSource::from(row.get::<_, String>(offset + 13)?.as_str()),
      external_etag: row.get(offset + 14)?,
      remote_url: row.get(offset + 15)?,
      dirty: row.get::<_, i64>(offset + 16)? != 0,
      deleted: row.get::<_, i64>(offset + 17)? != 0,
    })
  }

  fn reminders_for_event(&self, event_id: i64) -> rusqlite::Result<Vec<Reminder>> {
    let mut stmt = self.conn.prepare(
      "SELECT id, event_id, minutes_before, fired_at FROM reminders WHERE event_id = ?1",
    )?;
    let rows = stmt.query_map(params![event_id], |row| {
      Ok(Reminder {
        id: row.get(0)?,
        event_id: row.get(1)?,
        minutes_before: row.get(2)?,
        fired_at: parse_optional_datetime(row.get(3)?)?,
      })
    })?;
    rows.collect()
  }
}

fn parse_datetime(value: String) -> rusqlite::Result<DateTime<Utc>> {
  DateTime::parse_from_rfc3339(&value)
    .map(|dt| dt.with_timezone(&Utc))
    .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))
}

fn parse_optional_datetime(value: Option<String>) -> rusqlite::Result<Option<DateTime<Utc>>> {
  value.map(parse_datetime).transpose()
}

#[cfg(test)]
mod tests {
  use chrono::{Duration, TimeZone};

  use super::*;

  #[test]
  fn due_reminders_include_current_events_and_skip_future_or_expired_events() {
    let test_dir = std::env::temp_dir().join(format!(
      "argvus-calendar-reminders-{}",
      uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 18, 0, 0).unwrap();

    let mut due = CalendarEvent::new_local(1, "Due", now + Duration::minutes(5));
    db.upsert_event(&mut due).unwrap();

    let mut future = CalendarEvent::new_local(1, "Future", now + Duration::minutes(30));
    db.upsert_event(&mut future).unwrap();

    let mut expired = CalendarEvent::new_local(1, "Expired", now - Duration::hours(2));
    db.upsert_event(&mut expired).unwrap();

    let mut repeated = CalendarEvent::new_local(1, "Repeated", now - Duration::hours(1));
    repeated.end = now + Duration::hours(1);
    repeated.reminders = vec![Reminder {
      id: None,
      event_id: None,
      minutes_before: -60,
      fired_at: None,
    }];
    db.upsert_event(&mut repeated).unwrap();

    let titles: Vec<_> = db
      .due_reminders(now)
      .unwrap()
      .into_iter()
      .map(|(_, event)| event.title)
      .collect();
    assert_eq!(titles, vec!["Repeated", "Due"]);

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }

  #[test]
  fn due_reminders_include_recently_missed_zero_length_events() {
    let test_dir = std::env::temp_dir().join(format!(
      "argvus-calendar-missed-reminders-{}",
      uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 18, 0, 30).unwrap();

    let mut recent = CalendarEvent::new_local(1, "Recent", now - Duration::seconds(30));
    recent.end = recent.start;
    recent.reminders = vec![Reminder {
      id: None,
      event_id: None,
      minutes_before: 0,
      fired_at: None,
    }];
    db.upsert_event(&mut recent).unwrap();

    let mut old = CalendarEvent::new_local(1, "Old", now - Duration::minutes(6));
    old.end = old.start;
    old.reminders = vec![Reminder {
      id: None,
      event_id: None,
      minutes_before: 0,
      fired_at: None,
    }];
    db.upsert_event(&mut old).unwrap();

    let titles: Vec<_> = db
      .due_reminders(now)
      .unwrap()
      .into_iter()
      .map(|(_, event)| event.title)
      .collect();
    assert_eq!(titles, vec!["Recent"]);

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }

  #[test]
  fn unscheduled_due_events_get_a_start_reminder() {
    let test_dir = std::env::temp_dir().join(format!(
      "argvus-calendar-unscheduled-reminders-{}",
      uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 18, 0, 30).unwrap();

    let mut event = CalendarEvent::new_local(1, "Unscheduled", now - Duration::seconds(30));
    event.reminders.clear();
    db.upsert_event(&mut event).unwrap();

    assert_eq!(
      db.ensure_start_reminders_for_unscheduled_due_events(now)
        .unwrap(),
      1
    );
    let due = db.due_reminders(now).unwrap();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].1.title, "Unscheduled");
    assert_eq!(due[0].0.minutes_before, 0);

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }

  #[test]
  fn purge_removes_ended_events_and_their_reminders() {
    let test_dir =
      std::env::temp_dir().join(format!("argvus-calendar-purge-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let cutoff = Utc.with_ymd_and_hms(2026, 8, 12, 3, 0, 0).unwrap();

    let mut ended = CalendarEvent::new_local(1, "Ended", cutoff - Duration::hours(1));
    ended.end = cutoff;
    db.upsert_event(&mut ended).unwrap();
    let mut current = CalendarEvent::new_local(1, "Current", cutoff);
    db.upsert_event(&mut current).unwrap();

    assert_eq!(db.purge_events_ended_before(cutoff).unwrap(), 1);
    assert_eq!(
      db.all_events()
        .unwrap()
        .into_iter()
        .map(|event| event.title)
        .collect::<Vec<_>>(),
      vec!["Current"]
    );

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }

  #[test]
  fn purge_expired_local_single_events_keeps_recent_remote_and_recurring_events() {
    let test_dir =
      std::env::temp_dir().join(format!("argvus-calendar-expire-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 12, 18, 20, 0).unwrap();

    let mut expired = CalendarEvent::new_local(1, "Expired local", now - Duration::hours(1));
    expired.end = now - Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES + 1);
    db.upsert_event(&mut expired).unwrap();

    let mut recent = CalendarEvent::new_local(1, "Recent local", now - Duration::hours(1));
    recent.end = now - Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES - 1);
    db.upsert_event(&mut recent).unwrap();

    let mut recurring = CalendarEvent::new_local(1, "Recurring local", now - Duration::hours(1));
    recurring.end = now - Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES + 1);
    recurring.recurrence.rrule = Some("FREQ=DAILY".to_string());
    db.upsert_event(&mut recurring).unwrap();

    let mut remote = CalendarEvent::new_local(1, "Remote", now - Duration::hours(1));
    remote.end = now - Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES + 1);
    remote.source = CalendarSource::CalDav;
    db.upsert_event(&mut remote).unwrap();

    assert_eq!(db.purge_expired_local_single_events(now).unwrap(), 1);
    assert_eq!(
      db.all_events()
        .unwrap()
        .into_iter()
        .map(|event| event.title)
        .collect::<Vec<_>>(),
      vec!["Recent local", "Recurring local", "Remote"]
    );

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }

  #[test]
  fn purge_expired_local_single_events_removes_all_day_after_midnight_grace() {
    let test_dir = std::env::temp_dir().join(format!(
      "argvus-calendar-expire-all-day-{}",
      uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&test_dir).unwrap();
    let database_path = test_dir.join("calendar.db");
    let mut db = Database::open(&database_path).unwrap();
    let start = Utc.with_ymd_and_hms(2026, 8, 12, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 8, 13, 0, 0, 0).unwrap();
    let now = end + Duration::minutes(EXPIRED_LOCAL_EVENT_GRACE_MINUTES + 1);

    let mut event = CalendarEvent::new_local(1, "All day", start);
    event.end = end;
    event.all_day = true;
    db.upsert_event(&mut event).unwrap();

    assert_eq!(db.purge_expired_local_single_events(now).unwrap(), 1);
    assert!(db.all_events().unwrap().is_empty());

    drop(db);
    std::fs::remove_dir_all(test_dir).unwrap();
  }
}
