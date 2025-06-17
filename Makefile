common:
	mkdir -p src/constants
	touch src/constants/rook_magics.rs
	touch src/constants/bishop_magics.rs
	cargo run --bin rook_magics_gen
	cargo run --bin bishop_magics_gen

all: common
	cargo build

verbose: common
	cargo build --verbose
