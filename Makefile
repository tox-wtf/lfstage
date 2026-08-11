all: build build-man

build:
	cargo build --release

clean:
	cargo clean
	find man -mindepth 1 -maxdepth 1 -type f ! -name '*.scd' -exec rm -f {} +

check: test
	cargo clippy
	cargo audit

# TODO: Allow running the tests not as root
test:
	@echo -e "\x1b[37;1m[\x1b[31mWARNING\x1b[37m]\x1b[0m The test suite assumes LFStage is installed"                 >&2
	@echo -e "\x1b[37;1m[\x1b[31mWARNING\x1b[37m]\x1b[0m It also makes assumptions about the environment it's run in" >&2
	@echo -e "\x1b[37;1m[\x1b[31mWARNING\x1b[37m]\x1b[0m Lastly, it's meant to be run by the maintainer"              >&2
	@echo -e "\x1b[37;1m[\x1b[31mWARNING\x1b[37m]\x1b[0m In other words, take test failures with a grain of salt"     >&2
	@sleep 1
	@if cargo --list | grep -q nextest; then cargo nextest run; else cargo test; fi

install: install-man install-var
	install -Dm755 target/release/lfstage  -t    $(DESTDIR)/usr/bin/
	cp -af usr                                   $(DESTDIR)/
	cp -af etc                                   $(DESTDIR)/

build-man:
	@for m in man/*.scd; do  \
		out=$${m%.scd};  \
		scdoc < $$m > $$out; \
	done

install-man:
	@for m in man/*.[1-8]; do \
		m=$${m##*/}; \
		sect=$${m##*.}; \
		install -vDm644 man/$$m -t $(DESTDIR)/usr/share/man/man$$sect/; \
	done

uninstall:
	rm -f  $(DESTDIR)/usr/bin/lfstage
	rm -rf $(DESTDIR)/etc/lfstage                $(DESTDIR)/usr/lib/lfstage
	rm -rf $(DESTDIR)/var/lib/lfstage            $(DESTDIR)/var/cache/lfstage
	rm -rf $(DESTDIR)/var/log/lfstage            $(DESTDIR)/tmp/lfstage

install-var:
	install -dm755 $(DESTDIR)/var/lib/lfstage/mount        \
	               $(DESTDIR)/var/lib/lfstage/profiles     \
	               $(DESTDIR)/var/cache/lfstage/profiles   \
	               $(DESTDIR)/var/cache/lfstage/stages

.PHONY: all build build-man check clean install install-man install-var test uninstall 
