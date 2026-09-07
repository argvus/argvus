use gtk::prelude::*;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::time::Duration;

use crate::i18n::{Lang, na, tr};
use crate::system::SystemInfo;
use crate::theme::ThemeCss;

const DONATE_URL: &str = "https://argvus.github.io/#support";
const ARGVUS_URL: &str = "https://argvus.github.io";
const WILLIAM_CANIN_URL: &str = "https://williamcanin.github.io";

pub fn build(app: &gtk::Application) {
    install_css();

    let lang = Lang::detect();
    let info = SystemInfo::gather(na(lang));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(tr(lang, "Sobre o ARGVUS", "About ARGVUS"))
        .default_width(900)
        .default_height(720)
        .build();

    window.add_css_class("argvus-about");

    let header = gtk::HeaderBar::new();
    let title = gtk::Label::new(Some(tr(lang, "Sobre o ARGVUS", "About ARGVUS")));
    title.add_css_class("window-title");
    header.set_title_widget(Some(&title));
    window.set_titlebar(Some(&header));

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("root");

    let notebook = gtk::Notebook::new();
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    notebook.append_page(
        &system_tab(lang, &info),
        Some(&gtk::Label::new(Some(tr(lang, "Sistema", "System")))),
    );
    notebook.append_page(
        &about_tab(lang),
        Some(&gtk::Label::new(Some(tr(lang, "Sobre", "About")))),
    );
    notebook.append_page(&donate_tab(lang), Some(&gtk::Label::new(Some("Donate"))));
    notebook.append_page(
        &credits_tab(lang),
        Some(&gtk::Label::new(Some(tr(lang, "Créditos", "Credits")))),
    );
    notebook.append_page(
        &copyright_tab(lang),
        Some(&gtk::Label::new(Some(tr(
            lang,
            "Direitos autorais",
            "Copyright",
        )))),
    );
    root.append(&notebook);

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("actions");
    actions.set_halign(gtk::Align::End);

    let close = gtk::Button::with_label(tr(lang, "Fechar", "Close"));
    let window_weak = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(window) = window_weak.upgrade() {
            window.close();
        }
    });
    actions.append(&close);
    root.append(&actions);

    window.set_child(Some(&root));
    window.present();
}

fn install_css() {
    let provider = gtk::CssProvider::new();
    let theme = Rc::new(RefCell::new(ThemeCss::new()));
    let css = theme
        .borrow_mut()
        .render(include_str!("../assets/style.css"), &app_font_css());
    provider.load_from_string(&css);
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    gtk::glib::timeout_add_local(Duration::from_secs(1), move || {
        let mut theme = theme.borrow_mut();
        if theme.changed() {
            let css = theme.render(include_str!("../assets/style.css"), &app_font_css());
            provider.load_from_string(&css);
        }
        gtk::glib::ControlFlow::Continue
    });
}

fn app_font_css() -> String {
    let (family, size) = argvus_font("apps", "Terminus (TTF)", 13);
    format!(
        ".argvus-about {{ font-family: \"{}\", monospace; font-size: {}px; }}",
        css_escape(&family),
        size
    )
}

fn argvus_font(prefix: &str, fallback_family: &str, fallback_size: u32) -> (String, u32) {
    let config_home = std::env::var_os("ARGVUS_CONFIG_HOME")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                .join(".config")
        });
    let path = config_home.join("argvus").join("fonts.conf");
    let Ok(contents) = std::fs::read_to_string(path) else {
        return (fallback_family.to_string(), fallback_size);
    };

    let family = read_font_key(&contents, &format!("{prefix}_family"))
        .or_else(|| read_font_key(&contents, "default_family"))
        .unwrap_or_else(|| fallback_family.to_string());
    let size = read_font_key(&contents, &format!("{prefix}_size"))
        .or_else(|| read_font_key(&contents, "default_size"))
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(fallback_size)
        .clamp(8, 32);
    (family, size)
}

