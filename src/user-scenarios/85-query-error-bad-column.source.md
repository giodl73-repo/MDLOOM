# US-85 — ?select with a bogus column errors loudly

Negative test: a typo'd column name in `?select` should produce a clear
COMPILE-002 error naming the bad column rather than silently rendering an
empty table.

```proof:table
md://src/user-scenarios/data/models.md#:table:0?select=model,bogus
```

Compiling this file should fail with `?select references unknown column "bogus"`
and the directive's source line in the error message.
