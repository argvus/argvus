//! Implements isolated integration with system tools and APIs in crate `argvus control center diagnostics`. This separation keeps external effects from contaminating models, routes, or rendering.
//!
//! External tool dependencies remain in backend layers;
//! the UI consumes normalized models and results.
use crate::model::*;
use argvus_control_center_core::{
  capabilities::Capabilities,
  process::{ProcessRequest, ProcessRunner, SystemProcessRunner},
  sanitize::terminal_text,
};
use argvus_control_center_storage::model::{SmartHealth, UsageLevel, usage_level};
use argvus_i18n::{Lang, tr};
use std::{fs, time::Duration};

/// Executes the `collect` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
pub fn collect(cap: &Capabilities) -> DiagnosticFacts {
  let mut facts = DiagnosticFacts {
    storage: argvus_control_center_storage::backend::collect(cap).unwrap_or_default(),
    ..Default::default()
  };
  facts.failed_system_units = cap
    .has_systemd
    .then(|| failed_units(&["--system", "--type=service"]))
    .flatten();
  facts.failed_user_units = cap
    .has_systemd
    .then(|| failed_units(&["--user", "--type=service"]))
    .flatten();
  if cap.has_lspci {
    let devices = run("lspci", &["-nnk"]);
    facts.nvidia_present = Some(devices.to_ascii_lowercase().contains("nvidia"));
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
    let output = run("glxinfo", &["-B"]);
    facts.opengl_renderer = output
      .lines()
      .find_map(|line| line.strip_prefix("OpenGL renderer string:"))
      .map(|value| terminal_text(value.trim()));
  }
  if cap.has_vulkaninfo {
    facts.vulkan_available = Some(run_status("vulkaninfo", &["--summary"]));
  }
  facts
}

/// Executes the `failed_units` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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

/// Executes the `run` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run(program: &str, args: &[&str]) -> String {
  run_output(program, args).unwrap_or_default()
}

/// Executes the `run_output` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_output(program: &str, args: &[&str]) -> Option<String> {
  let request = args
    .iter()
    .fold(ProcessRequest::new(program), |request, arg| {
      request.arg(*arg)
    })
    .timeout(Duration::from_secs(5));
  SystemProcessRunner
    .run(&request)
    .ok()
    .filter(|output| output.status == Some(0) && !output.timed_out)
    .map(|output| terminal_text(&String::from_utf8_lossy(&output.stdout)))
}

/// Executes the `run_status` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn run_status(program: &str, args: &[&str]) -> bool {
  let request = args
    .iter()
    .fold(ProcessRequest::new(program), |request, arg| {
      request.arg(*arg)
    })
    .timeout(Duration::from_secs(5));
  SystemProcessRunner
    .run(&request)
    .is_ok_and(|output| output.status == Some(0) && !output.timed_out)
}

/// Executes the `evaluate` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
        Severity::Ok
      },
      title: tr(lang, "control_center.opengl_renderer").into(),
      summary: renderer.into(),
      details: if software {
        tr(lang, "control_center.opengl_software_rendering").into()
      } else {
        tr(lang, "control_center.opengl_hardware_rendering").into()
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
          UsageLevel::Critical => Severity::Error,
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
  if facts.storage.smart_available {
    for smart in &facts.storage.smart {
      let severity = match smart.health {
        SmartHealth::Failed => Severity::Error,
        SmartHealth::Warning => Severity::Warning,
        SmartHealth::Passed => Severity::Ok,
        SmartHealth::Unsupported => Severity::Info,
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
        summary: format!("{:?}", smart.health),
        details: tr(lang, "control_center.smart_evidence").into(),
        remediation_hint: None,
        route: Some("storage/smart".into()),
      });
    }
  } else {
    checks.push(info(
      lang,
      "storage.smart",
      "SMART",
      "smartctl is unavailable",
    ));
  }
  if facts.package_lock {
    checks.push(DiagnosticCheck {
      id: "packages.lock".into(),
      category: "packages".into(),
      severity: Severity::Warning,
      title: tr(lang, "control_center.package_database_lock").into(),
      summary: tr(lang, "control_center.package_operation_active").into(),
      details: tr(lang, "control_center.package_lock_evidence").into(),
      remediation_hint: None,
      route: Some("packages".into()),
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
/// Executes the `info` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
/// Executes the `unknown` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
fn unknown(lang: Lang, id: &str, title: &str, summary: &str) -> DiagnosticCheck {
  DiagnosticCheck {
    severity: Severity::Unknown,
    ..info(lang, id, title, summary)
  }
}
/// Executes the `ok` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `root_usage_severity_is_objective` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
        .any(|v| v.severity == Severity::Error && v.id == "storage.usage./")
    );
  }
  #[test]
  /// Executes the `unavailable_smart_is_info` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
  fn unavailable_smart_is_info() {
    let c = evaluate(
      Lang::for_locale("en-US"),
      &DiagnosticFacts::default(),
      &Capabilities::default(),
    );
    assert!(
      c.iter()
        .any(|v| v.id == "storage.smart" && v.severity == Severity::Info)
    );
  }

  #[test]
  /// Executes the `reports_failed_services_and_software_rendering` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
  /// Executes the `reports_nvidia_module_gap` step in this module. The behavior is encapsulated here so callers depend on a clear domain decision instead of duplicating system or UI details.
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
