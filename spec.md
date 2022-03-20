## Typesystem
There are several basic types:
* `Nil` (`nil` is only value of such type)
* `Bool` (`true` and `false` are the only values)
* `Int` (4 byte signed)
* `Float` (4 byte)
* `String`
**Note: functions are not first class citizens!**



There are 3 more built-in ways to modify those types:
* Nillable types
  Noted using a question mark: `String?` or `Int?`. This means that
  value might be either of a given type or nil.
  **This is not a wrapper, like Optional<T> or Maybe<T>**. Thus,
  `String??` is not allowed, as type is either nillable or not nillable
* Tuples
  Example: `(String, Int)`. You can access tuple data using `f1`, `f2`, `f<n>` fields.
  These fields are checked at compile-time.
* Lists:
  Example: `[String]` . Stores list of data

  


Imports:
```
from asd import Type, function;


```

  