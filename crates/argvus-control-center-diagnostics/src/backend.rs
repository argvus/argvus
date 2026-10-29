//! Isolated integration with system tools and APIs of the Diagnostics page.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_control_center_storage::model::{SmartHealth, UsageLevel, usage_level};
use argvus_i18n::{Lang, tr};
use std::{fs, time::Duration};

pub fn collect(
  cap: &Capabilities,
  cache: &argvus_control_center_storage::backend::StorageCache,
  force: bool,
) -> DiagnosticFacts {
  let storage_result = cache.collect(cap, force);
  let mut facts = DiagnosticFacts {
    storage_failed: storage_result.is_err() && cap.has_lsblk,
    storage: storage_result.unwrap_or_default(),
    ..Default::default()
  };
  if cap.has_pacman {
    facts.package_lock = std::path::Path::new("/var/lib/pacman/db.lck")
      .try_exists()
      .ok();
    match argvus_control_center_packages::backend::PackageBackend::new(
      SystemProcessRunner,
      cap.clone(),
    )
    .updates()
    {
      Ok(updates) => facts.package_updates = Some(updates.len()),
      Err(_) => facts.package_probe_failed = true,
    }
  }
  facts.failed_system_units = cap
    .has_systemd
    .then(|| failed_units(&["--system", "--type=service"]))
    .flatten();
  facts.failed_user_units = cap
    .has_systemd
    .then(|| failed_units(&["--user", "--type=service"]))
    .flatten();
  if cap.has_lspci {
    match run_output("lspci", &["-nnk"]) {
      Some(devices) => facts.nvidia_present = Some(devices.to_ascii_lowercase().contains("nvidia")),
      None => facts.nvidia_probe_failed = true,
    }
  }
  facts.nvidia_module_loaded = fs::read_to_string("/proc/modules").ok().map(|value| {
    value.lines().any(|line| {
      line
        .split_whitespace()
        .next()
        .is_some_and(|module| module == "nvidia" || module.starts_with("nvidia_"))
    })
  });
  if cap.has_glxinfo {
    let output = run_output("glxinfo", &["-B"]);
    facts.opengl_probe_failed = output.is_none();
    facts.opengl_renderer = output
      .as_deref()
      .unwrap_or_default()
      .lines()
      .find_map(|line| line.strip_prefix("OpenGL renderer string:"))
      .map(|value| terminal_text(value.trim()).chars().take(256).collect());
    facts.opengl_probe_failed |= facts.opengl_renderer.is_none();
  }
  if cap.has_vulkaninfo {
    facts.vulkan_available = run_output("vulkaninfo", &["--summary"]).map(|_| true);
    facts.vulkan_probe_failed = facts.vulkan_available.is_none();
  }
  facts
}

fn failed_units(args: &[&str]) -> Option<usize> {
  let mut arguments = vec!["--failed", "--no-legend", "--plain"];
  arguments.extend_from_slice(args);
  run_output("systemctl", &arguments).map(|output| {
    output
      .lines()
      .filter(|line| !line.trim().is_empty())
      .count()
  })
}

fn run_output(program: &str, args: &[&str]) -> Option<String> {
  run_output_with(&SystemProcessRunner, program, args)
}