fn read_font_key(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn css_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn system_tab(lang: Lang, info: &SystemInfo) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 36);
    content.add_css_class("tab-page");

    content.append(&logo_picture(300));

    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(18)
        .hexpand(true)
        .vexpand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    let rows = [
        (tr(lang, "Dispositivo", "Device"), info.hostname.as_str()),
        (tr(lang, "Nome do S.O.", "OS name"), info.os_name.as_str()),
        (tr(lang, "Tipo do S.O.", "OS type"), info.os_type.as_str()),
        (
            tr(lang, "Distribuidor", "Distributor"),
            info.distributor.as_str(),
        ),
        ("ARGVUS", env!("CARGO_PKG_VERSION")),
        ("GTK", &gtk_version()),
        (tr(lang, "Kernel", "Kernel"), info.kernel.as_str()),
        (
            tr(lang, "Sistema de janelas", "Window system"),
            info.window_system.as_str(),
        ),
        ("CPU", info.cpu.as_str()),
        (tr(lang, "Memória", "Memory"), info.memory.as_str()),
        ("GPU", info.gpus.as_str()),
    ];

    for (index, (label, value)) in rows.iter().enumerate() {
        let key = gtk::Label::new(Some(label));
        key.add_css_class("info-key");
        key.set_xalign(1.0);
        key.set_valign(gtk::Align::Start);

        let value = gtk::Label::new(Some(value));
        value.add_css_class("info-value");
        value.set_xalign(0.0);
        value.set_wrap(true);
        value.set_selectable(true);

        grid.attach(&key, 0, index as i32, 1, 1);
        grid.attach(&value, 1, index as i32, 1, 1);
    }

    content.append(&grid);
    scrolled(content).upcast()
}

fn about_tab(lang: Lang) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 20);
    content.add_css_class("tab-page");
    content.set_valign(gtk::Align::Start);

    let logo = logo_picture(180);
    logo.set_halign(gtk::Align::Center);
    content.append(&logo);

    let intro = gtk::Label::new(Some(tr(
        lang,
        "O ARGVUS é uma coleção modular de pacotes que juntos entregam um ambiente de desktop completo para Wayland e Hyprland.",
        "ARGVUS is a modular collection of packages that together provide a complete desktop environment for Wayland and Hyprland.",
    )));
    intro.add_css_class("paragraph");
    intro.set_wrap(true);
    intro.set_xalign(0.0);
    content.append(&intro);

    section_link(
        &content,
        tr(lang, "Site oficial", "Official website"),
        ARGVUS_URL,
        ARGVUS_URL,
    );

    let list = gtk::ListBox::new();
    list.add_css_class("module-list");
    for (name, desc_pt, desc_en) in modules() {
        let row = gtk::ListBoxRow::new();
        let row_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let title = gtk::Label::new(Some(name));
        title.add_css_class("module-title");
        title.set_xalign(0.0);

        let desc = gtk::Label::new(Some(tr(lang, desc_pt, desc_en)));
        desc.add_css_class("module-desc");
        desc.set_xalign(0.0);
        desc.set_wrap(true);

        row_box.append(&title);
        row_box.append(&desc);
        row.set_child(Some(&row_box));
        list.append(&row);
    }
    content.append(&list);

    scrolled(content).upcast()
}

fn credits_tab(lang: Lang) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.add_css_class("tab-page");
    content.set_valign(gtk::Align::Start);

    section_link(
        &content,
        tr(lang, "Desenvolvedor principal", "Lead developer"),
        WILLIAM_CANIN_URL,
        "William C. Canin - https://williamcanin.github.io",
    );
    section(
        &content,
        tr(lang, "Contribuidores", "Contributors"),
        tr(
            lang,
            "Comunidade ARGVUS e colaboradores listados nos repositórios de cada módulo.",
            "ARGVUS community and contributors listed in each module repository.",
        ),
    );
    section(
        &content,
        tr(lang, "Agradecimentos", "Thanks"),
        "Hyprland - https://hypr.land\nRust - https://rust-lang.org\nRofi - https://github.com/davatorium/rofi\nWaybar - https://github.com/Alexays/Waybar\nQuickshell - https://quickshell.org\nGTK - https://www.gtk.org\nYaru Theme - https://github.com/ubuntu/yaru\nKitty - https://sw.kovidgoyal.net/kitty\nsuperfile - https://superfile.dev\nnwg-look - https://github.com/nwg-piotr/nwg-look\nnwg-displays - https://github.com/nwg-piotr/nwg-displays\ngreetd - https://git.sr.ht/~kennylevinsen/greetd\n\ne todos os projetos livres que tornam este desktop possível.",
    );

    scrolled(content).upcast()
}

