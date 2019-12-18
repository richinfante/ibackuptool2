all:
	cargo build --release

install:
	cp ./target/release/ibackuptool2 /usr/local/bin/ibackuptool2
