CARGO ?= cargo
TMUX_REFERENCE ?= tmux

.PHONY: all build check-tmux check-buffer-memory check-collection-memory clean

all: build

build:
	$(CARGO) build

check-tmux:
	@version="$$($(TMUX_REFERENCE) -V)"; \
	if [ "$$version" != "tmux 3.7b" ]; then \
		echo "expected tmux 3.7b, found $$version" >&2; \
		exit 1; \
	fi

clean:
	$(CARGO) clean

check-buffer-memory:
	RUSTFLAGS="$(RUSTFLAGS) -Zsanitizer=address" $(CARGO) test --lib \
		--target x86_64-unknown-linux-gnu reactor::buffer::tests

check-collection-memory:
	RUSTFLAGS="$(RUSTFLAGS) -Zsanitizer=thread" $(CARGO) test \
		--manifest-path checks/collection-memory/Cargo.toml --locked \
		--target-dir target/collection-memory -Zbuild-std \
		--target x86_64-unknown-linux-gnu --lib -- --test-threads=1
