//! CLI for ARGVUS theme discovery and metadata.
//!
//! Usage:
//!   argvus-appearance themes list [--category dark|light] [--format tsv|json]
//!   argvus-appearance themes get <id> <field>

use argvus_theme::discovery::{ThemeCategory, discover_themes};
use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(name = "argvus-appearance")]
#[command(about = "ARGVUS theme discovery and configuration")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Manage themes
  Themes {
    #[command(subcommand)]
    action: ThemesAction,
  },
}

#[derive(Subcommand)]
enum ThemesAction {
  /// List available themes
  List {
    /// Filter by category: dark or light
    #[arg(long)]
    category: Option<String>,

    /// Output format: tsv or json
    #[arg(long, default_value = "tsv")]
    format: String,
  },

  /// Get theme field value
  Get {
    /// Theme ID (e.g., argvus-dark, dracula)
    id: String,

    /// Field name (e.g., name, accent, wallpaper, category)
    field: String,
  },
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Themes { action } => match action {
      ThemesAction::List { category, format } => {
        list_themes(&category, &format);
      }
      ThemesAction::Get { id, field } => {
        get_theme_field(&id, &field);
      }
    },
  }
}

#[allow(clippy::unnecessary_map_or)]
fn list_themes(category_filter: &Option<String>, format: &str) {
  let system_config = Path::new("/usr/share/argvus");
  let report = discover_themes(system_config);

  let filtered: Vec<_> = report
    .themes
    .iter()
    .filter(|t| {
      category_filter.as_ref().map_or(true, |cat| {
        let parsed_cat = match cat.as_str() {
          "dark" => Some(ThemeCategory::Dark),
          "light" => Some(ThemeCategory::Light),
          _ => None,
        };
        parsed_cat == Some(t.category)
      })
    })
    .collect();

  match format {
    "json" => {
      if let Ok(json) = serde_json::to_string(&filtered) {
        println!("{}", json);
      } else {
        eprintln!("error: failed to serialize themes as JSON");
        std::process::exit(1);
      }
    }
    "tsv" => {
      for theme in filtered {
        let cat = match theme.category {
          ThemeCategory::Dark => "dark",
          ThemeCategory::Light => "light",
        };
        println!("{}\t{}\t{}", theme.id, theme.name, cat);
      }
    }
    _ => {
      eprintln!("error: unknown format '{}' (supported: tsv, json)", format);
      std::process::exit(1);
    }
  }

  if !report.warnings.is_empty() {
    for warning in report.warnings {
      eprintln!("warning: {}", warning);
    }
  }
}

fn get_theme_field(id: &str, field: &str) {
  let system_config = Path::new("/usr/share/argvus");
  let report = discover_themes(system_config);

  let theme = match report.themes.iter().find(|t| t.id == id) {
    Some(t) => t,
    None => {
      eprintln!("error: theme '{}' not found", id);
      std::process::exit(1);
    }
  };

  let value = match field {
    "id" => theme.id.clone(),
    "name" => theme.name.clone(),
    "category" => match theme.category {
      ThemeCategory::Dark => "dark".to_string(),
      ThemeCategory::Light => "light".to_string(),
    },
    "accent" => theme.accent.clone(),
    "wallpaper" => theme.wallpaper.clone(),
    _ => {
      eprintln!(
        "error: unknown field '{}' (supported: id, name, category, accent, wallpaper)",
        field
      );
      std::process::exit(1);
    }
  };

  println!("{}", value);
}
