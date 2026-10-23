use std::{
  cell::RefCell,
  env,
  fs::File,
  io::Write,
  os::fd::FromRawFd,
  rc::Rc,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use argvus_i18n::I18n;
use argvus_loading_theme_core::{
  DEFAULT_ACCENT, DEFAULT_BACKGROUND, DEFAULT_FOREGROUND, hex_to_rgb, spinner,
};
use argvus_system::monotonic_ns;
use clap::Parser;
use gtk::{
  Align, Application, ApplicationWindow, Box as GtkBox, DrawingArea, Label, Orientation, gdk, glib,
  prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "org.argvus.LoadingTheme";
const FADE_IN_MS: u32 = 180;
const FADE_OUT_MS: u32 = 220;

#[derive(Debug, Parser, Clone)]
#[command(
  name = "argvus-loading-theme",
  about = "Overlay shown while ARGVUS changes theme or starts the desktop"
)]
struct Cli {
  #[arg(long)]
  theme: Option<String>,
  #[arg(long)]
  background: Option<String>,
  #[arg(long)]
  foreground: Option<String>,
  #[arg(long)]
  accent: Option<String>,
  #[arg(long)]
  logo: Option<String>,
  /// Write READY after every monitor surface has been mapped.
  #[arg(long)]
  ready_file: Option<String>,
  /// Write READY to an inherited file descriptor after mapping.
  #[arg(long)]
  ready_fd: Option<i32>,
  /// Render only the spinner, without the theme-transition message.
  #[arg(long)]
  spinner_only: bool,
  /// Marks a spinner started during a greeter-to-session handoff.
  ///
  /// The flag is intentionally behavioural-neutral: process lifetime remains
  /// controlled by the existing SIGTERM path.
  #[arg(long)]
  session_handoff: bool,
}

#[derive(Clone)]
struct Palette {
  background: String,
  foreground: String,
  accent: String,
}

impl Palette {
  fn from_cli(cli: &Cli) -> Self {
    let theme = cli
      .theme
      .as_deref()
      .map(ToOwned::to_owned)
      .unwrap_or_else(active_theme_name);
    let mapped = argvus_loading_theme_core::theme_colors(&theme);
    let fallback_background = mapped.map(|colors| colors.0).unwrap_or(DEFAULT_BACKGROUND);
    let fallback_foreground = mapped.map(|colors| colors.1).unwrap_or(DEFAULT_FOREGROUND);
    let fallback_accent = mapped.map(|colors| colors.2).unwrap_or(DEFAULT_ACCENT);
    let mut palette = Self {
      background: cli
        .background
        .as_deref()
        .map(|value| argvus_loading_theme_core::valid_color(value, fallback_background))
        .unwrap_or_else(|| fallback_background.to_owned()),
      foreground: cli
        .foreground
        .as_deref()
        .map(|value| argvus_loading_theme_core::valid_color(value, fallback_foreground))
        .unwrap_or_else(|| fallback_foreground.to_owned()),
      accent: fallback_accent.to_owned(),
    };
    if let Some(accent) = cli.accent.as_deref() {
      palette.accent = argvus_loading_theme_core::valid_color(accent, &palette.accent);
    }
    palette
  }
}

fn active_theme_name() -> String {
  let config_home = env::var_os("ARGVUS_CONFIG_HOME")
    .or_else(|| env::var_os("XDG_CONFIG_HOME"))
    .or_else(|| {
      env::var_os("HOME").map(|home| {
        std::path::PathBuf::from(home)
          .join(".config")
          .into_os_string()
      })
    })
    .map(std::path::PathBuf::from);

  config_home
    .map(|path| path.join("argvus/data/.active-theme"))
    .and_then(|path| std::fs::read_to_string(path).ok())
    .and_then(|theme| {
      let theme = theme.trim();
      (!theme.is_empty()).then(|| theme.to_owned())
    })
    .unwrap_or_else(|| "argvus-dark".to_owned())
}

fn css(palette: &Palette) -> String {
  format!(
    "window, #splash-surface, .splash-surface {{ background-color: {}; }} * {{ color: {}; }}",
    palette.background, palette.foreground
  )
}

fn tr_applying() -> String {
  I18n::new("appearance")
    .map(|catalog| catalog.tr("theme.applying"))
    .unwrap_or_else(|_| "theme.applying".to_owned())
}

