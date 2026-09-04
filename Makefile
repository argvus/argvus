BRANCH := $(shell git branch --show-current 2>/dev/null || echo "unknown")
REMOTES := $(shell git remote 2>/dev/null || echo "")

.DEFAULT_GOAL := help

.PHONY: help set-permissions install uninstall push push-lease build build-bin check validate clean

# ----- Menu help -----
help:
	@echo "Available targets:"
	@echo "  make build"
	@echo "  make build-bin"
	@echo "  make check"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make push"
	@echo "  make push-lease"

set-permissions:
	@if [ -d config ]; then find config -type f -name "*.sh" -exec chmod +x {} \;; fi
	@if [ -d bin ]; then find bin -type f -exec chmod +x {} \;; fi
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

check:
	cargo fmt --check
	cargo test --locked
	cargo clippy --locked -- -D warnings

validate: check
	@echo "argvus validation ok"

build-bin:
	cargo build --release --locked

install: build-bin
	@sh tools/sh/install.sh --all --force

uninstall:
	@sh tools/sh/uninstall.sh --all

# ----- GIT PUSH (development commands) -----
push:
	@echo "Push normal → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push $$remote $(BRANCH); \
	done

push-lease:
	@echo "Push --force-with-lease → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push --force-with-lease $$remote $(BRANCH); \
	done

# Swallow bare arguments passed to the targets above
%:
	@:

build:
	@tools/build-local-package.sh

clean:
	rm -f packaging/arch/*.zst packaging/arch/*.tar.gz
