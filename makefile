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
	# only show uses to check for cycles and god modules
	cargo modules generate graph --layout sfdp  --with-uses | grep -vwE owns | xdot -
