all:
	python main.py

get-exe:
	cp parser/.stack-work/dist/x86_64-linux-tinfo6/Cabal-2.4.0.1/build/frisbee-exe/frisbee-exe ./


force:
	cd parser && stack build --ghc-options -O2 --force-dirty

