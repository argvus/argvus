use super::*;

/// The repository part of a real `pacman.conf`: official sections, the
/// default commented examples, two inline repositories and two includes.
const PACMAN_CONF_SAMPLE: &str = "[options]
SigLevel    = Required DatabaseOptional

#[core-testing]
#Include = /etc/pacman.d/mirrorlist

[core]
Include = /etc/pacman.d/mirrorlist

#[custom]
#SigLevel = Optional TrustAll
#Server = file:///home/custompkgs

[sublime-text]
Server = https://download.sublimetext.com/arch/stable/x86_64

[canin]
SigLevel = Optional TrustAll
Server = https://williamcanin.gitlab.io/archlinux/stable/x86_64

# Include = /etc/pacman.d/argvus.conf

Include = /etc/pacman.d/openai-chatgpt.conf
";

const OPENAI_CONF: &str = "### THIS FILE IS AUTOMATICALLY CONFIGURED ###
[openai-chatgpt]
SigLevel = Required DatabaseRequired TrustedOnly
Server = https://persistent.oaistatic.com/codex-app-prod/linux/arch/$arch
";

const ARGVUS_CONF: &str = "# argvus-key: 0123456789ABCDEF0123456789ABCDEF01234567
[argvus]
Server = https://argvus.github.io/packages/arch/$arch
";

fn read_sample(path: &Path) -> Option<String> {
  match path.to_str()? {
    "/etc/pacman.d/openai-chatgpt.conf" => Some(OPENAI_CONF.into()),
    "/etc/pacman.d/argvus.conf" => Some(ARGVUS_CONF.into()),
    _ => None,
  }
}

fn names(repositories: &[Repository]) -> Vec<(&str, bool)> {
  repositories
    .iter()
    .map(|repository| (repository.name.as_str(), repository.enabled))
    .collect()
}

#[test]
fn lists_inline_and_included_repositories_but_not_official_or_examples() {
  let repositories = list_repositories(PACMAN_CONF_SAMPLE, read_sample);
  assert_eq!(
    names(&repositories),
    [
      ("sublime-text", true),
      ("canin", true),
      ("argvus", false),
      ("openai-chatgpt", true)
    ]
  );
  let canin = &repositories[1];
  assert_eq!(canin.origin, RepositoryOrigin::Inline);
  assert_eq!(canin.sig_level.as_deref(), Some("Optional TrustAll"));
  assert_eq!(canin.servers.len(), 1);
  let argvus = &repositories[2];
  assert_eq!(
    argvus.origin,
    RepositoryOrigin::Include("/etc/pacman.d/argvus.conf".into())
  );
  assert_eq!(
    argvus.key.as_deref(),
    Some("0123456789ABCDEF0123456789ABCDEF01234567")
  );
  assert_eq!(repositories[3].servers.len(), 1);
}

#[test]
fn a_disabled_include_without_its_file_is_not_listed() {
  let repositories = list_repositories(PACMAN_CONF_SAMPLE, |_| None);
  assert_eq!(
    names(&repositories),
    [
      ("sublime-text", true),
      ("canin", true),
      ("openai-chatgpt", true)
    ],
    "the active include stays listed by its file name"
  );
}

#[test]
fn includes_are_enabled_disabled_added_and_removed() {
  let argvus = Path::new("/etc/pacman.d/argvus.conf");
  let enabled = add_include(PACMAN_CONF_SAMPLE, argvus);
  assert!(enabled.contains("\nInclude = /etc/pacman.d/argvus.conf\n"));
  assert!(!enabled.contains("# Include = /etc/pacman.d/argvus.conf"));
  assert_eq!(enabled.lines().count(), PACMAN_CONF_SAMPLE.lines().count());

  let openai = Path::new("/etc/pacman.d/openai-chatgpt.conf");
  let disabled = set_include_enabled(PACMAN_CONF_SAMPLE, openai, false).unwrap();
  assert!(disabled.ends_with("# Include = /etc/pacman.d/openai-chatgpt.conf\n"));
  let again = set_include_enabled(&disabled, openai, true).unwrap();
  assert_eq!(again, PACMAN_CONF_SAMPLE);

  let removed = remove_include(PACMAN_CONF_SAMPLE, openai).unwrap();
  assert!(!removed.contains("openai-chatgpt"));
  assert!(remove_include(&removed, openai).is_err());

  let other = Path::new("/etc/pacman.d/new.conf");
  let added = add_include(PACMAN_CONF_SAMPLE, other);
  assert!(added.ends_with("openai-chatgpt.conf\n\nInclude = /etc/pacman.d/new.conf\n"));
}

