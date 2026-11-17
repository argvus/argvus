//! JSON-lines session. Input, provider refresh and activation never block QML.
use crate::{
  files, providers,
  search::{self, Action, Entry},
  settings::{SOURCES, Settings},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
  collections::BTreeMap,
  io::{self, BufRead, Write},
  process::ExitCode,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
  },
  time::{Duration, Instant},
};

#[derive(Default, Deserialize)]
#[serde(default)]
struct Request {
  op: String,
  sequence: u64,
  query: String,
  category: String,
  entries: Vec<Entry>,
  id: String,
}
enum Message {
  Request(Request),
  Provider(String, Vec<Entry>),
  Failure(String, String),
  Done,
  Activated(Result<(), String>),
  End,
}

fn emit(value: Value) {
  println!("{value}");
  let _ = io::stdout().flush();
}
fn response(sequence: u64, entries: &[Entry], error: Option<String>) {
  emit(json!({"event":"results","sequence":sequence,"entries":entries,"error":error}));
}

fn refresh(
  sender: mpsc::Sender<Message>,
  settings: Settings,
  stop: Arc<AtomicBool>,
  rebuild: bool,
) {
  std::thread::spawn(move || {
    for source in ["settings", "projects", "ssh"] {
      if stop.load(Ordering::Relaxed) {
        break;
      }
      let result =
        if !(settings.enabled(source) || source == "settings" && settings.enabled("help")) {
          Ok(Vec::new())
        } else {
          match source {
            "settings" => providers::settings(),
            "projects" => providers::projects(),
            _ => providers::ssh(),
          }
        };
      let message = match result {
        Ok(entries) => Message::Provider(source.into(), entries),
        Err(error) => Message::Failure(source.into(), error),
      };
      if sender.send(message).is_err() {
        return;
      }
    }
    if !stop.load(Ordering::Relaxed) {
      let mut index = files::load();
      if settings.index_enabled && !settings.index_paused {
        let next = files::scan(&settings, &stop);
        if !stop.load(Ordering::Relaxed) {
          // Periodic reconciliation updates the cache only when names change.
          if (rebuild
            || files::cache_path().is_ok_and(|path| !path.exists())
            || index.files != next.files)
            && let Err(error) = files::save(&next, &settings)
          {
            let _ = sender.send(Message::Failure("files".into(), error));
          }
          index = next;
        }
      }
      if !stop.load(Ordering::Relaxed) {
        let _ = sender.send(Message::Provider(
          "files".into(),
          files::entries(&index, &settings),
        ));
      }
    }
    let _ = sender.send(Message::Done);
  });
}

pub fn run_command(arguments: Vec<String>) -> ExitCode {
  if arguments.first().map(String::as_str) != Some("serve") {
    eprintln!("usage: argvus launcher-engine serve");
    return ExitCode::FAILURE;
  }
  match serve() {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("launcher engine: {error}");
      ExitCode::FAILURE
    }
  }
}

