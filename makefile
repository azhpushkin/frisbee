t:
	cargo test

format:
	cargo fmt -- ./src/*.rs
	cargo fmt -- ./src/**/*.rs
	cargo fmt -- ./src/**/**/*.rs

a:
	git add .

r:
	# make r f=examples/test.frisbee
	cargo run -- ${f}

g:
	# only show uses to check for cycles and god modules
	# (dot, neato, twopi, circo, fdp, sfdp)
	# alias, type, asts and td: #ff6161
	# codegen: ffe27a
	# vm: ca42ff
	# parser: 83e2f7
	# semantics: 8df783
	cargo modules generate graph --layout fdp  --with-uses | grep -vwE owns | xdot -

check:
	@python runner/test.py
	@rm examples/*.bytecode -rf
