generate:
	cd generator && cargo build --release && cargo run --release > ../src/tables.rs
	rustfmt src/tables.rs
