use super::*;

const FP_ALICE: &str = "AAAA111122223333AAAA111122223333AAAAAAAA";
const FP_BOB: &str = "CCCCCCCCCCCCCCCCDDDDDDDDDDDDDDDDEEEEEEEE";

fn filled_form() -> GpgForm {
  GpgForm {
    full_name: "Alice Example".into(),
    email: "alice@example.com".into(),
    expiry: "2099/05/03".into(),
    passphrase: "correct horse".into(),
    passphrase_confirm: "correct horse".into(),
    ..GpgForm::new(DateOrder::YearFirst)
  }
}

#[test]
fn parses_each_listed_primary_key() {
  let listing = format!(
    "{FP_ALICE}\tAAAA111122223333\ted25519\tnever\tyes\tu\tAlice <alice@example.com>\n\
     {FP_BOB}\tCCCCCCCCCCCCCCCC\trsa\t2030-03-17\tno\te\tBob: Test <bob@example.com>\n"
  );
  let entries = parse_list(&listing);
  assert_eq!(entries.len(), 2);
  assert_eq!(entries[0].fingerprint, FP_ALICE);
  assert_eq!(entries[0].key_type, "ed25519");
  assert_eq!(entries[0].expires, "never");
  assert!(entries[0].secret);
  assert_eq!(entries[1].expires, "2030-03-17");
  assert!(!entries[1].secret);
  assert_eq!(entries[1].validity, "e");
  assert_eq!(entries[1].user_id, "Bob: Test <bob@example.com>");
}

#[test]
fn skips_lines_with_another_column_count() {
  let entries = parse_list("AAAA\tAAAA\ted25519\n\n");
  assert!(entries.is_empty());
}

#[test]
fn skips_lines_without_a_fingerprint() {
  let entries = parse_list("\tAAAA\ted25519\tnever\tyes\tu\tAlice\n");
  assert!(entries.is_empty());
}

#[test]
fn reads_the_pinentry_line_and_its_origin() {
  let managed = parse_status("gnupg\tyes\npinentry\t/usr/bin/pinentry-gtk\targvus\n");
  assert_eq!(managed.program.as_deref(), Some("/usr/bin/pinentry-gtk"));
  assert_eq!(managed.origin, PinentryOrigin::Argvus);

  let user = parse_status("pinentry\t/usr/bin/pinentry-curses\tuser\n");
  assert_eq!(user.origin, PinentryOrigin::User);

  let none = parse_status("pinentry\tnone\tdefault\n");
  assert_eq!(none, Pinentry::default());
}

#[test]
fn an_unreadable_status_keeps_the_default_pinentry() {
  assert_eq!(parse_status("garbage\n"), Pinentry::default());
}

#[test]
fn expands_a_leading_tilde_only() {
  assert_eq!(expand_home("~/key.asc", "/home/u"), "/home/u/key.asc");
  assert_eq!(expand_home("~", "/home/u"), "/home/u");
  assert_eq!(expand_home("/tmp/key.asc", "/home/u"), "/tmp/key.asc");
  assert_eq!(expand_home("~other/key.asc", "/home/u"), "~other/key.asc");
}

#[test]
fn the_api_line_gives_the_launcher_version() {
  assert_eq!(parse_api("api\t2\ngnupg\tyes\n"), 2);
  assert_eq!(REQUIRED_LAUNCHER_API, 2);
}

#[test]
fn a_status_without_the_api_line_counts_as_the_first_launcher() {
  assert_eq!(parse_api("gnupg\tyes\npinentry\tnone\tdefault\n"), 1);
  assert_eq!(parse_api(""), 1);
}

#[test]
fn an_unreadable_api_line_counts_as_outdated() {
  assert_eq!(parse_api("api\tnew\n"), 1);
}

