BRANCH := $(shell git branch --show-current 2>/dev/null || echo "unknown")
REMOTES := $(shell git remote 2>/dev/null || echo "")
VERSION ?= 0.1.19
PKGDIR := packaging/arch
REPO_DIR ?= packaging/arch/repo/x86_64
GPGKEY ?= D1841B5111962DF1EA31249F3B2A783046E06762
PACKAGES_URL ?= https://github.com/argvus/packages.git
PACKAGES_REL_DIR := public/arch/x86_64
WORK_DIR ?= packaging/arch/repo-work

.DEFAULT_GOAL := help

.PHONY: help build repo publish set-permissions install uninstall test-storage test-storage-once test-storage-menu test-storage-waybar push push-lease

# ----- Menu help -----
help:
	@echo "Available targets:"
	@echo "  make build [GPGKEY=<fingerprint>]"
	@echo "  make repo [GPGKEY=<fingerprint>]    baixa argvus/packages e atualiza .db/.files com o pacote local"
	@echo "  make publish [GPGKEY=<fingerprint>] build + repo, tudo local"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make test-storage"
	@echo "  make test-storage-once"
	@echo "  make test-storage-menu"
	@echo "  make test-storage-waybar"
	@echo "  make push"
	@echo "  make push-lease"

build:
	@staging="$$(mktemp -d)" && \
	mkdir -p "$$staging/argvus-$(VERSION)" && \
	cp -a config bin LICENSE "$$staging/argvus-$(VERSION)/" && \
	tar -czf "$(PKGDIR)/argvus-$(VERSION).tar.gz" -C "$$staging" argvus-$(VERSION) && \
	rm -rf "$$staging"
	@echo "Source tarball created: $(PKGDIR)/argvus-$(VERSION).tar.gz"
	@if [ -n "$(GPGKEY)" ]; then export GPGKEY="$(GPGKEY)"; fi; \
	cd "$(PKGDIR)" && makepkg -s --sign -p PKGBUILD.local

define GEN_REPO
	@set -e; \
	pkg="argvus-$(VERSION)-1-x86_64.pkg.tar.zst"; \
	dir="$(REPO_DIR)"; \
	work="$(WORK_DIR)"; \
	rel="$(PACKAGES_REL_DIR)"; \
	if [ ! -f "$(PKGDIR)/$$pkg" ]; then \
		echo "Package '$(PKGDIR)/$$pkg' not found. Run 'make build' first."; \
		exit 1; \
	fi; \
	if [ -d "$$work" ] && [ ! -d "$$work/.git" ]; then \
		rm -rf "$$work"; \
	fi; \
	if [ ! -d "$$work/.git" ]; then \
		echo "Cloning $(PACKAGES_URL) ..."; \
		git clone --depth 1 "$(PACKAGES_URL)" "$$work"; \
	else \
		cd "$$work" && git fetch --depth 1 origin main && git reset --hard FETCH_HEAD; \
	fi; \
	mkdir -p "$$dir"; \
	cp -a "$$work/$$rel/." "$$dir/"; \
	cp -f "$(PKGDIR)/$$pkg" "$$dir/$$pkg"; \
	cd "$$dir"; \
	if [ -n "$(GPGKEY)" ]; then \
		sign() { gpg --yes --local-user "$(GPGKEY)" --detach-sign "$$1"; }; \
	else \
		sign() { gpg --yes --detach-sign "$$1"; }; \
	fi; \
	sign "$$pkg"; \
	repo-add -R argvus.db.tar.gz "$$pkg"; \
	sign argvus.db.tar.gz; \
	sign argvus.db; \
	sign argvus.files.tar.gz; \
	sign argvus.files; \
	echo "Repository files updated locally in: $$dir"; \
	ls -l "$$dir"
endef

repo:
	$(GEN_REPO)

publish: build
	$(GEN_REPO)

set-permissions:
	@find config -type f -name "*.sh" -exec chmod +x {} \;
	@find bin -type f -exec chmod +x {} \;
	@find tools/sh -type f -name "*.sh" -exec chmod +x {} \; 2>/dev/null || true

install:
	@sh tools/sh/install.sh --all --force

uninstall:
	@sh tools/sh/uninstall.sh --all

test-storage:
	@sh tools/sh/test-storage-waybar.sh check

test-storage-once:
	@sh tools/sh/test-storage-waybar.sh once

test-storage-menu:
	@sh tools/sh/test-storage-waybar.sh menu

test-storage-waybar:
	@sh tools/sh/test-storage-waybar.sh waybar

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