#[test]
fn inline_sections_are_disabled_with_a_prefix_and_restored() {
  let disabled = set_inline_enabled(PACMAN_CONF_SAMPLE, "canin", false).unwrap();
  assert!(disabled.contains(
    "#argvus-off [canin]\n#argvus-off SigLevel = Optional TrustAll\n#argvus-off Server ="
  ));
  // The comment and the include after the section are untouched.
  assert!(disabled.contains("\n# Include = /etc/pacman.d/argvus.conf\n"));
  assert!(disabled.contains("\nInclude = /etc/pacman.d/openai-chatgpt.conf\n"));
  let listed = list_repositories(&disabled, read_sample);
  let canin = listed
    .iter()
    .find(|repository| repository.name == "canin")
    .unwrap();
  assert!(!canin.enabled);
  assert_eq!(
    canin.servers.len(),
    1,
    "a disabled section keeps its details"
  );
  assert_eq!(
    set_inline_enabled(&disabled, "canin", true).unwrap(),
    PACMAN_CONF_SAMPLE
  );
}

#[test]
fn inline_sections_are_removed_without_leaving_a_double_gap() {
  let removed = remove_inline(PACMAN_CONF_SAMPLE, "sublime-text").unwrap();
  assert!(!removed.contains("sublime"));
  assert!(!removed.contains("\n\n\n"), "{removed}");
  assert!(removed.contains("\n[canin]\n"));
  assert!(remove_inline(PACMAN_CONF_SAMPLE, "core").is_err());
  assert!(remove_inline(PACMAN_CONF_SAMPLE, "missing").is_err());
}

#[test]
fn repository_configurations_are_validated() {
  let sections = parse_repository_conf(ARGVUS_CONF).unwrap();
  assert_eq!(sections[0].name, "argvus");
  assert_eq!(sections[0].servers.len(), 1);
  let form = form_conf(
    "mine",
    "https://example.org/$arch",
    Some("Optional TrustAll"),
  );
  assert_eq!(
    parse_repository_conf(&form).unwrap()[0]
      .sig_level
      .as_deref(),
    Some("Optional TrustAll")
  );
  for (text, expected) in [
    ("", ConfError::Empty),
    (
      "[core]\nServer = https://a\n",
      ConfError::Official("core".into()),
    ),
    ("[options]\nColor\n", ConfError::Official("options".into())),
    (
      "[a b]\nServer = https://a\n",
      ConfError::InvalidName("a b".into()),
    ),
    (
      "Server = https://a\n",
      ConfError::OutsideSection("Server = https://a".into()),
    ),
    (
      "[x]\nServer = ftp://a\n",
      ConfError::InvalidServer("ftp://a".into()),
    ),
    (
      "[x]\nSigLevel = Maybe\nServer = https://a\n",
      ConfError::InvalidSigLevel("Maybe".into()),
    ),
    (
      "[x]\nInclude = /etc/passwd\n",
      ConfError::InvalidInclude("/etc/passwd".into()),
    ),
    ("[x]\nSigLevel = Never\n", ConfError::NoServer("x".into())),
    (
      "[x]\nXferCommand = sh\n",
      ConfError::UnsupportedDirective("XferCommand".into()),
    ),
    (
      "[x]\nServer = https://a\n[x]\nServer = https://b\n",
      ConfError::Duplicate("x".into()),
    ),
  ] {
    assert_eq!(parse_repository_conf(text), Err(expected), "{text:?}");
  }
  assert!(
    parse_repository_conf("[chaotic-aur]\nInclude = /etc/pacman.d/chaotic-mirrorlist\n").is_ok()
  );
}

#[test]
fn key_listings_give_the_primary_fingerprint_and_user_id() {
  let listing = "pub:-:4096:1:0123456789ABCDEF:1700000000:::-:::scESC::::::23::0:
fpr:::::::::0123456789abcdef0123456789abcdef01234567:
uid:-::::1700000000::HASH::ARGVUS Packages <packages@argvus.org>::::::::::0:
sub:-:4096:1:FEDCBA9876543210:1700000000::::::e::::::23:
fpr:::::::::FEDCBA9876543210FEDCBA9876543210FEDCBA98:
";
  assert_eq!(
    parse_key_listing(listing),
    [KeyInfo {
      fingerprint: "0123456789ABCDEF0123456789ABCDEF01234567".into(),
      user_id: Some("ARGVUS Packages <packages@argvus.org>".into()),
    }]
  );
  assert!(parse_key_listing("").is_empty());
}

#[test]
fn keys_round_trip_through_hexadecimal_text() {
  let key = [0x99, 0x01, 0x0d, 0xff, 0x00];
  let text = encode_key(&key);
  assert_eq!(text, "99010dff00");
  assert_eq!(decode_key(&text).as_deref(), Some(&key[..]));
  for invalid in ["", "abc", "zz", "0g"] {
    assert_eq!(decode_key(invalid), None, "{invalid:?}");
  }
}

#[test]
fn names_and_fingerprints_are_checked() {
  assert!(is_valid_name("chaotic-aur"));
  assert!(!is_valid_name("../etc"));
  assert!(!is_valid_name("-x"));
  assert!(is_fingerprint("0123456789abcdef0123456789ABCDEF01234567"));
  assert!(!is_fingerprint("0123"));
  assert_eq!(
    repository_file("argvus"),
    Path::new("/etc/pacman.d/argvus.conf")
  );
}
