- wrap selector strings in Rc
- store selectors in stack frames for error traces
- compiler returns proper errors for var, which are then "decorated" with source

Figure out the rules for `at:`, `from:to:`. Proposal

given collection of length N:

```
0 <= at < N -> collection[at]
-N < at < 0 -> collection[length - at]
otherwise   -> error
```

frame - object - do block continuum of data-focused vs behavior-focused

Perf ideas:

- constant folding / propagation
- send direct to constants / self
- move common low-level calls (addition, equality etc) to IR
- inline method calls to constants / self
- only compile self-references if used
- IR to get Ival of target rather than self & inline getter methods to use this
- increment locals counter on unused expressions instead of dropping
