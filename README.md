# frisbee
Frisbee, actor-based programming language for distributed computing

:hourglass: Better readme incoming soon

Key ideas to describe this programming languages:
* actor-flavoured, implements [Active objects](https://en.wikipedia.org/wiki/Active_object) concept
  * currently, runs OS thread per active object
  * objects communicate using async messages, executed one-by-one
  * messages are serialized and deserialized on send and retrieval
  * no shared memory between objects
* compiles to bytecode, that runs on stack VM
* strictly-typed, performs semantic analysis before compiling
* statically-typed, features no overhead over simple types in runtime


Currently in progress:  
:x: simple GC, based on stack maps and types metadata  
:x: deduction of nullable types inside if-else statements  
:pray: WASM runtime so I can put this on my website without having backend calculations


### Quick basic types overview
Basic types are Int, Float, Bool and String.
```
// This is a comment

Int i = 12;  // 64-byte integer
Float f = 3.14;  // 64-byte double precision
Bool b = false;  // bool

// Simple string, immutable by design
// Both single and double quotes are fine
String s = "Hello world";
```

There are also 2 container types: List and Tuple
```
(String, Int) person = ("Anton", 24);

[String] names = ["Bob", "Alice"] ;
```

Each type can be marked as nullable using `?`.
```
String? someone = nil;
someone = "Mark";

[Int] wallet = [1, 2, 3];
wallet = nil;
```
Using `nil` with not-nullable types is restricted

### Defining own types
You can define own type as either class or active type.

**Classes** are just like classes in the other languages.
You can define methods and fields for them.
```
class Person {
   (String, String?) name;  //  first + last
   Int? age;
   
   fun void say_hello() {
       println("Hello, ", @name[0]);
   }
}

Person person = Person(("Anton", nil), 24);
person.say_hello()
```

Own fields and methods are accessible using `@` modifier, so calling
`say_hello` method from inside of the class would be look like `@say_hello()`

**Active** objects are very similar to classes, except they are not
__created__ but are rather __spawned__.
Spawning active object creates its own thread of execution, and performs all the method
processing in it.

You also can't simply use values and methods of an active objects. Fields are not available (yet!)
for them, and methods are accessible via Erlang-like message passing:
```
active Worker {
   fun void process_message(String message) {
      println(message);
   }
}

Worker w = spawn Worker();  // NOTE: spawn is used
w ! process_message("Say hi!");  // return value is not available
```

### Some other features

* if - elif - else branching
* while loops, break, continue
* foreach loops (iterate over lists)
* simple formatted prints
* null-operators :ok_hand:
  * `nullable_object?.method()` - access fields and methods for nullable objects
  * `nullable_int ?: -1` - elvis operator that narrows nullable type to non-nullable one
* simple imports system



## Some dummy program example:
```
class Person {
    String name;
    Int age;

    fun String name_extended(String prefix) {
        return prefix + " " + @name;
    }
}

fun void main() {
    Person p = Person("Anton", 244);
    println(p.name_extended("Mr.") + " is of age " + p.age.to_string());
    p.age = 24;
    println("Whoops, I meant " + p.age.to_string());
}

/* EXPECTED STDOUT
Mr. Anton is of age 244
Whoops, I meant 24
*/
```

## Examples
Make sure to go to [examples](examples) directory if you want to see more working examples.
Those are tested and validated :heavy_check_mark: with a simple script run (no CI/CD here yet)
