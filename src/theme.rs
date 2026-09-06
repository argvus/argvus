use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DEFAULT_THEME: &str = "argvus-dark-aether";

#[derive(Debug, Clone)]
pub struct ThemeCss {
    paths: ThemePaths,
    fingerprint: Option<ThemeFingerprint>,
}

#[derive(Debug, Clone)]
struct ThemePaths {
    style_file: PathBuf,
    theme_file: PathBuf,
    theme_dir: PathBuf,
    cache_theme_file: PathBuf,
    active_theme_file: PathBuf,
    dev_style_file: PathBuf,
    dev_theme_file: PathBuf,
    dev_theme_dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThemeFingerprint {
    active_theme: Option<FileStamp>,
    style: Option<FileStamp>,
    theme: Option<FileStamp>,
    active_css: Option<FileStamp>,
    cache_css: Option<FileStamp>,
}

impl ThemeCss {
    pub fn new() -> Self {
        Self {
            paths: ThemePaths::resolve(),
            fingerprint: None,
        }
    }

    pub fn changed(&self) -> bool {
        self.fingerprint.as_ref() != Some(&self.current_fingerprint())
    }

    pub fn render(&mut self, app_css: &str, font_css: &str) -> String {
        self.fingerprint = Some(self.current_fingerprint());
        let active = self.active_theme_name();
        let link_css = if active.starts_with("argvus-dark-silver") {
            ".argvus-about .argvus-link, .argvus-about .argvus-link label { color: #ffffff; }"
        } else {
            ".argvus-about .argvus-link, .argvus-about .argvus-link label { color: @argvus_accent; }"
        };

        [
            fallback_colors(),
            self.stylesheet_css().as_str(),
            app_css,
            link_css,
            font_css,
        ]
        .join("\n")
    }

    fn stylesheet_css(&self) -> String {
        self.stylesheet_paths()
            .into_iter()
            .filter_map(|path| read_css_recursive(&path, &mut Vec::new()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn stylesheet_paths(&self) -> Vec<PathBuf> {
        let theme_dir = existing_or_dev(&self.paths.theme_dir, &self.paths.dev_theme_dir);
        let active = self.active_theme_name();
        vec![
            existing_or_dev(&self.paths.style_file, &self.paths.dev_style_file),
            existing_or_dev(&self.paths.theme_file, &self.paths.dev_theme_file),
            theme_dir.join(format!("{active}.css")),
            self.paths.cache_theme_file.clone(),
        ]
    }

    fn active_theme_name(&self) -> String {
        fs::read_to_string(&self.paths.active_theme_file)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_THEME.to_string())
    }

    fn current_fingerprint(&self) -> ThemeFingerprint {
        let theme_dir = existing_or_dev(&self.paths.theme_dir, &self.paths.dev_theme_dir);
        let active_css = theme_dir.join(format!("{}.css", self.active_theme_name()));
        ThemeFingerprint {
            active_theme: stamp(&self.paths.active_theme_file),
            style: stamp(&existing_or_dev(
                &self.paths.style_file,
                &self.paths.dev_style_file,
            )),
            theme: stamp(&existing_or_dev(
                &self.paths.theme_file,
                &self.paths.dev_theme_file,
            )),
            active_css: stamp(&active_css),
            cache_css: stamp(&self.paths.cache_theme_file),
        }
    }
}

impl ThemePaths {
    fn resolve() -> Self {
        let system_dir = PathBuf::from("/etc/argvus-settings");
        let cache_home = cache_home();
        let dev_resources = std::env::var_os("ARGVUS_SETTINGS_RESOURCE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../argvus-settings/resources"
                ))
            });

        Self {
            style_file: system_dir.join("style.css"),
            theme_file: system_dir.join("theme.css"),
            theme_dir: system_dir.join("themes"),
            cache_theme_file: cache_home.join("argvus-calendar").join("theme.css"),
            active_theme_file: argvus_config_home().join(".active-theme"),
            dev_style_file: dev_resources.join("style.css"),
            dev_theme_file: dev_resources.join("theme.css"),
            dev_theme_dir: dev_resources.join("themes"),
        }
    }
}

fn read_css_recursive(path: &Path, stack: &mut Vec<PathBuf>) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    let mut css = String::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(imported) = import_path(trimmed, path) {
            if !stack.contains(&imported) {
                stack.push(imported.clone());
                if let Some(import_css) = read_css_recursive(&imported, stack) {
                    css.push_str(&import_css);
                    css.push('\n');
                }
                stack.pop();
            }
        } else {
            css.push_str(line);
            css.push('\n');
        }
    }

    Some(css)
}

fn import_path(line: &str, source: &Path) -> Option<PathBuf> {
    let inner = line
        .strip_prefix("@import url(")
        .and_then(|value| value.strip_suffix(");").or_else(|| value.strip_suffix(')')))?;
    let quoted = inner.trim().trim_matches('"').trim_matches('\'');
    (!quoted.is_empty()).then(|| source.parent().unwrap_or(Path::new(".")).join(quoted))
}

fn fallback_colors() -> &'static str {
    "@define-color argvus_bg #111316;
@define-color argvus_bg_alpha rgba(17, 19, 22, 0.99);
@define-color argvus_surface #201f27;
@define-color argvus_surface_alt #262933;
@define-color argvus_fg #dfe5ea;
@define-color argvus_muted #b0bfcb;
@define-color argvus_dim rgba(176, 191, 203, 0.34);
@define-color argvus_accent #3590bd;
@define-color argvus_accent_alpha rgba(53, 144, 189, 0.45);
@define-color argvus_border rgba(53, 144, 189, 0.28);
@define-color argvus_focus rgba(53, 144, 189, 0.75);"
}

fn existing_or_dev(system: &Path, dev: &Path) -> PathBuf {
    if system.exists() {
        system.to_path_buf()
    } else {
        dev.to_path_buf()
    }
}

fn stamp(path: &Path) -> Option<FileStamp> {
    let metadata = fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok(),
        len: metadata.len(),
    })
}

fn argvus_config_home() -> PathBuf {
    std::env::var_os("ARGVUS_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(config_home)
        .join("argvus")
}

fn config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"))
}

fn cache_home() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".cache"))
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_imports_relative_to_source_file() {
        let dir = std::env::temp_dir().join(format!("argvus-about-theme-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let base = dir.join("argvus-dark-aether.css");
        let float = dir.join("argvus-dark-aether-float.css");
        fs::write(&base, "@define-color argvus_bg #111316;\n").unwrap();
        fs::write(&float, "@import url(\"argvus-dark-aether.css\");\n").unwrap();

        let css = read_css_recursive(&float, &mut Vec::new()).unwrap();
        assert!(css.contains("@define-color argvus_bg #111316;"));

        fs::remove_dir_all(&dir).unwrap_or(());
    }
}
