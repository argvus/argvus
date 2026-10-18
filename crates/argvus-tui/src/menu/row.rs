//! Row model of the single vertical menu list.
//!
//! A row describes *what* an entry is (its [`RowKind`]) and carries its own
//! icon, so pages never pick glyphs by index or splice them into labels.

use crate::icons;

/// What a menu row is, which decides focus, markers and the activation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
  /// Label and value. Never receives focus and never shows a decorative icon.
  Info,
  /// Enter runs the action.
  Action,
  /// Enter opens another page; drawn with `›` on the right.
  Submenu,
  /// Enter/Space flips the state. Whether the change is immediate or goes
  /// into a draft is decided by the page, exactly as before the refactor.
  Toggle { on: bool },
  /// One of mutually exclusive options; the current one carries `●`.
  Choice { current: bool },
  /// A number or text. Enter edits it; with a `step`, `←/→` adjust it.
  Value { step: Option<i32> },
  /// An action that must go through the confirmation component.
  Destructive,
  /// Visual separator between groups of rows. Never receives focus.
  Separator,
  /// A section title the cursor can reach. Enter/Space expands or collapses
  /// it; the page decides which rows follow it, from the same state it stores.
  CollapsibleSection { collapsed: bool },
}

impl RowKind {
  /// Whether rows of this kind can ever hold the cursor.
  pub fn is_focusable(self) -> bool {
    !matches!(self, Self::Info | Self::Separator)
  }
}

/// Semantic emphasis resolved to theme colors when the row is drawn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Emphasis {
  #[default]
  Normal,
  /// The page's main action, such as `Apply`.
  Primary,
  /// Destructive or privileged actions.
  Danger,
}

/// One entry of the single vertical list.
///
/// `Id` is the page's stable item identifier (usually a small `Copy` enum).
/// Info rows and separators have no identifier because they never produce
/// events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row<Id> {
  id: Option<Id>,
  kind: RowKind,
  icon: Option<&'static str>,
  label: String,
  detail: Option<String>,
  enabled: bool,
  emphasis: Emphasis,
  marked: bool,
}

impl<Id> Row<Id> {
  fn new(id: Option<Id>, kind: RowKind, label: impl Into<String>) -> Self {
    Self {
      id,
      kind,
      icon: None,
      label: label.into(),
      detail: None,
      enabled: true,
      emphasis: Emphasis::Normal,
      marked: false,
    }
  }

  /// A row that runs an action on Enter.
  pub fn action(id: Id, label: impl Into<String>) -> Self {
    Self::new(Some(id), RowKind::Action, label)
  }

  /// A row that opens another page on Enter.
  pub fn submenu(id: Id, label: impl Into<String>) -> Self {
    Self::new(Some(id), RowKind::Submenu, label)
  }

  /// A row that flips a boolean setting.
  pub fn toggle(id: Id, label: impl Into<String>, on: bool) -> Self {
    Self::new(Some(id), RowKind::Toggle { on }, label)
  }

  /// One option of a mutually exclusive group.
  pub fn choice(id: Id, label: impl Into<String>, current: bool) -> Self {
    Self::new(Some(id), RowKind::Choice { current }, label)
  }

  /// An editable value shown on the right. `step` enables `←/→` adjustment.
  pub fn value(
    id: Id,
    label: impl Into<String>,
    text: impl Into<String>,
    step: Option<i32>,
  ) -> Self {
    Self::new(Some(id), RowKind::Value { step }, label).detail(text)
  }

  /// An action that always goes through the confirmation component.
  pub fn destructive(id: Id, label: impl Into<String>) -> Self {
    Self::new(Some(id), RowKind::Destructive, label).emphasis(Emphasis::Danger)
  }

  /// A read-only label/value pair that the cursor skips.
  pub fn info(label: impl Into<String>, value: impl Into<String>) -> Self {
    let value = value.into();
    let row = Self::new(None, RowKind::Info, label);
    if value.is_empty() {
      row
    } else {
      row.detail(value)
    }
  }

  /// A visual separator that the cursor skips.
  pub fn separator() -> Self {
    Self::new(None, RowKind::Separator, String::new())
  }

  /// A section title that the cursor skips. It is drawn in the theme's
  /// accent color, with no ASCII decoration, above the rows of its section.
  /// [`Row::detail`] adds a dimmed value on the right (for example a
  /// device's availability).
  pub fn section(title: impl Into<String>) -> Self {
    Self::new(None, RowKind::Separator, title)
  }

  /// A section title that expands or collapses on Enter/Space. The page keeps
  /// the state and leaves out the rows of a collapsed section.
  pub fn collapsible_section(id: Id, title: impl Into<String>, collapsed: bool) -> Self {
    Self::new(Some(id), RowKind::CollapsibleSection { collapsed }, title)
  }

  /// Whether this is a section title rather than a plain divider.
  pub fn is_section(&self) -> bool {
    self.kind == RowKind::Separator && !self.label.is_empty()
  }

