.PHONY: build build-all run test fmt check install clean help

# Default target
all: build

# Build in release mode
build:
	cargo build --release

# Build with all features
build-all:
	cargo build --release --all-features

# Run the project
# Usage: make run ARGS="--version"
run:
	cargo run -- $(ARGS)

# Run tests
test: fmt
	cargo test

# Format code and run clippy
fmt:
	cargo fmt
	cargo clippy --all-targets --all-features -- -D warnings

# Check formatting and clippy (CI style)
check:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings

# Install binary to ~/.local/bin
install: build-all
	mkdir -p ~/.local/bin
	cp target/release/zerostack ~/.local/bin/zerostack
	chmod +x ~/.local/bin/zerostack
	@echo "Installed to ~/.local/bin/zerostack"

# Clean build artifacts
clean:
	cargo clean

# Help
help:
	@echo "Available targets:"
	@echo "  build       Build in release mode"
	@echo "  build-all   Build with all features"
	@echo "  run         Run (use ARGS=\"...\" for flags)"
	@echo "  test        Run tests"
	@echo "  fmt         Format and clippy"
	@echo "  check       Check fmt and clippy"
	@echo "  install     Build all-features and copy to ~/.local/bin"
	@echo "  clean       Clean build artifacts"
