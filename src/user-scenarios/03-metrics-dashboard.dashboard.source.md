---
dashboard:
  width: 80
  height: 16
  title: "Daily Metrics"
  regions:
    header:  { x: 0,  y: 0,  width: 80, height: 2  }
    kpi1:    { x: 0,  y: 2,  width: 20, height: 5  }
    kpi2:    { x: 20, y: 2,  width: 20, height: 5  }
    kpi3:    { x: 40, y: 2,  width: 20, height: 5  }
    kpi4:    { x: 60, y: 2,  width: 20, height: 5  }
    trend:   { x: 0,  y: 7,  width: 80, height: 6  }
    footer:  { x: 0,  y: 13, width: 80, height: 3  }
---

```mdloom:region name=header
DAILY METRICS BOARD                                              2026-04-28
```

```mdloom:region name=kpi1
mdloom:element kind=label value="99.9%" width=14
mdloom:element kind=badge value="Uptime" width=8
```

```mdloom:region name=kpi2
mdloom:element kind=label value="142ms" width=14
mdloom:element kind=badge value="P50" width=8
```

```mdloom:region name=kpi3
mdloom:element kind=value value="1847" label="Req/sec" width=18
mdloom:element kind=delta value="+203" width=8
```

```mdloom:region name=kpi4
mdloom:element kind=value value="0" label="Errors" width=12
mdloom:element kind=badge value="clean" width=8
```

```mdloom:region name=trend
Throughput (7 days):
mdloom:element kind=sparkline value="1200,1350,1100,1600,1750,1820,1847" width=78
```

```mdloom:region name=footer
[sym:info] Auto-refresh every 60s  |  mdloom compile --watch  |  v0.5.0
```
