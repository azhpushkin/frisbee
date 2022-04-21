// TODO: should checks have some kind of error codes?

* check that return statement is required
* no return for constructors!!
* check that `this` in arguments is not allowed (this will ensure this is only for methods)
* check if multiline strings are OK and how they work
* check that variable is always assigned before using (all if branches)
* check divide and minus ordering if chained (just to check everything is fine)
* check that conditions are bool in if and else
* check foreach (conflict with other variable)
* check how everything works when there is multiple values with same name (for i in ...)
* check break and continue in loop only!
* check how single-item tuples work!
* check that function names cant overlap with std functions
* what if only part of tuple is defined? E.g. 
    ```example
    String a = 123;
    let b = (a, "asd");
    ```
* but in this case there is no expected at all due to the `let` (and we cant have `(String, _)`)
* next
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

