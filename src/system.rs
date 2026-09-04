use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_type: String,
    pub distributor: String,
    pub kernel: String,
    pub window_system: String,
    pub cpu: String,
    pub memory: String,
    pub gpus: String,
}

impl SystemInfo {
    pub fn gather(na: &str) -> Self {
        let os_release = parse_os_release(&read_to_string("/etc/os-release").unwrap_or_default());

        Self {
            hostname: hostname().unwrap_or_else(|| na.to_string()),
            os_name: os_release
                .get("PRETTY_NAME")
                .or_else(|| os_release.get("NAME"))
                .cloned()
                .unwrap_or_else(|| na.to_string()),
            os_type: format!("{} bits", usize::BITS),
            distributor: os_release
                .get("NAME")
                .cloned()
                .unwrap_or_else(|| na.to_string()),
            kernel: command_output("uname", &["-r"]).unwrap_or_else(|| na.to_string()),
            window_system: session_name().unwrap_or_else(|| na.to_string()),
            cpu: cpu_info(&read_to_string("/proc/cpuinfo").unwrap_or_default())
                .unwrap_or_else(|| na.to_string()),
            memory: memory_info(&read_to_string("/proc/meminfo").unwrap_or_default())
                .unwrap_or_else(|| na.to_string()),
            gpus: gpu_info().unwrap_or_else(|| na.to_string()),
        }
    }
}

pub fn parse_os_release(input: &str) -> HashMap<String, String> {
    input
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            let value = value.trim().trim_matches('"').replace("\\\"", "\"");
            Some((key.to_string(), value))
        })
        .collect()
}

pub fn cpu_info(input: &str) -> Option<String> {
    let model = input
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.trim() == "model name")
        })
        .map(|(_, value)| value.trim().to_string())?;
    let count = input
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    Some(if count > 1 {
        format!("{model} x {count}")
    } else {
        model
    })
}

pub fn memory_info(input: &str) -> Option<String> {
    let kib = input.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key == "MemTotal" {
            value.split_whitespace().next()?.parse::<u64>().ok()
        } else {
            None
        }
    })?;
    Some(format!("{:.1} GiB", kib as f64 / 1024.0 / 1024.0))
}

fn hostname() -> Option<String> {
    read_to_string("/etc/hostname")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| command_output("hostname", &[]))
}

fn session_name() -> Option<String> {
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .ok();

    match (session_type, desktop) {
        (Some(kind), Some(desktop)) if !desktop.is_empty() => Some(format!("{kind} ({desktop})")),
        (Some(kind), _) => Some(kind),
        (_, Some(desktop)) if !desktop.is_empty() => Some(desktop),
        _ => None,
    }
}

fn gpu_info() -> Option<String> {
    if let Some(output) = command_output("lspci", &[]) {
        let gpus = output
            .lines()
            .filter(|line| {
                line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
            })
            .filter_map(|line| line.split_once(": ").map(|(_, value)| value.to_string()))
            .collect::<Vec<_>>();
        if !gpus.is_empty() {
            return Some(gpus.join("\n"));
        }
    }

    let drm = Path::new("/sys/class/drm");
    let cards = fs::read_dir(drm)
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("card") && !name.contains('-'))
        .collect::<Vec<_>>();

    (!cards.is_empty()).then(|| cards.join(", "))
}

fn read_to_string(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_os_release_values() {
        let parsed = parse_os_release("NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\n");
        assert_eq!(parsed.get("NAME").map(String::as_str), Some("Arch Linux"));
    }

    #[test]
    fn parses_cpu_model_and_count() {
        let cpu = "processor\t: 0\nmodel name\t: Example CPU\nprocessor\t: 1\n";
        assert_eq!(cpu_info(cpu).as_deref(), Some("Example CPU x 2"));
    }

    #[test]
    fn formats_memory_as_gib() {
        assert_eq!(
            memory_info("MemTotal:       8388608 kB\n").as_deref(),
            Some("8.0 GiB")
        );
    }
}
