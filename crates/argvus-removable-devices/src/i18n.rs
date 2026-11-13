//! Shared catalog-backed translations for removable devices.
use std::sync::OnceLock;
static LANGUAGE: OnceLock<argvus_i18n::Lang> = OnceLock::new();
pub fn init() {
  let locale = argvus_i18n::resolve_installed_locale(&argvus_i18n::user_locale())
    .unwrap_or_else(|| "en-US".to_owned());
  let _ = LANGUAGE.set(argvus_i18n::Lang::for_domain(&locale, "removable-devices"));
}
pub fn tr(key: &str) -> &'static str {
  let language =
    *LANGUAGE.get_or_init(|| argvus_i18n::Lang::for_domain("en-US", "removable-devices"));
  argvus_i18n::tr(language, key)
}

pub fn is_rtl() -> bool {
  LANGUAGE
    .get()
    .copied()
    .unwrap_or_else(|| argvus_i18n::Lang::for_domain("en-US", "removable-devices"))
    .direction()
    == argvus_i18n::TextDirection::RightToLeft
}
