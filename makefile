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
	cargo modules generate graph --layout fdp  --with-uses | grep -vwE owns | xdot -

check: t
	 { echo Anton; echo Zhdan; } | cargo run -- examples/tuples.frisbee > /dev/null
	 echo Name | cargo run -- examples/strings.frisbee > /dev/null
	 echo Bodya | cargo run -- examples/list.frisbee > /dev/null

	 cargo run -- examples/print.frisbee > /dev/null
	 cargo run -- examples/loop.frisbee > /dev/null
	 cargo run -- examples/foreach.frisbee > /dev/null
	 cargo run -- examples/object.frisbee > /dev/null

	 @echo "ALL GOOD OLD FILES WORK FINE"
