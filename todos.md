- wrap selector strings in Rc
- store selectors in stack frames for error traces
- clean up var args so we're not just copying the whole arg array
- compiler returns proper errors for var, which are then "decorated" with source
- SendEffect callback for primitives that need send messages in a handler

Figure out the rules for `at:`, `from:to:`. Proposal

given collection of length N:

```
0 <= at < N -> collection[at]
-N < at < 0 -> collection[length - at]
otherwise   -> error
```
