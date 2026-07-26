# Platform Component Dependencies

## Repository structure

```mdloom:tree kind=dirtree root=src/user-scenarios max_depth=1
```

## Architecture hierarchy

mdloom:bullets
- platform
  - auth-service: user-db, cache, jwt-lib
  - api-gateway: auth-service, rate-limiter, router
  - data-pipeline: kafka, schema-registry, storage-client
  - ml-inference: model-store, data-pipeline, gpu-runtime
  - dashboard: api-gateway, websocket-server, chart-lib

## Crate dependencies

mdloom:bullets
- mdloom (CLI + lib)
  - mdloom-canvas: unicode-width
  - mdloom-math: unicode-width
  - mdpath: thiserror
- icelines
  - icelines-core: (no deps)
  - icelines-fetch: icelines-core
  - icelines-cli: all three above
