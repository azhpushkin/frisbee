all:
	python main.py

simple:
	python main.py simple.frisbee

exe:
	cp parser/.stack-work/dist/x86_64-linux-tinfo6/Cabal-2.4.0.1/build/frisbee-exe/frisbee-exe ./

gen-ast:
	python ast_generator.py

build-exe:
	cd parser && stack build --ghc-options -O2 --force-dirty

