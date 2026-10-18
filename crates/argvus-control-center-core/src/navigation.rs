//! Single source of truth for the navigation root and the Home cards.
//!
//! Breadcrumbs, the Home headings and the global search labels all read the
//! card names from here. A card is translated once, under the same i18n key,
//! so a page's breadcrumb can never drift from the card that lists it.
use argvus_i18n::{Lang, tr};

/// Catalog key of the navigation root (`Control Center` / `Central de Controle`).
pub const ROOT_KEY: &str = "control_center.breadcrumb.root";

/// Separator between the segments of a breadcrumb.
pub const SEPARATOR: &str = " > ";

/// A Home card. Each variant names the catalog key of its title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeCardId {
  Appearance,
  DesktopInput,
  Preferences,
  Hardware,
  System,
  Tools,
}

impl HomeCardId {
  /// Every card, in the order the Home lists them.
  pub const ALL: [HomeCardId; 6] = [
    HomeCardId::Appearance,
    HomeCardId::DesktopInput,
    HomeCardId::Preferences,
    HomeCardId::Hardware,
    HomeCardId::System,
    HomeCardId::Tools,
  ];

  /// Catalog key of the card title.
  pub fn title_key(self) -> &'static str {
    match self {
      HomeCardId::Appearance => "control_center.appearance",
      HomeCardId::DesktopInput => "control_center.card_desktop_input",
      HomeCardId::Preferences => "control_center.preferences",
      HomeCardId::Hardware => "control_center.card_hardware",
      HomeCardId::System => "control_center.card_system",
      HomeCardId::Tools => "control_center.card_tools",
    }
  }

  /// Translated card title for `lang`.
  pub fn label(self, lang: Lang) -> &'static str {
    tr(lang, self.title_key())
  }

  /// Resolves a search category that names a card. Other categories
  /// (domains such as `network`) return `None` and keep their own label.
  pub fn from_search_category(category: &str) -> Option<HomeCardId> {
    HomeCardId::ALL
      .into_iter()
      .find(|card| card.title_key().strip_prefix("control_center.") == Some(category))
  }
}

/// Translated navigation root for `lang`.
pub fn root_label(lang: Lang) -> &'static str {
  tr(lang, ROOT_KEY)
}

/// Joins breadcrumb segments with the shared separator.
pub fn join(segments: &[&str]) -> String {
  segments.join(SEPARATOR)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn en() -> Lang {
    Lang::for_locale("en-US")
  }

  fn pt() -> Lang {
    Lang::for_locale("pt-BR")
  }

  #[test]
  /// Every card and the root resolve to a real translation in both locales,
  /// so no raw key reaches the screen.
  fn cards_and_root_are_translated_in_both_locales() {
    for lang in [en(), pt()] {
      assert_ne!(root_label(lang), ROOT_KEY);
      for card in HomeCardId::ALL {
        assert_ne!(card.label(lang), card.title_key(), "{card:?}");
      }
    }
  }

  #[test]
  /// The root and the cards differ between locales, so the pt-BR screen is
  /// really translated and not a copy of the English text.
  fn pt_br_differs_from_en_us() {
    assert_eq!(root_label(en()), "Control Center");
    assert_eq!(root_label(pt()), "Central de Controle");
    for card in HomeCardId::ALL {
      assert_ne!(card.label(pt()), card.label(en()), "{card:?}");
    }
  }

  #[test]
  /// Search categories that name a card resolve to that card; domain
  /// categories do not.
  fn search_categories_resolve_to_cards() {
    assert_eq!(
      HomeCardId::from_search_category("card_desktop_input"),
      Some(HomeCardId::DesktopInput)
    );
    assert_eq!(
      HomeCardId::from_search_category("appearance"),
      Some(HomeCardId::Appearance)
    );
    assert_eq!(HomeCardId::from_search_category("network"), None);
  }

  #[test]
  /// Segments are joined with the one shared separator.
  fn join_uses_the_shared_separator() {
    assert_eq!(join(&["A", "B", "C"]), "A > B > C");
  }
}
