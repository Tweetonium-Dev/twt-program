.PHONY: build build-release clean update release deploy change-authority verify idl send

# ---- Build & Maintenance ----

build:
	@echo "🔧 Building Solana program..."
	@cargo build-sbf

build-release:
	@echo "🔧 Building Solana program (optimized for release)..."
	@cargo build-sbf --release

clean:
	@echo "🧹 Cleaning package"
	@cargo clean

update:
	@echo "🚀 Update package"
	@cargo update

release:
	@$(MAKE) clean
	@$(MAKE) build
	@$(MAKE) deploy
	@$(MAKE) verify

# ---- Deploy & Authority ----

deploy:
	@if [ -z "$(AUTH)" ]; then \
		echo "❌ Missing AUTH argument"; \
		echo "   Usage: make deploy AUTH=~/.config/solana/id.json"; \
		exit 1; \
	fi
	@echo "🚢 Deploying program to Solana..."
	@solana program deploy \
		--program-id ./target/deploy/tweetonium-keypair.json \
		--upgrade-authority $(AUTH) \
		./target/deploy/tweetonium.so
	@echo "✅ Deployment complete."

change-authority:
	@if [ -z "$(NEW_AUTH)" ]; then \
		echo "❌ Missing NEW_AUTH argument"; \
		echo "   Usage: make change-authority NEW_AUTH=~/.config/solana/new.json"; \
		exit 1; \
	fi
	@echo "🔑 Changing upgrade authority..."
	@solana program set-upgrade-authority \
		--program-id ./target/deploy/tweetonium-keypair.json \
		--new-upgrade-authority $(NEW_AUTH)
	@echo "✅ Upgrade authority changed to $(NEW_AUTH)"

verify:
	@echo ""
	@echo "🔍 Verifying program deployment..."
	@solana program show ./target/deploy/tweetonium-keypair.json

# ---- IDL ----

idl:
	@echo "🧩 Generating IDL..."
	@shank idl -r . -o ./idl
	@echo "✅ IDL generated at ./idl"

send:
	@if [ -z "$(DEST)" ]; then \
		echo "❌ Missing DEST argument"; \
		echo "   Usage: make send DEST=~/path/to/idl"; \
		exit 1; \
	fi
	@dest_expand=$$(eval echo $(DEST)); \
	echo "📦 Copying IDL to $$dest_expand"; \
	mkdir -p "$$dest_expand"; \
	cp ./idl/tweetonium.json "$$dest_expand"; \
	echo "✅ IDL copied successfully to $$dest_expand"
