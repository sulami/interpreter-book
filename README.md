# Losp

In this repo, I'm working through the second half of [Crafting
Interpreters](https://craftinginterpreters.com), the part that is about
compiling and running bytecode.

I'm using the book more as an inspiration than a guide, so not only am I working
in Rust instead of C (for learning purposes), but I am also implementing a
different grammar (for aesthetic purposes). I'm going to be building a Lisp,
assuming everything works out.

Note that this is my first ever Rust code, so it's probably terrible all-around.
Also, it's not actually very fast, because it's copying around a lot of data
rather needlessly.

# Usage

If none of the above deterred you and you actually want to run this, here is
how:

```sh
# Build with
rustc -O losp.rs
```

No external dependencies, no cargo.

```sh
$ ./losp # gives you a nice help print
usage:
losp repl         - start repl
losp depl         - start debug repl
losp run <file>   - run file
losp debug <file> - debug file
```

The `debug` print out bytecode as it's being run, which can be _a lot_ if you're
using functions or loop, so be advised.

The losp syntax is vaguely Scheme-inspired, with a dash of Clojure:

```scheme
; comments are from semicolons to the end of the line
; whitespace is ignored

; data types
nil     ; nil
true    ; booleans
3       ; 64-bit integers
.3      ; 64-bit floats
"3"     ; strings
foo     ; symbols
; techically functions are first class, but there is nothing useful
; you can do with them at this point
; there are also no lists, which is a bit ironic

; global variables with `def`
(def pi 3.14159)

; local variables with `let`
; scoping is lexical
(let ((a 1)
      (b 2))
  (+ a b))

; branching with `if` and `when`
(if false
  nil
  (when true
    (print "`when` has an implicit `do` block")
    4))

; `do` allows chaining of operations
(do (print "first")
    (print "second"))

; functions use `defn`, and have an implicit `do` block
; no TCO at this point
; arity is checked at call-time
(defn foo (a b)
  (+ a b))

; there is a `while` loop, not that it's very useful
; they also have an implicit `do` block
(def i 0)
(while (< i 10)
  (def i (+ i 1)))

; equality is by value, the following is true
(let ((a 1)
       b 1)
  (= a b))

; numbers are weakly typed, the following returns 1.1
(+ 1 .1)
(/ 2.2 2)

; no explicit type casts exist

; `print` always prints a trailing newline
```

You can see the included test file (in Losp) for more usage examples.
