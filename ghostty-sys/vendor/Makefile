REPO   := https://github.com/ghostty-org/ghostty
REF    := main
COMMIT := 2de5e7d38e1354759211722a8687c0815d2cf02c
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
	for patch in $(PATCHES); do \
		git apply "$(CURDIR)/$$patch"; \
	done
	@echo "Regenerated $(DIR)/ from $(REPO)@$(COMMIT)"

## clean: remove the temporary clone/work directory
clean:
	rm -rf $(WORK)
