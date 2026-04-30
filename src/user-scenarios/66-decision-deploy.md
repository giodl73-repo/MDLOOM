# US-66 — Decision tree: deployment escalation

<!-- proof:compiled from="proof:tree kind=decision" uri="" -->
```decision
Tests passing?
├── Yes → Coverage above 80%?
│   ├── Yes → Smoke tests green?
│   │   ├── Yes → Latency within SLO?
│   │   │   ├── Yes → done
│   │   │   └── No  → rollback
│   │   └── No  → rollback
│   └── No  → hold-for-review
└── No  → block
```
<!-- /proof:compiled -->

Five-node decision tree with two leaf states (`done`, `rollback`) plus
two early-exit leaves (`block`, `hold-for-review`).
