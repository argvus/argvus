#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Pt,
}

impl Lang {
    pub fn detect() -> Self {
        detect_from_vars([
            std::env::var("LC_ALL").ok(),
            std::env::var("LC_MESSAGES").ok(),
            std::env::var("LANGUAGE").ok(),
            std::env::var("LANG").ok(),
        ])
    }
}

pub fn detect_from_vars<I, S>(vars: I) -> Lang
where
    I: IntoIterator<Item = Option<S>>,
    S: AsRef<str>,
{
    for value in vars.into_iter().flatten() {
        let normalized = value.as_ref().to_ascii_lowercase();
        if normalized.starts_with("pt") || normalized.contains(":pt") {
            return Lang::Pt;
        }
    }
    Lang::En
}

pub fn tr(lang: Lang, pt: &'static str, en: &'static str) -> &'static str {
    match lang {
        Lang::Pt => pt,
        Lang::En => en,
    }
}

pub fn na(lang: Lang) -> &'static str {
    tr(lang, "N/D", "N/A")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_portuguese_from_lang() {
        assert_eq!(detect_from_vars([Some("pt_BR.UTF-8")]), Lang::Pt);
    }

    #[test]
    fn defaults_to_english() {
        assert_eq!(detect_from_vars([Some("en_US.UTF-8"), None]), Lang::En);
    }
}
