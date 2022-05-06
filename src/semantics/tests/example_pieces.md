// TODO: should checks have some kind of error codes?

* check that return statement is required
* no return for constructors!!
* check order of tuple indexes
* check that `this` in arguments is not allowed (this will ensure this is only for methods)
* check if multiline strings are OK and how they work
* check that type fields and methods cannot overlap cannot 
* check that variable is always assigned before using (all if branches)
* check divide and minus ordering if chained (just to check everything is fine)
* check that conditions are bool in if and else
* check foreach (conflict with other variable)
* check how everything works when there is multiple values with same name (for i in ...)
* check break and continue in loop only!
* check and implement something for cases when index is bigger than list
* Implement plus for lists
* check that initial list size is no more than 255
* check different variations of tuple typecheck (items should match)
* check that void and empty tuple cant be nillable in any way
* check assign to maybe tuple, e.g.
    `(Int, (Int, Bool))? a; a = (1, (1, true)); a[1][1] = 2;`
    meaning that if we know that certain part of local is not nil, we can assign INTO it
* check that double-negate is reduced away from ast and works fine
* check operator precedence and grouping way
* check different variations of name collisions with (for i in ..)
* check that `nil?:123` and `nil?.method()` expressions are checked and
    shown as errors (no sence + cant derive types for them)
* check how defining inside of if-else branches works 
* check AND and OR - if part is executed if it is not yet ready
    // e.g. true and obj.call()  -- call should not be executed..
* assignment - what is executed first, left or right?
   maybe typecheck left first, but execute right....
   should check this anyways..
* check uninitialized fields in a custom constructor..
* check that functions can't overlap with stdlib function names
* check how it works if variable is defined inside of the while loop
* check range() start and end (only pos? negative allowed?)
* what if there are function argument names "this"?
* check nested breaks and continue
* check that even after brake, loop variables are popped
* check that tuple size is no more than 255
* ensure that return is REQUIRED, but constructor must not have one
* check how single-item tuples work!
* check that main function is `void`
* check that function names cant overlap with std functions
* what if only part of tuple is defined? E.g. 
    ```example
    String a = 123;
    let b = (a, "asd");
    ```
* but in this case there is no expected at all due to the `let` (and we cant have `(String, _)`)
* check if empty tuple is allowed as a type
* change tuple inside of list, how....
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