// These vectors are the other side of the contract with `argvus gpg`; the
// argvus-keys' gpg command checks the same subcommands and argument counts.
#[test]
fn edit_actions_send_the_words_the_launcher_parses() {
  let fp = FP_ALICE;
  assert_eq!(passwd_args(fp), ["edit", "passwd", fp]);
  assert_eq!(
    trust_args(fp, TrustLevel::Marginal),
    ["edit", "trust", fp, "marginal"]
  );
  assert_eq!(
    trust_args(fp, TrustLevel::Full),
    ["edit", "trust", fp, "full"]
  );
  assert_eq!(
    trust_args(fp, TrustLevel::Ultimate),
    ["edit", "trust", fp, "ultimate"]
  );
  assert_eq!(
    revoke_args(fp, RevokeReason::Unused),
    ["edit", "revoke", fp, "--reason", "unused", "--yes"]
  );
  assert_eq!(sign_args(fp, FP_BOB), ["edit", "sign", fp, FP_BOB, "--yes"]);
  assert_eq!(
    expire_args(fp, "2031-06-30"),
    ["edit", "expire", fp, "2031-06-30"]
  );
}

#[test]
fn keyserver_and_delete_actions_send_the_words_the_launcher_parses() {
  let fp = FP_ALICE;
  assert_eq!(keyserver_recv_args(fp), ["keyserver", "recv", fp]);
  assert_eq!(keyserver_send_args(fp), ["keyserver", "send", fp, "--yes"]);
  assert_eq!(delete_args(fp), ["delete", fp, "--yes"]);
  assert_eq!(
    export_private_args(fp, "/tmp/k.asc"),
    ["export-private", fp, "/tmp/k.asc", "--yes"]
  );
  assert_eq!(
    import_private_args("/tmp/k.asc"),
    ["import-private", "/tmp/k.asc"]
  );
}

#[test]
fn revoke_reasons_and_trust_levels_use_the_launcher_words() {
  assert_eq!(RevokeReason::None.as_arg(), "none");
  assert_eq!(RevokeReason::Compromised.as_arg(), "compromised");
  assert_eq!(RevokeReason::Superseded.as_arg(), "superseded");
  assert_eq!(RevokeReason::Unused.as_arg(), "unused");
  assert_eq!(TrustLevel::Marginal.as_arg(), "marginal");
  assert_eq!(TrustLevel::Ultimate.as_arg(), "ultimate");
  assert_eq!(RevokeReason::ALL.len(), 4);
  assert_eq!(TrustLevel::ALL.len(), 4);
}

#[test]
fn an_edit_that_never_expires_sends_zero_whatever_the_text() {
  assert_eq!(
    expiry_argument(true, "not a date", DateOrder::YearFirst),
    Ok("0".into())
  );
  assert_eq!(
    expiry_argument(false, "2099/01/31", DateOrder::YearFirst),
    Ok("2099-01-31".into())
  );
}

#[test]
fn the_identity_puts_the_comment_between_name_and_email() {
  let mut form = filled_form();
  assert_eq!(form.identity(), "Alice Example <alice@example.com>");
  form.comment = " work ".into();
  assert_eq!(form.identity(), "Alice Example (work) <alice@example.com>");
}

#[test]
fn the_algorithm_and_size_rows_cycle_within_the_algorithm() {
  let mut form = GpgForm::new(DateOrder::YearFirst);
  assert_eq!((form.algorithm, form.key_size), (KeyAlgorithm::Rsa, 4096));
  form.next_key_size();
  assert_eq!(form.key_size, 1024);
  form.next_key_size();
  assert_eq!(form.key_size, 2048);

  form.toggle_algorithm();
  assert_eq!((form.algorithm, form.key_size), (KeyAlgorithm::Dsa, 2048));
  form.next_key_size();
  assert_eq!(form.key_size, 3072);
  form.next_key_size();
  assert_eq!(form.key_size, 1024);
}

#[test]
fn switching_to_dsa_moves_a_size_dsa_does_not_offer_to_its_default() {
  let mut form = GpgForm::new(DateOrder::YearFirst);
  assert_eq!(form.key_size, 4096);
  form.toggle_algorithm();
  assert_eq!(form.key_size, KeyAlgorithm::Dsa.default_key_size());
}

#[test]
fn the_default_form_asks_for_a_passphrase_and_expires_in_two_years() {
  let form = GpgForm::new(DateOrder::DayFirst);
  assert!(form.protect_with_passphrase);
  assert!(!form.never_expires);
  let expected = CalendarDate::today()
    .add_years(2)
    .display(DateOrder::DayFirst);
  assert_eq!(form.expiry, expected);
}

