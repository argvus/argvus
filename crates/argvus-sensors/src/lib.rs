//! The themed, live `sensors -j` temperature overview.

use argvus_i18n::I18n;
use argvus_theme::Theme;
use argvus_tui::chrome::{draw_live_footer, draw_live_header};
use crossterm::{
  event::{self, Event, KeyCode, KeyEventKind},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
  Terminal,
  backend::CrosstermBackend,
  layout::{Constraint, Layout},
  style::{Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};
use serde_json::Value;
use std::{
  io,
  process::Command,
  sync::mpsc::{self, Receiver},
  thread,
  time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct Catalog(Option<I18n>);

impl Catalog {
  fn load() -> Self {
    Self(
      I18n::new("sensors")
        .inspect_err(|error| log::error!("cannot load sensors translations: {error}"))
        .ok(),
    )
  }

  fn tr(&self, key: &str) -> String {
    self
      .0
      .as_ref()
      .map_or_else(|| key.to_owned(), |catalog| catalog.tr(key))
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
  pub label: String,
  pub celsius: f64,
  pub max: Option<f64>,
  pub crit: Option<f64>,
  pub alarm: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chip {
  pub name: String,
  pub readings: Vec<Reading>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorsState {
  Loading,
  Updated(Vec<Chip>),
  Error(String),
}

/// Parses lm_sensors JSON without making assumptions about hardware names.
pub fn parse_sensors_json(json: &str) -> Result<Vec<Chip>, String> {
  let root: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
  let object = root
    .as_object()
    .ok_or_else(|| "expected a JSON object".to_string())?;
  let mut chips = Vec::new();
  for (name, value) in object {
    let Some(properties) = value.as_object() else {
      continue;
    };
    let mut readings = Vec::new();
    for (_sensor_label, property) in properties {
      let Some(fields) = property.as_object() else {
        continue;
      };
      for (input_name, input) in fields {
        let Some(label) = input_name.strip_suffix("_input") else {
          continue;
        };
        let Some(celsius) = numeric(Some(input)) else {
          continue;
        };
        readings.push(Reading {
          label: _sensor_label.clone(),
          celsius,
          max: numeric(fields.get(&format!("{label}_max"))),
          crit: numeric(fields.get(&format!("{label}_crit"))),
          alarm: alarm(fields, label),
        });
      }
    }
    if !readings.is_empty() {
      chips.push(Chip {
        name: name.clone(),
        readings,
      });
    }
  }
  Ok(chips)
}

fn numeric(value: Option<&Value>) -> Option<f64> {
  value
    .and_then(Value::as_f64)
    .filter(|number| number.is_finite())
}

fn alarm(fields: &serde_json::Map<String, Value>, sensor: &str) -> bool {
  [format!("{sensor}_alarm"), format!("{sensor}_crit_alarm")]
    .iter()
    .filter_map(|key| fields.get(key))
    .any(|value| {
      value
        .as_bool()
        .unwrap_or_else(|| numeric(Some(value)).is_some_and(|number| number > 0.0))
    })
}

fn collect(catalog: &Catalog) -> Result<Vec<Chip>, String> {
  let output = Command::new("sensors")
    .arg("-j")
    .output()
    .map_err(|error| format!("{}: {error}", catalog.tr("sensors.command_failed")))?;
  if !output.status.success() {
    return Err(format!(
      "{} ({}): {}",
      catalog.tr("sensors.command_failed"),
      output.status,
      String::from_utf8_lossy(&output.stderr).trim()
    ));
  }
  let json = String::from_utf8(output.stdout).map_err(|error| format!("invalid UTF-8: {error}"))?;
  let chips = parse_sensors_json(&json)
    .map_err(|error| format!("{}: {error}", catalog.tr("sensors.invalid_data")))?;
  if chips.is_empty() {
    return Err(catalog.tr("sensors.no_sensors"));
  }
  Ok(chips)
}

pub fn run_command(arguments: Vec<String>) -> std::process::ExitCode {
  let _ = env_logger::try_init();
  if !arguments.is_empty() {
    eprintln!("usage: argvus sensors");
    return std::process::ExitCode::from(2);
  }
  match run_tui() {
    Ok(()) => std::process::ExitCode::SUCCESS,
    Err(error) => {
      log::error!("sensors TUI failed: {error:#}");
      eprintln!("argvus sensors: {error}");
      std::process::ExitCode::FAILURE
    }
  }
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
  let (sender, receiver) = mpsc::channel();
  let catalog = Catalog::load();
  let theme = Theme::load();
  thread::Builder::new().name("sensors-poll".into()).spawn({
    let catalog = catalog.clone();
    move || poll_loop(sender, catalog)
  })?;
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  let result = event_loop(&mut terminal, &receiver, &catalog, &theme);
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;
  Ok(result?)
}

fn poll_loop(sender: mpsc::Sender<SensorsState>, catalog: Catalog) {
  loop {
    let result = collect(&catalog);
    let mut state = SensorsState::Loading;
    apply_poll_result(&mut state, result);
    if let SensorsState::Error(error) = &state {
      log::error!("sensors poll failed: {error}");
    }
    if sender.send(state).is_err() {
      break;
    }
    thread::sleep(POLL_INTERVAL);
  }
}

fn apply_poll_result(state: &mut SensorsState, result: Result<Vec<Chip>, String>) {
  *state = match result {
    Ok(chips) => SensorsState::Updated(chips),
    Err(error) => SensorsState::Error(error),
  };
}

fn closes_panel(code: KeyCode) -> bool {
  matches!(code, KeyCode::Char('q') | KeyCode::Esc)
}

fn event_loop(
  terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
  receiver: &Receiver<SensorsState>,
  catalog: &Catalog,
  theme: &Theme,
) -> io::Result<()> {
  let mut state = SensorsState::Loading;
  loop {
    while let Ok(next) = receiver.try_recv() {
      state = next;
    }
    terminal.draw(|frame| draw(frame, &state, catalog, theme))?;
    if event::poll(Duration::from_millis(100))? {
      match event::read()? {
        Event::Key(key) if key.kind != KeyEventKind::Release && closes_panel(key.code) => {
          return Ok(());
        }
        Event::Resize(_, _) => terminal.clear()?,
        _ => {}
      }
    }
  }
}

fn draw(frame: &mut ratatui::Frame<'_>, state: &SensorsState, catalog: &Catalog, theme: &Theme) {
  let area = frame.area();
  let chunks = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(1),
    Constraint::Length(1),
  ])
  .split(area);
  draw_live_header(
    frame,
    chunks[0],
    theme,
    &catalog.tr("sensors.title"),
    Some(&catalog.tr("sensors.live")),
  );
  match state {
    SensorsState::Loading => frame.render_widget(
      Paragraph::new(catalog.tr("sensors.loading")).style(Style::default().fg(theme.muted)),
      chunks[1],
    ),
    SensorsState::Error(error) => frame.render_widget(
      Paragraph::new(format!(
        "{}\n\n{}",
        catalog.tr("sensors.unavailable"),
        error
      ))
      .style(Style::default().fg(theme.warning))
      .wrap(Wrap { trim: true }),
      chunks[1],
    ),
    SensorsState::Updated(chips) => render_chips(frame, chunks[1], chips, theme),
  }
  draw_live_footer(frame, chunks[2], theme, &catalog.tr("sensors.shortcuts"));
}

fn render_chips(
  frame: &mut ratatui::Frame<'_>,
  area: ratatui::layout::Rect,
  chips: &[Chip],
  theme: &Theme,
) {
  let rows = chips
    .iter()
    .map(|chip| Constraint::Length((chip.readings.len() as u16 + 2).max(3)))
    .collect::<Vec<_>>();
  let areas = Layout::vertical(rows).split(area);
  for (chip, chip_area) in chips.iter().zip(areas.iter()) {
    let block = Block::default()
      .title(format!(" {} ", chip.name))
      .borders(Borders::ALL)
      .border_style(Style::default().fg(theme.border_active));
    let inner = block.inner(*chip_area);
    frame.render_widget(block, *chip_area);
    for (index, reading) in chip.readings.iter().enumerate() {
      let y = inner.y + index as u16;
      if y >= inner.y + inner.height {
        break;
      }
      let threshold = reading.crit.or(reading.max);
      let hot = reading.alarm || threshold.is_some_and(|limit| reading.celsius >= limit);
      let accent = if hot { theme.error } else { theme.accent };
      let show_gauge = threshold.is_some_and(|limit| limit > 0.0) && inner.width > 88;
      let mut line = vec![Span::styled(
        format!(" {:<18} ", reading.label),
        Style::default().fg(theme.foreground),
      )];
      line.push(Span::styled(
        format!("{:>6.1} °C", reading.celsius),
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
      ));
      if let Some(max) = reading.max {
        line.push(Span::styled(
          format!("  max {:.1}°", max),
          Style::default().fg(theme.muted),
        ));
      }
      if let Some(crit) = reading.crit {
        line.push(Span::styled(
          format!("  crit {:.1}°", crit),
          Style::default().fg(theme.warning),
        ));
      }
      if reading.alarm {
        line.push(Span::styled(
          "  ALARM",
          Style::default()
            .fg(theme.error)
            .add_modifier(Modifier::BOLD),
        ));
      }
      frame.render_widget(
        Paragraph::new(Line::from(line)),
        ratatui::layout::Rect::new(
          inner.x,
          y,
          if show_gauge {
            inner.width.saturating_sub(17)
          } else {
            inner.width
          },
          1,
        ),
      );
      if show_gauge {
        let percent =
          (reading.celsius.max(0.0) / threshold.unwrap_or(1.0) * 100.0).clamp(0.0, 100.0) as u16;
        frame.render_widget(
          Gauge::default()
            .gauge_style(Style::default().fg(accent).bg(theme.surface))
            .percent(percent),
          ratatui::layout::Rect::new(inner.x + inner.width - 15, y, 15, 1),
        );
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ratatui::{Terminal, backend::TestBackend};

  const FIXTURE: &str = r#"{"chip one":{"Adapter":"ISA adapter","Package id 0":{"temp1_input":45.25,"temp1_max":80,"temp1_crit":100,"temp1_alarm":false},"Core 0":{"temp2_input":91,"temp2_max":80,"temp2_crit":null,"temp2_crit_alarm":1}},"chip two":{"GPU":{"temp1_input":55.0},"Broken":{"temp2_input":"invalid"}}}"#;

  #[test]
  fn parses_multiple_chips_readings_optional_bounds_and_alarm() {
    let chips = parse_sensors_json(FIXTURE).unwrap();
    assert_eq!(chips.len(), 2);
    assert_eq!(chips[0].readings.len(), 2);
    let core = chips[0]
      .readings
      .iter()
      .find(|reading| reading.label == "Core 0")
      .unwrap();
    let package = chips[0]
      .readings
      .iter()
      .find(|reading| reading.label == "Package id 0")
      .unwrap();
    assert_eq!(package.max, Some(80.0));
    assert_eq!(core.crit, None);
    assert!(core.alarm);
    assert_eq!(chips[1].readings.len(), 1);
  }

  #[test]
  fn invalid_json_and_empty_sensor_data_are_reported() {
    assert!(parse_sensors_json("{").is_err());
    assert!(parse_sensors_json("{}").unwrap().is_empty());
    assert!(
      parse_sensors_json(r#"{"chip":{"temp":{"temp1_input":"bad"}}}"#)
        .unwrap()
        .is_empty()
    );
  }

  #[test]
  fn updated_error_retry_and_resize_keep_model_and_exit_keys_are_supported() {
    let readings = parse_sensors_json(FIXTURE).unwrap();
    let mut state = SensorsState::Loading;
    apply_poll_result(&mut state, Ok(readings.clone()));
    assert_eq!(state, SensorsState::Updated(readings.clone()));
    apply_poll_result(&mut state, Err("offline".into()));
    assert_eq!(state, SensorsState::Error("offline".into()));
    apply_poll_result(&mut state, Ok(readings.clone()));
    assert_eq!(state, SensorsState::Updated(readings));
    assert!(closes_panel(KeyCode::Char('q')));
    assert!(closes_panel(KeyCode::Esc));
    assert!(!closes_panel(KeyCode::Enter));
    let expected = state.clone();
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let catalog = Catalog::load();
    let theme = Theme::load();
    terminal
      .draw(|frame| draw(frame, &state, &catalog, &theme))
      .unwrap();
    terminal
      .resize(ratatui::layout::Rect::new(0, 0, 42, 10))
      .unwrap();
    terminal
      .draw(|frame| draw(frame, &state, &catalog, &theme))
      .unwrap();
    assert_eq!(state, expected);
  }
}
