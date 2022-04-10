// TODO: should checks have some kind of error codes?

* check that return statement is required

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
fun Int hello() {}

FAILS: Int value not returned!

---
fun Int hello() {
    return 2.2;
}

FAILS: float value returned, instead of Int

---
fun Nil hello() {}

OK: implicit return;

---
fun Nil hello() { return; }

OK: implicit nil return
