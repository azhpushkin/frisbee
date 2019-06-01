#! /bin/sh

make build

cat "$@" | stack exec frisbee-exe