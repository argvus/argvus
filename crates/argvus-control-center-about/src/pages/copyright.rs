//! Copyright page of About.
use crate::app::App;
use crate::i18n::tr;

use super::{Doc, Row, simple_doc};

const LICENSE_TEXT: &str = include_str!("../../../../LICENSE");

pub fn doc(app: &App, width: usize, selected: usize) -> Doc<'static> {
  let lang = app.lang;
  let rows = vec![
    Row::Section(tr(lang, "control_center.copyright").to_string()),
    Row::Para(
      tr(
        lang,
        "control_center.copyright_2025_william_c_canin_argvus_and_its_modules_are_distributed_",
      )
      .to_string(),
    ),
    Row::Spacer,
    Row::Para(
      tr(
        lang,
        "control_center.this_program_is_free_software_you_can_redistribute_it_and_or_modify_it",
      )
      .to_string(),
    ),
    Row::Para(
      tr(
        lang,
        "control_center.this_program_is_distributed_in_the_hope_that_it_will_be_useful_but_wit",
      )
      .to_string(),
    ),
    Row::Para(
      tr(
        lang,
        "control_center.you_should_have_received_a_copy_of_the_gnu_general_public_license_alon",
      )
      .to_string(),
    ),
    Row::Spacer,
    Row::Section(tr(lang, "control_center.full_license_text").to_string()),
    Row::Para(LICENSE_TEXT.to_string()),
  ];

  simple_doc(&rows, &app.theme, width, selected)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::App;

  #[test]
  fn copyright_embeds_license() {
    let app = App::test();
    let doc = doc(&app, 100, 0);
    let text = doc.lines.iter().map(|l| l.to_string()).collect::<String>();
    assert!(text.contains("GNU GENERAL PUBLIC LICENSE"));
  }
}
