PREFIX ?= /usr
DESTDIR ?=
.DEFAULT_GOAL := help

WORKSPACE_VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)

.PHONY: help install uninstall set-permissions fmt validate build package clean

help:
	@echo "Available targets:"
	@echo "  make build           - build the package into build/"
	@echo "  make package         - alias for make build"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make fmt"
	@echo "  make validate"

set-permissions:
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

fmt:
	@cargo fmt --all

validate:
	@[ -n "$(WORKSPACE_VERSION)" ] || { echo "version not found in Cargo.toml" >&2; exit 1; }
	@cargo fmt --all -- --check
	@cargo clippy --workspace --all-targets --locked -- -D warnings
	@cargo test --workspace --locked
	@tools/sh/validate.sh

# Installs only the newest package in build/dist: older builds stay there, and
# passing several versions of argvus to pacman makes the transaction fail.
install:
	@pkg="$$(ls -t build/dist/argvus-[0-9]*.pkg.tar.zst 2>/dev/null | head -n 1)"; \
	[ -n "$$pkg" ] || { echo "no argvus package in build/dist: run make build" >&2; exit 1; }; \
	echo "Installing $$pkg"; \
	sudo pacman -U "$$pkg" --overwrite="*" --noconfirm

uninstall:
	@tools/sh/uninstall.sh --user

build:
	@tools/sh/pkgbuild_local.sh

package: build

clean:
	rm -rf build/
