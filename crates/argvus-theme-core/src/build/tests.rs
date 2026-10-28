use super::*;
use crate::registry;

const THEME: &str = r##"
schema = 2
id = "sunset"
name = "Sunset"
category = "dark"

[palette]
bg = "#0F0F0F"
fg = "#EADCCC"
accent = "#E2BE8A"
accent_dim = "@accent | alpha(0.13)"
"##;

struct Fixture {
  root: tempfile::TempDir,
  manifest: ThemeManifest,
  consumers: Vec<Consumer>,
  layout: Layout,
}

impl Fixture {
  fn new() -> Self {
    let root = tempfile::tempdir().unwrap();
    let system = root.path().join("system");
    let theme_dir = system.join("themes/sunset");
    fs::create_dir_all(&theme_dir).unwrap();
    fs::write(theme_dir.join("theme.toml"), THEME).unwrap();
    fs::create_dir_all(system.join("taskbar")).unwrap();
    fs::write(
      system.join("taskbar/theme.css"),
      "@define-color th-accent {{ @accent }};\n@define-color th-dim {{ @accent_dim | rgba }};\n",
    )
    .unwrap();
    fs::write(
      system.join("taskbar/borders.lua"),
      "return { border = \"{{ @accent | hypr }}\", rounding = {{ layout.rounding }}, variant = \"{{ theme.variant }}\" }\n",
    )
    .unwrap();
    let registry_dir = system.join("theme/consumers.d");
    fs::create_dir_all(&registry_dir).unwrap();
    fs::write(
      registry_dir.join("taskbar.toml"),
      "id = \"taskbar\"\ntemplate = \"taskbar/theme.css\"\noutput = \"taskbar/theme.css\"\n\
       [reload]\nkind = \"rewrite\"\npath = \"argvus/data/waybar/argvus-taskbar.css\"\n",
    )
    .unwrap();
    fs::write(
      registry_dir.join("hyprland.toml"),
      "id = \"hyprland\"\ntemplate = \"taskbar/borders.lua\"\noutput = \"hypr/theme.lua\"\n",
    )
    .unwrap();
    let consumers = registry::load(&system, &registry_dir).unwrap();
    let manifest = ThemeManifest::load(&theme_dir).unwrap();
    let layout = Layout::parse(Path::new("float.toml"), Variant::Float, "rounding = 8\n").unwrap();
    Self {
      root,
      manifest,
      consumers,
      layout,
    }
  }

  fn render(&self, accent: Option<Color>) -> Result<Rendered, Vec<ThemeError>> {
    render(&RenderRequest {
      manifest: &self.manifest,
      variant: Variant::Float,
      accent,
      layout: Some(&self.layout),
      consumers: &self.consumers,
    })
  }

  fn generated(&self) -> PathBuf {
    self.root.path().join("generated")
  }
}

#[test]
fn renders_every_consumer_with_palette_layout_and_metadata() {
  let fixture = Fixture::new();
  let rendered = fixture.render(None).unwrap();
  let file = |consumer: &str| {
    rendered
      .files
      .iter()
      .find(|file| file.consumer == consumer)
      .unwrap()
      .contents
      .clone()
  };
  assert_eq!(
    file("taskbar"),
    "@define-color th-accent #E2BE8A;\n@define-color th-dim rgba(226, 190, 138, 0.13);\n"
  );
  assert_eq!(
    file("hyprland"),
    "return { border = \"rgba(e2be8aff)\", rounding = 8, variant = \"float\" }\n"
  );
}

#[test]
fn every_broken_consumer_is_reported() {
  let mut fixture = Fixture::new();
  fs::write(&fixture.consumers[0].template, "{{ @missing }}").unwrap();
  fixture.consumers[1].template = fixture.root.path().join("absent.css");
  let errors = fixture.render(None).unwrap_err();
  assert_eq!(errors.len(), 2, "{errors:?}");
}

#[test]
fn build_names_follow_the_content() {
  let fixture = Fixture::new();
  let first = fixture.render(None).unwrap().build_name();
  assert_eq!(first, fixture.render(None).unwrap().build_name());
  assert!(first.starts_with("sunset-float-"));
  let other = fixture
    .render(Color::parse("#3590BD"))
    .unwrap()
    .build_name();
  assert_ne!(first, other);
}

#[test]
fn writes_reuses_commits_and_collects_builds() {
  let fixture = Fixture::new();
  let generated = fixture.generated();
  let builds = generated.join("builds");
  let current = generated.join("current");

  let first = fixture.render(None).unwrap().write_build(&builds).unwrap();
  assert!(first.join("taskbar/theme.css").is_file());
  let description = fs::read_to_string(first.join(BUILD_MANIFEST)).unwrap();
  assert!(description.contains("theme = \"sunset\""), "{description}");
  assert!(description.contains("kind = \"rewrite\""), "{description}");
  assert_eq!(
    fixture.render(None).unwrap().write_build(&builds).unwrap(),
    first
  );

  commit(&current, &first).unwrap();
  assert_eq!(active_build(&current).as_deref(), Some(first.as_path()));
  assert_eq!(current_theme(&current).as_deref(), Some("sunset-float"));
  assert!(current_file_for(&current, "sunset-float", "taskbar/theme.css").is_some());
  assert!(current_file_for(&current, "sunset", "taskbar/theme.css").is_none());
  assert!(current_file_for(&current, "sunset-float", "missing.css").is_none());
  assert_eq!(
    fs::read_link(&current).unwrap(),
    Path::new("builds").join(first.file_name().unwrap())
  );

  let second = fixture
    .render(Color::parse("#3590BD"))
    .unwrap()
    .write_build(&builds)
    .unwrap();
  let third = fixture
    .render(Color::parse("#00AA00"))
    .unwrap()
    .write_build(&builds)
    .unwrap();
  commit(&current, &third).unwrap();
  assert!(
    fs::read_to_string(current.join("taskbar/theme.css"))
      .unwrap()
      .contains("#00AA00")
  );

  let removed = collect_garbage(&builds, &current, 1).unwrap();
  assert_eq!(removed.len(), 1);
  assert!(third.is_dir(), "the active build stays");
  assert_eq!(
    first.is_dir() as u8 + second.is_dir() as u8,
    1,
    "one other build stays"
  );
}

#[test]
fn commit_refuses_directories_outside_builds() {
  let fixture = Fixture::new();
  let generated = fixture.generated();
  let elsewhere = fixture.root.path().join("elsewhere");
  fs::create_dir_all(&elsewhere).unwrap();
  fs::create_dir_all(&generated).unwrap();
  assert!(commit(&generated.join("current"), &elsewhere).is_err());
  assert!(
    commit(
      &generated.join("current"),
      &generated.join("builds/missing")
    )
    .is_err()
  );
}

#[test]
fn fnv1a_matches_the_reference_vectors() {
  let hash = |text: &str| {
    let mut hash = Fnv1a::new();
    hash.write(text.as_bytes());
    hash.finish()
  };
  assert_eq!(hash(""), 0xcbf2_9ce4_8422_2325);
  assert_eq!(hash("a"), 0xaf63_dc4c_8601_ec8c);
}
