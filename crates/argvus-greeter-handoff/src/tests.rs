use super::control::{Command, parse_command};
use super::drawing::{mix_rgb, spinner_pixel};
use super::outputs::{CrtcSample, CrtcWatch};
use super::overlay::{OverlayPlane, free_overlay_plane, overlay_expired};
use super::*;

#[test]
fn palette_uses_the_splash_theme_table() {
  assert_eq!(Palette::new("ARGVUS Light", None).background, 0xffffff);
  assert_eq!(Palette::new("tokyo-night", None).background, 0x1a1b26);
  assert_eq!(Palette::new("tokyo-night", None).accent, 0x7aa2f7);
  assert_eq!(Palette::new("solitude", None).background, 0x101315);
  assert_eq!(Palette::new("solitude", None).accent, 0x798186);
  assert_eq!(Palette::new("argvus-dark", None).background, 0x191b27);
}

#[test]
fn unknown_themes_use_the_splash_defaults() {
  let palette = Palette::new("custom", None);
  assert_eq!(Some(palette.background), hex_to_rgb(DEFAULT_BACKGROUND));
  assert_eq!(Some(palette.accent), hex_to_rgb(DEFAULT_ACCENT));
}

#[test]
fn the_user_accent_survives_a_theme_change() {
  let palette = Palette::new("argvus-dark", Some(0x12_3456));
  assert_eq!(palette.accent, 0x12_3456);
  let changed = palette.with_theme("tokyo-night");
  assert_eq!(changed.background, 0x1a1b26);
  assert_eq!(changed.accent, 0x12_3456);
}

#[test]
fn spinner_pixels_show_the_squares_and_the_background() {
  let palette = Palette::new("argvus-dark", Some(0xff_0000));
  let rest = spinner::frame(0);
  // The top-left square covers this pixel completely.
  assert_eq!(spinner_pixel(&palette, &rest, -20.0, -20.0), 0xff_0000);
  assert_eq!(
    spinner_pixel(&palette, &rest, 25.0, 25.0),
    palette.background
  );
  // A pixel that straddles the left edge of the top-left square is half covered.
  assert_eq!(
    spinner_pixel(&palette, &rest, -20.5, -20.0),
    mix_rgb(palette.background, 0xff_0000, 0.5)
  );
}

fn mode(clock: u32, vrefresh: u32) -> ModeInfo {
  ModeInfo {
    clock,
    hdisplay: 1920,
    hsync_start: 2008,
    hsync_end: 2052,
    htotal: 2200,
    hskew: 0,
    vdisplay: 1080,
    vsync_start: 1084,
    vsync_end: 1089,
    vtotal: 1125,
    vscan: 0,
    vrefresh,
    flags: 5,
    mode_type: 0,
    name: [0; 32],
  }
}

#[test]
fn modes_with_the_same_timings_need_no_modeset() {
  let mut named = mode(185_580, 75);
  named.mode_type = 0x48;
  named.name[0] = b'1' as libc::c_char;
  assert!(mode(185_580, 75).same_timings(&named));
  assert!(!mode(185_580, 75).same_timings(&mode(148_500, 60)));
}

#[test]
fn commands_are_parsed_from_their_line() {
  assert_eq!(parse_command("ping\n"), Command::Ping);
  assert_eq!(parse_command("release\n"), Command::Release);
  assert_eq!(parse_command("retire"), Command::Retire);
  assert_eq!(parse_command("theme=nord\n"), Command::Theme("nord"));
  assert_eq!(parse_command("reboot"), Command::Unknown);
}

#[test]
fn the_overlay_is_a_free_xrgb_plane_of_the_crtc() {
  let plane = |plane_id, crtc_id, possible_crtcs, xrgb8888| OverlayPlane {
    plane_id,
    crtc_id,
    possible_crtcs,
    xrgb8888,
  };
  let planes = [
    plane(58, 0, 0b010, true),
    plane(68, 57, 0b001, true),
    plane(78, 0, 0b001, false),
    plane(88, 0, 0b001, true),
  ];
  assert_eq!(free_overlay_plane(&planes, 0), Some(88));
  assert_eq!(free_overlay_plane(&planes, 1), Some(58));
  assert_eq!(free_overlay_plane(&planes, 2), None);
  assert_eq!(free_overlay_plane(&planes, 40), None);
}

