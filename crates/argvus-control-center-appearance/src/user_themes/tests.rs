use super::*;

fn write(path: &Path, bytes: &[u8]) {
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  fs::write(path, bytes).unwrap();
}

/// A tar.gz with the given (member, bytes) entries.
fn archive(path: &Path, members: &[(&str, &[u8])]) {
  let file = File::create(path).unwrap();
  let mut builder = Builder::new(GzEncoder::new(file, Compression::default()));
  for (member, bytes) in members {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_mode(0o644);
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    builder.append_data(&mut header, member, *bytes).unwrap();
  }
  builder.into_inner().unwrap().finish().unwrap();
}

#[test]
fn stage_accepts_one_theme_directory_only() {
  let root = tempdir().unwrap();
  let good = root.path().join("good.tar.gz");
  archive(
    &good,
    &[("mist/theme.toml", b"x"), ("mist/wallpaper.png", b"png")],
  );
  let staged = stage(&good).unwrap();
  assert!(staged.dir.ends_with("mist"));
  assert!(staged.dir.join("wallpaper.png").is_file());
  assert!(holds_a_theme(&good));

  let two = root.path().join("two.tar.gz");
  archive(&two, &[("a/theme.toml", b"x"), ("b/theme.toml", b"x")]);
  assert!(stage(&two).unwrap_err().contains("single theme"));

  let other = root.path().join("other.tar.gz");
  archive(&other, &[("notes.txt", b"x")]);
  assert!(!holds_a_theme(&other));

  let big = root.path().join("big.tar.gz");
  let large = vec![b'x'; (MAX_FILE + 1) as usize];
  archive(&big, &[("mist/theme.toml", &large)]);
  assert!(stage(&big).unwrap_err().contains("too large"));
}

#[test]
fn legacy_profiles_give_their_name_theme_accent_and_wallpaper() {
  let root = tempdir().unwrap();
  let v4 = root.path().join("v4.tar.gz");
  archive(
    &v4,
    &[
      (
        "argvus-theme-profile/manifest.json",
        br#"{"name":"dove","argvus":{"theme":"argvus-dark-float"}}"#,
      ),
      (
        "argvus-theme-profile/payload/config/argvus/config.json",
        br##"{"appearance":{"accent":"#B5655A","accent_custom":true}}"##,
      ),
      ("argvus-theme-profile/wallpaper/dove.jpg", b"jpg"),
    ],
  );
  let profile = LegacyProfile::read(&v4).unwrap();
  assert_eq!(profile.name, "dove");
  assert_eq!(profile.theme, "argvus-dark-float");
  assert_eq!(
    profile.accent.map(|color| color.hex()).as_deref(),
    Some("#B5655A")
  );
  assert_eq!(
    profile.wallpaper.map(|(name, _)| name).as_deref(),
    Some("dove.jpg")
  );

  let v1 = root.path().join("v1.tar.gz");
  archive(
    &v1,
    &[
      (
        "argvus-theme-profile/manifest.json",
        br#"{"name":"mint","argvus":{"theme":"argvus-dark-universe-float"}}"#,
      ),
      (
        "argvus-theme-profile/payload/config/argvus/.accent-color",
        b"#17d174\n",
      ),
    ],
  );
  let profile = LegacyProfile::read(&v1).unwrap();
  assert_eq!(canonical_theme_id(&profile.theme), "universe-float");
  assert_eq!(
    profile.accent.map(|color| color.hex()).as_deref(),
    Some("#17D174")
  );
  assert!(profile.wallpaper.is_none());
}

/// A session tree: an installed `sunset` theme with one consumer, the
/// active theme and accent, and a custom wallpaper.
fn session(root: &Path) {
  let system = root.join("system");
  write(
    &system.join("themes/sunset/theme.toml"),
    b"schema = 2\nid = \"sunset\"\nname = \"Sunset\"\ncategory = \"dark\"\n\
      [palette]\nbg = \"#000000\"\nfg = \"#FFFFFF\"\naccent = \"#E2BE8A\"\n\
      [overrides]\nterminal = \"overrides/terminal.conf\"\n",
  );
  write(
    &system.join("themes/sunset/overrides/terminal.conf"),
    b"fg {{ @accent }}\n",
  );
  write(&system.join("theme/layouts/sticky.toml"), b"gap = 1\n");
  write(&system.join("theme/layouts/float.toml"), b"gap = 8\n");
  write(&system.join("t/a.txt"), b"{{ @fg }} {{ layout.gap }}\n");
  write(
    &system.join("theme/consumers.d/terminal.toml"),
    b"id = \"terminal\"\ntemplate = \"t/a.txt\"\noutput = \"a.txt\"\n",
  );
  let data = root.join("config/argvus/data");
  write(&data.join(".active-theme"), b"sunset-float\n");
  write(&data.join(".accent-color"), b"#112233\n");
  write(&root.join("home/Pictures/mine.png"), b"png");
  write(
    &data.join(".wallpaper-custom"),
    root
      .join("home/Pictures/mine.png")
      .display()
      .to_string()
      .as_bytes(),
  );
  // SAFETY: the crate-wide lock serializes every test that changes the environment.
  unsafe {
    std::env::set_var("HOME", root.join("home"));
    std::env::set_var("XDG_DATA_HOME", root.join("share"));
    std::env::set_var("ARGVUS_CONFIG_HOME", root.join("config"));
    std::env::set_var("ARGVUS_SYSTEM_CONFIG", &system);
  }
}

