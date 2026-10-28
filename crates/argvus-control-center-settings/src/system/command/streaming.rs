//! Runs a command and forwards its output line by line, so long operations show live progress.

use super::*;

pub(super) fn locate_command(names: &[&str]) -> Option<PathBuf> {
  std::env::var_os("PATH").and_then(|path| {
    std::env::split_paths(&path)
      .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
      .find(|candidate| candidate.is_file())
  })
}

pub(in super::super) fn stream_command(command: &str, args: &[&str]) -> Result<(), String> {
  stream_command_display(command, Path::new(command), args)
}

pub(super) fn stream_command_path(command: &Path, args: &[&str]) -> Result<(), String> {
  stream_command_display(&command.display().to_string(), command, args)
}

pub(super) fn stream_command_display(
  display: &str,
  command: &Path,
  args: &[&str],
) -> Result<(), String> {
  let mut child = argvus_control_center_core::process::command(command)
    .args(args)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| format!("{display}: {error}"))?;
  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| format!("{display}: stdout was not captured"))?;
  let stderr = child
    .stderr
    .take()
    .ok_or_else(|| format!("{display}: stderr was not captured"))?;
  let stdout_forwarder = std::thread::spawn(move || forward_lines(BufReader::new(stdout)));
  let stderr_forwarder = std::thread::spawn(move || forward_lines(BufReader::new(stderr)));
  let status = child
    .wait()
    .map_err(|error| format!("{display}: {error}"))?;
  let stdout_lines = stdout_forwarder.join().unwrap_or_default();
  let stderr_lines = stderr_forwarder.join().unwrap_or_default();
  let any_output = !stdout_lines.is_empty() || !stderr_lines.is_empty();
  if status.success() {
    if !any_output {
      println!("ok");
    }
    Ok(())
  } else {
    let stderr = stderr_lines.concat().trim().to_string();
    Err(if stderr.is_empty() {
      format!("{display} exited with {status}")
    } else {
      stderr
    })
  }
}

pub(super) fn forward_lines(mut pipe: impl BufRead) -> Vec<String> {
  let mut lines = Vec::new();
  let mut line = String::new();
  loop {
    line.clear();
    match pipe.read_line(&mut line) {
      Ok(0) => break,
      Ok(_) => {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        let _ = handle.write_all(line.as_bytes());
        let _ = handle.flush();
        lines.push(std::mem::take(&mut line));
      }
      Err(_) => break,
    }
  }
  lines
}
