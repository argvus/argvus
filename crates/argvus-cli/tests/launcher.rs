//! Exercises the real JSON-lines session with isolated providers and user state.
use serde_json::{Value, json};
use std::{
  fs,
  io::{BufRead, BufReader, Write},
  os::unix::fs::PermissionsExt,
  path::Path,
  process::{Command, Stdio},
  sync::mpsc,
  time::Duration,
};

fn executable(path: &Path, source: &str) {
  fs::write(path, source).unwrap();
  fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn universal_session_isolates_failed_providers_and_keeps_query_sequences() {
  let temporary = tempfile::tempdir().unwrap();
  let root = temporary.path();
  let config = root.join("config");
  let modules = config.join("argvus/config");
  fs::create_dir_all(&modules).unwrap();
  // The projects provider runs before SSH and can call a host-configured
  // ARGVUS_CONFIG_BIN. Only enable the providers exercised by this test.
  fs::write(
    modules.join("launcher.json"),
    json!({"sources":["settings","ssh"],"recent_files":false,"index_enabled":false}).to_string(),
  )
  .unwrap();
  let commands = root.join("commands");
  fs::create_dir(&commands).unwrap();
  executable(
    &commands.join("control-center"),
    "#!/bin/sh\nprintf '%s\\n' '[{\"id\":\"audio.input\",\"title\":\"Microfone\",\"route\":\"audio/input\",\"keywords\":[\"microfone\"]}]'\n",
  );
  // A provider failure must not prevent other sources from answering queries.
  executable(&commands.join("ssh"), "#!/bin/sh\nexit 1\n");
  let catalogs = Path::new("../../resources/locales");
  let diagnostics = root.join("launcher.stderr");
  let mut child = Command::new(env!("CARGO_BIN_EXE_argvus"))
    .args(["launcher-engine", "serve"])
    .env("HOME", root)
    .env("ARGVUS_CONFIG_HOME", &config)
    .env("XDG_CONFIG_HOME", &config)
    .env_remove("ARGVUS_CONFIG_BIN")
    .env("XDG_CACHE_HOME", root.join("cache"))
    .env("ARGVUS_SYSTEM_CONFIG", root.join("system"))
    .env("ARGVUS_COMMANDS_DIR", &commands)
    .env("ARGVUS_I18N_DIR", catalogs)
    .env("LANG", "pt_BR.UTF-8")
    .env("PATH", &commands)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(fs::File::create(&diagnostics).unwrap())
    .spawn()
    .unwrap();
  let mut input = child.stdin.take().unwrap();
  let output = child.stdout.take().unwrap();
  let (sender, receiver) = mpsc::channel();
  let reader = std::thread::spawn(move || {
    for line in BufReader::new(output).lines() {
      let value: Value =
        serde_json::from_str(&line.unwrap()).expect("stdout must contain JSON only");
      if sender.send(value).is_err() {
        break;
      }
    }
  });
  let result = (|| {
    let mut failed = false;
    let mut found = false;
    writeln!(
      input,
      "{}",
      json!({"op":"query","sequence":10,"query":"microfone"})
    )
    .unwrap();
    for _ in 0..25 {
      let value = receiver
        .recv_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
      if value["event"] == "error" && value["source"] == "ssh" {
        failed = true;
      }
      if value["event"] == "results"
        && value["sequence"] == 10
        && value["entries"].as_array().is_some_and(|entries| {
          entries
            .iter()
            .any(|entry| entry["id"] == "settings:audio.input")
        })
      {
        found = true;
      }
      if failed && found {
        break;
      }
    }
    if !failed || !found {
      return Err("missing provider isolation or microphone result".into());
    }
    writeln!(
      input,
      "{}",
      json!({"op":"query","sequence":11,"query":"zzzz-no-match"})
    )
    .unwrap();
    loop {
      let value = receiver
        .recv_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
      if value["event"] == "results" && value["sequence"] == 11 {
        if value["entries"]
          .as_array()
          .is_none_or(|entries| !entries.is_empty())
        {
          return Err("stale results survived latest query".into());
        }
        break;
      }
    }
    Ok::<_, String>(())
  })();
  drop(input);
  if result.is_err() {
    let _ = child.kill();
  }
  let status = child.wait().unwrap();
  reader.join().unwrap();
  let diagnostics = fs::read_to_string(diagnostics).unwrap();
  assert!(
    result.is_ok(),
    "{result:?}\nlauncher stderr:\n{diagnostics}"
  );
  assert!(
    status.success(),
    "{status}\nlauncher stderr:\n{diagnostics}"
  );
}
