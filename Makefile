# Makefile for mdhavers - auto-detects LLVM and builds appropriately

.PHONY: build release test clean install check fmt clippy

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
	@echo "  make build        - Build with auto-detected features"
	@echo "  make release      - Build release with auto-detected features"
	@echo "  make test         - Run tests"
	@echo "  make check        - Check compilation"
	@echo "  make fmt          - Format code"
	@echo "  make clippy       - Run clippy lints"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make install      - Install mdhavers"
	@echo "  make status       - Show LLVM detection status"
	@echo "  make build-no-llvm    - Force build without LLVM"
	@echo "  make build-with-llvm  - Force build with LLVM"
