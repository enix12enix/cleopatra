# Makefile for Cleopatra Rust project

# Default database file
DB ?= cleopatra.db

# Default command for cargo
CARGO = cargo

# Run the Rust server
run:
	@echo "Running Cleopatra with DB=$(DB)"
	DATABASE_URL=sqlite://$(DB) $(CARGO) run

# Run SQL migrations manually
migrate:
	@echo "Running migrations using cleopatra.sql"
	sqlite3 $(DB) < migrations/cleopatra.sql

# Build project
build:
	$(CARGO) build --release

# Run tests
test:
	$(CARGO) test

# Clean target
clean:
	$(CARGO) clean
	rm -f $(DB)

# Rebuild & run
rebuild: clean build run

.PHONY: run migrate build test clean rebuild
