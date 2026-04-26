# Go Language Guide

# Duplicate H1 — proof catches this

## Overview

Go is a statically-typed compiled language from Google.

## Concurrency Model

```
┌───────────────────────────────────────────────┐
│            GOROUTINE SCHEDULER                 │
│  ┌────────────┐  ┌────────────┐               │
│  │ Goroutine  │  │ Goroutine  │  ← user code  │
│  └─────┬──────┘  └─────┬──────┘               │
│        └────────┬───────┘                      │
│           ┌─────▼──────┐                       │
│           │  OS Thread │                       │
│           └────────────┘                       │
└───────────────────────────────────────────────-┘
```

## Syntax Reference

Multi-column table with misaligned separator:

```
┌──────────────────────┬─────────────┐
│  Concept             │ Go Syntax    │
├─────────────────────┼─────────────┤
│  Variable            │ x := 42     │
│  Function            │ func f() {} │
│  Goroutine           │ go f()      │
│  Channel             │ ch <- val   │
└──────────────────────┴─────────────┘
```

## Type System Snapshot

| Axis         | Value      |
|--------------|------------|
| Binding      | Compile    |
| Typing       | Static     |
| Strength     | Strong     |
| Type system  | Structural |

<!-- Missing: Memory model row — proof.toml requires it -->

## Performance Notes

Go compiles to native code. Garbage collected.
