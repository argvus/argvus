use super::*;
#[test]
fn protocol_rejects_unknown_operations_and_fields() {
  assert!(serde_json::from_str::<Request>(r#"{"action":"shell","command":"id"}"#).is_err());
  assert!(
    serde_json::from_str::<Request>(r#"{"action":"delete","user":"root","force":true}"#).is_err()
  );
}
#[test]
fn wheel_and_sudo_are_the_admin_groups() {
  assert!(is_admin_group("wheel"));
  assert!(is_admin_group("sudo"));
  assert!(!is_admin_group("users"));
  assert!(!is_admin_group("Wheel"));
}

#[test]
fn protects_root_and_system_users() {
  assert!(protect_account("root").is_err());
  assert!(protect_account("../etc/passwd").is_err());
}

#[test]
fn edits_cannot_target_system_accounts() {
  assert!(protect_system_account("root").is_err());
  assert!(protect_system_account("../etc/passwd").is_err());
}

#[test]
fn create_uses_shadow_argument_vector_and_leaves_password_locked() {
  let mut calls = Vec::new();
  let request = Request::Create {
    user: "argvus_test_created".into(),
    name: "Alice $(id)".into(),
    shell: shells().unwrap()[0].clone(),
    groups: Vec::new(),
    password: None,
  };
  let mut set_password_calls = Vec::new();
  apply_request(
    request,
    &|_| Ok((1000, Vec::new())),
    &mut |program, args| {
      calls.push((
        program.to_string(),
        args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
      ));
      Ok(())
    },
    &mut |user, new| {
      set_password_calls.push((user.to_string(), new.to_string()));
      Ok(())
    },
  )
  .unwrap();
  assert_eq!(calls.len(), 1);
  assert_eq!(calls[0].0, "/usr/bin/useradd");
  assert!(calls[0].1.contains(&"--create-home".into()));
  assert!(calls[0].1.contains(&"Alice $(id)".into()));
  assert!(!calls[0].1.contains(&"--password".into()));
  assert_eq!(calls[0].1.last().unwrap(), "argvus_test_created");
  assert!(set_password_calls.is_empty());
}

#[test]
fn create_with_password_sets_it_without_passing_secret_via_argv() {
  let secret = "correct horse battery staple";
  let mut calls = Vec::new();
  let mut set_password_calls = Vec::new();
  let request = Request::Create {
    user: "argvus_test_created".into(),
    name: "Alice".into(),
    shell: shells().unwrap()[0].clone(),
    groups: Vec::new(),
    password: Some(secret.into()),
  };
  apply_request(
    request,
    &|_| Ok((1000, Vec::new())),
    &mut |program, args| {
      calls.push((
        program.to_string(),
        args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
      ));
      Ok(())
    },
    &mut |user, new| {
      set_password_calls.push((user.to_string(), new.to_string()));
      Ok(())
    },
  )
  .unwrap();
  assert_eq!(calls.len(), 1);
  assert_eq!(calls[0].0, "/usr/bin/useradd");
  assert!(!calls[0].1.contains(&"--password".into()));
  assert_eq!(calls[0].1.last().unwrap(), "argvus_test_created");
  for (_program, args) in &calls {
    for arg in args {
      assert!(
        !arg.contains(secret),
        "secret must never reach argv: {arg:?}"
      );
    }
  }
  assert_eq!(set_password_calls.len(), 1);
  assert_eq!(set_password_calls[0].0, "argvus_test_created");
  assert_eq!(set_password_calls[0].1, secret);
}

#[test]
fn create_rejects_invalid_password_before_running_any_system_command() {
  let request = Request::Create {
    user: "argvus_test_created".into(),
    name: "Alice".into(),
    shell: shells().unwrap()[0].clone(),
    groups: Vec::new(),
    password: Some("line\nbreak".into()),
  };
  assert!(
    apply_request(
      request,
      &|_| Ok((1000, Vec::new())),
      &mut |_, _| panic!("must not run a system command for an invalid password"),
      &mut |_, _| panic!("must not set a password for an invalid password"),
    )
    .is_err()
  );
}

#[test]
fn invalid_creation_never_reaches_shadow_tools() {
  let request = Request::Create {
    user: "--root".into(),
    name: "Invalid".into(),
    shell: "/bin/sh".into(),
    groups: Vec::new(),
    password: None,
  };
  assert!(
    apply_request(
      request,
      &|_| Ok((1000, Vec::new())),
      &mut |_, _| panic!("must not run a system command"),
      &mut |_, _| panic!("must not set a password"),
    )
    .is_err()
  );
}

#[test]
fn cannot_delete_root_even_after_authorization() {
  assert!(
    apply_request(
      Request::Delete {
        user: "root".into(),
        remove_home: false,
      },
      &|_| Ok((1000, Vec::new())),
      &mut |_, _| panic!("must not delete root"),
      &mut |_, _| panic!("must not set a password"),
    )
    .is_err()
  );
}

#[test]
fn edit_group_renames_and_syncs_membership() {
  let mut calls = Vec::new();
  let request = Request::EditGroup {
    group: "staff".into(),
    name: "team".into(),
    members: vec!["alice".into(), "bob".into()],
  };
  let current = |_group: &str| Ok((1001, vec!["carol".into()]));
  apply_request(
    request,
    &current,
    &mut |program, args| {
      calls.push((
        program.to_string(),
        args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
      ));
      Ok(())
    },
    &mut |_, _| panic!("EditGroup must not set a password"),
  )
  .unwrap();
  assert!(calls.contains(&(
    "/usr/bin/groupmod".into(),
    vec![
      "--new-name".into(),
      "team".into(),
      "--".into(),
      "staff".into()
    ]
  )));
  assert!(calls.contains(&(
    "/usr/bin/gpasswd".into(),
    vec![
      "--delete".into(),
      "carol".into(),
      "--".into(),
      "team".into()
    ]
  )));
  assert!(calls.iter().any(|call| call.0 == "/usr/bin/usermod"));
  assert!(calls.iter().any(|call| call.1.contains(&"--append".into())));
}

#[test]
fn delete_group_runs_groupdel() {
  let mut calls = Vec::new();
  let current = |_group: &str| Ok((1001, vec!["alice".into()]));
  apply_request(
    Request::DeleteGroup {
      group: "legacy".into(),
    },
    &current,
    &mut |program, _| {
      calls.push(program.to_string());
      Ok(())
    },
    &mut |_, _| panic!("DeleteGroup must not set a password"),
  )
  .unwrap();
  assert!(calls.contains(&"/usr/bin/groupdel".into()));
}

#[test]
fn system_and_administrator_groups_cannot_be_renamed_or_deleted() {
  for (group, gid) in [("video", 985), ("wheel", 998), ("sudo", 1005)] {
    assert!(
      apply_request(
        Request::DeleteGroup {
          group: group.into(),
        },
        &|_| Ok((gid, Vec::new())),
        &mut |_, _| panic!("must not delete {group}"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err(),
      "{group}"
    );
    assert!(
      apply_request(
        Request::EditGroup {
          group: group.into(),
          name: "renamed".into(),
          members: Vec::new(),
        },
        &|_| Ok((gid, Vec::new())),
        &mut |_, _| panic!("must not rename {group}"),
        &mut |_, _| panic!("must not set a password"),
      )
      .is_err(),
      "{group}"
    );
  }
}

#[test]
fn system_group_membership_can_still_change() {
  let mut calls = Vec::new();
  apply_request(
    Request::EditGroup {
      group: "video".into(),
      name: "video".into(),
      members: vec!["alice".into()],
    },
    &|_| Ok((985, Vec::new())),
    &mut |program, _| {
      calls.push(program.to_string());
      Ok(())
    },
    &mut |_, _| panic!("EditGroup must not set a password"),
  )
  .unwrap();
  assert_eq!(calls, ["/usr/bin/usermod"]);
}

#[test]
fn administrator_cannot_leave_an_admin_group_through_a_group_edit() {
  let me = passwd::current_user().unwrap().username;
  let existing = vec![me.clone()];
  assert!(
    apply_request(
      Request::EditGroup {
        group: "wheel".into(),
        name: "wheel".into(),
        members: Vec::new(),
      },
      &|_| Ok((998, existing.clone())),
      &mut |_, _| panic!("must not drop {me} from wheel"),
      &mut |_, _| panic!("must not set a password"),
    )
    .is_err()
  );
}

#[test]
fn group_operations_reject_missing_or_invalid_groups() {
  assert!(
    apply_request(
      Request::DeleteGroup {
        group: "../etc/passwd".into()
      },
      &|_| Ok((1000, Vec::new())),
      &mut |_, _| panic!("must not delete an invalid group"),
      &mut |_, _| panic!("must not set a password"),
    )
    .is_err()
  );
}

fn user_with_home(home: &std::path::Path) -> passwd::UserInfo {
  passwd::UserInfo {
    username: "alice".into(),
    uid: 1000,
    gid: 1000,
    gecos: String::new(),
    home: home.to_path_buf(),
    shell: "/bin/bash".into(),
  }
}

#[test]
fn snapshot_avatar_is_the_deployed_face_path_or_empty() {
  let home = tempfile::tempdir().expect("temporary home");
  let info = user_with_home(home.path());
  assert_eq!(avatar_of(&info), "");

  let face = home.path().join(avatar::AVATAR_FILENAME);
  std::fs::write(&face, b"png").expect("avatar file");
  assert_eq!(avatar_of(&info), face.display().to_string());
}
