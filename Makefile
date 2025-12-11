.DEFAULT_GOAL = help
.PHONY: help check fmt clippy test audit build release demo full-check diagrams

## —— Slides-rs Makefile ——————————————————————————————————————————————————————

help: ## Show this help
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}{printf "\033[32m%-15s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'

## —— Development —————————————————————————————————————————————————————————————

check: ## Quick compile check (no codegen)
	@cargo check

fmt: ## Check code formatting
	@cargo fmt --check

clippy: ## Run linter
	@cargo clippy -- -D warnings

test: ## Run all tests
	@cargo test

audit: ## Check for security vulnerabilities
	@cargo audit

## —— Build ———————————————————————————————————————————————————————————————————

build: ## Build debug binary
	@cargo build

release: ## Build optimized release binary
	@cargo build --release

demo: release ## Build and test init+build in /tmp/slides-test
	@rm -rf /tmp/slides-test && mkdir /tmp/slides-test
	@cd /tmp/slides-test && $(CURDIR)/target/release/slides init && $(CURDIR)/target/release/slides build

## —— CI ——————————————————————————————————————————————————————————————————————

full-check: fmt clippy test audit ## Run all checks (CI)
	@echo "✅ All checks passed!"

## —— Documentation ————————————————————————————————————————————————————————————

PLANTUML_VERSION = 1.2024.7
PLANTUML_JAR = /tmp/plantuml-$(PLANTUML_VERSION).jar
DIAGRAMS_DIR = documentation/architecture/diagrams

diagrams: $(PLANTUML_JAR) ## Generate SVG diagrams from PlantUML sources
	@mkdir -p $(DIAGRAMS_DIR)/images
	@java -jar $(PLANTUML_JAR) -tsvg -o images $(DIAGRAMS_DIR)/*.puml
	@echo "✅ Diagrams generated in $(DIAGRAMS_DIR)/images/"

$(PLANTUML_JAR):
	@echo "Downloading PlantUML $(PLANTUML_VERSION)..."
	@curl -sSL -o $(PLANTUML_JAR) https://github.com/plantuml/plantuml/releases/download/v$(PLANTUML_VERSION)/plantuml-$(PLANTUML_VERSION).jar
