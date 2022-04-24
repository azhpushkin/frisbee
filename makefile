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
	cargo modules generate graph --layout sfdp  --with-uses | grep -vwE owns | xdot -

check:
	 { echo Anton; echo Zhdan; } | cargo run -- examples/tuples.frisbee > /dev/null
	 echo Name | cargo run -- examples/strings.frisbee > /dev/null

	 cargo run -- examples/print.frisbee > /dev/null
	 cargo run -- examples/loop.frisbee > /dev/null

	 @echo "ALL GOOD OLD FILES WORK FINE"
