//! Validation of package names, search terms and reflector options before they reach a command line.

use super::*;

pub(super) fn settle_count(
  name: &str,
  result: Result<Result<usize, PackageError>, Box<dyn std::any::Any + Send>>,
  slot: &mut usize,
  error: &mut Option<String>,
) {
  match result {
    Ok(Ok(value)) => *slot = value,
    Ok(Err(failure)) => {
      if error.is_none() {
        *error = Some(failure.to_string());
      }
    }
    Err(_) => {
      if error.is_none() {
        *error = Some(format!("dashboard {name} query panicked"));
      }
    }
  }
}

pub fn validate_package_name(v: &str) -> Result<(), PackageError> {
  if v.is_empty()
    || v.len() > 128
    || v
      .chars()
      .any(|c| !c.is_ascii_alphanumeric() && !matches!(c, '@' | '.' | '_' | '+' | '-'))
  {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}

pub fn validate_search(v: &str) -> Result<(), PackageError> {
  if v.trim().is_empty()
    || v.len() > 128
    || v
      .chars()
      .any(|c| c.is_control() || matches!(c, '\'' | '"' | ';' | '|' | '&'))
  {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}

pub fn validate_aur_search(v: &str) -> Result<(), PackageError> {
  validate_search(v)?;
  if v.trim().chars().count() < 2 {
    Err(PackageError::InvalidPackageName)
  } else {
    Ok(())
  }
}

pub fn validate_package_names(values: &[String]) -> Result<(), PackageError> {
  if values.is_empty() {
    return Err(PackageError::InvalidPackageName);
  }
  values
    .iter()
    .try_for_each(|value| validate_package_name(value))
}

pub fn validate_reflector_options(options: &ReflectorOptions) -> Result<(), PackageError> {
  if options.age_hours == 0
    || options.age_hours > 24 * 365
    || options.count == 0
    || options.count > 100
  {
    return Err(PackageError::InvalidReflectorOption);
  }
  if options.countries.iter().any(|country| {
    country.is_empty()
      || country.len() > 64
      || country
        .chars()
        .any(|value| !value.is_ascii_alphabetic() && value != ' ' && value != '-')
  }) {
    return Err(PackageError::InvalidReflectorOption);
  }
  if options
    .protocols
    .iter()
    .any(|protocol| !matches!(protocol.as_str(), "http" | "https" | "rsync"))
    || !matches!(
      options.sort.as_str(),
      "rate" | "age" | "score" | "delay" | "country"
    )
  {
    return Err(PackageError::InvalidReflectorOption);
  }
  Ok(())
}

pub fn reflector_args(options: &ReflectorOptions) -> Result<Vec<String>, PackageError> {
  validate_reflector_options(options)?;
  let mut args = Vec::new();
  if !options.countries.is_empty() {
    args.push("--country".into());
    args.push(options.countries.join(","));
  }
  if !options.protocols.is_empty() {
    args.push("--protocol".into());
    args.push(options.protocols.join(","));
  }
  args.push("--age".into());
  args.push(options.age_hours.to_string());
  args.push("--latest".into());
  args.push(options.count.to_string());
  args.push("--sort".into());
  args.push(options.sort.clone());
  Ok(args)
}