fn donate_tab(lang: Lang) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.add_css_class("tab-page");
    content.set_valign(gtk::Align::Start);

    let title = gtk::Label::new(Some(tr(
        lang,
        "Apoie o desenvolvimento do ARGVUS",
        "Support ARGVUS development",
    )));
    title.add_css_class("section-title");
    title.set_xalign(0.0);
    content.append(&title);

    let text = gtk::Label::new(Some(tr(
        lang,
        "Se o ARGVUS é útil para você, considere apoiar o desenvolvimento do projeto. Sua contribuição ajuda a manter a infraestrutura e a evolução contínua do desktop.",
        "If ARGVUS is useful to you, please consider supporting the project's development. Your contribution helps maintain the infrastructure and ongoing evolution of the desktop.",
    )));
    text.add_css_class("paragraph");
    text.set_wrap(true);
    text.set_xalign(0.0);
    content.append(&text);

    let button = gtk::Button::with_label("Donate");
    button.add_css_class("suggested-action");
    button.set_halign(gtk::Align::Start);
    button.connect_clicked(|_| open_donate_url());
    content.append(&button);

    scrolled(content).upcast()
}

fn open_donate_url() {
    let _ = Command::new("xdg-open").arg(DONATE_URL).spawn();
}

fn copyright_tab(lang: Lang) -> gtk::Widget {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.add_css_class("tab-page");
    content.set_valign(gtk::Align::Start);

    let text = gtk::Label::new(Some(tr(
        lang,
        "Copyright (C) William C. Canin e contribuidores do ARGVUS.\n\nARGVUS About e os módulos oficiais do ARGVUS são distribuídos sob os termos da GNU General Public License v3.0, salvo indicação em contrário em um pacote específico.\n\nOs nomes de projetos de terceiros, bibliotecas e aplicativos pertencem aos seus respectivos autores.",
        "Copyright (C) William C. Canin and ARGVUS contributors.\n\nARGVUS About and the official ARGVUS modules are distributed under the terms of the GNU General Public License v3.0 unless a specific package states otherwise.\n\nThird-party project names, libraries and applications belong to their respective authors.",
    )));
    text.add_css_class("paragraph");
    text.set_wrap(true);
    text.set_xalign(0.0);
    text.set_valign(gtk::Align::Start);
    content.append(&text);

    let license_title = gtk::Label::new(Some(tr(lang, "Licença", "License")));
    license_title.add_css_class("section-title");
    license_title.set_xalign(0.0);
    content.append(&license_title);

    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(include_str!("../LICENSE"));

    let license = gtk::TextView::with_buffer(&buffer);
    license.add_css_class("license-view");
    license.set_editable(false);
    license.set_cursor_visible(false);
    license.set_monospace(true);
    license.set_wrap_mode(gtk::WrapMode::WordChar);

    let license_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(300)
        .child(&license)
        .build();
    license_scroll.add_css_class("license-scroll");

    let frame = gtk::Frame::builder().child(&license_scroll).build();
    frame.add_css_class("license-frame");
    content.append(&frame);

    scrolled(content).upcast()
}

fn section(parent: &gtk::Box, title: &str, body: &str) {
    let title = gtk::Label::new(Some(title));
    title.add_css_class("section-title");
    title.set_xalign(0.0);
    parent.append(&title);

    let body = gtk::Label::new(Some(body));
    body.add_css_class("paragraph");
    body.set_xalign(0.0);
    body.set_wrap(true);
    body.set_selectable(true);
    parent.append(&body);
}

