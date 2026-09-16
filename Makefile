PREFIX ?= /usr
DESTDIR ?=
.DEFAULT_GOAL := help

SCRIPT_VERSION := $(shell sed -n 's/^VERSION="\(.*\)"/\1/p' src/usr/bin/argvus)

.PHONY: help install uninstall set-permissions validate build package clean

help:
	@echo "Available targets:"
	@echo "  make build           - build the package into build/"
	@echo "  make package         - alias for make build"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make validate"

set-permissions:
	@if [ -d src/usr/bin ]; then find src/usr/bin -type f -exec chmod +x {} \;; fi
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

validate:
	@test -x src/usr/bin/argvus
	@sh -n src/usr/bin/argvus
	@[ -n "$(SCRIPT_VERSION)" ] || { echo "VERSION not found in src/usr/bin/argvus" >&2; exit 1; }
	@tools/sh/validate.sh

install:
	@sudo pacman -U build/dist/argvus*.zst --overwrite="*" --noconfirm

build:
	@tools/sh/pkgbuild_local.sh

package: build

clean:
	rm -rf build/
