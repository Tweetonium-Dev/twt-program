.PHONY: build build-release clean update test cov fmt lint deploy change-authority verify release idl send

# ---- Colors ----

GREEN := \033[0;32m
CYAN := \033[0;36m
YELLOW := \033[1;33m
RED := \033[0;31m
RESET := \033[0m

# ---- Build & Maintenance ----

build:
	@clear
	@echo "$(CYAN)🔧 [BUILD] Compiling Solana program...$(RESET)"
	@cargo build-sbf

build-release:
	@echo "$(CYAN)🚀 [BUILD] Compiling Solana program (release optimized)...$(RESET)"
	@cargo build-sbf --release

clean:
	@echo "$(YELLOW)🧹 [CLEAN] Removing build artifacts...$(RESET)"
	@cargo clean

update:
	@echo "$(GREEN)⬆️ [UPDATE] Updating dependencies...$(RESET)"
	@cargo update

# ---- Testing ----

test:
	@clear
	@echo "$(CYAN)🔍 [TEST] Running unit tests...$(RESET)"
	@cargo test

cov:
	@clear
	@echo "$(CYAN)🔦 [COVERAGE] Running test coverage...$(RESET)"
	@cargo tarpaulin \
		--workspace \
		--all-features \
		--exclude-files "target/*" \
		--out Html \
		--fail-under 80

# ---- Formatting & Linting ----

fmt:
	@clear
	@echo "$(GREEN)🎨 [FMT] Formatting codebase...$(RESET)"
	@cargo fmt

lint:
	@clear
	@echo "$(YELLOW)🧹 [LINT] Running Clippy linter...$(RESET)"
	@cargo clippy 

# ---- Deploy & Authority ----

deploy:
	@if [ -z "$(AUTH)" ]; then \
		echo "$(RED)🔴 [DEPLOY] Missing AUTH argument$(RESET)"; \
		echo "   Usage: make deploy AUTH=~/.config/solana/id.json"; \
		exit 1; \
	fi
	@echo "$(CYAN)🚢 [DEPLOY] Deploying program to Solana...$(RESET)"
	@solana program deploy \
		--program-id ./target/deploy/tweetonium-keypair.json \
		--upgrade-authority $(AUTH) \
		./target/deploy/tweetonium.so
	@echo "$(GREEN)🟢 [DEPLOY] Deployment complete.$(RESET)"

change-authority:
	@if [ -z "$(NEW_AUTH)" ]; then \
		echo "$(RED)🔴 [AUTH] Missing NEW_AUTH argument$(RESET)"; \
		echo "   Usage: make change-authority NEW_AUTH=~/.config/solana/new.json"; \
		exit 1; \
	fi
	@echo "$(CYAN)🔑 [AUTH] Changing upgrade authority...$(RESET)"
	@solana program set-upgrade-authority \
		--program-id ./target/deploy/tweetonium-keypair.json \
		--new-upgrade-authority $(NEW_AUTH)
	@echo "$(GREEN)🟢 [AUTH] Upgrade authority changed to $(NEW_AUTH)$(RESET)"

verify:
	@echo ""
	@echo "$(CYAN)🔎 [VERIFY] Checking deployed program info...$(RESET)"
	@solana program show ./target/deploy/tweetonium-keypair.json

release:
	@$(MAKE) build
	@$(MAKE) deploy
	@$(MAKE) verify

# ---- IDL ----

idl:
	@echo "$(CYAN)🧩 [IDL] Generating IDL schema...$(RESET)"
	@shank idl -r . -o ./idl
	@echo "$(GREEN)🟢 [IDL] IDL generated at ./idl$(RESET)"

send:
	@if [ -z "$(DEST)" ]; then \
		echo "$(RED)🔴 [SEND] Missing DEST argument$(RESET)"; \
		echo "   Usage: make send DEST=~/path/to/idl"; \
		exit 1; \
	fi
	@dest_expand=$$(eval echo $(DEST)); \
	echo "$(CYAN)📦 [SEND] Copying IDL to $$dest_expand...$(RESET)"; \
	mkdir -p "$$dest_expand"; \
	cp ./idl/tweetonium.json "$$dest_expand"; \
	echo "$(GREEN)🟢 [SEND] IDL copied successfully to $$dest_expand$(RESET)"