fn main() -> glib::ExitCode {
  configure_parent_death();
  let cli = Cli::parse();
  let palette = Palette::from_cli(&cli);
  let application = Application::builder().application_id(APP_ID).build();
  application.connect_activate(move |application| {
    build_surfaces(application, &cli, &palette);
  });
  // GTK parses its own argv after clap. Pass only the application name so
  // splash-specific options are not rejected by GApplication.
  application.run_with_args(&[APP_ID])
}

fn configure_parent_death() {
  // A theme switch owns this process. If that owner disappears, ask the
  // normal SIGTERM path to fade the overlay instead of leaving it orphaned.
  unsafe {
    let _ = libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
    if libc::getppid() == 1 {
      let _ = libc::raise(libc::SIGTERM);
    }
  }
}

fn build_surfaces(application: &Application, cli: &Cli, palette: &Palette) {
  let display = gdk::Display::default().expect("a Wayland display is required");
  let monitors = display.monitors();
  let total = monitors.n_items();
  let surfaces: Rc<RefCell<Vec<ApplicationWindow>>> = Rc::new(RefCell::new(Vec::new()));

  for index in 0..total {
    let Some(item) = monitors.item(index) else {
      continue;
    };
    let Ok(monitor) = item.downcast::<gdk::Monitor>() else {
      continue;
    };
    let window = make_surface(application, &monitor, palette, cli.spinner_only);
    window.present();
    surfaces.borrow_mut().push(window);
  }

  // Do not report readiness from main or an idle callback. A layer-shell
  // surface has not necessarily entered a compositor frame at either point.
  // Every output reports from its first GTK frame callback instead.
  let ready_file = cli.ready_file.clone();
  let ready_fd = cli.ready_fd;
  let pending_frames = Rc::new(RefCell::new(surfaces.borrow().len()));
  if *pending_frames.borrow() == 0 {
    signal_ready(ready_file.as_deref(), ready_fd);
  } else {
    for window in surfaces.borrow().iter() {
      let pending_frames = Rc::clone(&pending_frames);
      let ready_file = ready_file.clone();
      window.add_tick_callback(move |_, _| {
        let mut pending = pending_frames.borrow_mut();
        *pending = pending.saturating_sub(1);
        if *pending == 0 {
          let monotonic_ns = monotonic_ns();
          eprintln!(
            "argvus-loading-theme: monotonic_ns={monotonic_ns} first layer-shell frame callback"
          );
          signal_ready(ready_file.as_deref(), ready_fd);
        }
        glib::ControlFlow::Break
      });
    }
  }

  let fade_state = Rc::new(RefCell::new(FadeState::new(surfaces.borrow().clone())));
  install_signal_handlers(&fade_state);
  if cli.spinner_only {
    // Session startup must expose feedback on the first mapped frame; the
    // transition fade is reserved for interactive theme changes.
    for window in &fade_state.borrow().windows {
      window.set_opacity(1.0);
    }
  } else {
    start_fade_in(&fade_state);
  }
}

fn make_surface(
  application: &Application,
  monitor: &gdk::Monitor,
  palette: &Palette,
  spinner_only: bool,
) -> ApplicationWindow {
  let window = ApplicationWindow::builder()
    .application(application)
    .decorated(false)
    .focusable(false)
    .focus_on_click(false)
    .build();
  window.init_layer_shell();
  window.set_namespace(Some("argvus-loading-theme"));
  window.set_layer(Layer::Overlay);
  window.set_monitor(Some(monitor));
  window.set_keyboard_mode(KeyboardMode::None);
  // Keep the surface in the compositor's full-output overlay area. Hyprland
  // otherwise places a zero-zone overlay below the exclusive Waybar strip.
  window.set_exclusive_zone(-1);
  for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
    window.set_anchor(edge, true);
    window.set_margin(edge, 0);
  }

  let provider = gtk::CssProvider::new();
  provider.load_from_string(&css(palette));
  gtk::style_context_add_provider_for_display(
    &gtk::prelude::WidgetExt::display(&window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );

  let content = GtkBox::new(Orientation::Vertical, 12);
  content.set_widget_name("splash-surface");
  content.add_css_class("splash-surface");
  content.set_halign(Align::Center);
  content.set_valign(Align::Center);
  content.set_hexpand(true);
  content.set_vexpand(true);
  content.set_can_focus(false);
  content.set_focusable(false);

  content.append(&spinner_area(palette));
  if !spinner_only {
    let applying = Label::new(Some(&tr_applying()));
    applying.add_css_class("caption");
    applying.set_halign(Align::Center);
    content.append(&applying);
  }
  window.set_child(Some(&content));
  window.set_opacity(if spinner_only { 1.0 } else { 0.0 });
  window
}

