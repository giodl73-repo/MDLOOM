# Platform Component Dependencies

## Repository structure

```proof:tree kind=dirtree root=src/user-scenarios max_depth=1
```

## Architecture hierarchy

proof:bullets
- platform
  - auth-service: user-db, cache, jwt-lib
  - api-gateway: auth-service, rate-limiter, router
  - data-pipeline: kafka, schema-registry, storage-client
  - ml-inference: model-store, data-pipeline, gpu-runtime
  - dashboard: api-gateway, websocket-server, chart-lib

## Crate dependencies

proof:bullets
- proof (CLI + lib)
  - proof-canvas: unicode-width
  - proof-math: unicode-width
  - mdpath: thiserror
- icelines
  - icelines-core: (no deps)
  - icelines-fetch: icelines-core
  - icelines-cli: all three above
