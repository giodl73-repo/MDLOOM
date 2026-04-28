# Benchmark Results — proof compile throughput

Compile times measured on the MAXIM 2,703-file corpus.

## Throughput by cache tier (files/sec)

```proof:chart kind=bar width=60
Cold (no cache): 120
Tier 3 hit: 890
Tier 2 + 3: 1240
All tiers warm: 2100
```

## Latency by operation (ms)

```proof:chart kind=bar width=50
Parse source: 2
Resolve URI: 8
Render math: 5
Write output: 1
```

## Speedup

The Tier 2 resolve cache eliminates repeated mdpath calls for figures
referenced by multiple source documents in the same compile run.
