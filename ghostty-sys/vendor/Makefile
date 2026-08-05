REPO   := https://github.com/ghostty-org/ghostty
REF    := main
COMMIT := 2346c4fe4767373cda50b5977d95191f714b1cca
DIR  := libghostty-vt
WORK := .vendor-work
ZIG  ?= zig
PATCHES := $(sort $(wildcard patches/*.patch))

.PHONY: vendor clean

## vendor: regenerate $(DIR)/ from $(REPO) at $(COMMIT)
## Downloading dependencies from internet - try zigfetch or scp from internet connected host
vendor:
	rm -rf $(WORK)
	git clone $(REPO) $(WORK)
	git -C $(WORK) checkout $(COMMIT)
	cd $(WORK) && $(ZIG) build dist -Demit-lib-vt
	rm -rf $(DIR)
	mkdir -p $(DIR)
	tar xzf $(WORK)/zig-out/dist/$(DIR)-*.tar.gz -C $(DIR) --strip-components=1
	rm -rf $(WORK)
	# Patch paths are rooted at the hmux workspace, so strip the leading
	# `a/ghostty-sys/vendor` while applying from this directory.
	for patch in $(PATCHES); do \
		git apply -p3 "$(CURDIR)/$$patch"; \
	done
	@echo "Regenerated $(DIR)/ from $(REPO)@$(COMMIT)"

## clean: remove the temporary clone/work directory
clean:
	rm -rf $(WORK)