fn section_link(parent: &gtk::Box, title: &str, uri: &str, label: &str) {
    let title = gtk::Label::new(Some(title));
    title.add_css_class("section-title");
    title.set_xalign(0.0);
    parent.append(&title);

    let link = gtk::LinkButton::with_label(uri, label);
    link.add_css_class("argvus-link");
    link.set_halign(gtk::Align::Start);
    parent.append(&link);
}

fn scrolled<W: IsA<gtk::Widget>>(child: W) -> gtk::ScrolledWindow {
    child.set_vexpand(true);
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .child(&child)
        .build()
}

fn logo_picture(size: i32) -> gtk::Picture {
    let picture = logo_path()
        .map(gtk::Picture::for_filename)
        .unwrap_or_default();
    picture.set_size_request(size, size);
    picture.set_can_shrink(true);
    picture
}

fn logo_path() -> Option<String> {
    [
        "/usr/share/argvus-about/argvus-about.svg",
        "/usr/share/argvus-logo/svg/logotype.svg",
        "/usr/share/argvus-logo/svg/argvus-banner.svg",
        "/usr/share/pixmaps/argvus.svg",
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/argvus-about.svg"),
        "../argvus-logo/svg/logotype.svg",
        "../argvus-logo/svg/argvus-banner.svg",
    ]
    .into_iter()
    .find(|path| std::path::Path::new(path).exists())
    .map(str::to_string)
}

fn gtk_version() -> String {
    format!(
        "{}.{}.{}",
        gtk::major_version(),
        gtk::minor_version(),
        gtk::micro_version()
    )
}

fn modules() -> [(&'static str, &'static str, &'static str); 14] {
    [
        (
            "argvus-session",
            "Ciclo de vida da sessão, targets e integração Hyprland.",
            "Session lifecycle, targets and Hyprland integration.",
        ),
        (
            "argvus-shell",
            "Painel Quickshell, Waybar e interfaces de shell.",
            "Quickshell panel, Waybar and shell interfaces.",
        ),
        (
            "argvus-appearance",
            "Temas, fontes, wallpapers e integração visual.",
            "Themes, fonts, wallpapers and visual integration.",
        ),
        (
            "argvus-settings",
            "Configurações do ARGVUS, incluindo fontes e aplicativos padrão.",
            "ARGVUS settings, including fonts and default applications.",
        ),
        (
            "argvus-about",
            "Informações do sistema, créditos e licença do ARGVUS.",
            "System information, credits and ARGVUS license.",
        ),
        (
            "argvus-calendar",
            "Calendário e popup integrado à taskbar.",
            "Calendar and taskbar popup integration.",
        ),
        (
            "argvus-storage",
            "Módulo de dispositivos removíveis e armazenamento.",
            "Removable device and storage module.",
        ),
        (
            "argvus-greeter",
            "Tela gráfica de login do ARGVUS.",
            "ARGVUS graphical login screen.",
        ),
        (
            "argvus-accounts",
            "Configurações de conta e usuário.",
            "Account and user settings.",
        ),
        (
            "argvus-display",
            "Gerenciamento de monitores e layouts.",
            "Monitor and layout management.",
        ),
        (
            "argvus-network",
            "NetworkManager, Wi-Fi e Bluetooth.",
            "NetworkManager, Wi-Fi and Bluetooth.",
        ),
        (
            "argvus-power",
            "Menu de energia, idle e ações de sessão.",
            "Power menu, idle and session actions.",
        ),
        (
            "argvus-lock",
            "Bloqueio de tela e temas do lock screen.",
            "Screen locking and lock screen themes.",
        ),
        (
            "argvus-portal",
            "Portais Wayland, DBus e preferências de integração.",
            "Wayland portals, DBus and integration preferences.",
        ),
    ]
}
