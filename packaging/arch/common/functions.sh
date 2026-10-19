#!/usr/bin/env bash
# shellcheck shell=bash
# shellcheck disable=SC2154
# srcdir, pkgdir, pkgbase, pkgname and pkgver are supplied by makepkg.
#
# The argvus package base builds one Cargo workspace. `argvus` carries the
# public CLI and the data of the repositories absorbed into the workspace.
# `argvus-gtk` carries the GTK 4 helpers, which stay out of the command path.

# Helpers that are GTK 4 programs, installed in the private commands directory
# (argvus-taskbar-calendar, argvus-control-center, argvus-removable-devices).
ARGVUS_GTK_COMMANDS=(
	argvus-control-center
	argvus-removable-devices
	argvus-taskbar-calendar
)

arch_source_root() {
	printf '%s\n' "${srcdir}/${pkgbase}-${pkgver}"
}

# Where cargo writes release binaries: CARGO_TARGET_DIR when a local build sets it,
# otherwise the target directory inside the source tree.
arch_cargo_target_dir() {
	printf '%s\n' "${CARGO_TARGET_DIR:-$(arch_source_root)/target}"
}

arch_normalize_source_tree() {
	local expected="${srcdir}/${pkgbase}-${pkgver}"
	local -a roots=()

	while IFS= read -r -d '' root; do
		roots+=("$root")
	done < <(find "$srcdir" -mindepth 1 -maxdepth 1 -type d -print0)

	if (( ${#roots[@]} != 1 )); then
		printf 'error: expected exactly one extracted source directory in %s\n' "$srcdir" >&2
		return 1
	fi

	if [[ "${roots[0]}" != "$expected" ]]; then
		[[ ! -e "$expected" ]] || {
			printf 'error: source destination already exists: %s\n' "$expected" >&2
			return 1
		}
		mv -- "${roots[0]}" "$expected"
	fi
}

arch_build_argvus_workspace() {
	local source_root
	source_root="$(arch_source_root)"
	cd -- "$source_root" || return 1

	cargo build --release --locked \
		-p argvus-cli \
		-p argvus-control-center \
		-p argvus-loading-theme \
		-p argvus-removable-devices \
		-p argvus-taskbar-calendar

	# The greeter is a separate Cargo workspace inside this repository, with its own
	# lockfile. It shares argvus-i18n, argvus-theme and argvus-tui by path.
	(cd greeter && cargo build --release --locked)
}

arch_check_argvus_workspace() {
	local source_root
	source_root="$(arch_source_root)"
	cd -- "$source_root" || return 1

	test -x "$(arch_cargo_target_dir)/release/argvus"
	cargo test --workspace --locked
	(cd greeter && cargo test --workspace --locked)
}

# Installs the files of the package payload (`src/`) at their system paths. The
# `src/` tree mirrors the installed root: `usr/` and `etc/`.
arch_install_payload() {
	local source_root
	source_root="$(arch_source_root)"

	cp -R --no-preserve=ownership "${source_root}/src/usr/." "${pkgdir}/usr/"
	if [[ -d "${source_root}/src/etc" ]]; then
		install -dm755 "${pkgdir}/etc"
		cp -R --no-preserve=ownership "${source_root}/src/etc/." "${pkgdir}/etc/"
	fi

	# The taskbar calendar files belong to argvus-gtk. Removing them here keeps the
	# two packages from installing the same files.
	rm -rf -- "${pkgdir}/usr/lib/argvus-taskbar-calendar" \
		"${pkgdir}/usr/lib/systemd/user/argvus-taskbar-calendar.service" \
		"${pkgdir}/etc/argvus/taskbar/calendar"

	# The i18n QML module is imported from the Qt module path, not the ARGVUS data.
	install -dm755 "${pkgdir}/usr/lib/qt6/qml/org/argvus/i18n"
	install -Dm644 "${source_root}/src/usr/share/argvus/qml/argvus-i18n/I18n.qml" \
		"${pkgdir}/usr/lib/qt6/qml/org/argvus/i18n/I18n.qml"
	install -Dm644 "${source_root}/src/usr/share/argvus/qml/argvus-i18n/qmldir" \
		"${pkgdir}/usr/lib/qt6/qml/org/argvus/i18n/qmldir"
	rm -rf -- "${pkgdir}/usr/share/argvus/qml"

	# The system monitor reads its btop configuration from its own directory.
	install -Dm644 "${source_root}/src/usr/share/argvus/system-monitor/config/btop/btop.conf" \
		"${pkgdir}/usr/share/argvus-system-monitor/btop/btop.conf"
	install -dm755 "${pkgdir}/usr/share/argvus-system-monitor/btop/themes"
	install -m644 "${source_root}/src/usr/share/argvus/system-monitor/config/btop/themes/"*.theme \
		"${pkgdir}/usr/share/argvus-system-monitor/btop/themes/"
}

arch_package_argvus() {
	local source_root
	source_root="$(arch_source_root)"

	install -Dm755 "$(arch_cargo_target_dir)/release/argvus" "${pkgdir}/usr/bin/argvus"
	arch_install_payload

	# The i18n catalogs of the two supported locales.
	install -dm755 "${pkgdir}/usr/share/argvus/i18n/en-US" "${pkgdir}/usr/share/argvus/i18n/pt-BR"
	install -m644 "${source_root}/locales/en-US/"*.json "${pkgdir}/usr/share/argvus/i18n/en-US/"
	install -m644 "${source_root}/locales/pt-BR/"*.json "${pkgdir}/usr/share/argvus/i18n/pt-BR/"

	install -Dm644 "${source_root}/LICENSE" "${pkgdir}/usr/share/licenses/argvus/LICENSE"
}

arch_package_argvus_gtk() {
	local source_root command
	source_root="$(arch_source_root)"

	install -dm755 "${pkgdir}/usr/lib/argvus/commands"
	for command in "${ARGVUS_GTK_COMMANDS[@]}"; do
		install -Dm755 "$(arch_cargo_target_dir)/release/${command}" \
			"${pkgdir}/usr/lib/argvus/commands/${command}"
	done

	# The loading overlay stays at the path the greeter and the session launch.
	install -Dm755 "$(arch_cargo_target_dir)/release/splash" \
		"${pkgdir}/usr/lib/argvus/loading-theme/splash"

	# Taskbar calendar: the Waybar launcher, its configuration and its user unit.
	install -Dm755 "${source_root}/src/usr/lib/argvus-taskbar-calendar/waybar-launcher" \
		"${pkgdir}/usr/lib/argvus-taskbar-calendar/waybar-launcher"
	install -Dm644 "${source_root}/src/usr/lib/systemd/user/argvus-taskbar-calendar.service" \
		"${pkgdir}/usr/lib/systemd/user/argvus-taskbar-calendar.service"
	install -dm755 "${pkgdir}/etc/argvus/taskbar/calendar/themes"
	install -m644 "${source_root}/src/etc/argvus/taskbar/calendar/"*.toml \
		"${source_root}/src/etc/argvus/taskbar/calendar/"*.css \
		"${pkgdir}/etc/argvus/taskbar/calendar/"
	install -m644 "${source_root}/src/etc/argvus/taskbar/calendar/themes/"*.css \
		"${pkgdir}/etc/argvus/taskbar/calendar/themes/"

	install -Dm644 "${source_root}/LICENSE" "${pkgdir}/usr/share/licenses/argvus-gtk/LICENSE"
}

arch_package_argvus_greeter() {
	local source_root greeter
	source_root="$(arch_source_root)"
	greeter="${source_root}/greeter"

	install -Dm755 "$(arch_cargo_target_dir)/release/argvus-greeter" \
		"${pkgdir}/usr/lib/argvus/commands/argvus-greeter"
	install -Dm755 "${greeter}/packaging/greetd/argvus-greeter-setup" \
		"${pkgdir}/usr/lib/argvus/commands/argvus-greeter-setup"
	install -Dm755 "${greeter}/packaging/greetd/argvus-greeter-session" \
		"${pkgdir}/usr/lib/argvus/commands/argvus-greeter-session"
	install -Dm755 "${greeter}/packaging/greetd/argvus-greeter-tui" \
		"${pkgdir}/usr/lib/argvus/commands/argvus-greeter-tui"
	install -Dm755 "$(arch_cargo_target_dir)/release/argvus-greeter-handoff" \
		"${pkgdir}/usr/lib/argvus/greeter/handoff"

	install -Dm644 "${greeter}/packaging/greetd/greeter.toml" "${pkgdir}/etc/argvus/greeter.toml"
	install -Dm644 "${greeter}/packaging/greetd/hyprland-argvus-greeter.conf" \
		"${pkgdir}/etc/argvus/hyprland-argvus-greeter.conf"
	install -Dm644 "${greeter}/packaging/greetd/hyprland-argvus-greeter.lua" \
		"${pkgdir}/etc/argvus/hyprland-argvus-greeter.lua"
	install -Dm644 "${greeter}/packaging/greetd/argvus-greeter-tmpfiles.conf" \
		"${pkgdir}/usr/lib/tmpfiles.d/argvus-greeter.conf"
	install -Dm644 "${greeter}/packaging/greetd/config.toml" \
		"${pkgdir}/usr/share/doc/argvus-greeter/greetd-config.toml"
	install -Dm644 "${greeter}/README.md" "${pkgdir}/usr/share/doc/argvus-greeter/README.md"
	install -Dm644 "${source_root}/LICENSE" "${pkgdir}/usr/share/licenses/argvus-greeter/LICENSE"
}
