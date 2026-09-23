export HEADROOM_TELEMETRY := off

.PHONY: run release build aider

build:
	cargo build

run:
	cargo run --release

release:
	cargo build --release

aider:
	headroom wrap aider --watch-files --config .aider.conf.yml