  /// Sets the item's own icon (a glyph from [`crate::icons`]). Ignored on
  /// Info rows and separators, which never carry decorative icons.
  pub fn icon(mut self, glyph: &'static str) -> Self {
    self.icon = Some(glyph);
    self
  }

  /// Sets the value drawn on the right side of the row.
  pub fn detail(mut self, text: impl Into<String>) -> Self {
    self.detail = Some(text.into());
    self
  }

  /// Enables or disables the row. Disabled rows are dimmed and skipped by
  /// the cursor, for options that are temporarily invalid.
  pub fn enabled(mut self, enabled: bool) -> Self {
    self.enabled = enabled;
    self
  }

  /// Sets the semantic emphasis.
  pub fn emphasis(mut self, emphasis: Emphasis) -> Self {
    self.emphasis = emphasis;
    self
  }

  /// Marks the row as the page's default, drawn with `●` in a column of its
  /// own, before the state marker. The column only appears in lists that
  /// have at least one marked row, so other lists keep their layout.
  pub fn marked(mut self, marked: bool) -> Self {
    self.marked = marked;
    self
  }

  pub fn id(&self) -> Option<&Id> {
    self.id.as_ref()
  }

  pub fn kind(&self) -> RowKind {
    self.kind
  }

  /// The icon to draw, or `None` for Info rows and separators.
  pub fn icon_glyph(&self) -> Option<&'static str> {
    if self.kind.is_focusable() {
      self.icon
    } else {
      None
    }
  }

  pub fn label(&self) -> &str {
    &self.label
  }

  pub fn detail_text(&self) -> Option<&str> {
    self.detail.as_deref()
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn emphasis_kind(&self) -> Emphasis {
    self.emphasis
  }

  pub fn is_marked(&self) -> bool {
    self.marked
  }

  /// Whether the cursor may stop on this row.
  pub fn is_selectable(&self) -> bool {
    self.kind.is_focusable() && self.enabled && self.id.is_some()
  }
}

/// Builds the trailing `Apply` group of a page that edits a draft: a
/// separator followed by the `Apply` action, disabled (and therefore skipped
/// by the cursor) while there are no pending changes.
pub fn draft_actions<Id>(
  apply: Id,
  label: impl Into<String>,
  has_pending_changes: bool,
) -> Vec<Row<Id>> {
  vec![
    Row::separator(),
    Row::action(apply, label)
      .icon(icons::APPLY)
      .emphasis(Emphasis::Primary)
      .enabled(has_pending_changes),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn info_rows_and_separators_are_never_selectable() {
    let info: Row<u8> = Row::info("Kernel", "6.18");
    let separator: Row<u8> = Row::separator();
    assert!(!info.is_selectable());
    assert!(!separator.is_selectable());
    assert_eq!(info.detail_text(), Some("6.18"));
  }

  #[test]
  fn info_rows_drop_decorative_icons() {
    let info: Row<u8> = Row::info("Kernel", "6.18").icon(icons::INFO);
    let action = Row::action(1u8, "Refresh").icon(icons::REFRESH);
    assert_eq!(info.icon_glyph(), None);
    assert_eq!(action.icon_glyph(), Some(icons::REFRESH));
  }

  #[test]
  fn section_titles_are_separators_that_carry_a_title() {
    let section: Row<u8> = Row::section("Account").detail("2");
    assert_eq!(section.kind(), RowKind::Separator);
    assert!(section.is_section());
    assert!(!section.is_selectable());
    assert!(!Row::<u8>::separator().is_section());
  }

  #[test]
  fn disabled_rows_are_not_selectable() {
    let rounding = Row::value(1u8, "Rounding", "8", Some(1)).enabled(false);
    assert!(!rounding.is_selectable());
  }

  #[test]
  fn destructive_rows_default_to_danger_emphasis() {
    assert_eq!(
      Row::destructive(1u8, "Delete").emphasis_kind(),
      Emphasis::Danger
    );
  }

  #[test]
  fn marked_rows_keep_the_mark_flag_and_default_to_unmarked() {
    let plain: Row<u8> = Row::toggle(1, "us", true);
    let default: Row<u8> = Row::toggle(2, "br", true).marked(true);
    assert!(!plain.is_marked());
    assert!(default.is_marked());
  }

  #[test]
  fn draft_apply_is_disabled_without_pending_changes() {
    let clean = draft_actions(7u8, "Apply", false);
    let dirty = draft_actions(7u8, "Apply", true);
    assert_eq!(clean[0].kind(), RowKind::Separator);
    assert!(!clean[1].is_selectable());
    assert!(dirty[1].is_selectable());
    assert_eq!(dirty[1].emphasis_kind(), Emphasis::Primary);
    assert_eq!(dirty[1].icon_glyph(), Some(icons::APPLY));
  }
}
