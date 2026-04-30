# US-66 — Decision tree: deployment escalation

```proof:tree kind=decision
| Node | Condition | Yes | No |
|------|-----------|-----|-----|
| root | Tests passing? | merge | block |
| merge | Coverage above 80%? | deploy-staging | hold-for-review |
| deploy-staging | Smoke tests green? | deploy-prod | rollback |
| deploy-prod | Latency within SLO? | done | rollback |
```

Five-node decision tree with two leaf states (`done`, `rollback`) plus
two early-exit leaves (`block`, `hold-for-review`).