#[test]
fn export_import_convert_and_delete_user_themes() {
  let _lock = crate::test_support::env_lock();
  let root = tempdir().unwrap();
  session(root.path());

  assert_eq!(
    preview_export_path("My Sunset"),
    Some(root.path().join("home/argvus-theme-my-sunset.tar.gz"))
  );
  let archive = export_named("My Sunset").unwrap();
  let (name, duplicate) = inspect_archive(&archive).unwrap();
  assert_eq!((name.as_str(), duplicate), ("My Sunset", false));
  assert_eq!(list_import_archives(), vec![archive.clone()]);

  let theme = import_archive(&archive).unwrap();
  assert_eq!(theme.id, "my-sunset");
  let installed = root.path().join("share/argvus/themes/my-sunset");
  let manifest = argvus_theme_core::manifest::ThemeManifest::load(&installed).unwrap();
  assert_eq!(manifest.token("accent").unwrap().hex(), "#112233");
  assert_eq!(manifest.wallpaper, Some(installed.join("wallpaper.png")));
  assert!(installed.join("overrides/terminal.conf").is_file());
  assert!(inspect_archive(&archive).unwrap().1, "now a duplicate");
  assert_eq!(custom_themes(), vec![theme.clone()]);
  assert_eq!(active_custom_theme(), None);

  // An exported official id is refused on import.
  let staging = tempdir().unwrap();
  let official = staging.path().join("sunset");
  copy_files(&installed, &official).unwrap();
  let text = fs::read_to_string(official.join("theme.toml")).unwrap();
  fs::write(
    official.join("theme.toml"),
    text.replace("my-sunset", "sunset"),
  )
  .unwrap();
  let official_archive = root.path().join("home/official.tar.gz");
  write_archive(&official, "sunset", &official_archive).unwrap();
  assert!(
    inspect_archive(&official_archive)
      .unwrap_err()
      .contains("official")
  );

  // A legacy appearance profile becomes a theme and moves to a backup.
  let legacy = root.path().join("share/argvus/themes/dove");
  fs::create_dir_all(&legacy).unwrap();
  archive_members(
    &legacy.join("profile.tar.gz"),
    &[
      (
        "argvus-theme-profile/manifest.json",
        br#"{"name":"dove","argvus":{"theme":"sunset-float"}}"#,
      ),
      (
        "argvus-theme-profile/payload/config/argvus/.accent-color",
        b"#B5655A\n",
      ),
      ("argvus-theme-profile/wallpaper/dove.jpg", b"jpg"),
    ],
  );
  let report = convert_legacy_profiles();
  assert_eq!(report, vec!["dove → dove".to_string()]);
  let dove = root.path().join("share/argvus/themes/dove");
  assert!(dove.join("theme.toml").is_file());
  assert!(dove.join("wallpaper.jpg").is_file());
  assert!(!dove.join("profile.tar.gz").exists());
  let backups = root.path().join("config/argvus/data/backups");
  assert_eq!(fs::read_dir(&backups).unwrap().count(), 1);
  assert!(convert_legacy_profiles().is_empty(), "runs once");

  delete_custom_theme(&theme).unwrap();
  assert!(!installed.exists());
  assert_eq!(custom_themes().len(), 1);
}

fn archive_members(path: &Path, members: &[(&str, &[u8])]) {
  archive(path, members);
}

#[test]
fn an_exported_theme_directory_is_archived_under_its_id() {
  let root = tempdir().unwrap();
  let dir = root.path().join("mist");
  write(&dir.join("theme.toml"), b"t");
  write(&dir.join("overrides/terminal.conf"), b"o");
  let output = root.path().join("argvus-theme-mist.tar.gz");
  write_archive(&dir, "mist", &output).unwrap();
  let staged = stage(&output).unwrap();
  assert_eq!(
    fs::read(staged.dir.join("overrides/terminal.conf")).unwrap(),
    b"o"
  );
  assert!(!root.path().join("argvus-theme-mist.gz.tmp").exists());
}
