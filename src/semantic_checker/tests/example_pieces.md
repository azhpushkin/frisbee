// TODO: should checks have some kind of error codes?


---
fun Nil hello(Int a, Int a) {}

FAILS: cant have arguments with the same name

---
fun Nil hello(Int a) {
    Int a = 123;
}

FAILS: redefinition of the `a` variable (same as above)

---
fun Nil hello() {
    a = 123;
}

FAILS: variable a not defined;

---