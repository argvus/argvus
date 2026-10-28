use super::*;

/// Creates a temporary directory for one test and removes it on drop.
struct TempTree(PathBuf);

impl TempTree {
  fn new(label: &str) -> Self {
    let path =
      std::env::temp_dir().join(format!("argvus-icons-test-{}-{label}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    Self(path)
  }

  fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for TempTree {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

/// Writes an `index.theme` for a theme directory under `root`.
fn write_theme(root: &Path, directory: &str, index_theme: &str) {
  let theme_dir = root.join(directory);
  fs::create_dir_all(&theme_dir).unwrap();
  fs::write(theme_dir.join("index.theme"), index_theme).unwrap();
}

/// Writes an `icon-theme.toml` manifest under `manifests_root`.
fn write_manifest(manifests_root: &Path, id: &str, content: &str) {
  let manifest_dir = manifests_root.join(id);
  fs::create_dir_all(&manifest_dir).unwrap();
  fs::write(manifest_dir.join("icon-theme.toml"), content).unwrap();
}

const ICON_THEME_WITH_DIRECTORIES: &str =
  "[Icon Theme]\nName=Papirus\nComment=Paper icons\nDirectories=16x16/apps\n";
const INHERITING_ICON_THEME: &str =
  "[Icon Theme]\nName=Argvus Icons\nInherits=Argvus Dark Icons,Adwaita,hicolor\nDirectories=\n";
const CURSOR_ONLY_THEME: &str = "[Icon Theme]\nName=Bibata\nInherits=default\n";

#[test]
fn parses_only_the_icon_theme_section() {
  let theme = parse_index_theme(
    "[Other]\nName=ignored\n[Icon Theme]\nName=Papirus\nComment=Paper icons\nInherits=hicolor, Adwaita,\nDirectories=16x16/apps, 22x22/apps\n",
  );

  assert_eq!(theme.name.as_deref(), Some("Papirus"));
  assert_eq!(theme.comment.as_deref(), Some("Paper icons"));
  assert_eq!(theme.inherits, vec!["hicolor", "Adwaita"]);
  assert_eq!(theme.directories, vec!["16x16/apps", "22x22/apps"]);
}

#[test]
fn cursor_only_and_hidden_themes_are_not_choices() {
  assert!(!parse_index_theme(CURSOR_ONLY_THEME).is_choice());
  assert!(
    !parse_index_theme("[Icon Theme]\nName=Hidden\nHidden=true\nDirectories=a\n").is_choice()
  );
  assert!(parse_index_theme(INHERITING_ICON_THEME).is_choice());
  assert!(parse_index_theme(ICON_THEME_WITH_DIRECTORIES).is_choice());
}

#[test]
fn lists_installed_themes_and_skips_fallbacks_and_cursor_themes() {
  let tree = TempTree::new("list");
  let root = tree.path().join("icons");
  write_theme(&root, "Papirus", ICON_THEME_WITH_DIRECTORIES);
  write_theme(&root, "Bibata", CURSOR_ONLY_THEME);
  write_theme(
    &root,
    "hicolor",
    "[Icon Theme]\nName=hicolor\nDirectories=scalable\n",
  );
  let roots = [IconRoot {
    source: IconThemeSource::User,
    path: root,
  }];

  let entries = discover_icon_themes(&roots, &tree.path().join("manifests"));

  let names: Vec<&str> = entries
    .iter()
    .map(|entry| entry.directory_name.as_str())
    .collect();
  assert_eq!(names, vec!["Papirus"]);
  assert_eq!(entries[0].comment.as_deref(), Some("Paper icons"));
  assert_eq!(entries[0].official, None);
}

#[test]
fn user_root_overrides_system_root_for_the_same_directory() {
  let tree = TempTree::new("precedence");
  let data_home = tree.path().join("data");
  let system_root = tree.path().join("system");
  write_theme(
    &data_home.join("icons"),
    "Papirus",
    "[Icon Theme]\nName=Papirus (mine)\nDirectories=a\n",
  );
  write_theme(&system_root, "Papirus", ICON_THEME_WITH_DIRECTORIES);
  let roots = icon_roots(&tree.path().join("home"), &data_home, &system_root);

  let entries = discover_icon_themes(&roots, &tree.path().join("manifests"));

  let papirus: Vec<&IconThemeEntry> = entries
    .iter()
    .filter(|entry| entry.directory_name == "Papirus")
    .collect();
  assert_eq!(papirus.len(), 1);
  assert_eq!(papirus[0].display_name, "Papirus (mine)");
  assert_eq!(papirus[0].source, IconThemeSource::User);
}

#[test]
fn official_manifests_mark_only_system_themes() {
  let tree = TempTree::new("official");
  let system_root = tree.path().join("system");
  let data_home = tree.path().join("data");
  write_theme(&system_root, "Argvus Icons", INHERITING_ICON_THEME);
  write_theme(
    &data_home.join("icons"),
    "Argvus Icons",
    INHERITING_ICON_THEME,
  );
  let manifests = tree.path().join("manifests");
  write_manifest(
    &manifests,
    "argvus-icons",
    "id = \"argvus-icons\"\nname = \"Argvus Icons\"\nvariant = \"default\"\nofficial = true\n",
  );
  let roots = icon_roots(&tree.path().join("home"), &data_home, &system_root);

  let entries = discover_icon_themes(&roots, &manifests);

  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].source, IconThemeSource::User);
  assert_eq!(entries[0].official, None);

  let system_only = discover_icon_themes(
    &[IconRoot {
      source: IconThemeSource::System,
      path: system_root,
    }],
    &manifests,
  );
  assert_eq!(system_only[0].official, Some(IconThemeVariant::Default));
}

#[test]
fn invalid_or_non_official_manifests_are_ignored() {
  let tree = TempTree::new("manifests");
  let manifests = tree.path().join("manifests");
  write_manifest(&manifests, "broken", "this is not = [toml");
  write_manifest(
    &manifests,
    "unofficial",
    "name = \"Papirus\"\nvariant = \"dark\"\nofficial = false\n",
  );
  write_manifest(
    &manifests,
    "unknown-variant",
    "name = \"Argvus Icons\"\nvariant = \"sepia\"\nofficial = true\n",
  );
  write_manifest(
    &manifests,
    "argvus-dark-icons",
    "name = \"Argvus Dark Icons\"\nvariant = \"dark\"\nofficial = true\n",
  );

  let official = official_manifests(&manifests);

  assert_eq!(
    official,
    vec![("Argvus Dark Icons".to_string(), IconThemeVariant::Dark)]
  );
}

#[test]
fn official_themes_sort_before_other_themes() {
  let tree = TempTree::new("sort");
  let system_root = tree.path().join("system");
  write_theme(
    &system_root,
    "Adwaita-custom",
    "[Icon Theme]\nName=Zeta\nDirectories=a\n",
  );
  write_theme(&system_root, "Argvus Icons", INHERITING_ICON_THEME);
  let manifests = tree.path().join("manifests");
  write_manifest(
    &manifests,
    "argvus-icons",
    "name = \"Argvus Icons\"\nvariant = \"default\"\nofficial = true\n",
  );

  let entries = discover_icon_themes(
    &[IconRoot {
      source: IconThemeSource::System,
      path: system_root,
    }],
    &manifests,
  );

  assert_eq!(entries[0].directory_name, "Argvus Icons");
  assert_eq!(entries[1].display_name, "Zeta");
}

#[test]
fn missing_roots_yield_no_themes() {
  let tree = TempTree::new("missing");
  let roots = icon_roots(
    &tree.path().join("home"),
    &tree.path().join("data"),
    &tree.path().join("system"),
  );

  assert!(discover_icon_themes(&roots, &tree.path().join("manifests")).is_empty());
}

#[test]
fn validates_plain_icon_theme_names() {
  assert!(validate_icon_theme_name("Argvus Icons").is_ok());
  assert!(validate_icon_theme_name("").is_err());
  assert!(validate_icon_theme_name(".").is_err());
  assert!(validate_icon_theme_name("..").is_err());
  assert!(validate_icon_theme_name("../etc").is_err());
  assert!(validate_icon_theme_name("nested/theme").is_err());
  assert!(validate_icon_theme_name(&"x".repeat(MAX_ICON_THEME_NAME_LENGTH + 1)).is_err());
}

#[test]
fn empty_or_missing_configuration_resolves_to_the_mode_default() {
  assert_eq!(
    configured_icon_theme(None, DEFAULT_LIGHT_ICON_THEME),
    DEFAULT_LIGHT_ICON_THEME
  );
  assert_eq!(
    configured_icon_theme(Some(""), DEFAULT_ICON_THEME),
    DEFAULT_ICON_THEME
  );
  assert_eq!(
    configured_icon_theme(Some("\"\"\n"), DEFAULT_ICON_THEME),
    DEFAULT_ICON_THEME
  );
  assert_eq!(
    configured_icon_theme(Some("Papirus\n"), DEFAULT_ICON_THEME),
    "Papirus"
  );
  assert_eq!(
    configured_icon_theme(Some("\"Papirus\""), DEFAULT_ICON_THEME),
    "Papirus"
  );
}

#[test]
fn default_follows_the_mode_of_the_theme() {
  let official = [argvus_theme::discovery::ThemeEntry {
    id: "nord-light".into(),
    name: "Nord Light".into(),
    category: argvus_theme::discovery::ThemeCategory::Light,
    accent: String::new(),
    wallpaper: String::new(),
    name_i18n: std::collections::HashMap::new(),
    user: false,
  }];
  assert_eq!(
    default_icon_theme_for("argvus-dark", &[]),
    DEFAULT_ICON_THEME
  );
  assert_eq!(
    default_icon_theme_for("argvus-light", &[]),
    DEFAULT_LIGHT_ICON_THEME
  );
  assert_eq!(
    default_icon_theme_for("argvus-light-float", &[]),
    DEFAULT_LIGHT_ICON_THEME
  );
  assert_eq!(
    default_icon_theme_for("nord-light-float", &official),
    DEFAULT_LIGHT_ICON_THEME
  );
  assert_eq!(
    default_icon_theme_for("monokai-dark-float", &official),
    DEFAULT_ICON_THEME
  );
}
