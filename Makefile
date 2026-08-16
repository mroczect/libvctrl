SHELL = /bin/bash
.SHELLFLAGS = -euo pipefail -c

CARGO   = cargo
MEMBERS = libvctrl_handler libvctrl_core libvctrl_plumbing libvctrl_porcelain libvctrl libvctrl_sha512

# Default package jika ingin menjalankan CI untuk satu package
PKG ?= libvctrl_handler

# Flag tambahan untuk Clippy (kosong = santai)
CLIPPY_FLAGS ?=

.PHONY: all
all: build

.PHONY: help
help:
	@echo "Usage: make <target> [PKG=<package>]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# ---------------------------------------------------------------------------
# Global
# ---------------------------------------------------------------------------
.PHONY: build
build:
	$(CARGO) build --workspace

.PHONY: release
release:
	$(CARGO) build --release --workspace

.PHONY: check
check:
	$(CARGO) check --workspace

.PHONY: test
test:
	$(CARGO) test --workspace

.PHONY: test-verbose
test-verbose:
	RUST_BACKTRACE=1 $(CARGO) test --workspace -- --nocapture

.PHONY: watch-test
watch-test:
	$(CARGO) watch -x 'test --workspace'

.PHONY: watch-build
watch-build:
	$(CARGO) watch -x 'check --workspace'

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

# Clippy santai (tidak -D warnings)
.PHONY: clippy
clippy:
	$(CARGO) clippy --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: lint
lint: fmt clippy

.PHONY: ci
ci: fmt-check clippy test-verbose

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: doc
doc:
	$(CARGO) doc --workspace --no-deps

.PHONY: doc-open
doc-open: doc
	$(CARGO) doc --workspace --no-deps --open

.PHONY: bench
bench:
	$(CARGO) bench --workspace

.PHONY: update
update:
	$(CARGO) update

.PHONY: audit
audit:
	@if command -v cargo-audit >/dev/null 2>&1; then \
		$(CARGO) audit; \
	else \
		echo "cargo-audit not installed. Run: cargo install cargo-audit"; \
	fi

.PHONY: publish-check
publish-check: check-readmes
	@for crate in $(MEMBERS); do \
		echo "Packaging $$crate"; \
		$(CARGO) package -p "$$crate" --no-verify || exit 1; \
	done
	@echo "All crates are ready for publish."

.PHONY: publish-all
publish-all: check-readmes
	@echo "Publishing libvctrl_handler ..."
	$(CARGO) publish -p libvctrl_handler
	@sleep 5
	@echo "Publishing libvctrl_core ..."
	$(CARGO) publish -p libvctrl_core
	@sleep 5
	@echo "Publishing libvctrl_plumbing ..."
	$(CARGO) publish -p libvctrl_plumbing
	@sleep 5
	@echo "Publishing libvctrl_porcelain ..."
	$(CARGO) publish -p libvctrl_porcelain
	@sleep 5
	@echo "Publishing libvctrl (root) ..."
	$(CARGO) publish -p libvctrl
	@echo "All crates published successfully."

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov --workspace --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

.PHONY: version
version:
	@if [ -z "$(V)" ]; then \
		echo "Usage: make version V=<major|minor|patch|X.Y.Z>"; \
		exit 1; \
	fi
	@if [ ! -x "dev/version_bump.sh" ]; then \
		echo "ERROR: dev/version_bump.sh not found or not executable"; \
		exit 1; \
	fi
	dev/version_bump.sh $(V)

.PHONY: snap
snap:
	mkdir -p dev
	@for crate in $(MEMBERS); do \
		if [ -d "$$crate" ]; then \
			echo "Snapping $$crate/src"; \
			snapcat "$$crate/src" -f markdown -o "dev/$$crate.src.snapcat.md" || true; \
		fi; \
		if [ -d "$$crate/tests" ]; then \
			echo "Snapping $$crate/tests"; \
			snapcat "$$crate/tests" -f markdown -o "dev/$$crate.tests.snapcat.md" || true; \
		fi; \
	done
	@echo "Merging all snapshots into dev/root.md"
	cat dev/*.snapcat.md > dev/root.md 2>/dev/null || true
	@echo "Done. See dev/root.md"

.PHONY: run
run:
	$(CARGO) run

.PHONY: install
install:
	$(CARGO) install --path .

.PHONY: uninstall
uninstall:
	$(CARGO) uninstall libvctrl || true

.PHONY: rebuild
rebuild: release install

# ---------------------------------------------------------------------------
# Package-specific targets (pkg=<name>)
# ---------------------------------------------------------------------------
.PHONY: build-pkg
build-pkg:
	$(CARGO) build -p $(PKG)

.PHONY: release-pkg
release-pkg:
	$(CARGO) build --release -p $(PKG)

.PHONY: check-pkg
check-pkg:
	$(CARGO) check -p $(PKG)

.PHONY: test-pkg
test-pkg:
	$(CARGO) test -p $(PKG)

.PHONY: test-verbose-pkg
test-verbose-pkg:
	RUST_BACKTRACE=1 $(CARGO) test -p $(PKG) -- --nocapture

.PHONY: fmt-pkg
fmt-pkg:
	$(CARGO) fmt -p $(PKG)

.PHONY: fmt-check-pkg
fmt-check-pkg:
	$(CARGO) fmt -p $(PKG) -- --check

# Clippy per package (santai)
.PHONY: clippy-pkg
clippy-pkg:
	$(CARGO) clippy -p $(PKG) --all-targets --all-features $(CLIPPY_FLAGS)

# Alias backward-compatible
.PHONY: clippy-pkg-unwarn
clippy-pkg-unwarn: clippy-pkg

.PHONY: ci-pkg
ci-pkg: fmt-check-pkg clippy-pkg test-verbose-pkg

.PHONY: ci-pkg-unwarn
ci-pkg-unwarn: ci-pkg

.PHONY: doc-pkg
doc-pkg:
	$(CARGO) doc -p $(PKG) --no-deps

.PHONY: watch-test-pkg
watch-test-pkg:
	$(CARGO) watch -x 'test -p $(PKG)'

.PHONY: watch-build-pkg
watch-build-pkg:
	$(CARGO) watch -x 'check -p $(PKG)'

# ---------------------------------------------------------------------------
# Convenience aliases for common packages
# ---------------------------------------------------------------------------
.PHONY: handler
handler: PKG=libvctrl_handler
handler: ci-pkg

.PHONY: core
core: PKG=libvctrl_core
core: ci-pkg

.PHONY: plumbing
plumbing: PKG=libvctrl_plumbing
plumbing: ci-pkg

.PHONY: porcelain
porcelain: PKG=libvctrl_porcelain
porcelain: ci-pkg

.PHONY: root-pkg
root-pkg: PKG=libvctrl
root-pkg: ci-pkg

.PHONY: sha512
sha512: PKG=libvctrl_sha512
sha512: ci-pkg

# Target khusus kalau mau lebih ketat
.PHONY: clippy-strict
clippy-strict:
	$(CARGO) clippy --all-targets --all-features -- -D warnings
