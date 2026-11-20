//! GTK front ends for the existing brightness and accent commands.

use std::{ffi::OsString, io, process::ExitCode};

use argvus_i18n::{Lang, tr};
use argvus_theme_core::color::Color;
use argvus_theme_selector::picker::{self, Choice, SearchPicker};
use serde::{Deserialize, de::DeserializeOwned};

use crate::exec::{Failure, Outcome};

#[derive(Deserialize)]
struct BrightnessStatus {
  backlight: bool,
  percent: Option<u8>,
  night_light: Option<NightLightStatus>,
}

#[derive(Deserialize)]
struct NightLightStatus {
  available: bool,
  active: Option<bool>,
  effective_temperature: Option<u32>,
}

pub fn run(args: &[OsString]) -> Outcome {
  let locale = argvus_i18n::user_locale();
  let lang = Lang::for_domain(&locale, "appearance");
  match args {
    [mode] if mode == "brightness" => brightness(&locale, lang, read_input()?),
    [mode] if mode == "accent" => accent(&locale, lang, read_input()?),
    _ => Err(Failure::new(
      "usage: argvus appearance-picker brightness|accent",
    )),
  }
}

fn read_input<T: DeserializeOwned>() -> Result<T, Failure> {
  serde_json::from_reader(io::stdin().lock())
    .map_err(|error| Failure::new(format!("invalid appearance picker input: {error}")))
}

fn show(
  locale: &str,
  lang: Lang,
  namespace: &str,
  title: &str,
  empty: &str,
  rows: &[Choice<'_>],
) -> Result<Option<usize>, Failure> {
  let options = SearchPicker {
    namespace,
    title,
    prompt: tr(lang, "picker.search"),
    empty,
    no_results: tr(lang, "picker.no_results"),
    activate: tr(lang, "picker.apply"),
  };
  picker::choose(locale, lang.direction(), options, rows)
    .map_err(|error| Failure::new(format!("GTK: {error}")))
}

fn brightness(locale: &str, lang: Lang, status: BrightnessStatus) -> Outcome {
  let mut actions = Vec::new();
  if status.backlight {
    for (value, key, label) in [
      ("+10%", "brightness.increase", "+10%"),
      ("+5%", "brightness.increase", "+5%"),
      ("5%-", "brightness.decrease", "−5%"),
      ("10%-", "brightness.decrease", "−10%"),
    ] {
      actions.push((
        value,
        format!("{} {label}", tr(lang, key)),
        "display-brightness-symbolic",
      ));
    }
    for value in ["10%", "25%", "50%", "75%", "90%", "100%"] {
      actions.push((value, value.to_owned(), "display-brightness-symbolic"));
    }
  }
  if let Some(night) = status.night_light.filter(|night| night.available) {
    let label = match (night.active, night.effective_temperature) {
      (Some(true), Some(temperature)) => lang.tr_args(
        "brightness.night_light_on",
        [("temperature", temperature.to_string())],
      ),
      (Some(true), None) => tr(lang, "brightness.night_light_active").to_owned(),
      (Some(false), _) => tr(lang, "brightness.night_light_off").to_owned(),
      (None, _) => tr(lang, "brightness.night_light_toggle").to_owned(),
    };
    actions.push(("night-light", label, "weather-clear-night-symbolic"));
  }
  let title = status
    .percent
    .filter(|percent| *percent <= 100)
    .map_or_else(
      || tr(lang, "brightness.title_short").to_owned(),
      |percent| lang.tr_args("brightness.title", [("percent", percent.to_string())]),
    );
  let rows: Vec<_> = actions
    .iter()
    .map(|(_, label, icon)| Choice {
      icon,
      title: label,
      detail: "",
      shortcut: "",
      swatch: None,
    })
    .collect();
  if let Some(index) = show(
    locale,
    lang,
    "argvus-brightness",
    &title,
    tr(lang, "brightness.controls_unavailable"),
    &rows,
  )? {
    println!("{}", actions[index].0);
  }
  Ok(ExitCode::SUCCESS)
}

fn accent(locale: &str, lang: Lang, palette: Vec<(String, String)>) -> Outcome {
  let rows: Vec<_> = palette
    .iter()
    .map(|(key, hex)| Choice {
      icon: "color-select-symbolic",
      title: tr(lang, key),
      detail: hex,
      shortcut: "",
      swatch: Color::parse(hex),
    })
    .collect();
  let title = tr(lang, "accent.title");
  if let Some(index) = show(
    locale,
    lang,
    "argvus-accent",
    title,
    tr(lang, "picker.no_results"),
    &rows,
  )? {
    println!("{index}");
  }
  Ok(ExitCode::SUCCESS)
}
