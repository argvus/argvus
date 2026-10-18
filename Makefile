PREFIX ?= /usr
DESTDIR ?=
.DEFAULT_GOAL := help

WORKSPACE_VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)

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
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

validate:
	@sh tools/sh/sync-skeleton.sh --check
	@[ -n "$(WORKSPACE_VERSION)" ] || { echo "version not found in Cargo.toml" >&2; exit 1; }
	@cargo fmt --all -- --check
	@cargo clippy --workspace --all-targets --locked -- -D warnings
	@cargo test --workspace --locked
	@cd greeter && cargo fmt --all -- --check && cargo clippy --workspace --all-targets --locked -- -D warnings && cargo test --workspace --locked
	@tools/sh/validate.sh

install:
	@sudo pacman -U build/dist/argvus*.zst --overwrite="*" --noconfirm

uninstall:
	@tools/sh/uninstall.sh --user

build:
	@tools/sh/pkgbuild_local.sh

package: build

clean:
	rm -rf build/
