.PHONY: all clean build wasm deploy dev serve

WASM_PACK := $(shell command -v wasm-pack 2> /dev/null)
RUSTUP  := $(shell command -v rustup 2> /dev/null)

all: build

clean:
	rm -rf pkg docs target

ensure-rust:
	@command -v rustup >/dev/null 2>&1 || { echo "Install Rust: https://rustup.rs"; exit 1; }
	@rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
	@command -v wasm-pack >/dev/null 2>&1 || cargo install wasm-pack

ensure-node:
	@command -v npm >/dev/null 2>&1 || { echo "Install Node.js: https://nodejs.org"; exit 1; }
	@[ -d node_modules ] || npm install

wasm: ensure-rust
	wasm-pack build --target web --out-dir pkg

build: clean wasm ensure-node
	npm run build
	echo "record.itsash.in" > docs/CNAME
	touch docs/.nojekyll

dev: wasm ensure-node
	npm run dev

serve: build
	@echo "Serving docs/ at http://localhost:4173"
	cd docs && python3 -m http.server 4173

deploy: build
	@echo "Deploying to GitHub Pages (record.itsash.in)..."
	@if [ -d docs ]; then \
		git add docs/; \
		git commit -m "Deploy to GitHub Pages: $$(date +'%Y-%m-%d %H:%M')"; \
		git push origin main; \
		echo "Deployed. Configure GitHub Pages: Settings > Pages > Source: Deploy from branch (main / docs)"; \
	else \
		echo "Build failed: docs/ directory not found"; \
		exit 1; \
	fi

help:
	@echo "Makefile targets:"
	@echo "  make all      - clean + build (alias for build)"
	@echo "  make build    - clean, build WASM, install deps, build to docs/"
	@echo "  make wasm     - build only the Rust WASM artifact"
	@echo "  make dev      - build WASM + start Vite dev server"
	@echo "  make serve    - build + preview from docs/ on port 4173"
	@echo "  make deploy   - build + commit + push docs/ to GitHub Pages"
	@echo "  make clean    - remove pkg/ docs/ target/"
	@echo "  make help     - show this help"