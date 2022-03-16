t:
	cargo test

f:
	cargo fmt -- ./src/**/*.rs

a: f
	git add .

r:
	# make r f=../examples/test.frisbee
	cargo run -- ${f}