fn serve() -> Result<(), String> {
  let mut settings = Settings::load()?;
  let (sender, receiver) = mpsc::channel();
  let input = sender.clone();
  std::thread::spawn(move || {
    for line in io::stdin().lock().lines() {
      let Ok(line) = line else {
        break;
      };
      if line.len() > 4 * 1024 * 1024 {
        continue;
      }
      match serde_json::from_str(&line) {
        Ok(request) => {
          if input.send(Message::Request(request)).is_err() {
            return;
          }
        }
        Err(error) => {
          let _ = input.send(Message::Failure("protocol".into(), error.to_string()));
        }
      }
    }
    let _ = input.send(Message::End);
  });
  let mut sources: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
  for entry in providers::builtin() {
    sources
      .entry(entry.category.clone())
      .or_default()
      .push(entry);
  }
  let mut active_stop = Arc::new(AtomicBool::new(false));
  let mut busy = false;
  let mut pending = true;
  let mut rebuild = false;
  let mut last = Instant::now();
  let mut checked = Instant::now();
  let mut current = Request::default();
  let mut devices: Option<Vec<Entry>> = None;
  let mut visible: Vec<Entry> = Vec::new();
  emit(json!({"event":"ready","settings":settings}));
  loop {
    if checked.elapsed() > Duration::from_secs(2) {
      if settings.index_enabled
        && !settings.index_paused
        && files::cache_path().is_ok_and(|path| !path.exists())
      {
        pending = true;
      }
      checked = Instant::now();
      match Settings::load() {
        Ok(next) if next != settings => {
          settings = next;
          active_stop.store(true, Ordering::Relaxed);
          pending = true;
          sources.remove("files");
          devices = None;
          emit(json!({"event":"ready","settings":settings}));
        }
        Err(error) => eprintln!("launcher preferences: {error}"),
        _ => {}
      }
    }
    if !busy && (pending || last.elapsed() > Duration::from_secs(30)) {
      active_stop = Arc::new(AtomicBool::new(false));
      refresh(
        sender.clone(),
        settings.clone(),
        Arc::clone(&active_stop),
        rebuild,
      );
      pending = false;
      rebuild = false;
      busy = true;
      last = Instant::now();
    }
    let message = match receiver.recv_timeout(Duration::from_millis(100)) {
      Ok(message) => message,
      Err(mpsc::RecvTimeoutError::Timeout) => continue,
      Err(_) => break,
    };
    let mut redraw = true;
    match message {
      Message::End => {
        active_stop.store(true, Ordering::Relaxed);
        break;
      }
      Message::Done => {
        busy = false;
        redraw = false;
        emit(
          json!({"event":"index_status","count":sources.get("files").map_or(0,Vec::len),"paused":settings.index_paused}),
        );
      }
      Message::Failure(source, error) => {
        eprintln!("launcher provider {source}: {error}");
        if source != "protocol" {
          sources.remove(&source);
        }
        emit(
          json!({"event":"error","source":source,"message":argvus_i18n::tr(providers::language(),"universal.provider_error")}),
        );
      }
      Message::Provider(source, entries) => {
        if !active_stop.load(Ordering::Relaxed) {
          sources.insert(source, entries);
        }
      }
      Message::Activated(result) => {
        match result {
          Ok(()) => emit(json!({"event":"close"})),
          Err(error) => {
            eprintln!("launcher action: {error}");
            emit(
              json!({"event":"error","message":argvus_i18n::tr(providers::language(),"universal.action_error")}),
            );
          }
        }
        redraw = false;
      }
      Message::Request(request) => match request.op.as_str() {
        "select_category" => {
          let previous = settings.clone();
          if request.category.is_empty() || settings.enabled(&request.category) {
            settings.selected_category = request.category;
            match settings.save_changes(&previous) {
              Ok(()) => emit(json!({"event":"ready","settings":settings})),
              Err(error) => {
                settings = previous;
                eprintln!("launcher preferences: {error}");
                emit(
                  json!({"event":"error","message":argvus_i18n::tr(providers::language(),"universal.action_error")}),
                );
              }
            }
          }
        }
        "query" => {
          current = request;
        }
        "clients" => {
          for source in ["applications", "windows"] {
            let entries = request
              .entries
              .iter()
              .filter(|entry| {
                entry.category == source
                  && matches!(
                    (&entry.action, source),
                    (Action::Application(_), "applications") | (Action::Window(_), "windows")
                  )
              })
              .take(20000)
              .cloned()
              .collect();
            sources.insert(source.into(), entries);
          }
        }
        "refresh" => {
          settings = Settings::load()?;
          active_stop.store(true, Ordering::Relaxed);
          pending = true;
          // Hide old file results immediately after a policy change.
          sources.remove("files");
          devices = None;
          emit(json!({"event":"ready","settings":settings}));
        }
        "rebuild" => {
          active_stop.store(true, Ordering::Relaxed);
          pending = true;
          rebuild = true;
        }
        "clear_index" => {
          let previous = settings.clone();
          settings.index_paused = true;
          settings.save_changes(&previous)?;
          files::clear()?;
          sources.remove("files");
          active_stop.store(true, Ordering::Relaxed);
          pending = true;
          emit(json!({"event":"ready","settings":settings}));
        }
        "back" => {
          devices = None;
          current.query.clear();
        }
        "configure" => {
          let tx = sender.clone();
          std::thread::spawn(move || {
            let _ = tx.send(Message::Activated(providers::activate(&Action::Configure)));
          });
          redraw = false;
        }
        "favorite" => {
          let previous = settings.clone();
          if settings.favorites.contains(&request.id) {
            settings.favorites.retain(|id| id != &request.id);
          } else if visible.iter().any(|entry| entry.id == request.id) {
            settings.favorites.push(request.id);
          }
          settings.save_changes(&previous)?;
          emit(json!({"event":"ready","settings":settings}));
        }
        "activate" => {
          if let Some(entry) = visible.iter().find(|entry| entry.id == request.id) {
            match &entry.action {
              Action::AudioOutputs => match providers::audio_outputs() {
                Ok(entries) => {
                  devices = Some(entries);
                  current.query.clear();
                  emit(json!({"event":"submenu"}));
                }
                Err(error) => {
                  let _ = sender.send(Message::Failure("audio".into(), error));
                }
              },
              Action::Application(_) | Action::Window(_) => {
                let application_id = match &entry.action {
                  Action::Application(id) => Some(id.as_str()),
                  Action::Window(_) if !entry.usage_id.is_empty() => Some(entry.usage_id.as_str()),
                  _ => None,
                };
                if let Some(application_id) = application_id
                  && let Err(error) = crate::usage::record(application_id)
                {
                  eprintln!("launcher usage history: {error}");
                }
                emit(json!({"event":"activate","action":entry.action}));
                redraw = false;
              }
              action => {
                let action = action.clone();
                let tx = sender.clone();
                std::thread::spawn(move || {
                  let _ = tx.send(Message::Activated(providers::activate(&action)));
                });
                redraw = false;
              }
            }
          }
        }
        _ => {
          redraw = false;
        }
      },
    }
    if redraw {
      if let Some(entries) = &devices {
        let mut device_settings = settings.clone();
        device_settings.sources = SOURCES.iter().map(|s| (*s).into()).collect();
        visible = if current.query.trim().is_empty() {
          entries.clone()
        } else {
          search::rank(entries, &current.query, "actions", &device_settings)
        };
      } else {
        let entries: Vec<_> = sources.values().flatten().cloned().collect();
        let recent_order = crate::usage::recent_order().unwrap_or_else(|error| {
          eprintln!("launcher usage history: {error}");
          Vec::new()
        });
        visible = search::rank_with_recent_order(
          &entries,
          &current.query,
          &current.category,
          &settings,
          &recent_order,
        );
      }
      response(current.sequence, &visible, None);
    }
  }
  Ok(())
}
