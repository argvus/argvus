//! Temporary Wayland selections used by the native paste shortcut.
use std::{
  env,
  io::{self, BufRead, BufReader, BufWriter, Read, Write},
  process::{Child, Command, Stdio},
  thread,
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use wl_clipboard_rs::{
  copy::{MimeSource, MimeType, Options, Source},
  paste::{self, ClipboardType, Seat},
};

type Offer = (String, Vec<u8>);
const MAX_BACKUP_BYTES: u64 = 32 * 1024 * 1024;
const OWNER_COMMAND: &str = "--serve-paste-selection";

pub fn is_owner_command(argument: &str) -> bool {
  argument == OWNER_COMMAND
}

// The restored selection must remain available after the picker exits.
pub fn serve(primary: bool) -> Result<(), Box<dyn std::error::Error>> {
  let offers: Vec<Offer> = serde_json::from_reader(io::stdin().lock())?;
  let sources = offers
    .into_iter()
    .map(|(mime, bytes)| MimeSource {
      source: Source::Bytes(bytes.into_boxed_slice()),
      mime_type: MimeType::Specific(mime),
    })
    .collect();
  let mut options = Options::new();
  options
    .foreground(true)
    .omit_additional_text_mime_types(true)
    .clipboard(if primary {
      wl_clipboard_rs::copy::ClipboardType::Primary
    } else {
      wl_clipboard_rs::copy::ClipboardType::Regular
    });
  let copy = options.prepare_copy_multi(sources)?;
  println!("ready");
  io::stdout().flush()?;
  copy.serve()?;
  Ok(())
}

fn start_owner(primary: bool, offers: &[Offer]) -> io::Result<Child> {
  let mut child = Command::new(env::current_exe()?)
    .args([OWNER_COMMAND, if primary { "primary" } else { "regular" }])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
  let ready = (|| -> io::Result<bool> {
    let stdin = child
      .stdin
      .take()
      .ok_or_else(|| io::Error::other("selection owner stdin"))?;
    {
      let mut input = BufWriter::new(stdin);
      serde_json::to_writer(&mut input, offers)?;
      input.flush()?;
    }
    let output = child
      .stdout
      .take()
      .ok_or_else(|| io::Error::other("selection owner stdout"))?;
    let mut status = String::new();
    BufReader::new(output).read_line(&mut status)?;
    Ok(status.trim_end() == "ready")
  })();
  if matches!(ready, Ok(true)) {
    return Ok(child);
  }
  let _ = child.kill();
  let _ = child.wait();
  Err(
    ready
      .err()
      .unwrap_or_else(|| io::Error::other("selection owner failed")),
  )
}

fn snapshot(clipboard: ClipboardType) -> Result<Vec<Offer>, Box<dyn std::error::Error>> {
  let types = match paste::get_mime_types_ordered(clipboard, Seat::Unspecified) {
    Ok(types) => types,
    Err(paste::Error::ClipboardEmpty) => return Ok(Vec::new()),
    Err(error) => return Err(error.into()),
  };
  let mut remaining = MAX_BACKUP_BYTES;
  let mut offers = Vec::new();
  for mime in types {
    let (pipe, _) = paste::get_contents(
      clipboard,
      Seat::Unspecified,
      paste::MimeType::Specific(&mime),
    )?;
    let mut bytes = Vec::new();
    let len = pipe.take(remaining + 1).read_to_end(&mut bytes)? as u64;
    remaining = remaining
      .checked_sub(len)
      .ok_or("selection exceeds backup limit")?;
    offers.push((mime, bytes));
  }
  Ok(offers)
}

struct Selection {
  kind: ClipboardType,
  original: Vec<Offer>,
  marker: String,
  owner: Option<Child>,
}

impl Selection {
  fn temporary(
    kind: ClipboardType,
    original: Vec<Offer>,
    sequence: &str,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let generation = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let marker = format!(
      "application/x-argvus-emoji-{}-{generation}",
      std::process::id()
    );
    let offers = ["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"]
      .into_iter()
      .map(|mime| (mime.to_owned(), sequence.as_bytes().to_vec()))
      .chain(std::iter::once((marker.clone(), Vec::new())))
      .collect::<Vec<_>>();
    let owner = start_owner(kind == ClipboardType::Primary, &offers)?;
    let selection = Self {
      kind,
      original,
      marker,
      owner: Some(owner),
    };
    // The owner acknowledges preparation before dispatching its Wayland queue.
    for _ in 0..20 {
      if selection.is_current() {
        return Ok(selection);
      }
      thread::sleep(Duration::from_millis(10));
    }
    Err(format!("temporary Wayland selection {kind:?} was not installed").into())
  }

  fn is_current(&self) -> bool {
    paste::get_mime_types(self.kind, Seat::Unspecified)
      .is_ok_and(|types| types.contains(&self.marker))
  }
}

impl Drop for Selection {
  fn drop(&mut self) {
    if let Some(mut owner) = self.owner.take() {
      if self.is_current() && !self.original.is_empty() {
        match start_owner(self.kind == ClipboardType::Primary, &self.original) {
          Ok(_restored) => {
            // Let the compositor replace the offer before closing its old owner.
            for _ in 0..20 {
              if owner.try_wait().is_ok_and(|status| status.is_some()) {
                return;
              }
              thread::sleep(Duration::from_millis(10));
            }
          }
          Err(error) => eprintln!("argvus emoji-picker: could not restore selection: {error}"),
        }
      }
      let _ = owner.kill();
      let _ = owner.wait();
    }
  }
}

pub fn paste(sequence: &str, can_send: impl FnOnce() -> bool) -> Result<(), &'static str> {
  let regular = snapshot(ClipboardType::Regular).map_err(report)?;
  let primary = snapshot(ClipboardType::Primary);
  let regular = Selection::temporary(ClipboardType::Regular, regular, sequence).map_err(report)?;
  let primary = primary
    .and_then(|offers| Selection::temporary(ClipboardType::Primary, offers, sequence))
    .ok();
  if !regular.is_current() {
    return Err("insert_error");
  }
  let (modifier, key) = if primary.as_ref().is_some_and(Selection::is_current) {
    ("shift", "Insert")
  } else {
    ("ctrl", "v")
  };
  if !can_send() {
    return Err("focus_error");
  }
  Command::new("wtype")
    .args(["-M", modifier, "-P", key, "-p", key, "-m", modifier])
    .status()
    .ok()
    .filter(|status| status.success())
    .ok_or("insert_error")?;
  // GUI clients may request the selection asynchronously after processing the shortcut.
  thread::sleep(Duration::from_millis(350));
  Ok(())
}

fn report(error: Box<dyn std::error::Error>) -> &'static str {
  eprintln!("argvus emoji-picker: could not prepare selection: {error}");
  "insert_error"
}