#[test]
fn the_overlay_leaves_on_its_own_only_after_a_timeout() {
  assert_eq!(overlay_expired(Duration::from_secs(1), None), None);
  assert_eq!(
    overlay_expired(Duration::from_secs(5), Some(Duration::from_secs(1))),
    None
  );
  assert_eq!(
    overlay_expired(Duration::from_secs(5), Some(RETIRE_TIMEOUT)),
    Some("handoff.retire-timeout")
  );
  assert_eq!(
    overlay_expired(REPLACEMENT_TIMEOUT, None),
    Some("handoff.replacement-timeout")
  );
}

#[test]
fn the_crtc_watch_reports_only_changes() {
  let on = CrtcSample {
    crtc_id: 57,
    framebuffer: 111,
    mode_valid: true,
  };
  let off = CrtcSample {
    framebuffer: 0,
    mode_valid: false,
    ..on
  };
  let mut watch = CrtcWatch::default();
  assert_eq!(watch.changes(vec![on]), vec![on]);
  assert!(watch.changes(vec![on]).is_empty());
  assert_eq!(watch.changes(vec![off]), vec![off]);
}

/// Binds a socket in `directory` and serves one request through
/// `handle_command`, returning the socket path so a client can connect.
fn serve_once(directory: &Path, palette_theme: &'static str) -> (PathBuf, thread::JoinHandle<()>) {
  let socket = directory.join("handoff.sock");
  let listener = UnixListener::bind(&socket).expect("bind test socket");
  let handle = thread::spawn(move || {
    let (stream, _) = listener.accept().expect("accept test client");
    let mut palette = Palette::new(palette_theme, None);
    if let Ok(Some(mut release)) = handle_command(stream, &mut palette) {
      // Mirrors main: the acknowledgement follows the DRM release.
      release
        .write_all(b"released\n")
        .expect("acknowledge release");
    }
  });
  (socket, handle)
}

fn test_directory(name: &str) -> PathBuf {
  let directory =
    std::env::temp_dir().join(format!("argvus-handoff-test-{name}-{}", std::process::id()));
  let _ = fs::remove_dir_all(&directory);
  fs::create_dir_all(&directory).expect("create test directory");
  directory
}

#[test]
fn a_silent_client_cannot_stall_the_surface() {
  let directory = test_directory("silent");
  let socket = directory.join("handoff.sock");
  let listener = UnixListener::bind(&socket).expect("bind test socket");
  let _client = UnixStream::connect(&socket).expect("connect silent client");
  let (stream, _) = listener.accept().expect("accept silent client");
  let started = Instant::now();
  let mut palette = Palette::new("argvus-dark", None);
  assert!(handle_command(stream, &mut palette).is_err());
  assert!(started.elapsed() < CLIENT_TIMEOUT + Duration::from_secs(1));
  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn sending_without_a_running_surface_succeeds() {
  let directory = test_directory("absent");
  assert!(send_command(&directory.join("missing.sock"), "release").is_ok());
  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn release_is_acknowledged_by_the_surface() {
  let directory = test_directory("release");
  let (socket, server) = serve_once(&directory, "argvus-dark");
  assert!(send_command(&socket, "release").is_ok());
  server.join().expect("surface thread");
  let _ = fs::remove_dir_all(&directory);
}

#[test]
fn theme_update_is_acknowledged_and_keeps_the_surface_running() {
  let directory = test_directory("theme");
  let (socket, server) = serve_once(&directory, "argvus-dark");
  assert!(send_command(&socket, "theme=tokyo-night").is_ok());
  server.join().expect("surface thread");
  let _ = fs::remove_dir_all(&directory);
}
