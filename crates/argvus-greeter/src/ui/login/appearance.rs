//! Per-user avatar and the public theme projection the greeter reads before login.

use super::*;

pub(super) fn valid_public_accent(value: &str) -> Option<String> {
  let value = value.trim();
  (value.len() == 7
    && value.starts_with('#')
    && value[1..]
      .chars()
      .all(|character| character.is_ascii_hexdigit()))
  .then(|| value.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PublicThemeProjection {
  pub(super) name: String,
  pub(super) accent: Option<String>,
}

pub(super) fn read_owned_regular_file(path: &Path, uid: u32) -> Option<String> {
  let metadata = fs::symlink_metadata(path).ok()?;
  (metadata.is_file() && metadata.uid() == uid).then(|| fs::read_to_string(path).ok())?
}

pub(super) fn load_public_theme_projection(directory: &Path, uid: u32) -> PublicThemeProjection {
  let projection = directory.join(uid.to_string());
  let name = read_owned_regular_file(&projection, uid)
    .map(|value| normalize_theme_name(value.trim()).to_owned())
    .unwrap_or_else(|| DEFAULT_THEME.to_owned());
  let accent = read_owned_regular_file(&projection.with_extension("accent"), uid)
    .and_then(|value| valid_public_accent(&value));

  PublicThemeProjection { name, accent }
}

impl LoginApp {
  /// Reuses the decoded image when the selected account has not changed.
  ///
  /// The fallback is loaded from bytes so no user-controlled path is needed
  /// for the default presentation.
  pub(super) fn refresh_avatar(&mut self) {
    let key = self
      .users
      .get(self.selected_user)
      .and_then(|user| user.avatar.clone());
    // Avoid decoding and reinitializing the Kitty/iTerm/sixel protocol on
    // every frame when keyboard input did not change the selected account.
    if key == self.avatar_key && self.avatar.is_some() {
      return;
    }
    self.avatar_key = key.clone();
    self.avatar = key
      .as_deref()
      .and_then(ImageSurface::from_path)
      .or_else(|| ImageSurface::from_bytes(DEFAULT_AVATAR, true));
  }

  /// Loads the selected account's pre-login theme projection.
  ///
  /// Only the UID-named projection is read here. The greeter intentionally
  /// never traverses a user's private home because it runs before PAM has
  /// authenticated that account.
  pub(super) fn refresh_theme(&mut self) {
    let Some(user) = self.users.get(self.selected_user) else {
      return;
    };
    if self.theme_user_uid == Some(user.uid) {
      return;
    }

    let projection = load_public_theme_projection(Path::new(GREETER_THEME_STATE_DIR), user.uid);
    self.theme =
      Theme::load_for_theme_name_and_accent(&projection.name, projection.accent.as_deref());
    self.theme_name = projection.name;
    self.theme_accent = projection.accent;
    self.theme_user_uid = Some(user.uid);
  }
}
