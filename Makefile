SHELL = /bin/bash
.SHELLFLAGS = -euo pipefail -c

CARGO   = cargo
MEMBERS = libvctrl_handler libvctrl_core libvctrl_plumbing libvctrl_porcelain

.PHONY: all
all: build

.PHONY: help
help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

.PHONY: init-readmes
init-readmes:
	@for crate in $(MEMBERS); do \
		if [ -d "$$crate" ]; then \
			readme="$$crate/README.md"; \
			if [ ! -f "$$readme" ]; then \
				echo "Creating $$readme"; \
				echo "# $$crate\n\nPart of the libvctrl workspace.\n\nSee [README](../README.md) for full documentation." > "$$readme"; \
			else \
				echo "$$readme already exists"; \
			fi; \
		else \
			echo "ERROR: Folder $$crate not found"; \
			exit 1; \
		fi; \
	done

.PHONY: check-readmes
check-readmes:
	@missing=0; \
	for crate in $(MEMBERS); do \
		if [ ! -f "$$crate/README.md" ]; then \
			echo "MISSING: $$crate/README.md"; \
			missing=$$((missing + 1)); \
		fi; \
	done; \
	if [ $$missing -gt 0 ]; then \
		echo "ERROR: $$missing README files missing. Run 'make init-readmes'"; \
		exit 1; \
	else \
		echo "All READMEs present."; \
	fi

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

.PHONY: clippy
clippy: 
	$(CARGO) clippy --all-targets --all-features -- -D warnings

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
	@echo "All crates published successfully."

.PHONY: coverage
coverage: 
	$(CARGO) llvm-cov --workspace --html
	@echo "Coverage report: target/llvm-cov/html/index.html"
