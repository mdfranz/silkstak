.PHONY: build build-all dev run test fmt check check-ci install clean help

# Export environment variables for sub-processes
export RUST_LOG
export RUST_LOG_FILE

# --- Compiler Optimizations ---

# --- OS & Binary Name Auto-detection ---
ifeq ($(OS),Windows_NT)
    BIN_NAME := silkstak
    EXE := .exe
else
    BIN_NAME := silkstak
    EXE :=
    # Auto-detect sccache (Unix only)
    ifeq ($(shell which sccache 2>/dev/null),)
        # sccache not found
    else
        export RUSTC_WRAPPER := sccache
    endif

    # Auto-detect mold or lld for faster linking (only on Linux, as macOS clang has issues with -fuse-ld)
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        ifeq ($(shell which mold 2>/dev/null),)
            ifeq ($(shell which lld 2>/dev/null),)
                # No fast linker found, use default
            else
                export RUSTFLAGS := -C link-arg=-fuse-ld=lld
            endif
        else
            export RUSTFLAGS := -C link-arg=-fuse-ld=mold
        endif
    endif
endif

# Default target
all: build

# Build in release mode
build:
	cargo build --release

# Build with all features
build-all:
	cargo build --release --all-features

# Fast unoptimized build for local development
dev:
	cargo build

# Run the project
# Usage: make run ARGS="--version"
run:
	cargo run -- $(ARGS)

# Run with debug logging to silkstak.log (app) and rig.log (framework)
# App logs go to silkstak.log, rig framework logs go to rig.log
debug:
	$(MAKE) run RUST_LOG=silkstak=debug,rig=info RUST_LOG_FILE=1 ARGS="$(ARGS)"

# Run tests
test: fmt
	cargo test

# Fast type checking without binary generation
check:
	cargo check --all-targets --all-features

# Format code and run clippy
fmt:
	cargo fmt
	cargo clippy --all-targets --all-features -- -D warnings

# Check formatting and clippy (CI style)
check-ci:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings

# Install binary
install: build-all
ifeq ($(OS),Windows_NT)
	cargo install --path .
else
	mkdir -p ~/.local/bin
	cp target/release/$(BIN_NAME) ~/.local/bin/$(BIN_NAME)
	chmod +x ~/.local/bin/$(BIN_NAME)
	@echo "Installed to ~/.local/bin/$(BIN_NAME)"
endif

uninstall:
ifeq ($(OS),Windows_NT)
	cargo uninstall $(BIN_NAME)
else
	rm -vf ~/.local/bin/$(BIN_NAME)
	rm -vf ~/.cargo/bin/$(BIN_NAME)
endif

# Clean build artifacts and logs
clean:
	cargo clean
ifeq ($(OS),Windows_NT)
	-del /f /q silkstak.log rig.log zerostack.log zerostack.log.rig 2>nul
else
	rm -f silkstak.log rig.log zerostack.log zerostack.log.rig
endif

# Help
help:
	@echo "Available targets:"
	@echo "  build       Build in release mode (default)"
	@echo "  build-all   Build with all features"
	@echo "  dev         Build unoptimized (faster for dev)"
	@echo "  check       Run cargo check (fastest feedback)"
	@echo "  run         Run (use ARGS=\"...\" for flags)"
	@echo "  debug       Run with RUST_LOG=debug and log to silkstak.log and rig.log"
	@echo "  test        Run tests"
	@echo "  fmt         Format and clippy"
	@echo "  check-ci    Check fmt and clippy (CI style)"
	@echo "  install     Build all-features and copy to ~/.local/bin"
	@echo "  clean       Clean build artifacts"
