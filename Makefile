PREFIX ?= /usr
DESTDIR ?=
BRANCH := $(shell git branch --show-current 2>/dev/null || echo "unknown")
REMOTES := $(shell git remote 2>/dev/null || echo "")

.DEFAULT_GOAL := help

SCRIPT_VERSION := $(shell sed -n 's/^VERSION="\(.*\)"/\1/p' usr/bin/argvus)

.PHONY: help install uninstall set-permissions validate build clean push push-lease

help:
	@echo "Available targets:"
	@echo "  make build"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make validate"
	@echo "  make push"
	@echo "  make push-lease"

set-permissions:
	@if [ -d bin ]; then find bin -type f -exec chmod +x {} \;; fi
	@if [ -d usr/bin ]; then find usr/bin -type f -exec chmod +x {} \;; fi
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

validate:
	@test -x usr/bin/argvus
	@sh -n usr/bin/argvus
	@[ -n "$(SCRIPT_VERSION)" ] || { echo "VERSION not found in usr/bin/argvus" >&2; exit 1; }
	@for f in packaging/arch/PKGBUILD packaging/arch/PKGBUILD.local; do \
		[ -f "$$f" ] || continue; \
		pkgver=$$(sed -n 's/^pkgver=\(.*\)/\1/p' "$$f"); \
		if [ "$$pkgver" != "$(SCRIPT_VERSION)" ]; then \
			echo "pkgver mismatch in $$f: expected $(SCRIPT_VERSION), got $$pkgver" >&2; \
			exit 1; \
		fi; \
	done
	@echo "argvus validation ok"

install:
	install -Dm755 usr/bin/argvus \
		"$(DESTDIR)$(PREFIX)/bin/argvus"
	install -Dm644 LICENSE \
		"$(DESTDIR)$(PREFIX)/share/licenses/argvus/LICENSE"

uninstall:
	find "$(DESTDIR)$(PREFIX)/bin" -name 'argvus' -type f -delete
	find "$(DESTDIR)$(PREFIX)/share/licenses/argvus" -type f -delete

# Swallow bare arguments passed to the targets above
%:
	@:

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

build:
	@tools/build-local-package.sh

clean:
	rm -rf dist
	rm -f packaging/arch/*.zst packaging/arch/*.tar.gz
