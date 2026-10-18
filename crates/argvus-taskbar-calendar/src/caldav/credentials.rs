use std::collections::HashMap;

use secret_service::{EncryptionType, SecretService};

use crate::error::{ArgvusError, Result};

pub async fn store_password(account_key: &str, username: &str, password: &str) -> Result<String> {
  let ss = SecretService::connect(EncryptionType::Dh)
    .await
    .map_err(|err| ArgvusError::Configuration(format!("secret service unavailable: {err}")))?;
  let collection = ss
    .get_default_collection()
    .await
    .map_err(|err| ArgvusError::Configuration(format!("cannot open default keyring: {err}")))?;
  let lookup = format!("caldav:{account_key}:{username}");
  collection
    .create_item(
      "Argvus Calendar CalDAV credential",
      HashMap::from([
        ("application", "argvus-calendar"),
        ("kind", "caldav"),
        ("lookup", lookup.as_str()),
      ]),
      password.as_bytes(),
      true,
      "text/plain",
    )
    .await
    .map_err(|err| ArgvusError::Configuration(format!("cannot store credential: {err}")))?;
  Ok(lookup)
}

pub async fn load_password(lookup: &str) -> Result<String> {
  let ss = SecretService::connect(EncryptionType::Dh)
    .await
    .map_err(|err| ArgvusError::Configuration(format!("secret service unavailable: {err}")))?;
  let found = ss
    .search_items(HashMap::from([
      ("application", "argvus-calendar"),
      ("lookup", lookup),
    ]))
    .await
    .map_err(|err| ArgvusError::Configuration(format!("cannot search keyring: {err}")))?;
  let item = found.unlocked.first().ok_or_else(|| {
    ArgvusError::Configuration("credential not found in Secret Service".to_string())
  })?;
  let secret = item
    .get_secret()
    .await
    .map_err(|err| ArgvusError::Configuration(format!("cannot read credential: {err}")))?;
  String::from_utf8(secret)
    .map_err(|_| ArgvusError::Configuration("credential is not valid UTF-8".to_string()))
}
