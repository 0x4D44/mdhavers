# Makefile for mdhavers - auto-detects LLVM and builds appropriately

.PHONY: build release test clean install install-local uninstall check fmt clippy package

# Auto-detect LLVM - check for llvm-config variants
LLVM_CONFIG := $(shell which llvm-config-15 2>/dev/null || which llvm-config-14 2>/dev/null || which llvm-config-16 2>/dev/null || which llvm-config-17 2>/dev/null || which llvm-config-18 2>/dev/null || which llvm-config 2>/dev/null)

# Check if we have all LLVM dependencies (including libzstd-dev for linking)
# We need the static library or the .so symlink that -dev packages provide
HAVE_ZSTD := $(shell test -f /usr/lib/x86_64-linux-gnu/libzstd.a -o -f /usr/lib/x86_64-linux-gnu/libzstd.so && echo "yes" || echo "no")

# Determine features based on LLVM availability
ifdef LLVM_CONFIG
ifeq ($(HAVE_ZSTD),yes)
    FEATURES := --features llvm
    LLVM_STATUS := enabled ($(LLVM_CONFIG))
else
    FEATURES :=
    LLVM_STATUS := disabled (missing libzstd-dev)
endif
else
    FEATURES :=
    LLVM_STATUS := disabled (LLVM not found)
endif

# Default target
build:
	@echo "Building mdhavers (LLVM: $(LLVM_STATUS))"
	cargo build $(FEATURES)

release:
	@echo "Building mdhavers release (LLVM: $(LLVM_STATUS))"
	cargo build --release $(FEATURES)

test:
	@echo "Testing mdhavers (LLVM: $(LLVM_STATUS))"
	cargo test $(FEATURES)

check:
	@echo "Checking mdhavers (LLVM: $(LLVM_STATUS))"
	cargo check $(FEATURES)

fmt:
	cargo fmt

clippy:
	cargo clippy $(FEATURES) -- -D warnings

clean:
	cargo clean

install: release
	@echo "Installing mdhavers..."
	cargo install --path . $(FEATURES)

# Install using the custom installer (to ~/.mdhavers)
install-local: release
	@echo "Installing mdhavers to ~/.mdhavers..."
	@chmod +x installer/install.sh
	@./installer/install.sh --local --yes

# Uninstall mdhavers
uninstall:
	@echo "Uninstalling mdhavers..."
	@chmod +x installer/uninstall.sh
	@./installer/uninstall.sh --yes

# Create distribution package
package: release
	@echo "Creating distribution package..."
	@mkdir -p dist
	@VERSION=$$(cargo pkgid | cut -d# -f2 | cut -d: -f2) && \
	ARCH=$$(uname -m) && \
	OS=$$(uname -s | tr '[:upper:]' '[:lower:]') && \
	PKG_NAME="mdhavers-$$VERSION-$$OS-$$ARCH" && \
	mkdir -p "dist/$$PKG_NAME/bin" && \
	mkdir -p "dist/$$PKG_NAME/completions" && \
	cp target/release/mdhavers "dist/$$PKG_NAME/bin/" && \
	(cp target/release/mdhavers-lsp "dist/$$PKG_NAME/bin/" 2>/dev/null || true) && \
	cp installer/completions/* "dist/$$PKG_NAME/completions/" && \
	cp installer/install.sh "dist/$$PKG_NAME/" && \
	cp installer/uninstall.sh "dist/$$PKG_NAME/" && \
	cp installer/scripts/env.sh "dist/$$PKG_NAME/" && \
	cp README.md "dist/$$PKG_NAME/" && \
	(cp -r examples "dist/$$PKG_NAME/" 2>/dev/null || true) && \
	(cp -r stdlib "dist/$$PKG_NAME/" 2>/dev/null || true) && \
	cd dist && tar -czvf "$$PKG_NAME.tar.gz" "$$PKG_NAME" && \
	rm -rf "$$PKG_NAME" && \
	echo "Created dist/$$PKG_NAME.tar.gz"

# Force build without LLVM
build-no-llvm:
	@echo "Building mdhavers without LLVM"
	cargo build

# Force build with LLVM (will fail if LLVM not available)
build-with-llvm:
	@echo "Building mdhavers with LLVM"
	cargo build --features llvm

# Show LLVM detection status
status:
	@echo "LLVM Config: $(if $(LLVM_CONFIG),$(LLVM_CONFIG),not found)"
	@echo "libzstd: $(HAVE_ZSTD)"
	@echo "LLVM Feature: $(LLVM_STATUS)"

help:
	@echo "mdhavers build targets:"
	@echo "  make build           - Build with auto-detected features"
	@echo "  make release         - Build release with auto-detected features"
	@echo "  make test            - Run tests"
	@echo "  make check           - Check compilation"
	@echo "  make fmt             - Format code"
	@echo "  make clippy          - Run clippy lints"
	@echo "  make clean           - Clean build artifacts"
	@echo "  make install         - Install via cargo install"
	@echo "  make install-local   - Install to ~/.mdhavers with completions"
	@echo "  make uninstall       - Uninstall from ~/.mdhavers"
	@echo "  make package         - Create distribution tarball"
	@echo "  make status          - Show LLVM detection status"
	@echo "  make build-no-llvm   - Force build without LLVM"
	@echo "  make build-with-llvm - Force build with LLVM"
