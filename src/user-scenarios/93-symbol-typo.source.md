# US-93 — Did-you-mean for unknown symbol

Negative test: a typo'd symbol name produces a SYMBOL-001 warning with a
suggestion drawn from the built-in library.

```mdloom:symbol checkmrk size=2
```

Compile output should warn `Unknown symbol 'checkmrk' — did you mean 'checkmark'?`