#[test]
fn clearing_the_form_restores_the_defaults() {
  let mut form = filled_form();
  form.clear(DateOrder::YearFirst);
  assert_eq!(form, GpgForm::new(DateOrder::YearFirst));
}

#[test]
fn never_expires_passes_zero_and_ignores_the_date() {
  let form = GpgForm {
    never_expires: true,
    expiry: "not a date".into(),
    ..filled_form()
  };
  assert_eq!(form.expiry_argument(DateOrder::YearFirst), Ok("0".into()));
}

#[test]
fn a_typed_date_becomes_iso_for_gpg() {
  let form = GpgForm {
    expiry: "03/05/2099".into(),
    ..filled_form()
  };
  assert_eq!(
    form.expiry_argument(DateOrder::DayFirst),
    Ok("2099-05-03".into())
  );
}

#[test]
fn a_past_or_impossible_date_is_refused() {
  let past = GpgForm {
    expiry: "2001/01/01".into(),
    ..filled_form()
  };
  assert_eq!(
    past.expiry_argument(DateOrder::YearFirst),
    Err(FormError::DateNotFuture)
  );
  let impossible = GpgForm {
    expiry: "2099/02/30".into(),
    ..filled_form()
  };
  assert_eq!(
    impossible.expiry_argument(DateOrder::YearFirst),
    Err(FormError::DateInvalid)
  );
}

#[test]
fn the_form_needs_a_name_and_a_plausible_email() {
  let no_name = GpgForm {
    full_name: "  ".into(),
    ..filled_form()
  };
  assert_eq!(
    no_name.validate(DateOrder::YearFirst),
    Err(FormError::NameRequired)
  );
  let bad_email = GpgForm {
    email: "alice at example".into(),
    ..filled_form()
  };
  assert_eq!(
    bad_email.validate(DateOrder::YearFirst),
    Err(FormError::EmailInvalid)
  );
}

#[test]
fn a_protected_key_needs_matching_passphrases_without_newlines() {
  let empty = GpgForm {
    passphrase: String::new(),
    passphrase_confirm: String::new(),
    ..filled_form()
  };
  assert_eq!(
    empty.validate(DateOrder::YearFirst),
    Err(FormError::PassphraseEmpty)
  );
  let mismatch = GpgForm {
    passphrase_confirm: "other".into(),
    ..filled_form()
  };
  assert_eq!(
    mismatch.validate(DateOrder::YearFirst),
    Err(FormError::PassphraseMismatch)
  );
  let multiline = GpgForm {
    passphrase: "a\nb".into(),
    passphrase_confirm: "a\nb".into(),
    ..filled_form()
  };
  assert_eq!(
    multiline.validate(DateOrder::YearFirst),
    Err(FormError::PassphraseInvalid)
  );
}

#[test]
fn a_protected_key_sends_the_passphrase_on_stdin_and_never_in_arguments() {
  let request = generate_request(&filled_form(), DateOrder::YearFirst).expect("valid form");
  assert!(request.args.contains(&"--passphrase-stdin".to_owned()));
  assert!(
    request
      .args
      .iter()
      .all(|arg| !arg.contains("correct horse"))
  );
  assert_eq!(request.passphrase.as_deref(), Some("correct horse"));
}

#[test]
fn an_unprotected_key_asks_the_launcher_for_confirmation_and_sends_no_passphrase() {
  let form = GpgForm {
    protect_with_passphrase: false,
    ..filled_form()
  };
  let request = generate_request(&form, DateOrder::YearFirst).expect("valid form");
  assert!(request.args.contains(&"--no-passphrase".to_owned()));
  assert!(request.args.contains(&"--yes".to_owned()));
  assert_eq!(request.passphrase, None);
}

#[test]
fn generation_arguments_carry_the_algorithm_size_and_expiry() {
  let form = GpgForm {
    algorithm: KeyAlgorithm::Dsa,
    key_size: 2048,
    ..filled_form()
  };
  let request = generate_request(&form, DateOrder::YearFirst).expect("valid form");
  assert_eq!(
    request.args,
    [
      "generate",
      "Alice Example <alice@example.com>",
      "--algo",
      "dsa",
      "--bits",
      "2048",
      "--expire",
      "2099-05-03",
      "--passphrase-stdin",
    ]
  );
}
