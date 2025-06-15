# Linsl

Linsl -- Linsl is not Scheme or Lisp -- is a *very* simple Lisp/Scheme like
language. It is currently technically functional and (should be) turing
complete, although far from complete (or even really pleasant to use).

## Usage

Linsl code is -- just as other lisp dialects -- based around expressions. An
expression is either

- an atom, or
- a list.

### Atoms
There are (currently) three kinds of atoms:

- Numbers, currently 64-bit floats,
- bools[^bools], either `#t` or `#f` and
- symbols, which can be any string.

[^bools]: Note that unlike Lisp -- and like Scheme -- Linsl considers booleans
    to be a type in their own right, and does not consider lists valid truth
    values. Consequently, expressions like `(if (+ 1 2) (...) (...))` will
    cause an error.

Numbers and bools are self-evaluating, unlike symbols; symbols must first be
defined (see [here](#define)), and when evaluated will evaluate
to whatever they are defined as evaluates to.

### Lists

A list is a sequence of expressions, separated by white space and surrounded by
regular parentheses. Furthermore, for a list to be a valid expression, its head
must be either a [primitive](#primitives) or a [special
form](#special-forms). For example, `(1 2 3)` is technically a valid list,
but not a valid expression, while `(+ 1 2)` is both.

### Primitives

Primitives are built in 'functions', i.e. transformations of expressions into
other expressions. Here follows a complete list of the currently existing
primitives.

#### The `+`-primitive

`+` takes a list of numbers, and returns their sum. For example, `(+ 1 2 3)`
evaluates to `6`.

#### The `neg`-primitive

`neg` takes a single number and returns its negation. For example, `(neg 1)`
evaluates to `-1`.

Note that -- unlike other lisp/scheme dialects -- there is no built in `-`.
This is in an effort to keep the interpreter as minimal as possible, since this
function can be defined in Linsl using the `+` and `neg` primitives.

#### The `*`-primitive

`*` takes a list of numbers and returns their product. For example, `(* 2 3 4)`
evaluates to `24`.

#### The `inv`-primitive

`inv` takes a single, non-zero number and returns its reciprocal. For example,
`(inv 2)` evaluates to `0.5`.

Note that -- for the same reasons as there is no `-` -- there is no built in `/`.

#### The `=`-primitive

`=` tests two expressions for equality, after evaluation. For example, `(= 1 (+
0 1))` evaluates to `#t`. 

Only booleans and numbers can be compared,
and only two expressions of the same type; in other words, the expression `(=
#t 1)` will generate an error, since `1` and `#t` are not the same type.

#### The `>`-primitive

`>` takes two numbers `a` and `b`, and returns `#t` if `a` is greater than `b`
and `#f` otherwise.


### Special Forms

Special forms act like primitives or functions, but differ in that they change
how evaluation is done. While a primitive always evaluates all its parameters
first, this is not the case with special forms. Below follows a description of all special forms.

#### The `define` Special Form

`define` takes two expressions, the first of which must be a symbol. It then
adds this symbol to the environment, bound to the evaluation of second
expression.

#### The `if` Special Form

`if` takes three expressions, the first of which must evaluate to a boolean b.
Then, if b is true it evaluates the second form, otherwise it evaluates the
third form.

#### the `lambda` Special Form

`lambda` takes two expressions, the first of which must be a list of symbols.
The second can be any expression, and can use the symbols listed in the first
list.

Essentially, the `lambda` special form allows you to create parameterized
expressions. If a `lambda` form is combined with a `define` form, a name can be
given to this parameterized expression, and it can then be used as the head of
a list, just as any of the primitives.

# Acknowledgements

This implementation was heavily inspired by [Risp by Stepan
Parunashvili](https://stopa.io/post/222) and [rustylisp by
galzmarc](https://dev.to/galzmarc/building-a-lisp-interpreter-in-rust-2njj).
Further inspiration and tricks were taken from [Lispy by Peter
Norvig](https://norvig.com/lispy.html) and [tinylisp by Robert van
Engelen](https://github.com/Robert-van-Engelen/tinylisp)
