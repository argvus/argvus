//! Contextual help footer derived from the selected row and the page.
//!
//! Instead of one i18n key per screen, the footer is assembled from the
//! selected row's [`RowKind`] plus a few page capabilities, and shows only
//! the keys that apply to the current row. The text keeps the established
//! footer format: `↑/↓ Navigate   →/Enter Open   ←/Esc Back   ? Help`.

use argvus_i18n::{Lang, tr};

use crate::menu::RowKind;

/// Gap between footer segments, matching the existing footer strings.
const SEGMENT_GAP: &str = "   ";

/// Action named by a footer segment; each maps to one translation key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintAction {
  Navigate,
  Scroll,
  Open,
  Run,
  Toggle,
  Select,
  Edit,
  Adjust,
  Back,
  Search,
  Refresh,
  Help,
  Quit,
  Confirm,
  Cancel,
}

impl HintAction {
  fn i18n_key(self) -> &'static str {
    match self {
      Self::Navigate => "control_center.hint.navigate",
      Self::Scroll => "control_center.hint.scroll",
      Self::Open => "control_center.hint.open",
      Self::Run => "control_center.hint.run",
      Self::Toggle => "control_center.hint.toggle",
      Self::Select => "control_center.hint.select",
      Self::Edit => "control_center.hint.edit",
      Self::Adjust => "control_center.hint.adjust",
      Self::Back => "control_center.hint.back",
      Self::Search => "control_center.hint.search",
      Self::Refresh => "control_center.hint.refresh",
      Self::Help => "control_center.hint.help",
      Self::Quit => "control_center.hint.quit",
      Self::Confirm => "control_center.hint.confirm",
      Self::Cancel => "control_center.hint.cancel",
    }
  }
}

/// One footer segment: the keys and the action they trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hint {
  pub keys: &'static str,
  pub action: HintAction,
}

const fn hint(keys: &'static str, action: HintAction) -> Hint {
  Hint { keys, action }
}

/// What the page offers besides the selected row.
#[derive(Debug, Clone, Copy, Default)]
pub struct HintContext<'a> {
  /// Kind of the selected row; `None` when the page is scroll-only.
  pub row: Option<RowKind>,
  /// The page has a parent to go back to.
  pub can_go_back: bool,
  /// `/` opens a search on this page.
  pub search: bool,
  /// `r` refreshes this page.
  pub refresh: bool,
  /// `q` quits from this page (the Control Center Home).
  pub quit: bool,
  /// Page-specific shortcuts, already translated: `(keys, label)`.
  pub extra: &'a [(&'a str, &'a str)],
}

/// The ordered footer segments for `context`, without translation.
pub fn hint_keys(context: &HintContext<'_>) -> Vec<Hint> {
  let mut hints = page_hints(context);
  hints.extend(global_hints(context));
  hints
}

/// Row and page segments: navigation, the row's activation, search, refresh.
fn page_hints(context: &HintContext<'_>) -> Vec<Hint> {
  let mut hints = Vec::new();
  match context.row {
    None => hints.push(hint("↑/↓", HintAction::Scroll)),
    Some(kind) => {
      hints.push(hint("↑/↓", HintAction::Navigate));
      hints.extend(row_hints(kind));
    }
  }
  if context.search {
    hints.push(hint("/", HintAction::Search));
  }
  if context.refresh {
    hints.push(hint("r", HintAction::Refresh));
  }
  hints
}

/// Segments that close every footer: Back, Help and Quit.
fn global_hints(context: &HintContext<'_>) -> Vec<Hint> {
  let mut hints = Vec::new();
  if context.can_go_back {
    // On a Value row with a step, `←` adjusts the value, so only Esc goes back.
    let adjusts = matches!(context.row, Some(RowKind::Value { step: Some(_) }));
    hints.push(hint(
      if adjusts { "Esc" } else { "←/Esc" },
      HintAction::Back,
    ));
  }
  hints.push(hint("?", HintAction::Help));
  if context.quit {
    hints.push(hint("q", HintAction::Quit));
  }
  hints
}

