all:
	stack build --fast
	cat asd.asd | stack exec frisbee-exe

alex:
	rm -f src/Tokens.hs
	alex src/Tokens.x -o src/Tokens.hs