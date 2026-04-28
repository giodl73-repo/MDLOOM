# Inherited Docs — Before Fix Pass

These files were inherited with 47 alignment errors.
The boxes below show a sample of the problems.

## Broken Box 1 — width mismatch

```
┌────────────────────────────────────┐
│ Service A                          │
│  → depends on Service B            │
│  → depends on Service C            │
└────────────────────────────────┘
```

## Broken Box 2 — short bottom border

```
┌──────────────────────────────────────┐
│ Pipeline Stage                       │
│   Source → Transform → Sink          │
└──────────────────────────┘
```

## Broken Box 3 — short top border

```
┌────────────────────┐
│ Cache Layer         │
│  L1: in-process     │
│  L2: Redis          │
│  L3: CDN            │
└──────────────────────┘
```

## Good Box — should be left alone

```
┌──────────────────────┐
│ Load Balancer        │
│  round-robin policy  │
└──────────────────────┘
```