/// Segments specific to the selected row's kind.
fn row_hints(kind: RowKind) -> Vec<Hint> {
  match kind {
    RowKind::Action | RowKind::Destructive => vec![hint("→/Enter", HintAction::Run)],
    RowKind::Submenu => vec![hint("→/Enter", HintAction::Open)],
    RowKind::Toggle { .. } => vec![hint("Enter/Space", HintAction::Toggle)],
    RowKind::Choice { .. } => vec![hint("Enter", HintAction::Select)],
    RowKind::Value { step: Some(_) } => vec![
      hint("←/→", HintAction::Adjust),
      hint("Enter", HintAction::Edit),
    ],
    RowKind::Value { step: None } => vec![hint("Enter", HintAction::Edit)],
    // Never selected; listed for exhaustiveness.
    RowKind::CollapsibleSection { .. } => vec![hint("Enter/Space", HintAction::Toggle)],
    RowKind::Info | RowKind::Separator => Vec::new(),
  }
}

/// The translated footer for a page built with [`crate::menu`]. Page
/// shortcuts from `context.extra` go between the row segments and the
/// global keys, so Back/Help/Quit always close the footer.
pub fn hints(lang: Lang, context: &HintContext<'_>) -> String {
  // A page that names its own `←/→` shortcut replaces the generic Adjust hint.
  let replaces_adjust = context.extra.iter().any(|(keys, _)| *keys == "←/→");
  let page = page_hints(context)
    .into_iter()
    .filter(|hint| !(replaces_adjust && hint.keys == "←/→"))
    .map(|hint| translate(lang, hint));
  let extra = context
    .extra
    .iter()
    .map(|(keys, label)| format!("{keys} {label}"));
  let global = global_hints(context)
    .into_iter()
    .map(|hint| translate(lang, hint));
  page
    .chain(extra)
    .chain(global)
    .collect::<Vec<_>>()
    .join(SEGMENT_GAP)
}

fn translate(lang: Lang, hint: Hint) -> String {
  format!("{} {}", hint.keys, tr(lang, hint.action.i18n_key()))
}

/// Segments shown while the confirmation component is open.
pub fn confirm_hint_keys() -> Vec<Hint> {
  vec![
    hint("↑/↓", HintAction::Select),
    hint("y", HintAction::Confirm),
    hint("n/Esc", HintAction::Cancel),
  ]
}

/// Segments shown while the choice dialog is open: it has no `y`, the
/// option is chosen with Enter.
pub fn choice_hint_keys() -> Vec<Hint> {
  vec![
    hint("↑/↓", HintAction::Navigate),
    hint("Enter", HintAction::Select),
    hint("n/Esc", HintAction::Cancel),
  ]
}

/// The translated footer while the choice dialog is open.
pub fn choice_hints(lang: Lang) -> String {
  choice_hint_keys()
    .into_iter()
    .map(|hint| translate(lang, hint))
    .collect::<Vec<_>>()
    .join(SEGMENT_GAP)
}

/// The translated footer while the confirmation component is open.
pub fn confirm_hints(lang: Lang) -> String {
  confirm_hint_keys()
    .into_iter()
    .map(|hint| translate(lang, hint))
    .collect::<Vec<_>>()
    .join(SEGMENT_GAP)
}

#[cfg(test)]
mod tests {
  use super::*;

  const ALL_ACTIONS: &[HintAction] = &[
    HintAction::Navigate,
    HintAction::Scroll,
    HintAction::Open,
    HintAction::Run,
    HintAction::Toggle,
    HintAction::Select,
    HintAction::Edit,
    HintAction::Adjust,
    HintAction::Back,
    HintAction::Search,
    HintAction::Refresh,
    HintAction::Help,
    HintAction::Quit,
    HintAction::Confirm,
    HintAction::Cancel,
  ];

