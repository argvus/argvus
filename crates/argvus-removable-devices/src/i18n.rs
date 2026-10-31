// UI language follows the user locale.conf through the shared ARGVUS resolver.
// Detection is cached at application startup; missing configuration means English.

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
  En,
  Pt,
}

static LANG: OnceLock<Lang> = OnceLock::new();

// Called once from main(); tests and early errors keep the English default
// until then.
pub fn init() {
  let _ = LANG.set(detect());
}

pub fn lang() -> Lang {
  *LANG.get().unwrap_or(&Lang::En)
}

// Picks the translated variant at the call site: tr("english", "portugues").
pub fn tr<'a>(en: &'a str, pt: &'a str) -> &'a str {
  match lang() {
    Lang::Pt => pt,
    Lang::En => en,
  }
}

fn detect() -> Lang {
  language_for_locale(&argvus_i18n::user_locale())
}

fn language_for_locale(locale: &str) -> Lang {
  match argvus_i18n::normalize_locale(locale).as_str() {
    "pt" | "pt-BR" => Lang::Pt,
    _ => Lang::En,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tr_defaults_to_english_before_init() {
    assert_eq!(tr("available", "disponível"), "available");
  }

  #[test]
  fn personal_locale_maps_to_supported_languages() {
    for locale in ["pt_BR.UTF-8", "pt-BR"] {
      assert_eq!(language_for_locale(locale), Lang::Pt);
    }
    for locale in ["en_US.UTF-8", "C.UTF-8", "", "xx_XX"] {
      assert_eq!(language_for_locale(locale), Lang::En);
    }
  }
}
