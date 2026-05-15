# US-109 — Decision tree from external table

Drive the decision tree from a markdown table stored elsewhere. Useful when
the decision logic comes from a process spreadsheet maintained by a
non-engineering stakeholder.

```proof:tree kind=decision source=md://src/user-scenarios/data/triage.md#:table:0
```

Note: this scenario references `triage.md` which doesn't ship with the
corpus — compiling produces a COMPILE-002 file-not-found, useful as a
negative test for resolution error reporting.
