mod actions;
mod config;
mod device;
mod filesystem;
mod gui;
mod i18n;
mod menu;
mod monitor;
mod process;
mod renderer;
mod text;
mod udisks;

use std::collections::HashMap;
use std::io::Write;

use actions::{ActionKind, Actions};
use config::Config;
use device::{Device, build_devices};
use menu::Menu;
use monitor::Monitor;
use renderer::Renderer;
use udisks::UdisksClient;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_usage(out: &mut dyn Write) {
  let usage = i18n::tr("help.usage").replace("{VERSION}", VERSION);
  let _ = writeln!(out, "{}\n", usage);
}

pub(crate) async fn snapshot(cfg: &Config) -> Vec<Device> {
  let mut client = UdisksClient::new();
  if !client.connect().await {
    eprintln!("argvus-removable-devices: {}", client.last_error());
    return Vec::new();
  }
  let raw = client.enumerate().await;
  let mut mt: HashMap<String, i64> = HashMap::new();
  let mut seq: HashMap<String, u64> = HashMap::new();
  let mut next = 0u64;
  build_devices(&raw, cfg, &mut mt, &mut seq, &mut next)
}

fn pick<'a>(devs: &'a [Device], token: &str) -> Option<&'a Device> {
  if !token.is_empty() {
    for d in devs {
      if d.block == token || d.name == token || d.object_path == token {
        return Some(d);
      }
    }
    for d in devs {
      let base = d.object_path.rsplit('/').next().unwrap_or("");
      if base == token {
        return Some(d);
      }
    }
  }
  devs.first()
}

async fn run_watch(cfg: &Config, once: bool) -> i32 {
  let renderer = Renderer::new(cfg);
  let emit = |devs: &[Device]| {
    let result = renderer.render(devs);
    let mut stdout = std::io::stdout().lock();
    if stdout.write_all(result.json.as_bytes()).is_err() || stdout.write_all(b"\n").is_err() {
      // waybar restarted the stream; leave quietly.
      std::process::exit(0);
    }
    let _ = stdout.flush();
  };

  if once {
    emit(&snapshot(cfg).await);
    return 0;
  }

  let mut monitor = Monitor::new(cfg);
  match monitor.run(emit).await {
    Ok(()) => 0,
    Err(e) => {
      eprintln!("argvus-removable-devices: {}", e);
      1
    }
  }
}

async fn run_action(cfg: &Config, cmd: &str, token: &str) -> i32 {
  let kind = match cmd {
    "open" => ActionKind::Open,
    "mount" => ActionKind::Mount,
    "unmount" => ActionKind::Unmount,
    "eject" => ActionKind::Eject,
    "poweroff" => ActionKind::PowerOff,
    "lock" => ActionKind::Lock,
    "unlock" => ActionKind::Unlock,
    "copy" => ActionKind::Copy,
    _ => return -1,
  };
  let devs = snapshot(cfg).await;
  let Some(d) = pick(&devs, token) else {
    eprintln!(
      "argvus-removable-devices: {}",
      i18n::tr("device.no_removable_storage_devices_30")
    );
    return 1;
  };
  let actions = Actions::new(cfg);
  actions.perform(kind, d).await
}

fn run_list(devs: &[Device]) -> i32 {
  for d in devs {
    println!(
      "{}\t{}\t{}\t{}\t{}",
      d.block,
      d.name,
      d.filesystem,
      if d.mounted { &d.mount_point } else { "-" },
      text::human_bytes(d.capacity)
    );
  }
  0
}

#[tokio::main]
async fn main() {
  i18n::init();

  // Never die on a closed stdout (waybar restarting the stream).
  unsafe {
    libc::signal(libc::SIGPIPE, libc::SIG_IGN);
  }

  let args: Vec<String> = std::env::args().skip(1).collect();
  let mut config_path = String::new();
  let mut positional: Vec<String> = Vec::new();
  let mut fixed_x: Option<i32> = None;
  let mut fixed_y: Option<i32> = None;
  let mut bar_bottom: Option<i32> = None;
  let mut i = 0;
  while i < args.len() {
    let arg = &args[i];
    if arg == "--config" {
      i += 1;
      if i >= args.len() {
        eprintln!(
          "argvus-removable-devices: {}",
          i18n::tr("device.config_requires_a_path")
        );
        std::process::exit(2);
      }
      config_path = args[i].clone();
    } else if arg == "--x" {
      i += 1;
      if i >= args.len() {
        eprintln!(
          "argvus-removable-devices: {}",
          i18n::tr("device.x_requires_a_value")
        );
        std::process::exit(2);
      }
      fixed_x = args[i].parse().ok();
    } else if arg == "--y" {
      i += 1;
      if i >= args.len() {
        eprintln!(
          "argvus-removable-devices: {}",
          i18n::tr("device.y_requires_a_value")
        );
        std::process::exit(2);
      }
      fixed_y = args[i].parse().ok();
    } else if arg == "--bar-bottom" {
      i += 1;
      if i >= args.len() {
        eprintln!(
          "argvus-removable-devices: {}",
          i18n::tr("device.bar_bottom_requires_a_value")
        );
        std::process::exit(2);
      }
      bar_bottom = args[i].parse().ok();
    } else if arg == "-h" || arg == "--help" {
      print_usage(&mut std::io::stdout());
      return;
    } else if arg == "-v" || arg == "--version" {
      println!("argvus-removable-devices {}", VERSION);
      return;
    } else {
      positional.push(arg.clone());
    }
    i += 1;
  }

  let cmd = positional
    .first()
    .cloned()
    .unwrap_or_else(|| "watch".to_string());
  let token = positional.get(1).cloned().unwrap_or_default();

  let cfg = Config::load(&config_path);

  let code = match cmd.as_str() {
    "watch" => run_watch(&cfg, false).await,
    "once" => run_watch(&cfg, true).await,
    "list" => run_list(&snapshot(&cfg).await),
    "menu" => {
      let devs = snapshot(&cfg).await;
      match fixed_x.zip(fixed_y) {
        Some(pos) => {
          if cfg.mode == "gui" {
            gui::run(&cfg, &devs, &config_path, pos, bar_bottom)
          } else {
            Menu::new(&cfg).run(&devs).await
          }
        }
        // No anchor position (keybind / CLI launch, e.g.
        // SUPER+SHIFT+D): GUI mode can only open anchored to a
        // waybar click, so fall back to the rofi/dmenu menu.
        None => Menu::new(&cfg).run(&devs).await,
      }
    }
    "devices" => {
      let devs = snapshot(&cfg).await;
      Menu::new(&cfg).run_devices(&devs).await
    }
    _ => run_action(&cfg, &cmd, &token).await,
  };
  if code == -1 {
    eprintln!(
      "argvus-removable-devices: {}",
      i18n::tr("device.unknown_command").replace("{command}", &cmd)
    );
    print_usage(&mut std::io::stderr());
    std::process::exit(2);
  }
  std::process::exit(code);
}
