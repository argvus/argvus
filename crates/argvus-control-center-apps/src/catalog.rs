//! Implements default-application catalog definitions in crate `argvus control center apps`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! The catalog maps every supported category to the applications Argvus knows
//! about. It is intentionally independent from any desktop environment
//! (GNOME/KDE/Xfce) and from the Argvus shell code: this table is the single
//! source of truth both for the GUI/CLI and for detecting what is installed.

use serde::{Deserialize, Serialize};

/// Version recorded in `defaults.json`, to allow graceful future migrations.
pub const STATE_VERSION: u32 = 1;

/// A supported default-program category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
  Terminal,
  FileManager,
  TextEditor,
  TerminalEditor,
  Browser,
  ImageViewer,
  PdfViewer,
  VideoPlayer,
  AudioPlayer,
  Archive,
  Launcher,
}

/// A single known application inside a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownApp {
  /// Executable name used to run/validate the app (`command -v <binary>`).
  pub binary: &'static str,
  /// Friendly display name (branding) shown in the GUI.
  pub display: &'static str,
  /// Bundled `.desktop` id, used for XDG/MIME registration. Empty string for
  /// terminal-based apps that have no desktop file.
  pub desktop_id: &'static str,
  /// `true` when the app runs inside a terminal (TUI).
  pub tui: bool,
}

/// A category plus the applications Argvus knows about for it.
#[derive(Debug, Clone, Copy)]
pub struct CategorySpec {
  pub category: Category,
  pub apps: &'static [KnownApp],
}

impl Category {
  /// Defines the constant `ORDER`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
  pub const ORDER: [Category; 11] = [
    Category::Terminal,
    Category::FileManager,
    Category::TextEditor,
    Category::TerminalEditor,
    Category::Browser,
    Category::ImageViewer,
    Category::PdfViewer,
    Category::VideoPlayer,
    Category::AudioPlayer,
    Category::Archive,
    Category::Launcher,
  ];

