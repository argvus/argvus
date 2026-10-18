//! Stable identifiers of the Settings menu rows.
//!
//! Every selectable row carries an [`Item`] instead of its position, so the
//! action of a row never depends on how many rows precede it. Items of data
//! lists keep the index into their *unfiltered* source, which keeps them
//! valid while a search narrows the list.
use argvus_control_center_apps::catalog::Category;

use crate::config::fonts::{FontTarget, SettingKind};

/// Rows of the hardware-mouse (libratbag) block on Mouse & Touchpad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatbagRow {
  Device,
  Profile,
  Dpi,
  ReportRate,
}

/// The collapsible sections of the GPG keys page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpgSection {
  /// The generate form.
  Generate,
  /// The import actions.
  Import,
}

/// Identity of one Settings menu row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
  // Settings home.
  DefaultApps,
  Fonts,
  LocaleRegion,
  System,
  /// "Restore defaults" of the app, font and font-setting pages (D6).
  ResetDefaults,

  // Default applications.
  Category(Category),
  /// Index into the installed applications of the open category.
  App(usize),

  // Fonts.
  FontTarget(FontTarget),
  FontSetting(SettingKind),
  FontSize,
  /// Index into the system font list.
  Font(usize),
  /// Index into the options of the open rendering setting.
  SettingOption(usize),

  // Locale and region.
  TimeZone,
  DateTime,
  RegionalLocale,
  SystemLocales,
  Keyboard,
  /// Index into the time zone list. Enter only changes the draft.
  Zone(usize),
  /// Writes the Time Zone draft to the system with `timedatectl`.
  ApplyTimezone,
  LocalDateTime,
  Ntp,
  /// Index into the generated locales.
  Locale(usize),
  /// Index into the `/etc/locale.gen` entries.
  LocaleGen(usize),
  ApplyLocales,
  KeyboardLayout,
  KeyboardVariant,
  ConsoleKeymap,
  /// Index into the XKB layouts.
  Layout(usize),
  /// Writes the Keyboard > Layout draft to the Hyprland configuration.
  ApplyKeyboardLayout,
  /// Index into the variants of the current layout.
  Variant(usize),
  /// Index into the console keymaps.
  Keymap(usize),
  /// 0 = English (US), 1 = Portuguese (Brazil).
  Language(usize),

  // System.
  Hostname,
  Users,
  Groups,
  DoNotDisturb,

  // Mouse and touchpad. The index is the one understood by
  // `InputSettings::toggle` / `InputSettings::cycle_by`.
  Input(usize),
  Ratbag(RatbagRow),

  // Keyboard shortcuts.
  /// Index into the loaded window rules. Every per-rule row edits a draft
  /// until `ApplyWindowRule` saves it.
  WindowRuleName(usize),
  WindowRuleWorkspace(usize),
  WindowRuleClasses(usize),
  ApplyWindowRule(usize),
  RemoveWindowRule(usize),
  /// The name typed for the next rule, used by `AddWindowRule`.
  NewWindowRuleName,
  AddWindowRule,
  /// Opens the editor for a new project path, or a new root when true.
  AddProjectPath,
  AddProjectRoot,
  /// Index into the loaded project entries.
  RemoveProject(usize),

  // Snippets.
  /// Name field of the new-snippet form.
  NewSnippetName,
  /// Content field of the new-snippet form.
  NewSnippetContent,
  /// Saves the new-snippet form.
  AddSnippet,
  /// Opens the picker, typed after a short delay.
  OpenSnippetPicker,
  /// Index into the loaded snippets; Enter edits the content.
  SnippetContent(usize),
  /// Types the snippet at this index after a short delay.
  TypeSnippet(usize),
  /// Index into the loaded snippets.
  RemoveSnippet(usize),

  // SSH keys.
  /// Name field of the generate form.
  NewSshKeyName,
  /// Cycles the algorithm of the generate form between ed25519 and rsa.
  SshKeyType,
  /// Generates a pair with the name and algorithm of the form.
  GenerateSshKey,
  /// Index into the loaded key pairs. Copies its `.pub` file to the clipboard.
  CopySshPublicKey(usize),
  /// Index into the loaded key pairs. Opens a terminal for the passphrase.
  LoadSshKey(usize),
  /// Index into the loaded key pairs.
  UnloadSshKey(usize),
  /// Removes every key from the agent. The key files stay on disk.
  UnloadAllSshKeys,
  /// Lists the key pairs and the agent state again, after a terminal load.
  RefreshSshKeys,
  /// Index into the loaded key pairs. Asks for confirmation first.
  DeleteSshKey(usize),

  // GPG keys.
  /// Expands or collapses one section of the GPG keys page.
  ToggleGpgSection(GpgSection),
  /// Full name field of the generate form.
  GpgFullName,
  /// E-mail field of the generate form.
  GpgEmail,
  /// Optional comment field of the generate form.
  GpgComment,
  /// Cycles the algorithm of the generate form between RSA and DSA.
  GpgAlgorithm,
  /// Cycles the key size the algorithm offers.
  GpgKeySize,
  /// Opens the expiration page of the generate form.
  OpenGpgExpiration,
  /// Expiry date field on the expiration page, in the user's date layout.
  GpgExpiry,
  /// Whether the key never expires. Turns the date field off.
  GpgNeverExpires,
  /// Opens the passphrase page of the generate form.
  OpenGpgPassphrase,
  /// Whether the key is protected by a passphrase.
  GpgProtectPassphrase,
  /// Passphrase and its confirmation, on the passphrase page.
  GpgPassphrase,
  GpgPassphraseConfirm,
  /// Generates a key from the form. Without a passphrase it asks first.
  GenerateGpgKey,
  /// Clears every field of the generate form.
  CancelGpgForm,
  /// Opens the file to import public keys from.
  ImportGpgKey,
  /// Asks for confirmation, then opens the file to import a private key from.
  ImportGpgPrivate,
  /// Opens the edit page of the key at this index in the listing.
  OpenGpgKeyEdit(usize),
  /// Asks for confirmation, then opens the path editor for a private export.
  ExportGpgPrivate(usize),
  /// Copies the public key or the fingerprint of the key at this index.
  CopyGpgPublic(usize),
  CopyGpgFingerprint(usize),
  /// Asks for confirmation before the key at this index is deleted.
  DeleteGpgKey(usize),
  /// Opens the expiration page of the key being edited.
  OpenGpgEditExpiration,
  /// Expiry date field of the edit page, and its "never expires" switch.
  GpgEditExpiry,
  GpgEditNeverExpires,
  /// Applies the expiry typed on the edit page.
  ApplyGpgExpiry,
  /// Changes the passphrase of the key being edited.
  GpgEditPasswd,
  /// Opens the page to certify the key being edited with another key.
  OpenGpgSign,
  /// Certifies the key being edited with the key at this index. Asks first.
  GpgSignWith(usize),
  /// Opens the page to revoke the key being edited.
  OpenGpgRevoke,
  /// Revokes the key being edited with this reason. Asks first.
  GpgRevokeWith(crate::system::gpg_keys::RevokeReason),
  /// Sets the trust of the key being edited.
  GpgTrustWith(crate::system::gpg_keys::TrustLevel),
  /// Publishes the key being edited to the keyserver. Asks first.
  GpgKeyserverSend,
  /// Fetches the key being edited from the keyserver.
  GpgKeyserverRecv,
  /// Opens the list of keys in the keyring.
  OpenGpgKeys,
  /// Index into the listed keys. Opens the editor for the export path.
  ExportGpgKey(usize),
  /// Lists the keyring again.
  RefreshGpgKeys,
  /// Index into the loaded keybindings.
  Keybinding(usize),
  RestoreAllShortcuts,
  ChangeShortcut,
  DisableShortcut,
  RestoreShortcut,
  /// Apply the detected shortcut, or capture again when none was detected.
  ApplyCapture,
  CancelCapture,

  // Accounts.
  CreateUser,
  UserList,
  SystemUsers,
  /// Index into the listed users.
  UserEntry(usize),
  Username,
  FullName,
  Shell,
  PrimaryGroup,
  SupplementaryGroups,
  /// 0 = current, 1 = new, 2 = confirmation.
  Password(usize),
  CreateAccount,
  /// Create form: give the new account administration rights (`sudo` group).
  CreateAdmin,
  /// Username field of its own page on the create form (Enter opens the editor).
  UsernameField,
  /// Cancel of the create form: discards it and goes back to the list.
  CancelCreate,
  /// User page: administration rights of the account (`sudo` membership), applied by Save.
  UserAdmin,
  SaveUser,
  ChangePassword,
  LockPassword,
  UnlockPassword,
  ExpirePassword,
  /// User page: starts this account automatically at the next boot, skipping
  /// the login screen (greetd's `initial_session`). Applied immediately.
  ToggleAutoLogin,
  AvatarImage,
  EditAvatar,
  RemoveAvatar,
  /// Full name field of its own page (Enter opens the editor).
  FullNameField,
  /// Ok of the password page: returns to the user page, the change waits for Save.
  OkPassword,
  /// Cancel of the user page: discards the draft and goes back.
  CancelUser,
  /// Delete of the user page: opens the Keep home / Delete home choice.
  DeleteUserMenu,
  DeleteUser,
  DeleteUserAndHome,
  /// Index into the available shells.
  ShellOption(usize),
  /// Index into the system groups (primary group picker).
  PrimaryGroupOption(usize),
  /// Index into the system groups (supplementary groups picker).
  GroupOption(usize),

  // Groups.
  CreateGroup,
  GroupList,
  /// Index into the listed groups.
  GroupEntry(usize),
  GroupName,
  Members,
  /// Index into all users (members picker).
  Member(usize),
  SubmitGroup,
  SaveGroup,
  DeleteGroup,

  // Firewall.
  LoadFirewall,
  FirewallService,
  FirewallBoot,
  /// Index into the firewall configuration fields.
  FirewallField(usize),
  SaveFirewall,
  DiscardFirewall,
  AddRules,
  ApplyRules,
}
