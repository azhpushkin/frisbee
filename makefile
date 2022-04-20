t:
	cargo test

format:
	cargo fmt -- ./src/*.rs
	cargo fmt -- ./src/**/*.rs
	cargo fmt -- ./src/**/**/*.rs

a:
	git add .

r:
	# make r f=../examples/test.frisbee
	cargo run -- ${f}

g:
	cargo modules generate graph --layout sfdp  --with-uses | xdot -