  /// State-file key, e.g. `file_manager`.
  pub fn key(self) -> &'static str {
    match self {
      Category::Terminal => "terminal",
      Category::FileManager => "file_manager",
      Category::TextEditor => "text_editor",
      Category::TerminalEditor => "terminal_editor",
      Category::Browser => "browser",
      Category::ImageViewer => "image_viewer",
      Category::PdfViewer => "pdf_viewer",
      Category::VideoPlayer => "video_player",
      Category::AudioPlayer => "audio_player",
      Category::Archive => "archive",
      Category::Launcher => "launcher",
    }
  }

  /// Argvus fallback when `defaults.json` has no value for this category.
  pub fn fallback(self) -> &'static str {
    match self {
      Category::Terminal => "argvus-terminal",
      Category::FileManager => "spf",
      Category::TextEditor => "mousepad",
      Category::TerminalEditor => "vim",
      Category::Browser => "",
      Category::ImageViewer => "imv",
      Category::PdfViewer => "zathura",
      Category::VideoPlayer => "mpv",
      Category::AudioPlayer => "audacious",
      Category::Archive => "xarchiver",
      Category::Launcher => "rofi",
    }
  }

  /// Categories that are registered through XDG when applied.
  pub fn uses_xdg(self) -> bool {
    matches!(
      self,
      Category::Browser
        | Category::TextEditor
        | Category::ImageViewer
        | Category::PdfViewer
        | Category::VideoPlayer
        | Category::AudioPlayer
        | Category::Archive
    )
  }

  /// MIME types associated to this category (empty when not XDG-managed).
  pub fn mimes(self) -> &'static [&'static str] {
    match self {
      Category::Browser => &[
        "x-scheme-handler/http",
        "x-scheme-handler/https",
        "text/html",
      ],
      Category::TextEditor => &[
        "application/ecmascript",
        "application/javascript",
        "application/json",
        "application/sql",
        "application/toml",
        "application/xml",
        "application/x-shellscript",
        "application/x-toml",
        "application/x-yaml",
        "inode/x-empty",
        "text/css",
        "text/csv",
        "text/html",
        "text/javascript",
        "text/markdown",
        "text/plain",
        "text/rust",
        "text/x-c",
        "text/x-c++src",
        "text/x-chdr",
        "text/x-csrc",
        "text/x-go",
        "text/x-java",
        "text/x-log",
        "text/x-lua",
        "text/x-makefile",
        "text/x-markdown",
        "text/x-python",
        "text/x-readme",
        "text/x-script",
        "text/x-shellscript",
        "text/x-toml",
        "text/x-yaml",
        "text/xml",
      ],
      Category::ImageViewer => &[
        "image/avif",
        "image/bmp",
        "image/gif",
        "image/heic",
        "image/heif",
        "image/jp2",
        "image/jpeg",
        "image/jxl",
        "image/png",
        "image/svg+xml",
        "image/tiff",
        "image/vnd.adobe.photoshop",
        "image/vnd.microsoft.icon",
        "image/vnd.radiance",
        "image/webp",
        "image/x-canon-cr2",
        "image/x-canon-cr3",
        "image/x-dds",
        "image/x-exr",
        "image/x-fuji-raf",
        "image/x-icon",
        "image/x-nikon-nef",
        "image/x-olympus-orf",
        "image/x-panasonic-rw2",
        "image/x-portable-anymap",
        "image/x-portable-bitmap",
        "image/x-portable-graymap",
        "image/x-portable-pixmap",
        "image/x-sony-arw",
        "image/x-tga",
        "image/x-xbitmap",
        "image/x-xpixmap",
      ],
      Category::PdfViewer => &[
        "application/acrobat",
        "application/pdf",
        "application/vnd.ms-xpsdocument",
        "application/x-bzpdf",
        "application/x-gzpdf",
        "application/x-pdf",
        "application/x-xps",
        "image/vnd.djvu",
        "image/vnd.djvu+multipage",
        "image/vnd.pdf",
      ],
      Category::VideoPlayer => &[
        "application/sdp",
        "application/smil",
        "application/streamingmedia",
        "application/vnd.apple.mpegurl",
        "application/vnd.ms-asf",
        "application/x-cue",
        "application/x-matroska",
        "video/3gpp",
        "video/3gpp2",
        "video/annodex",
        "video/divx",
        "video/dv",
        "video/fli",
        "video/flv",
        "video/mp2t",
        "video/mp4",
        "video/mp4v-es",
        "video/mpeg",
        "video/msvideo",
        "video/ogg",
        "video/quicktime",
        "video/vnd.mpegurl",
        "video/vnd.rn-realvideo",
        "video/webm",
        "video/x-flv",
        "video/x-m4v",
        "video/x-matroska",
        "video/x-mpeg",
        "video/x-ms-asf",
        "video/x-ms-wmv",
        "video/x-msvideo",
        "video/x-ogm",
        "video/x-theora",
        "video/x-theora+ogg",
      ],
      Category::AudioPlayer => &[
        "application/ogg",
        "audio/aac",
        "audio/ac3",
        "audio/annodex",
        "audio/basic",
        "audio/flac",
        "audio/m4a",
        "audio/mp2",
        "audio/mp4",
        "audio/mpeg",
        "audio/ogg",
        "audio/opus",
        "audio/vnd.rn-realaudio",
        "audio/wav",
        "audio/webm",
        "audio/x-aac",
        "audio/x-aiff",
        "audio/x-ape",
        "audio/x-flac",
        "audio/x-m4a",
        "audio/x-matroska",
        "audio/x-mpeg",
        "audio/x-ms-wma",
        "audio/x-musepack",
        "audio/x-opus+ogg",
        "audio/x-vorbis+ogg",
        "audio/x-wav",
        "audio/x-wavpack",
      ],
      Category::Archive => &[
        "application/bzip2",
        "application/gzip",
        "application/vnd.debian.binary-package",
        "application/vnd.rar",
        "application/x-7z-compressed-tar",
        "application/x-7z-compressed",
        "application/x-archive",
        "application/x-bzip",
        "application/x-bzip2",
        "application/x-bzip2-compressed-tar",
        "application/x-compress",
        "application/x-compressed-tar",
        "application/x-cpio",
        "application/x-gzip",
        "application/x-lha",
        "application/x-lzip",
        "application/x-lzip-compressed-tar",
        "application/x-lzma",
        "application/x-lzma-compressed-tar",
        "application/x-lzop",
        "application/x-lzop-compressed-tar",
        "application/x-rar",
        "application/x-rar-compressed",
        "application/x-rpm",
        "application/x-tar",
        "application/x-xar",
        "application/x-xz",
        "application/x-xz-compressed-tar",
        "application/zstd",
        "application/zip",
      ],
      _ => &[],
    }
  }

  /// Parse a state-file key into a category.
  pub fn from_key(key: &str) -> Option<Category> {
    Category::ORDER.iter().copied().find(|c| c.key() == key)
  }

  /// Categories that are only meaningful inside a terminal (TUI).
  #[allow(dead_code)]
  pub fn is_tui(self) -> bool {
    matches!(self, Category::Terminal | Category::TerminalEditor)
  }
}

