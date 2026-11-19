//! Offline Unicode sequences and localized CLDR annotations.
use gtk::glib::{self, Unichar};
use std::collections::HashMap;

pub struct Emoji {
  pub sequence: String,
  pub category: &'static str,
  pub name: String,
  pub unicode: String,
  pub search: String,
}

type Annotations = HashMap<String, [String; 2]>;

pub fn normalize(text: &str) -> String {
  glib::normalize(glib::casefold(text), glib::NormalizeMode::Default)
    .chars()
    .filter(|character| {
      !matches!(character, '\u{0300}'..='\u{036f}') || character.combining_class() == 0
    })
    .collect()
}

pub fn load(locale: &str) -> Result<Vec<Emoji>, serde_json::Error> {
  let annotations = match locale {
    "ar" => include_str!("../data/ar.json"),
    "bn-BD" => include_str!("../data/bn-BD.json"),
    "de-DE" => include_str!("../data/de-DE.json"),
    "es-ES" => include_str!("../data/es-ES.json"),
    "fr-FR" => include_str!("../data/fr-FR.json"),
    "hi-IN" => include_str!("../data/hi-IN.json"),
    "id-ID" => include_str!("../data/id-ID.json"),
    "it-IT" => include_str!("../data/it-IT.json"),
    "ja-JP" => include_str!("../data/ja-JP.json"),
    "ko-KR" => include_str!("../data/ko-KR.json"),
    "pt-BR" => include_str!("../data/pt-BR.json"),
    "pt-PT" => include_str!("../data/pt-PT.json"),
    "ru-RU" => include_str!("../data/ru-RU.json"),
    "tr-TR" => include_str!("../data/tr-TR.json"),
    "ur-PK" => include_str!("../data/ur-PK.json"),
    "vi-VN" => include_str!("../data/vi-VN.json"),
    "zh-CN" => include_str!("../data/zh-CN.json"),
    _ => "{}",
  };
  let local: Annotations = serde_json::from_str(annotations)?;
  let fallback: Annotations = serde_json::from_str(include_str!("../data/en-US.json"))?;
  let entries: Vec<[String; 3]> = serde_json::from_str(include_str!("../data/emoji.json"))?;
  Ok(
    entries
      .into_iter()
      .map(|[sequence, group, description]| {
        let key = sequence.replace('\u{fe0f}', "");
        let annotation = |index: usize| {
          local
            .get(&key)
            .and_then(|fields| (!fields[index].is_empty()).then_some(fields[index].as_str()))
            .or_else(|| {
              fallback
                .get(&key)
                .map(|fields| fields[index].as_str())
                .filter(|text| !text.is_empty())
            })
        };
        let name = annotation(0).unwrap_or(&description).to_owned();
        let unicode = sequence
          .chars()
          .map(|character| format!("U+{:04X}", u32::from(character)))
          .collect::<Vec<_>>()
          .join(" ");
        let search = normalize(&format!(
          "{name} {} {unicode} {sequence}",
          annotation(1).unwrap_or("")
        ));
        Emoji {
          sequence,
          name,
          unicode,
          search,
          category: match group.as_str() {
            "Smileys & Emotion" => "smileys",
            "People & Body" => "people",
            "Animals & Nature" => "nature",
            "Food & Drink" => "food",
            "Travel & Places" => "travel",
            "Objects" | "Activities" => "objects",
            _ => "symbols",
          },
        }
      })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn localized_names_and_sequences() {
    let entries = load("pt-BR").unwrap();
    let heart = entries.iter().find(|entry| entry.sequence == "❤️").unwrap();
    assert!(heart.search.contains("coracao"));
    assert_eq!(heart.unicode, "U+2764 U+FE0F");
    assert_eq!(heart.category, "smileys");
  }
}
