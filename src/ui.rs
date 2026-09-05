use gtk::prelude::*;

use crate::i18n::{Lang, na, tr};
use crate::system::SystemInfo;

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
    provider.load_from_string(include_str!("../assets/style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
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

    section(
        &content,
        tr(lang, "Desenvolvedor principal", "Lead developer"),
        "William C. Canin",
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
        tr(
            lang,
            "Hyprland, GTK, Rust, Waybar, Quickshell e todos os projetos livres que tornam este desktop possível.",
            "Hyprland, GTK, Rust, Waybar, Quickshell and every free software project that makes this desktop possible.",
        ),
    );

    scrolled(content).upcast()
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

fn scrolled<W: IsA<gtk::Widget>>(child: W) -> gtk::ScrolledWindow {
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
