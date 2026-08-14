BRANCH := $(shell git branch --show-current 2>/dev/null || echo "unknown")
REMOTES := $(shell git remote 2>/dev/null || echo "")

.DEFAULT_GOAL := help

.PHONY: help set-permissions install uninstall test-storage test-storage-once test-storage-menu test-storage-waybar push push-lease

# ----- Menu help -----
help:
	@echo "Available targets:"
	@echo "  make set-permissions"
	@echo "  make install"
	@echo "  make uninstall"
	@echo "  make test-storage"
	@echo "  make test-storage-once"
	@echo "  make test-storage-menu"
	@echo "  make test-storage-waybar"
	@echo "  make push"
	@echo "  make push-lease"

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
