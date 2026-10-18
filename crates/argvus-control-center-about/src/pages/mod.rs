//! Implements module declarations for the `pages` subsystem in crate `argvus about`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use argvus_tui::text::{display_width, truncate_to_width};

use crate::app::{App, Tab};
use crate::theme::Theme;

pub mod about;
pub mod copyright;
pub mod credits;
pub mod donate;
pub mod system;

/// Defines the constant `DONATE_URL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const DONATE_URL: &str = "https://argvus.github.io/#support";
/// Defines the constant `ARGVUS_URL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const ARGVUS_URL: &str = "https://argvus.github.io";
/// Defines the constant `WILLIAM_CANIN_URL`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub const WILLIAM_CANIN_URL: &str = "https://williamcanin.github.io";

/// Executes the `doc_for` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn doc_for(app: &App, width: usize, selected: usize) -> Doc<'static> {
  match app.active_tab {
    Tab::System => system::doc(app, width, selected),
    Tab::About => about::doc(app, width, selected),
    Tab::Donate => donate::doc(app, width, selected),
    Tab::Credits => credits::doc(app, width, selected),
    Tab::Copyright => copyright::doc(app, width, selected),
  }
}

#[derive(Debug, Clone)]
/// Represents `Action`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Action {
  pub line: usize,
  pub url: String,
}

#[derive(Debug, Clone, Default)]
/// Represents `Doc`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub struct Doc<'a> {
  pub lines: Vec<Line<'a>>,
  pub actions: Vec<Action>,
}

impl Doc<'_> {
  /// Executes the `height` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  pub fn height(&self) -> usize {
    self.lines.len()
  }
}

/// Defines `Row`. Its explicit shape preserves the contract consumed by the rest of the workspace and keeps the intent visible as the module evolves.
pub enum Row {
  Spacer,
  Section(String),
  Sub(String),
  Para(String),
  Lead(String),
  Divider { label: Option<String> },
  Module { name: String, description: String },
  KeyValue { key: String, value: String },
  Link { label: String, url: String },
}

/// Executes the `simple_doc` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn simple_doc(rows: &[Row], theme: &Theme, width: usize, selected: usize) -> Doc<'static> {
  let width = width.max(10);
  let mut doc = Doc::default();
  let mut link_index = 0;

  for row in rows {
    match row {
      Row::Spacer => doc.lines.push(Line::from("")),
      Row::Section(title) => doc.lines.push(Line::styled(
        title.clone(),
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
      )),
      Row::Sub(title) => doc.lines.push(Line::styled(
        title.clone(),
        Style::new()
          .fg(theme.foreground)
          .add_modifier(Modifier::BOLD),
      )),
      Row::Para(text) => {
        for part in wrap_paragraphs(text, width) {
          doc
            .lines
            .push(Line::styled(part, Style::new().fg(theme.foreground)));
        }
      }
      Row::Lead(text) => {
        for part in wrap_paragraphs(text, width) {
          doc.lines.push(Line::styled(
            part,
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
          ));
        }
      }
      Row::Divider { label } => {
        doc.lines.push(divider_line(label.as_deref(), theme, width));
      }
      Row::Module { name, description } => {
        append_module(&mut doc, name, description, theme, width);
      }
      Row::KeyValue { key, value } => {
        append_key_value(&mut doc, key, value, theme, width);
      }
      Row::Link { label, url } => {
        let is_selected = link_index == selected;
        doc
          .lines
          .push(line_for_link(label, url, theme, is_selected, width));
        doc.actions.push(Action {
          line: doc.lines.len() - 1,
          url: url.clone(),
        });
        link_index += 1;
      }
    }
  }

  doc
}

/// Executes the `append_key_value` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn append_key_value(doc: &mut Doc, key: &str, value: &str, theme: &Theme, width: usize) {
  let key_span = Span::styled(key.to_string(), Style::new().fg(theme.muted));
  let separator = Span::raw("  ");
  let indent = display_width(key).saturating_add(2);

  let mut first = true;
  for part in value.lines() {
    let joined_width = indent.saturating_add(display_width(part));
    if joined_width > width {
      for wrapped in wrap_paragraphs(part, width.saturating_sub(indent)) {
        let line = if first {
          Line::from(vec![
            key_span.clone(),
            separator.clone(),
            Span::styled(wrapped, Style::new().fg(theme.foreground)),
          ])
        } else {
          continuation_line(&wrapped, indent, theme)
        };
        doc.lines.push(line);
        first = false;
      }
    } else {
      let line = if first {
        Line::from(vec![
          key_span.clone(),
          separator.clone(),
          Span::styled(part.to_string(), Style::new().fg(theme.foreground)),
        ])
      } else {
        continuation_line(part, indent, theme)
      };
      doc.lines.push(line);
      first = false;
    }
  }
  if first {
    let line = Line::from(vec![
      key_span,
      separator,
      Span::styled("", Style::new().fg(theme.foreground)),
    ]);
    doc.lines.push(line);
  }
}

/// Executes the `continuation_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn continuation_line(text: &str, indent: usize, theme: &Theme) -> Line<'static> {
  Line::from(vec![
    Span::styled(" ".repeat(indent), Style::new().fg(theme.muted)),
    Span::styled(text.to_string(), Style::new().fg(theme.foreground)),
  ])
}

