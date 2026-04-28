# Symbol Typo Demo

This document intentionally contains a typo in a symbol name to demonstrate
the did-you-mean diagnostic.

## Valid symbols

[sym:checkmark] Task complete
[sym:warning] Attention required
[sym:star] Featured item

## Typo — will trigger SYMBOL-001 with did-you-mean

[sym:checkmar] This will produce:
  warning [SYMBOL-001]: Unknown symbol 'checkmar' — did you mean 'checkmark'?

[sym:starr] This will produce:
  warning [SYMBOL-001]: Unknown symbol 'starr' — did you mean 'star'?

[sym:arroww-right] This will produce:
  warning [SYMBOL-001]: Unknown symbol 'arroww-right' — did you mean 'arrow-right'?
