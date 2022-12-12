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
