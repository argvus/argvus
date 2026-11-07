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
    if let Ok(Some(mut release)) = handle_command(stream, &mut palette, ClientPolicy::current(None))
    {
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
    Path::new("/tmp/argvus").join(format!("handoff-test-{name}-{}", std::process::id()));
  let _ = fs::remove_dir_all(&directory);
  fs::create_dir_all(&directory).expect("create test directory");
  directory
}

#[test]
fn release_cancels_acquisition_without_an_available_drm_card() {
  let directory = test_directory("acquiring-release");
  let socket = directory.join("handoff.sock");
  let listener = prepare_socket(&socket).unwrap();
  let mut ping = UnixStream::connect(&socket).unwrap();
  ping.write_all(b"ping\n").unwrap();
  let mut release = UnixStream::connect(&socket).unwrap();
  release.write_all(b"release\n").unwrap();
  release
    .set_read_timeout(Some(Duration::from_secs(2)))
    .unwrap();

  let started = Instant::now();
  let result = acquire_card(
    &[],
    &mut Vec::new(),
    None,
    &listener,
    &mut Palette::new("argvus-dark", None),
    ClientPolicy::current(None),
  )
  .unwrap();
  let outputs::Acquisition::Released(mut stream) = result else {
    panic!("no DRM card can have been acquired");
  };
  assert!(started.elapsed() < Duration::from_secs(2));
  stream.write_all(b"released\n").unwrap();
  let mut answer = String::new();
  BufReader::new(&mut release).read_line(&mut answer).unwrap();
  assert_eq!(answer, "released\n");
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn an_unprivileged_prepared_observer_reopens_the_same_gpu_to_acquire_master() {
  use std::os::fd::FromRawFd;
  use std::os::unix::fs::MetadataExt;

  let directory = test_directory("unprivileged-master");
  // Both files stand in for cards. The ioctl seam reproduces the kernel's
  // EACCES for an old non-master descriptor, without touching /dev/dri.
  let cards = [directory.join("card0"), directory.join("card1")];
  for card in &cards {
    fs::write(card, b"").unwrap();
  }
  let observer = OpenOptions::new()
    .read(true)
    .write(true)
    .open(&cards[1])
    .unwrap();
  let original_fd = observer.as_raw_fd();
  let mut observers = vec![outputs::Observer {
    name: "card1".into(),
    file: observer,
    watch: CrtcWatch::default(),
  }];
  let listener = prepare_socket(&directory.join("handoff.sock")).unwrap();
  let mut attempts = 0;
  let started = Instant::now();
  let acquired = outputs::acquire_card_with(
    &cards,
    &mut observers,
    Some(original_fd),
    &listener,
    &mut Palette::new("argvus-dark", None),
    ClientPolicy::current(None),
    |fd| {
      attempts += 1;
      if fd == original_fd {
        Err(io::Error::from_raw_os_error(libc::EACCES))
      } else {
        Ok(())
      }
    },
  )
  .unwrap();
  let outputs::Acquisition::Acquired { fd, .. } = acquired else {
    panic!("expected a reopened master");
  };
  // SAFETY: acquisition transfers ownership of this new descriptor to us.
  let master = unsafe { fs::File::from_raw_fd(fd) };
  assert_ne!(fd, original_fd);
  assert_eq!(
    master.metadata().unwrap().ino(),
    fs::metadata(&cards[1]).unwrap().ino()
  );
  assert_ne!(
    master.metadata().unwrap().ino(),
    fs::metadata(&cards[0]).unwrap().ino()
  );
  assert!(
    observers[0].file.metadata().is_ok(),
    "keep the old scanout descriptor alive"
  );
  assert_eq!(attempts, 2);
  assert!(started.elapsed() < Duration::from_secs(1));
  assert!(prepared_for_fd(fd, Some((original_fd, "old framebuffer"))).is_none());
  drop(master);
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn acquisition_ping_is_acknowledged_only_after_the_first_frame() {
  let directory = test_directory("acquiring-ping");
  let socket = directory.join("handoff.sock");
  let listener = prepare_socket(&socket).unwrap();
  let mut client = UnixStream::connect(&socket).unwrap();
  client.write_all(b"ping\n").unwrap();
  let mut requests = control::AcquisitionRequests::default();
  assert!(
    requests
      .poll(
        &listener,
        &mut Palette::new("argvus-dark", None),
        ClientPolicy::current(None)
      )
      .unwrap()
      .is_none()
  );
  client.set_nonblocking(true).unwrap();
  let mut answer = [0; 3];
  assert_eq!(
    std::io::Read::read(&mut client, &mut answer)
      .unwrap_err()
      .kind(),
    io::ErrorKind::WouldBlock
  );
  requests.first_frame();
  client.set_nonblocking(false).unwrap();
  client
    .set_read_timeout(Some(Duration::from_secs(1)))
    .unwrap();
  std::io::Read::read_exact(&mut client, &mut answer).unwrap();
  assert_eq!(&answer, b"ok\n");
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn an_acquisition_client_that_disconnects_does_not_prevent_release() {
  let directory = test_directory("acquiring-disconnect");
  let socket = directory.join("handoff.sock");
  let listener = prepare_socket(&socket).unwrap();
  let mut requests = control::AcquisitionRequests::default();
  let mut palette = Palette::new("argvus-dark", None);
  let policy = ClientPolicy::current(None);
  let client = UnixStream::connect(&socket).unwrap();
  drop(client);
  assert!(
    requests
      .poll(&listener, &mut palette, policy)
      .unwrap()
      .is_none()
  );
  let mut client = UnixStream::connect(&socket).unwrap();
  client.write_all(b"release\n").unwrap();
  assert!(
    requests
      .poll(&listener, &mut palette, policy)
      .unwrap()
      .is_some()
  );
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn acquisition_theme_update_preserves_the_user_accent() {
  let directory = test_directory("acquiring-theme");
  let socket = directory.join("handoff.sock");
  let listener = prepare_socket(&socket).unwrap();
  let mut client = UnixStream::connect(&socket).unwrap();
  client.write_all(b"theme=tokyo-night\n").unwrap();
  let mut palette = Palette::new("argvus-dark", Some(0x12_3456));
  assert!(
    control::AcquisitionRequests::default()
      .poll(&listener, &mut palette, ClientPolicy::current(None))
      .unwrap()
      .is_none()
  );
  assert_eq!(palette.background, 0x1a1b26);
  assert_eq!(palette.accent, 0x12_3456);
  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn failed_presentation_closes_the_ping_without_acknowledging_a_frame() {
  let directory = test_directory("acquiring-no-frame");
  let socket = directory.join("handoff.sock");
  let listener = prepare_socket(&socket).unwrap();
  let mut client = UnixStream::connect(&socket).unwrap();
  client.write_all(b"ping\n").unwrap();
  client
    .set_read_timeout(Some(Duration::from_secs(1)))
    .unwrap();
  let mut requests = control::AcquisitionRequests::default();
  requests
    .poll(
      &listener,
      &mut Palette::new("argvus-dark", None),
      ClientPolicy::current(None),
    )
    .unwrap();
  drop(requests);
  let mut answer = String::new();
  assert_eq!(BufReader::new(client).read_line(&mut answer).unwrap(), 0);
  fs::remove_dir_all(directory).unwrap();
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
  assert!(handle_command(stream, &mut palette, ClientPolicy::current(None)).is_err());
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

#[test]
fn p10_unrelated_peer_cannot_release_or_change_the_surface() {
  if ClientPolicy::current(None).owner_uid == 0 {
    return;
  } // root is intentionally authorized.
  for command in ["release\n", "theme=argvus-light\n", "retire\n"] {
    let (mut client, server) = UnixStream::pair().unwrap();
    client.write_all(command.as_bytes()).unwrap();
    let mut palette = Palette::new("argvus-dark", None);
    let policy = ClientPolicy {
      owner_uid: u32::MAX,
      session_uid: None,
    };
    let result = handle_command(server, &mut palette, policy);
    assert!(matches!(result, Err(ref error) if error.kind() == io::ErrorKind::PermissionDenied));
  }
}

#[test]
fn p10_prepared_surface_is_never_reused_on_another_descriptor() {
  assert_eq!(prepared_for_fd(11, Some((10, "buffer from card0"))), None);
  assert_eq!(
    prepared_for_fd(10, Some((10, "buffer from card0"))),
    Some("buffer from card0")
  );
}

#[test]
fn p10_only_owner_root_and_authenticated_uid_are_authorized() {
  let policy = ClientPolicy {
    owner_uid: 900,
    session_uid: Some(1000),
  };
  for uid in [0, 900, 1000] {
    assert!(policy.allows(uid));
  }
  for uid in [901, 1001, 65534] {
    assert!(!policy.allows(uid));
  }
  assert!(
    !ClientPolicy {
      owner_uid: 900,
      session_uid: None
    }
    .allows(1000)
  );
}

#[test]
fn p10_real_peer_credentials_allow_the_selected_session_uid() {
  let uid = ClientPolicy::current(None).owner_uid;
  let (mut client, mut server) = UnixStream::pair().unwrap();
  client.write_all(b"release\n").unwrap();
  assert_eq!(
    control::read_command(
      &mut server,
      ClientPolicy {
        owner_uid: u32::MAX,
        session_uid: Some(uid)
      }
    )
    .unwrap(),
    "release\n"
  );
}

#[test]
fn p10_authorized_commands_have_a_bounded_size() {
  let (mut client, mut server) = UnixStream::pair().unwrap();
  client
    .write_all(format!("theme={}\n", "x".repeat(5000)).as_bytes())
    .unwrap();
  assert!(control::read_command(&mut server, ClientPolicy::current(None)).is_err());
}

#[test]
fn p10_refused_acknowledgement_is_not_success() {
  let directory = test_directory("refused");
  let path = directory.join("handoff.sock");
  let listener = UnixListener::bind(&path).unwrap();
  let server = thread::spawn(move || {
    let (mut stream, _) = listener.accept().unwrap();
    let mut line = String::new();
    BufReader::new(&mut stream).read_line(&mut line).unwrap();
    stream.write_all(b"error\n").unwrap();
  });
  assert_eq!(
    send_command(&path, "release").unwrap_err().kind(),
    io::ErrorKind::PermissionDenied
  );
  server.join().unwrap();
  fs::remove_dir_all(directory).unwrap();
}
