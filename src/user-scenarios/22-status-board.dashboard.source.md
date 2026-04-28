---
dashboard:
  width: 80
  height: 20
  title: "Service Status"
  regions:
    header:   { x: 0,  y: 0,  width: 80, height: 2  }
    svc-a:    { x: 0,  y: 2,  width: 38, height: 8  }
    svc-b:    { x: 42, y: 2,  width: 38, height: 8  }
    divider:  { x: 39, y: 2,  width: 2,  height: 8  }
    summary:  { x: 0,  y: 10, width: 80, height: 7  }
    footer:   { x: 0,  y: 17, width: 80, height: 3  }
---

```proof:region name=header
SERVICE STATUS BOARD                                    [sym:checkmark] All systems operational
```

```proof:region name=svc-a
API Gateway
proof:element kind=label value="99.9% uptime" width=16
proof:element kind=label value="142ms p50" width=16
proof:element kind=sparkline value="140,145,138,142,150,141,142" width=36
proof:element kind=badge value="healthy" width=10
```

```proof:region name=divider
│
│
│
│
│
│
│
│
```

```proof:region name=svc-b
Data Pipeline
proof:element kind=label value="99.7% uptime" width=16
proof:element kind=label value="890ms p50" width=16
proof:element kind=sparkline value="800,820,880,910,870,890,890" width=36
proof:element kind=badge value="healthy" width=10
```

```proof:region name=summary
Last 24 hours:
proof:element kind=label value="2.1M API reqs" width=16
proof:element kind=label value="8.4M events" width=16
proof:element kind=value value="0" label="incidents" width=12
proof:element kind=label value="-23ms trend" width=14
```

```proof:region name=footer
Updated: 2026-04-28 02:30 UTC  |  proof compile --watch  |  Next refresh: 60s
```
