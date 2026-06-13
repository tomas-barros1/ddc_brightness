PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
DESKTOPDIR ?= $(DATADIR)/applications

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install: build
	install -Dm755 target/release/ddc_brightness $(DESTDIR)$(BINDIR)/ddc_brightness
	install -Dm644 ddc_brightness.desktop $(DESTDIR)$(DESKTOPDIR)/ddc_brightness.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/ddc_brightness
	rm -f $(DESTDIR)$(DESKTOPDIR)/ddc_brightness.desktop

clean:
	cargo clean
