debug:
	cargo build
	cd target/debug && ./chessica

release:
	cargo build --release
	cd target/release && ./chessica