/// Executes the `divider_line` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn divider_line(label: Option<&str>, theme: &Theme, width: usize) -> Line<'static> {
  let width = width.max(1);
  let Some(label) = label.filter(|label| !label.trim().is_empty()) else {
    return Line::styled("─".repeat(width), Style::new().fg(theme.muted));
  };
  let label = label.trim();
  let available = width.saturating_sub(1);
  let labeled = if display_width(label) + 4 <= available {
    format!(" {} ", label)
  } else {
    format!(
      " {}… ",
      truncate_to_width(label, available.saturating_sub(4))
    )
  };
  let fill = width
    .saturating_sub(1)
    .saturating_sub(display_width(&labeled));
  Line::from(vec![
    Span::styled("─", Style::new().fg(theme.muted)),
    Span::styled(
      labeled,
      Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    ),
    Span::styled("─".repeat(fill), Style::new().fg(theme.muted)),
  ])
}

/// Executes the `append_module` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn append_module(doc: &mut Doc, name: &str, description: &str, theme: &Theme, width: usize) {
  doc.lines.push(Line::styled(
    format!("▪ {}", name),
    Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
  ));
  let indent = 3;
  for part in wrap_paragraphs(description, width.saturating_sub(indent)) {
    doc.lines.push(continuation_line(&part, indent, theme));
  }
}

/// Executes the `line_for_link` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn line_for_link(
  label: &str,
  _url: &str,
  theme: &Theme,
  selected: bool,
  width: usize,
) -> Line<'static> {
  let mut text = label.to_string();
  if display_width(&text) > width {
    text = format!("{}...", truncate_to_width(&text, width.saturating_sub(3)));
  }
  let style = if selected {
    Style::new()
      .fg(theme.selected_foreground)
      .bg(theme.selected_background)
      .add_modifier(Modifier::BOLD)
  } else {
    Style::new().fg(theme.link)
  };
  Line::styled(text, style)
}

/// Executes the `wrap_paragraphs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn wrap_paragraphs(text: &str, width: usize) -> Vec<String> {
  let width = width.max(1);
  text
    .split('\n')
    .flat_map(|paragraph| {
      if paragraph.trim().is_empty() {
        vec![String::new()]
      } else {
        wrap_words(paragraph, width)
      }
    })
    .collect()
}

/// Executes the `wrap_words` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn wrap_words(paragraph: &str, width: usize) -> Vec<String> {
  let mut lines = Vec::new();
  let mut current = String::new();
  for word in paragraph.split_whitespace() {
    if current.is_empty() {
      current = word.to_string();
    } else if display_width(&current) + 1 + display_width(word) <= width {
      current.push(' ');
      current.push_str(word);
    } else {
      lines.push(std::mem::take(&mut current));
      current = word.to_string();
    }
  }
  if !current.is_empty() {
    lines.push(current);
  }
  lines
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Executes the `wraps_long_paragraphs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn wraps_long_paragraphs() {
    let lines = wrap_paragraphs("aaa bbb ccc ddd", 7);
    assert!(lines.iter().all(|line| line.chars().count() <= 7));
    assert!(lines.join(" ") == "aaa bbb ccc ddd");
  }

  #[test]
  /// Executes the `preserves_paragraph_breaks` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn preserves_paragraph_breaks() {
    let lines = wrap_paragraphs("one\n\ntwo", 80);
    assert_eq!(
      lines,
      vec!["one".to_string(), String::new(), "two".to_string()]
    );
  }

  #[test]
  /// Executes the `link_rows_become_actions` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn link_rows_become_actions() {
    let rows = vec![
      Row::Spacer,
      Row::Link {
        label: "site".into(),
        url: "https://argvus.github.io".into(),
      },
      Row::Para("body".into()),
      Row::Link {
        label: "donate".into(),
        url: "https://argvus.github.io/#support".into(),
      },
    ];
    let doc = simple_doc(&rows, &Theme::load(), 80, 1);
    assert_eq!(doc.actions.len(), 2);
    assert_eq!(doc.actions[0].line, 1);
    assert_eq!(doc.actions[1].line, 3);
    assert_eq!(doc.actions[1].url, "https://argvus.github.io/#support");
  }

  #[test]
  /// Executes the `dividers_fit_the_full_width_and_keep_labels` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn dividers_fit_the_full_width_and_keep_labels() {
    let rows = vec![Row::Divider {
      label: Some("Núcleo".into()),
    }];
    let doc = simple_doc(&rows, &Theme::load(), 40, 0);
    assert_eq!(doc.lines.len(), 1);
    let rendered = doc.lines[0].to_string();
    assert!(rendered.contains("Núcleo"));
    assert_eq!(display_width(&rendered), 40);
  }

  #[test]
  /// Executes the `module_rows_bullet_and_indent_description` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn module_rows_bullet_and_indent_description() {
    let rows = vec![Row::Module {
      name: "argvus-session".into(),
      description: "Session lifecycle".into(),
    }];
    let doc = simple_doc(&rows, &Theme::load(), 40, 0);
    assert!(doc.lines[0].to_string().contains("▪ argvus-session"));
    assert!(doc.lines[1].to_string().contains("Session lifecycle"));
    assert!(doc.lines[1].to_string().starts_with("   "));
  }

  #[test]
  /// Executes the `lead_is_emphasized_and_wraps_like_paragraphs` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn lead_is_emphasized_and_wraps_like_paragraphs() {
    let rows = vec![Row::Lead("aaa bbb ccc ddd".into())];
    let doc = simple_doc(&rows, &Theme::load(), 7, 0);
    assert!(doc.lines.len() > 1);
    let rendered = doc.lines[0].to_string();
    assert!(rendered.contains("aaa"));
  }
}
