CARGO ?= cargo
TMUX_REFERENCE ?= tmux

.PHONY: all build check-tmux clean

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