/// The shared loading spinner (`argvus_loading_theme_core::spinner`).
///
/// It replaces `gtk::Spinner` so the greeter splash, the DRM handoff surface
/// and this overlay show one animation: same squares, same physical size and,
/// because the phase comes from `CLOCK_MONOTONIC`, the same frame at the same instant.
fn spinner_area(palette: &Palette) -> DrawingArea {
  let accent = hex_to_rgb(&palette.accent)
    .or_else(|| hex_to_rgb(DEFAULT_ACCENT))
    .unwrap_or_default();
  let area = DrawingArea::new();
  area.set_halign(Align::Center);
  area.set_valign(Align::Center);
  // Large enough at scale 1; on a scaled output the squares shrink to keep their
  // physical size and stay centered in the same area.
  let extent = spinner::EXTENT.ceil() as i32;
  area.set_content_width(extent);
  area.set_content_height(extent);
  area.set_draw_func(move |area, context, width, height| {
    let scale = f64::from(area.scale_factor().max(1));
    let monotonic_millis = u64::try_from(glib::monotonic_time() / 1000).unwrap_or_default();
    let frame = spinner::frame(monotonic_millis);
    let (center_x, center_y) = (f64::from(width) / 2.0, f64::from(height) / 2.0);
    let (red, green, blue) = rgb_channels(accent);
    context.set_source_rgb(red, green, blue);
    for square in frame.squares {
      context.rectangle(
        center_x + square.x / scale,
        center_y + square.y / scale,
        spinner::SQUARE / scale,
        spinner::SQUARE / scale,
      );
    }
    let _ = context.fill();
  });
  area.add_tick_callback(|area, _clock| {
    area.queue_draw();
    glib::ControlFlow::Continue
  });
  area
}

/// `0xrrggbb` as cairo channels between 0 and 1.
fn rgb_channels(rgb: u32) -> (f64, f64, f64) {
  let channel = |shift: u32| f64::from((rgb >> shift) & 0xff) / 255.0;
  (channel(16), channel(8), channel(0))
}

struct FadeState {
  windows: Vec<ApplicationWindow>,
  closing: bool,
}

impl FadeState {
  fn new(windows: Vec<ApplicationWindow>) -> Self {
    Self {
      windows,
      closing: false,
    }
  }
}

fn start_fade_in(state: &Rc<RefCell<FadeState>>) {
  animate(Rc::clone(state), 0.0, 1.0, FADE_IN_MS, false);
}

fn start_fade_out(state: &Rc<RefCell<FadeState>>) {
  let mut current = state.borrow_mut();
  if current.closing {
    return;
  }
  current.closing = true;
  drop(current);
  animate(Rc::clone(state), 1.0, 0.0, FADE_OUT_MS, true);
}

fn animate(state: Rc<RefCell<FadeState>>, from: f64, to: f64, duration_ms: u32, quit: bool) {
  let started = std::time::Instant::now();
  glib::timeout_add_local(Duration::from_millis(16), move || {
    let progress = (started.elapsed().as_millis() as f64 / duration_ms as f64).min(1.0);
    let value = from + (to - from) * progress;
    let finished = progress >= 1.0;
    for window in &state.borrow().windows {
      window.set_opacity(value);
    }
    if finished {
      if quit {
        for window in &state.borrow().windows {
          window.close();
        }
        gtk::Application::default().quit();
      }
      glib::ControlFlow::Break
    } else {
      glib::ControlFlow::Continue
    }
  });
}

fn install_signal_handlers(state: &Rc<RefCell<FadeState>>) {
  let requested = Arc::new(AtomicBool::new(false));
  for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
    signal_hook::flag::register(signal, Arc::clone(&requested))
      .expect("signal handler registration must succeed");
  }
  let signal_state = Rc::clone(state);
  glib::timeout_add_local(Duration::from_millis(40), move || {
    if requested.load(Ordering::Relaxed) {
      start_fade_out(&signal_state);
      glib::ControlFlow::Break
    } else {
      glib::ControlFlow::Continue
    }
  });
}