/// The full catalog, in display order.
pub const CATALOG: &[CategorySpec] = &[
  CategorySpec {
    category: Category::Terminal,
    apps: &[
      known(
        "argvus-terminal",
        "ARGVUS Terminal",
        "argvus-terminal.desktop",
      ),
      known("kitty", "Kitty", "org.kitt.humans.kitty.desktop"),
      known("alacritty", "Alacritty", "org.alacritty.Alacritty.desktop"),
      known("wezterm", "WezTerm", "org.wezterm.WezTerm.desktop"),
      known("ghostty", "Ghostty", "com.mitchellh.ghostty.desktop"),
      known("xfce4-terminal", "Xfce Terminal", "xfce4-terminal.desktop"),
      known(
        "gnome-terminal",
        "GNOME Terminal",
        "org.gnome.Terminal.desktop",
      ),
      known("konsole", "Konsole", "org.kde.konsole.desktop"),
      known("tilix", "Tilix", "com.gexperts.Tilix.desktop"),
      known("terminator", "Terminator", "terminator.desktop"),
      known("st", "st", "st.desktop"),
    ],
  },
  CategorySpec {
    category: Category::FileManager,
    apps: &[
      tui("spf", "superfile"),
      tui("yazi", "Yazi"),
      tui("ranger", "Ranger"),
      tui("lf", "Lf"),
      tui("joshuto", "Joshuto"),
      tui("broot", "Broot"),
      tui("mc", "Midnight Commander"),
      tui("nnn", "nnn"),
      known("nautilus", "Nautilus", "org.gnome.Nautilus.desktop"),
      known("nemo", "Nemo", "org.nemo.Nemo.desktop"),
      known("thunar", "Thunar", "Thunar.desktop"),
      known("dolphin", "Dolphin", "org.kde.dolphin.desktop"),
      known("pcmanfm", "PCManFM", "pcmanfm.desktop"),
      known("krusader", "Krusader", "org.kde.krusader.desktop"),
      known("caja", "Caja", "org.mate.Caja.desktop"),
      known("doublecmd", "Double Commander", "doublecmd.desktop"),
    ],
  },
  CategorySpec {
    category: Category::TextEditor,
    apps: &[
      known("code", "Visual Studio Code", "code.desktop"),
      known(
        "code-insiders",
        "Visual Studio Code Insiders",
        "code-insiders.desktop",
      ),
      known("codium", "VSCodium", "codium.desktop"),
      known("sublime_text", "Sublime Text", "sublime_text.desktop"),
      known("gedit", "GNOME Text Editor", "org.gnome.TextEditor.desktop"),
      known("gnome-text-editor", "GNOME Text Editor (legacy)", ""),
      known("kate", "Kate", "org.kde.kate.desktop"),
      known("mousepad", "Mousepad", "mousepad.desktop"),
      known("geany", "Geany", "geany.desktop"),
      known("leafpad", "Leafpad", "leafpad.desktop"),
      known("pluma", "Pluma", "pluma.desktop"),
      known("emacs", "Emacs", "emacs.desktop"),
    ],
  },
  CategorySpec {
    category: Category::TerminalEditor,
    apps: &[
      tui("nvim", "Neovim"),
      tui("vim", "Vim"),
      tui("nano", "Nano"),
      tui("micro", "Micro"),
      tui("helix", "Helix"),
      tui("kak", "Kakoune"),
      tui("ed", "ed"),
    ],
  },
  CategorySpec {
    category: Category::Browser,
    apps: &[
      known("firefox", "Firefox", "firefox.desktop"),
      known("chromium", "Chromium", "chromium.desktop"),
      known("google-chrome", "Google Chrome", "google-chrome.desktop"),
      known(
        "google-chrome-stable",
        "Google Chrome (stable)",
        "google-chrome-stable.desktop",
      ),
      known("librewolf", "LibreWolf", "librewolf.desktop"),
      known(
        "ungoogled-chromium",
        "Ungoogled Chromium",
        "ungoogled-chromium.desktop",
      ),
      known("brave-browser", "Brave", "brave-browser.desktop"),
      known("vivaldi", "Vivaldi", "vivaldi-stable.desktop"),
      known("opera", "Opera", "opera.desktop"),
      known("epiphany", "GNOME Web", "org.gnome.Epiphany.desktop"),
      known("qutebrowser", "Qutebrowser", "qutebrowser.desktop"),
      known("falkon", "Falkon", "org.kde.falkon.desktop"),
      known("midori", "Midori", "midori.desktop"),
      known("min", "Min", "min.desktop"),
    ],
  },
  CategorySpec {
    category: Category::ImageViewer,
    apps: &[
      known("imv", "Imv", "imv.desktop"),
      known("ristretto", "Ristretto", "org.xfce.ristretto.desktop"),
      known("sxiv", "Sxiv", "sxiv.desktop"),
      known("feh", "feh", "feh.desktop"),
      known("gpicview", "GPicView", "gpicview.desktop"),
      known("loupe", "Loupe", "org.gnome.Loupe.desktop"),
      known("eog", "Eye of GNOME", "org.gnome.eog.desktop"),
      known("gwenview", "Gwenview", "org.kde.gwenview.desktop"),
      known("viewnior", "Viewnior", "viewnior.desktop"),
      known("pqiv", "Pqiv", "pqiv.desktop"),
    ],
  },
  CategorySpec {
    category: Category::PdfViewer,
    apps: &[
      known("zathura", "Zathura", "org.pwmt.zathura.desktop"),
      known("evince", "Evince", "org.gnome.Evince.desktop"),
      known("okular", "Okular", "org.kde.okular.desktop"),
      known("mupdf", "MuPDF", "mupdf.desktop"),
      known("qpdfview", "qpdfview", "qpdfview.desktop"),
      known("llpp", "llpp", "llpp.desktop"),
      known("xpdf", "xpdf", "xpdf.desktop"),
    ],
  },
  CategorySpec {
    category: Category::VideoPlayer,
    apps: &[
      known("mpv", "mpv", "mpv.desktop"),
      known("vlc", "VLC", "org.videolan.VLC.desktop"),
      known(
        "celluloid",
        "Celluloid",
        "io.github.celluloid_player.Celluloid.desktop",
      ),
      known("totem", "Videos (Totem)", "org.gnome.Totem.desktop"),
      known("ffplay", "ffplay", "ffplay.desktop"),
    ],
  },
  CategorySpec {
    category: Category::AudioPlayer,
    apps: &[
      known("mpv", "mpv", "mpv.desktop"),
      known("vlc", "VLC", "org.videolan.VLC.desktop"),
      known("cmus", "cmus", "cmus.desktop"),
      known("ncspot", "ncspot", "ncspot.desktop"),
      known("spotify", "Spotify", "spotify.desktop"),
      known("tauon", "Tauon", "tauon.desktop"),
      known("amberol", "Amberol", "io.bassi.Amberol.desktop"),
      known("lollypop", "Lollypop", "org.gnome.Lollypop.desktop"),
      known("rhythmbox", "Rhythmbox", "org.gnome.Rhythmbox.desktop"),
      known("strawberry", "Strawberry", "org.kde.strawberry.desktop"),
      known("audacious", "Audacious", "audacious.desktop"),
    ],
  },
  CategorySpec {
    category: Category::Archive,
    apps: &[
      known("file-roller", "File Roller", "org.gnome.FileRoller.desktop"),
      known("ark", "Ark", "org.kde.ark.desktop"),
      known("xarchiver", "Xarchiver", "xarchiver.desktop"),
      known("peazip", "PeaZip", "peazip.desktop"),
      known("engrampa", "Engrampa", "org.mate.engrampa.desktop"),
    ],
  },
  CategorySpec {
    category: Category::Launcher,
    apps: &[
      known("rofi", "Rofi", "rofi.desktop"),
      known("fuzzel", "Fuzzel", "fuzzel.desktop"),
      known("tofi", "tofi", "tofi.desktop"),
      known("wmenu", "wmenu", "wmenu.desktop"),
      known("dmenu", "dmenu", "dmenu.desktop"),
      known("ulauncher", "Ulauncher", "ulauncher.desktop"),
      known("albert", "Albert", "albert.desktop"),
    ],
  },
];

/// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const fn known(binary: &'static str, display: &'static str, desktop_id: &'static str) -> KnownApp {
  KnownApp {
    binary,
    display,
    desktop_id,
    tui: false,
  }
}

/// Executes the const function documented in this module. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
const fn tui(binary: &'static str, display: &'static str) -> KnownApp {
  KnownApp {
    binary,
    display,
    desktop_id: "",
    tui: true,
  }
}

/// Look up the catalog spec for a category.
pub fn spec(category: Category) -> &'static CategorySpec {
  CATALOG
    .iter()
    .find(|s| s.category == category)
    .expect("catalog covers every category")
}

/// Look up a known app (by binary, display name or desktop id) in a category.
pub fn find_app(category: Category, query: &str) -> Option<&'static KnownApp> {
  let q = query.trim().to_lowercase();
  spec(category).apps.iter().find(|a| {
    a.binary.eq_ignore_ascii_case(&q)
      || a.display.to_lowercase() == q
      || (!a.desktop_id.is_empty() && a.desktop_id.eq_ignore_ascii_case(&q))
      || a
        .desktop_id
        .strip_suffix(".desktop")
        .is_some_and(|d| d.eq_ignore_ascii_case(&q))
  })
}

/// Every category with its effective value (state value, or the Argvus fallback).
pub fn effective_values(stored: &[(Category, Option<String>)]) -> Vec<(Category, String)> {
  Category::ORDER
    .iter()
    .copied()
    .map(|c| {
      let value = stored
        .iter()
        .find(|(cat, _)| *cat == c)
        .and_then(|(_, v)| v.clone())
        .unwrap_or_default();
      let effective = if value.is_empty() {
        c.fallback().to_string()
      } else {
        value
      };
      (c, effective)
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `category_keys_round_trip` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn category_keys_round_trip() {
    for cat in Category::ORDER {
      assert_eq!(Category::from_key(cat.key()), Some(cat));
    }
    assert_eq!(Category::from_key("bogus"), None);
  }

  #[test]
  /// Executes the `every_category_has_a_spec_and_apps` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn every_category_has_a_spec_and_apps() {
    for cat in Category::ORDER {
      let s = spec(cat);
      assert_eq!(s.category, cat);
      assert!(!s.apps.is_empty(), "{} catalog is empty", cat.key());
    }
  }

  #[test]
  /// Executes the `every_default_is_present_in_catalog` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn every_default_is_present_in_catalog() {
    for cat in Category::ORDER {
      let fallback = cat.fallback();
      if fallback.is_empty() {
        continue;
      }
      assert!(
        spec(cat).apps.iter().any(|a| a.binary == fallback),
        "fallback {fallback} missing from {}",
        cat.key()
      );
    }
  }

  #[test]
  /// Executes the `desktop_ids_are_unique_within_category` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn desktop_ids_are_unique_within_category() {
    for cat in Category::ORDER {
      let mut ids: Vec<&str> = spec(cat)
        .apps
        .iter()
        .filter(|a| !a.desktop_id.is_empty())
        .map(|a| a.desktop_id)
        .collect();
      let before = ids.len();
      ids.sort_unstable();
      ids.dedup();
      assert_eq!(ids.len(), before, "duplicate desktop id in {}", cat.key());
    }
  }

  #[test]
  /// Executes the `find_app_matches_by_binary` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn find_app_matches_by_binary() {
    let a = find_app(Category::Browser, "firefox").expect("firefox in catalog");
    assert_eq!(a.binary, "firefox");
    assert_eq!(find_app(Category::Browser, "not-a-browser"), None);
  }

  #[test]
  /// Executes the `find_app_matches_by_desktop_id` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn find_app_matches_by_desktop_id() {
    let a = find_app(Category::PdfViewer, "org.pwmt.zathura").unwrap();
    assert_eq!(a.binary, "zathura");
  }

  #[test]
  /// Executes the `effective_values_uses_fallback` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn effective_values_uses_fallback() {
    let stored = vec![(Category::Browser, Some("chromium".to_string()))];
    let values = effective_values(&stored);
    let map: std::collections::HashMap<_, _> = values.into_iter().collect();
    assert_eq!(map[&Category::Browser], "chromium");
    assert_eq!(map[&Category::Terminal], "argvus-terminal");
    assert_eq!(map[&Category::TextEditor], "mousepad");
  }

  #[test]
  /// Executes the `common_file_mimes_are_registered` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn common_file_mimes_are_registered() {
    assert!(Category::ImageViewer.mimes().contains(&"image/jpeg"));
    assert!(Category::ImageViewer.mimes().contains(&"image/png"));
    assert!(Category::ImageViewer.mimes().contains(&"image/webp"));

    assert!(Category::TextEditor.mimes().contains(&"text/plain"));
    assert!(Category::TextEditor.mimes().contains(&"text/markdown"));
    assert!(Category::TextEditor.mimes().contains(&"inode/x-empty"));

    assert!(Category::AudioPlayer.mimes().contains(&"audio/mpeg"));
    assert!(Category::AudioPlayer.mimes().contains(&"audio/flac"));
    assert!(Category::VideoPlayer.mimes().contains(&"video/mp4"));
    assert!(Category::PdfViewer.mimes().contains(&"application/pdf"));
    assert!(Category::Archive.mimes().contains(&"application/zip"));
  }
}
