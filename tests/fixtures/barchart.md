# Bar Chart Fixtures

## Clean bar chart — proportional bars, aligned values

Bar widths are proportional to percentages relative to 100%.
Max bar width = 30 chars (represents 100%).
78% → 23 chars, 45% → 14 chars, 12% → 4 chars.

```
Item A  ███████████████████████        78%
Item B  ██████████████                 45%
Item C  ████                           12%
```

## Clean ASCII bar chart (hash bars) — no errors

Max = 80% → 24 chars. 52% → 16 chars. 19% → 6 chars.

```
Option 1  ████████████████████████ 80%
Option 2  ████████████████         52%
Option 3  ██████                   19%
```

## Clean no-value bar chart — no errors (no proportionality check without values)

```
Short      ████
Medium     ████████████
Long       ████████████████████
Very long  ████████████████████████████
```

## Disproportionate bars — should detect scale error

Item A at 78% has a bar that fills the full width — looks like 100%.
Correct proportional bar for 78% should be ~23 chars, not 30.

```
Item A  ██████████████████████████████ 78%
Item B  █████████████                  45%
Item C  ████                           12%
```
