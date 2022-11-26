# Goblin

a programming language for perverts

---

Three goals of Goblin:

- disguise a functional language as an object-oriented one
- create a syntax that bothers people as much as Lisp without making something bad-on-purpose
-

## Notable features

### Syntax

Smalltalk-style method syntax, where method name & parameters are combined into a single "selector". Operators are limited to left-associative binary operators with a single level of precedence, but are user-configurable. Most control flow done with blocks, like Ruby / Smalltalk, rather than keyword statements.

### Frames

General purpose data structure, which can be used as tuples, records, enums, and functions

### Local mutability

`var` bindings and parameters allow you to write in an imperative style while using immutable data. Similar to Swift's copy-on-write data structures & `inout` parameters

### Dynamic scope

`provide` and `use` provide language-level support for app context, dependency injection, singletons, feature flags & OS-level resources, and allow for the elimination of globals, static variables, and "ambient authority"

### Modules

