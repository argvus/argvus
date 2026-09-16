#!/usr/bin/env bash
# shellcheck shell=bash
# shellcheck disable=SC2154
# srcdir, pkgdir, pkgname, and pkgver are supplied by makepkg.

arch_normalize_source_tree() {
	local expected="${srcdir}/${pkgname}-${pkgver}"
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

arch_check_argvus_payload() {
	local source_root="${srcdir}/${pkgname}-${pkgver}"

	test -x "${source_root}/src/usr/bin/argvus"
	sh -n "${source_root}/src/usr/bin/argvus"
	test "$(sed -n 's/^VERSION=\"\(.*\)\"/\1/p' "${source_root}/src/usr/bin/argvus")" = "$pkgver"
}

arch_package_argvus_payload() {
	local source_root="${srcdir}/${pkgname}-${pkgver}"

	install -Dm755 "${source_root}/src/usr/bin/argvus" \
		"${pkgdir}/usr/bin/argvus"
	install -Dm644 "${source_root}/LICENSE" \
		"${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
