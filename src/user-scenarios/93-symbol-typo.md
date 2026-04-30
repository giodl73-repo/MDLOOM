# US-93 — Did-you-mean for unknown symbol

Negative test: a typo'd symbol name produces a SYMBOL-001 warning with a
suggestion drawn from the built-in library.

```proof:symbol checkmrk size=2
```

Compile output should warn `Unknown symbol 'checkmrk' — did you mean 'checkmark'?`
