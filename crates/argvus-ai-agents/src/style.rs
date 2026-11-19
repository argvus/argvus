fn search_backdrop_opacity(window_opacity: f64) -> f64 {
  let target = argvus_theme_selector::render::SELECTOR_BACKDROP_OPACITY;
  // Source-over composition of the tint with the existing window background.
  if window_opacity < target {
    (target - window_opacity) / (1.0 - window_opacity)
  } else {
    0.0
  }
}

pub fn apply_theme(window: &gtk::ApplicationWindow, locale: &str) {
  use argvus_config_core::ConfigStore;
  use argvus_theme_core::color::Color;
  use argvus_theme_selector::model::Catalog;

  let document = ConfigStore::from_environment()
    .and_then(|store| store.load_effective())
    .ok();
  let theme_id = document
    .as_ref()
    .and_then(|document| document.get("/appearance/theme"))
    .and_then(serde_json::Value::as_str)
    .unwrap_or("argvus-dark");
  let theme = Catalog::load(locale, theme_id.to_owned())
    .ok()
    .and_then(|catalog| {
      catalog
        .themes
        .into_iter()
        .find(|theme| theme.id == theme_id)
    });
  let accent = document
    .as_ref()
    .and_then(|document| document.get("/appearance/accent"))
    .and_then(serde_json::Value::as_str)
    .and_then(Color::parse)
    .or_else(|| theme.as_ref().and_then(|theme| theme.token("accent").ok()))
    .unwrap_or_else(|| Color::opaque(53, 144, 189));
  let effect = |name: &str| document.as_ref().and_then(|doc| doc.get(name));
  let enabled = |name| {
    effect(name)
      .and_then(serde_json::Value::as_bool)
      .unwrap_or(true)
  };
  let opacity = if enabled("/effects/transparency_control-center_enabled")
    && enabled("/effects/blur_control-center_enabled")
    && enabled("/effects/blur_global_enabled")
  {
    1.0
      - effect("/effects/transparency_control-center_value")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(15)
        .min(100) as f64
        / 100.0
  } else {
    1.0
  };
  let token = |name, fallback| {
    theme
      .as_ref()
      .and_then(|theme| theme.token(name).ok())
      .unwrap_or(fallback)
      .hex()
  };
  let bg = token("bg", Color::opaque(17, 19, 22));
  let fg = token("fg", Color::opaque(238, 238, 238));
  let on_accent = accent.contrast().hex();
  let accent = accent.hex();
  let radius = document
    .as_ref()
    .map_or(0, |document| document.window_rounding());
  let border = effect("/layout/window/border_size")
    .and_then(serde_json::Value::as_u64)
    .unwrap_or(1)
    .min(10);
  let search_style =
    argvus_theme_selector::render::search_css(".ai-agents", &fg, &accent, &format!("{radius}px"));
  let search_backdrop = search_backdrop_opacity(opacity);
  // GTK 4.22+ derives the Wayland blur region from this filter. Hyprland
  // uses its own blur settings for that region.
  let blur = if opacity < 1.0 && (gtk::major_version(), gtk::minor_version()) >= (4, 22) {
    "backdrop-filter: blur(20px);"
  } else {
    ""
  };
  let provider = gtk::CssProvider::new();
  provider.load_from_string(&format!(
    "window.ai-agents {{ background: transparent; color: {fg}; font: 15px monospace; }}
     .ai-agents .panel {{ background: alpha({bg}, {opacity}); {blur} border: {border}px solid alpha({fg}, .38); border-radius: {radius}px; padding: 20px; }}
     .ai-agents .agent-grid {{ background: transparent; }}
     .ai-agents .agent-card {{ background: alpha({bg}, {search_backdrop}); color: {fg}; border: 1px solid alpha({fg}, .28); border-radius: {radius}px; padding: 10px; }}
     .ai-agents .agent-card:selected {{ background: alpha({accent}, .24); border-color: {accent}; }}
     .ai-agents .agent-card:focus-visible {{ outline: 2px solid {accent}; outline-offset: -2px; }}
     .ai-agents .agent-icon {{ background: #929ba7; border-radius: {radius}px; }}
     .ai-agents .agent-name {{ font-weight: bold; }}
     .ai-agents .heading, .ai-agents .brand {{ font-size: 22px; font-weight: bold; }}
     .ai-agents .brand, .ai-agents .primary {{ background: {accent}; color: {on_accent}; padding: 4px 10px; }}
     .ai-agents .footer {{ border-top: 1px solid {accent}; padding-top: 12px; }}
     {search_style}"
  ));
  gtk::style_context_add_provider_for_display(
    &gtk::prelude::WidgetExt::display(window),
    &provider,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
}
