.PHONY: test maintainer-tools maintainer-toolcheck maintainer-check maintainer-build

test:
	cargo test

maintainer-tools:
	@set -eu; \
	if command -v nix >/dev/null 2>&1; then \
		echo "nix found; maintainer tools are available through: nix develop ./whitepaper"; \
	else \
		echo "nix not found; installing local maintainer tools when missing"; \
		command -v mdbook >/dev/null 2>&1 || cargo install mdbook; \
		command -v mdbook-katex >/dev/null 2>&1 || cargo install mdbook-katex; \
		if ! command -v lake >/dev/null 2>&1; then \
			if command -v elan >/dev/null 2>&1; then \
				elan default leanprover/lean4:stable; \
			else \
				echo "lake not found. Install Nix, or install Lean with elan:"; \
				echo "  curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh"; \
				exit 1; \
			fi; \
		fi; \
	fi

maintainer-toolcheck:
	@set -eu; \
	if command -v nix >/dev/null 2>&1; then \
		exit 0; \
	fi; \
	missing=0; \
	for tool in lake mdbook mdbook-katex; do \
		if ! command -v "$$tool" >/dev/null 2>&1; then \
			echo "missing $$tool"; \
			missing=1; \
		fi; \
	done; \
	if [ "$$missing" -ne 0 ]; then \
		echo "Run 'make maintainer-tools'."; \
		exit 1; \
	fi

maintainer-check:
	@set -eu; \
	if command -v nix >/dev/null 2>&1; then \
		nix flake check ./whitepaper; \
	else \
		make maintainer-toolcheck; \
		cd whitepaper/lean && lake build; \
		mdbook build whitepaper; \
	fi

maintainer-build:
	@set -eu; \
	if command -v nix >/dev/null 2>&1; then \
		nix build ./whitepaper#whitepaper-html; \
	else \
		make maintainer-toolcheck; \
		cd whitepaper/lean && lake build; \
		mdbook build whitepaper; \
	fi