fn run_output_with(runner: &impl ProcessRunner, program: &str, args: &[&str]) -> Option<String> {
  let request = args
    .iter()
    .fold(ProcessRequest::new(program), |request, arg| {
      request.arg(*arg)
    })
    .timeout(Duration::from_secs(5))
    .output_limit(1024 * 1024);
  runner
    .run(&request)
    .ok()
    .filter(|output| output.status == Some(0) && !output.timed_out)
    .map(|output| terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

pub fn evaluate(lang: Lang, facts: &DiagnosticFacts, cap: &Capabilities) -> Vec<DiagnosticCheck> {
  let mut checks = Vec::new();
  match facts.failed_system_units {
    Some(0) => checks.push(ok(
      lang,
      "services.system",
      tr(lang, "control_center.system_services"),
      tr(lang, "control_center.no_failed_system_units"),
    )),
    Some(n) => checks.push(DiagnosticCheck {
      id: "system.failed_units".into(),
      category: "services".into(),
      severity: Severity::Warning,
      title: tr(lang, "control_center.failed_system_units").into(),
      summary: lang.tr_args(
        "control_center.system_unit_failure_summary",
        [("count", n.to_string())],
      ),
      details: tr(lang, "control_center.system_manager_failed_units").into(),
      remediation_hint: Some(tr(lang, "control_center.open_services_failed").into()),
      route: Some("services/failed".into()),
    }),
    None => checks.push(unknown(
      lang,
      "services.system",
      tr(lang, "control_center.system_services"),
      tr(lang, "control_center.service_state_unavailable"),
    )),
  }
  if facts.failed_user_units == Some(0) {
    let mut check = ok(
      lang,
      "user.failed_units",
      tr(lang, "control_center.failed_user_units"),
      tr(lang, "control_center.doctor_user_units_ok"),
    );
    check.category = "services".into();
    checks.push(check);
  } else if facts.failed_user_units.is_none() {
    let mut check = unknown(
      lang,
      "user.failed_units",
      tr(lang, "control_center.failed_user_units"),
      tr(lang, "control_center.service_state_unavailable"),
    );
    check.category = "services".into();
    checks.push(check);
  }
  if facts.failed_user_units.is_some_and(|n| n > 0) {
    checks.push(DiagnosticCheck {
      id: "user.failed_units".into(),
      category: "services".into(),
      severity: Severity::Warning,
      title: tr(lang, "control_center.failed_user_units").into(),
      summary: lang.tr_args(
        "control_center.user_unit_failure_summary",
        [("count", facts.failed_user_units.unwrap().to_string())],
      ),
      details: tr(lang, "control_center.user_manager_failed_units").into(),
      remediation_hint: Some(tr(lang, "control_center.open_services_user").into()),
      route: Some("services/user".into()),
    });
  }
  if facts.nvidia_present == Some(true) {
    match facts.nvidia_module_loaded {
      Some(true) => checks.push(ok(
        lang,
        "graphics.nvidia",
        tr(lang, "control_center.nvidia_driver"),
        tr(lang, "control_center.nvidia_available"),
      )),
      Some(false) => checks.push(DiagnosticCheck {
        id: "graphics.nvidia".into(),
        category: "graphics".into(),
        severity: Severity::Warning,
        title: tr(lang, "control_center.nvidia_driver").into(),
        summary: tr(lang, "control_center.nvidia_module_missing").into(),
        details: tr(lang, "control_center.nvidia_module_missing").into(),
        remediation_hint: Some(tr(lang, "control_center.check_hardware_gpu").into()),
        route: Some("hardware/gpu".into()),
      }),
      None => checks.push(unknown(
        lang,
        "graphics.nvidia",
        tr(lang, "control_center.nvidia_driver"),
        tr(lang, "control_center.nvidia_module_unavailable"),
      )),
    }
  }
  if let Some(renderer) = facts.opengl_renderer.as_deref() {
    let renderer_lower = renderer.to_ascii_lowercase();
    let software = renderer_lower.contains("llvmpipe")
      || renderer_lower.contains("softpipe")
      || renderer_lower.contains("software rasterizer");
    checks.push(DiagnosticCheck {
      id: "graphics.opengl".into(),
      category: "graphics".into(),
      severity: if software {
        Severity::Warning
      } else {
        Severity::Info
      },
      title: tr(lang, "control_center.opengl_renderer").into(),
      summary: renderer.into(),
      details: if software {
        tr(lang, "control_center.opengl_software_rendering").into()
      } else {
        tr(lang, "control_center.doctor_renderer_reported").into()
      },
      remediation_hint: software.then(|| tr(lang, "control_center.check_hardware_gpu").into()),
      route: Some("hardware/gpu".into()),
    });
  }
  if let Some(available) = facts.vulkan_available {
    checks.push(if available {
      ok(
        lang,
        "graphics.vulkan",
        tr(lang, "control_center.vulkan"),
        tr(lang, "control_center.vulkan_available"),
      )
    } else {
      DiagnosticCheck {
        id: "graphics.vulkan".into(),
        category: "graphics".into(),
        severity: Severity::Warning,
        title: tr(lang, "control_center.vulkan").into(),
        summary: tr(lang, "control_center.vulkan_unavailable").into(),
        details: tr(lang, "control_center.vulkan_init_failed").into(),
        remediation_hint: Some(tr(lang, "control_center.check_graphics_driver").into()),
        route: Some("hardware/gpu".into()),
      }
    });
  }
  let relevant = facts.storage.filesystems.iter().filter(|f| {
    f.mountpoint
      .as_deref()
      .is_some_and(|p| p == "/" || p == "/home")
  });
  for fs in relevant {
    if let (Some(total), Some(available)) = (fs.total_bytes, fs.available_bytes) {
      let level = usage_level(total, available);
      checks.push(DiagnosticCheck {
        id: format!(
          "storage.usage.{}",
          fs.mountpoint.as_deref().unwrap_or("unknown")
        ),
        category: "storage".into(),
        severity: match level {
          UsageLevel::Ok => Severity::Ok,
          UsageLevel::Warning => Severity::Warning,
          UsageLevel::Critical => Severity::Problem,
        },
        title: lang.tr_args(
          "control_center.filesystem_usage",
          [("path", fs.mountpoint.as_deref().unwrap_or("?"))],
        ),
        summary: lang.tr_args(
          "control_center.percent_used",
          [(
            "percent",
            (total.saturating_sub(available).saturating_mul(100) / total.max(1)).to_string(),
          )],
        ),
        details: lang.tr_args(
          "control_center.bytes_available",
          [("bytes", available.to_string())],
        ),
        remediation_hint: Some(tr(lang, "control_center.open_storage_disk_usage").into()),
        route: Some("storage/usage".into()),
      });
    }
  }
  if !checks
    .iter()
    .any(|check| check.id.starts_with("storage.usage."))
  {
    let mut check = unknown(
      lang,
      "storage.usage",
      tr(lang, "control_center.storage"),
      tr(lang, "control_center.doctor_storage_usage_unavailable"),
    );
    check.category = "storage".into();
    checks.push(check);
  }
  if facts.storage.smart_available && !facts.storage.smart.is_empty() {
    for smart in &facts.storage.smart {
      let severity = match smart.health {
        SmartHealth::Failed => Severity::Problem,
        SmartHealth::Warning => Severity::Warning,
        SmartHealth::Passed => Severity::Ok,
        SmartHealth::Unsupported => Severity::Unknown,
        SmartHealth::Unknown => Severity::Unknown,
      };
      checks.push(DiagnosticCheck {
        id: format!("storage.smart.{}", smart.device),
        category: "storage".into(),
        severity,
        title: lang.tr_args(
          "control_center.smart_device",
          [("device", smart.device.as_str())],
        ),
        summary: tr(
          lang,
          match smart.health {
            SmartHealth::Failed => "control_center.doctor_smart_failed",
            SmartHealth::Warning => "control_center.warning",
            SmartHealth::Passed => "control_center.ok",
            SmartHealth::Unsupported | SmartHealth::Unknown => "control_center.doctor_unavailable",
          },
        )
        .into(),
        details: tr(lang, "control_center.smart_evidence").into(),
        remediation_hint: None,
        route: Some("storage/smart".into()),
      });
    }
  } else {
    let mut check = unknown(
      lang,
      "storage.smart",
      "SMART",
      tr(lang, "control_center.doctor_smart_unavailable"),
    );
    check.category = "storage".into();
    checks.push(check);
  }
  if facts.package_lock == Some(true) {
    checks.push(DiagnosticCheck {
      id: "packages.lock".into(),
      category: "packages".into(),
      severity: Severity::Warning,
      title: tr(lang, "control_center.package_database_lock").into(),
      summary: tr(lang, "control_center.doctor_package_lock_present").into(),
      details: tr(lang, "control_center.package_lock_evidence").into(),
      remediation_hint: None,
      route: Some("packages/home".into()),
    });
  }
  if let Some(n) = facts.package_updates {
    checks.push(DiagnosticCheck {
      id: "packages.updates".into(),
      category: "packages".into(),
      severity: if n == 0 { Severity::Ok } else { Severity::Info },
      title: tr(lang, "control_center.package_updates").into(),
      summary: lang.tr_args(
        "control_center.updates_available",
        [("count", n.to_string())],
      ),
      details: tr(lang, "control_center.package_backend_status").into(),
      remediation_hint: None,
      route: Some("packages/updates".into()),
    });
  }
  if !cap.has_pacman || facts.package_lock.is_none() {
    let mut check = unknown(
      lang,
      "packages.lock",
      tr(lang, "control_center.package_database_lock"),
      tr(lang, "control_center.doctor_package_unavailable"),
    );
    check.category = "packages".into();
    checks.push(check);
  }
  if !cap.has_pacman {
    let mut check = unknown(
      lang,
      "packages.updates",
      tr(lang, "control_center.package_updates"),
      tr(lang, "control_center.doctor_package_unavailable"),
    );
    check.category = "packages".into();
    checks.push(check);
  }
  for (id, category, title, failed) in [
    (
      "packages.updates",
      "packages",
      tr(lang, "control_center.package_updates"),
      facts.package_probe_failed,
    ),
    (
      "storage.collect",
      "storage",
      tr(lang, "control_center.storage"),
      facts.storage_failed,
    ),
    (
      "graphics.pci",
      "graphics",
      tr(lang, "control_center.graphics"),
      facts.nvidia_probe_failed,
    ),
    (
      "graphics.opengl",
      "graphics",
      tr(lang, "control_center.opengl_renderer"),
      facts.opengl_probe_failed,
    ),
    (
      "graphics.vulkan",
      "graphics",
      tr(lang, "control_center.vulkan"),
      facts.vulkan_probe_failed,
    ),
  ] {
    if failed {
      checks.push(DiagnosticCheck {
        id: id.into(),
        category: category.into(),
        severity: Severity::Error,
        title: title.into(),
        summary: tr(lang, "control_center.doctor_check_failed").into(),
        details: tr(lang, "control_center.doctor_check_failed_detail").into(),
        ..Default::default()
      });
    }
  }
  for (available, id, title) in [
    (
      cap.has_lspci,
      "graphics.pci",
      tr(lang, "control_center.graphics"),
    ),
    (
      cap.has_glxinfo,
      "graphics.opengl",
      tr(lang, "control_center.opengl_renderer"),
    ),
    (
      cap.has_vulkaninfo,
      "graphics.vulkan",
      tr(lang, "control_center.vulkan"),
    ),
  ] {
    if !available {
      let mut check = unknown(
        lang,
        id,
        title,
        tr(lang, "control_center.doctor_tool_unavailable"),
      );
      check.category = "graphics".into();
      checks.push(check);
    }
  }
  if !cap.has_lsblk {
    let mut check = unknown(
      lang,
      "storage.collect",
      tr(lang, "control_center.storage"),
      tr(lang, "control_center.doctor_storage_unavailable"),
    );
    check.category = "storage".into();
    checks.push(check);
  }
  if let Some(check) = checks
    .iter_mut()
    .find(|check| check.id == "services.system")
  {
    check.category = "services".into();
  }
  checks.push(if cap.is_virtual_machine {
    info(
      lang,
      "graphics.virtual-machine",
      tr(lang, "control_center.graphics_environment"),
      tr(lang, "control_center.virtual_machine_detected"),
    )
  } else {
    info(
      lang,
      "graphics.environment",
      tr(lang, "control_center.graphics_environment"),
      tr(lang, "control_center.hardware_graphics_checks"),
    )
  });
  checks.sort_by_key(|c| c.severity);
  checks
}
fn info(_lang: Lang, id: &str, title: &str, summary: &str) -> DiagnosticCheck {
  DiagnosticCheck {
    id: id.into(),
    category: "system".into(),
    severity: Severity::Info,
    title: title.into(),
    summary: summary.into(),
    details: summary.into(),
    ..Default::default()
  }
}
fn unknown(lang: Lang, id: &str, title: &str, summary: &str) -> DiagnosticCheck {
  DiagnosticCheck {
    severity: Severity::Unknown,
    ..info(lang, id, title, summary)
  }
}
fn ok(lang: Lang, id: &str, title: &str, summary: &str) -> DiagnosticCheck {
  DiagnosticCheck {
    severity: Severity::Ok,
    ..info(lang, id, title, summary)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use argvus_control_center_storage::model::Filesystem;
  #[test]
  fn package_updates_are_informational_and_missing_data_is_unavailable() {
    let capabilities = Capabilities {
      has_pacman: true,
      ..Default::default()
    };
    let facts = DiagnosticFacts {
      package_updates: Some(3),
      package_lock: Some(true),
      ..Default::default()
    };
    let checks = evaluate(Lang::for_locale("en-US"), &facts, &capabilities);
    assert!(checks.iter().any(|check| check.id == "packages.updates"
      && check.severity == Severity::Info
      && check.summary.contains('3')));
    assert!(checks.iter().any(|check| check.id == "packages.lock"
      && check.severity == Severity::Warning
      && check.summary.contains("may")));
    let checks = evaluate(
      Lang::for_locale("en-US"),
      &DiagnosticFacts::default(),
      &Capabilities::default(),
    );
    assert!(
      checks
        .iter()
        .any(|check| check.id == "packages.updates" && check.severity == Severity::Unknown)
    );
  }

  #[test]
  fn timed_out_probe_is_inconclusive_and_has_limits() {
    use argvus_control_center_core::process::{ProcessError, ProcessOutput};
    struct TimeoutRunner;
    impl ProcessRunner for TimeoutRunner {
      fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        assert_eq!(request.program, "vulkaninfo");
        assert_eq!(request.args, ["--summary"]);
        assert_eq!(request.timeout, Some(Duration::from_secs(5)));
        assert_eq!(request.output_limit, Some(1024 * 1024));
        Ok(ProcessOutput {
          stdout: b"partial".to_vec(),
          stderr: vec![],
          status: None,
          timed_out: true,
        })
      }
    }
    assert_eq!(
      run_output_with(&TimeoutRunner, "vulkaninfo", &["--summary"]),
      None
    );
  }

  #[test]
  fn failed_probes_do_not_claim_a_hardware_problem() {
    let facts = DiagnosticFacts {
      nvidia_probe_failed: true,
      vulkan_probe_failed: true,
      storage_failed: true,
      ..Default::default()
    };
    let checks = evaluate(Lang::for_locale("en-US"), &facts, &Capabilities::default());
    for id in ["graphics.pci", "graphics.vulkan", "storage.collect"] {
      assert!(
        checks
          .iter()
          .any(|check| check.id == id && check.severity == Severity::Error)
      );
    }
    assert!(!checks.iter().any(|check| check.id == "graphics.nvidia"));
  }

  #[test]
  fn optional_hardware_and_smart_are_not_problems() {
    let checks = evaluate(
      Lang::for_locale("en-US"),
      &DiagnosticFacts::default(),
      &Capabilities::default(),
    );
    assert!(
      !checks
        .iter()
        .any(|check| check.severity == Severity::Problem)
    );
    assert!(
      checks
        .iter()
        .any(|check| check.id == "storage.smart" && check.severity == Severity::Unknown)
    );
  }

  #[test]
  fn root_usage_severity_is_objective() {
    let mut f = DiagnosticFacts::default();
    f.storage.filesystems.push(Filesystem {
      mountpoint: Some("/".into()),
      total_bytes: Some(100),
      available_bytes: Some(3),
      ..Default::default()
    });
    let c = evaluate(Lang::for_locale("en-US"), &f, &Capabilities::default());
    assert!(
      c.iter()
        .any(|v| v.severity == Severity::Problem && v.id == "storage.usage./")
    );
  }
  #[test]
  fn unavailable_smart_is_info() {
    let c = evaluate(
      Lang::for_locale("en-US"),
      &DiagnosticFacts::default(),
      &Capabilities::default(),
    );
    assert!(
      c.iter()
        .any(|v| v.id == "storage.smart" && v.severity == Severity::Unknown)
    );
  }

  #[test]
  fn reports_failed_services_and_software_rendering() {
    let facts = DiagnosticFacts {
      failed_system_units: Some(2),
      opengl_renderer: Some("llvmpipe (LLVM 20.0.0, 256 bits)".into()),
      ..Default::default()
    };
    let checks = evaluate(Lang::for_locale("en-US"), &facts, &Capabilities::default());
    assert!(
      checks
        .iter()
        .any(|check| { check.id == "system.failed_units" && check.severity == Severity::Warning })
    );
    assert!(
      checks
        .iter()
        .any(|check| { check.id == "graphics.opengl" && check.severity == Severity::Warning })
    );
  }

  #[test]
  fn reports_nvidia_module_gap() {
    let facts = DiagnosticFacts {
      nvidia_present: Some(true),
      nvidia_module_loaded: Some(false),
      ..Default::default()
    };
    let checks = evaluate(Lang::for_locale("en-US"), &facts, &Capabilities::default());
    let check = checks
      .iter()
      .find(|check| check.id == "graphics.nvidia")
      .unwrap();
    assert_eq!(check.severity, Severity::Warning);
    assert!(check.summary.contains("not loaded"));
  }
}