fn signal_ready(path: Option<&str>, fd: Option<i32>) {
  if let Some(path) = path
    && let Ok(mut file) = File::create(path)
  {
    let _ = file.write_all(b"READY\n");
  }
  if let Some(fd) = fd.filter(|value| *value >= 0) {
    // The descriptor is explicitly handed to this process by the caller.
    let mut file = unsafe { File::from_raw_fd(fd) };
    let _ = file.write_all(b"READY\n");
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accepts_supported_hex_colors() {
    assert_eq!(
      argvus_loading_theme_core::valid_color("#abc", DEFAULT_BACKGROUND),
      "#abc"
    );
    assert_eq!(
      argvus_loading_theme_core::valid_color("#AABBCC", DEFAULT_BACKGROUND),
      "#AABBCC"
    );
    assert_eq!(
      argvus_loading_theme_core::valid_color("#AABBCCDD", DEFAULT_BACKGROUND),
      "#AABBCCDD"
    );
  }

  #[test]
  fn rejects_css_injection_and_bad_lengths() {
    assert_eq!(
      argvus_loading_theme_core::valid_color("red", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
    assert_eq!(
      argvus_loading_theme_core::valid_color("#123456;", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
    assert_eq!(
      argvus_loading_theme_core::valid_color("#12345", DEFAULT_BACKGROUND),
      DEFAULT_BACKGROUND
    );
  }

  #[test]
  fn cli_defaults_are_safe() {
    let cli = Cli::try_parse_from(["argvus-loading-theme", "--theme", "argvus-dark"]).unwrap();
    assert_eq!(Palette::from_cli(&cli).background, "#191b27");
    assert_eq!(Palette::from_cli(&cli).foreground, "#3590bd");
    assert_eq!(Palette::from_cli(&cli).accent, DEFAULT_ACCENT);
  }

  #[test]
  fn known_theme_names_select_their_main_colors() {
    let cases = [
      ("ARGVUS Dark", "#191b27", "#3590bd", DEFAULT_ACCENT),
      ("ARGVUS Dracula", "#282A36", "#F8F8F2", "#BD93F9"),
      ("silver-dark-float", "#595959", "#121518", "#121518"),
      ("rose-pine", "#191724", "#E0DEF4", "#C4A7E7"),
      ("ARGVUS Dark Slate", "#3b4352", "#7391a5", DEFAULT_ACCENT),
      ("universe", "#000000", "#ffffff", DEFAULT_ACCENT),
      ("gruvbox-high-dark", "#282828", "#EBDBB2", "#D79921"),
      ("gruvbox-dark", "#282828", "#EBDBB2", "#D4BE98"),
      ("solitude", "#101315", "#CACCCC", "#798186"),
      ("sunset", "#0F0F0F", "#EADCCC", "#E2BE8A"),
      ("matrix", "#0B0C16", "#DDF7FF", "#82FB9C"),
      ("monokai-dark", "#2D2A2E", "#FCFCFA", "#78DCE8"),
      ("ARGVUS Light", "#ffffff", "#000000", DEFAULT_ACCENT),
    ];

    for (theme, background, foreground, accent) in cases {
      let cli = Cli::try_parse_from(["argvus-loading-theme", "--theme", theme]).unwrap();
      let palette = Palette::from_cli(&cli);
      assert_eq!(palette.background, background);
      assert_eq!(palette.foreground, foreground);
      assert_eq!(palette.accent, accent);
    }
  }

  #[test]
  fn explicit_transition_colors_override_known_theme_fallbacks() {
    let cli = Cli::try_parse_from([
      "argvus-loading-theme",
      "--theme",
      "silver-dark",
      "--background",
      "#abcdef",
      "--foreground",
      "#123456",
      "--accent",
      "#654321",
    ])
    .unwrap();
    let palette = Palette::from_cli(&cli);
    assert_eq!(palette.background, "#abcdef");
    assert_eq!(palette.foreground, "#123456");
    assert_eq!(palette.accent, "#654321");
  }

  #[test]
  fn explicit_accent_overrides_known_theme_accent() {
    let cli = Cli::try_parse_from([
      "argvus-loading-theme",
      "--theme",
      "silver-dark",
      "--accent",
      "#123456",
    ])
    .unwrap();
    assert_eq!(Palette::from_cli(&cli).accent, "#123456");
  }

  #[test]
  fn unknown_theme_keeps_explicit_cli_colors() {
    let cli = Cli::try_parse_from([
      "argvus-loading-theme",
      "--theme",
      "custom-theme",
      "--background",
      "#abcdef",
      "--foreground",
      "#123456",
    ])
    .unwrap();
    let palette = Palette::from_cli(&cli);
    assert_eq!(palette.background, "#abcdef");
    assert_eq!(palette.foreground, "#123456");
  }
}
