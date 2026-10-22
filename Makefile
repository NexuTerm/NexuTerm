PREFIX ?= /usr/local
DESTDIR ?=
CARGO ?= cargo
BINARY ?= target/release/nexuterm

.PHONY: build install
build:
	$(CARGO) build --locked --release

# Run make build first. DESTDIR allows packaging without changing the system.
install:
	install -Dm755 "$(BINARY)" "$(DESTDIR)$(PREFIX)/bin/nexuterm"
	install -Dm644 packaging/io.nexuterm.NexuTerm.desktop "$(DESTDIR)$(PREFIX)/share/applications/io.nexuterm.NexuTerm.desktop"
	install -Dm644 logo/NexuTerm.png "$(DESTDIR)$(PREFIX)/share/pixmaps/io.nexuterm.NexuTerm.png"
