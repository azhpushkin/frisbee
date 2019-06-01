all:
	stack build --fast
	cat ../simple.frisbee | stack exec frisbee-exe

build:
	stack build --fast

alex:
	rm -f src/Tokens.hs
	alex src/Tokens.x -o src/Tokens.hs