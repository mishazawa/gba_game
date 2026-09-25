export HEADROOM_TELEMETRY := off

NAME := my_game
MGBA := /Applications/mGBA.app/Contents/MacOS/mGBA


.PHONY: run release build aider rom-fix rom-run

build:
	cargo build

run:
	cargo run

release:
	cargo build --release

aider:
	headroom wrap aider --watch-files --config .aider.conf.yml

rom-fix:
	agb-gbafix target/thumbv4t-none-eabi/release/$(NAME) -o roms/$(NAME).gba 

rom-run:
	$(MGBA) -C logToStdout=1 -C logLevel.gba.debug=127 roms/$(NAME).gba