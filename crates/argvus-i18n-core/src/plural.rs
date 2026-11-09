//! CLDR cardinal rules for integer counts supported by `tr_plural`.
//! Decimal and ordinal formatting are deliberately outside this API.

pub(crate) fn category(locale: &str, count: i64) -> &'static str {
  let count = count.unsigned_abs();
  let language = locale.split('-').next().unwrap_or("en");
  match language {
    "zh" | "ja" | "ko" => "other",
    "ru" => match (count % 10, count % 100) {
      (1, last_two) if last_two != 11 => "one",
      (2..=4, last_two) if !(12..=14).contains(&last_two) => "few",
      _ => "many",
    },
    "fr" | "pt" if count <= 1 => "one",
    "fr" | "pt" | "es" | "it" if count != 0 && count.is_multiple_of(1_000_000) => "many",
    _ if count == 1 => "one",
    _ => "other",
  }
}

pub(crate) fn extra_categories(locale: &str) -> &'static [&'static str] {
  match locale.split('-').next().unwrap_or("en") {
    "ru" => &["few", "many"],
    "fr" | "pt" | "es" | "it" => &["many"],
    _ => &[],
  }
}

#[cfg(test)]
mod tests {
  use super::category;

  #[test]
  fn integer_cardinals_cover_supported_languages_and_boundaries() {
    for locale in ["zh-CN", "ja-JP", "ko-KR"] {
      for count in [0, 1, 2, 21, i64::MIN] {
        assert_eq!(category(locale, count), "other");
      }
    }
    for (count, expected) in [
      (0, "many"),
      (1, "one"),
      (2, "few"),
      (5, "many"),
      (11, "many"),
      (12, "many"),
      (14, "many"),
      (21, "one"),
      (22, "few"),
      (-22, "few"),
    ] {
      assert_eq!(category("ru-RU", count), expected);
    }
    for locale in ["fr-FR", "pt-BR"] {
      assert_eq!(category(locale, 0), "one");
    }
    for locale in ["es-ES", "it-IT", "fr-FR", "pt-BR"] {
      assert_eq!(category(locale, 1_000_000), "many");
    }
    for locale in ["en-US", "de-DE", "es-ES", "it-IT"] {
      assert_eq!(category(locale, 0), "other");
      assert_eq!(category(locale, 1), "one");
      assert_eq!(category(locale, 2), "other");
    }
  }
}