  /// The footer must never fall back to raw keys: every action needs an
  /// entry in the canonical (en-US) and the pt-BR control-center catalogs,
  /// which live in the workspace `locales` directory this crate builds
  /// against.
  #[test]
  fn every_hint_action_is_translated_in_en_us_and_pt_br() {
    let locales = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../resources/locales");
    for locale in ["en-US", "pt-BR"] {
      let path = locales.join(locale).join("control-center.json");
      let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
      let catalog: serde_json::Value = serde_json::from_str(&content).unwrap();
      for action in ALL_ACTIONS {
        let key = action.i18n_key();
        let text = catalog.get(key).and_then(serde_json::Value::as_str);
        assert!(
          text.is_some_and(|text| !text.is_empty()),
          "{locale} misses {key}"
        );
      }
    }
  }

  fn actions(context: HintContext<'_>) -> Vec<(&'static str, HintAction)> {
    hint_keys(&context)
      .into_iter()
      .map(|hint| (hint.keys, hint.action))
      .collect()
  }

  fn context(row: Option<RowKind>) -> HintContext<'static> {
    HintContext {
      row,
      can_go_back: true,
      ..HintContext::default()
    }
  }

  #[test]
  fn submenu_rows_offer_open_and_back() {
    assert_eq!(
      actions(context(Some(RowKind::Submenu))),
      vec![
        ("↑/↓", HintAction::Navigate),
        ("→/Enter", HintAction::Open),
        ("←/Esc", HintAction::Back),
        ("?", HintAction::Help),
      ]
    );
  }

  #[test]
  fn each_row_kind_shows_only_its_own_activation() {
    let row_actions = |kind| {
      actions(context(Some(kind)))
        .into_iter()
        .map(|(_, action)| action)
        .collect::<Vec<_>>()
    };
    assert!(row_actions(RowKind::Action).contains(&HintAction::Run));
    assert!(row_actions(RowKind::Destructive).contains(&HintAction::Run));
    assert!(row_actions(RowKind::Toggle { on: true }).contains(&HintAction::Toggle));
    assert!(row_actions(RowKind::Choice { current: false }).contains(&HintAction::Select));
    assert!(!row_actions(RowKind::Action).contains(&HintAction::Toggle));
    assert!(!row_actions(RowKind::Submenu).contains(&HintAction::Adjust));
  }

  #[test]
  fn value_rows_with_step_show_adjust_and_back_only_on_esc() {
    let hints = actions(context(Some(RowKind::Value { step: Some(5) })));
    assert!(hints.contains(&("←/→", HintAction::Adjust)));
    assert!(hints.contains(&("Enter", HintAction::Edit)));
    assert!(
      hints.contains(&("Esc", HintAction::Back)),
      "← adjusts here: {hints:?}"
    );

    let plain = actions(context(Some(RowKind::Value { step: None })));
    assert!(
      !plain
        .iter()
        .any(|(_, action)| *action == HintAction::Adjust)
    );
    assert!(plain.contains(&("←/Esc", HintAction::Back)));
  }

  #[test]
  fn scroll_only_pages_show_scroll_instead_of_navigate() {
    let hints = actions(context(None));
    assert_eq!(hints[0], ("↑/↓", HintAction::Scroll));
    assert!(
      !hints
        .iter()
        .any(|(_, action)| *action == HintAction::Navigate)
    );
  }

  #[test]
  fn page_capabilities_add_search_refresh_and_quit() {
    let hints = actions(HintContext {
      row: Some(RowKind::Submenu),
      search: true,
      refresh: true,
      quit: true,
      ..HintContext::default()
    });
    assert!(hints.contains(&("/", HintAction::Search)));
    assert!(hints.contains(&("r", HintAction::Refresh)));
    assert_eq!(hints.last(), Some(&("q", HintAction::Quit)));
    assert!(
      !hints.iter().any(|(_, action)| *action == HintAction::Back),
      "Home has no Back"
    );
  }

  #[test]
  fn confirmation_footer_lists_y_and_n() {
    let actions = confirm_hint_keys()
      .into_iter()
      .map(|hint| (hint.keys, hint.action))
      .collect::<Vec<_>>();
    assert!(actions.contains(&("y", HintAction::Confirm)));
    assert!(actions.contains(&("n/Esc", HintAction::Cancel)));
  }
}
