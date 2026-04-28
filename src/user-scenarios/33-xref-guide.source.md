# API Guide

## Overview

This guide covers authentication and endpoints. For background on the data model,

```proof:xref uri="md://src/user-scenarios/33-xref-guide.source.md#data-model" format=note
```

## Authentication

API keys are passed as Bearer tokens. Every request must include:

```
Authorization: Bearer <api-key>
```

For token generation, see:

```proof:xref uri="md://src/user-scenarios/33-xref-guide.source.md#token-generation"
```

## Endpoints

The base URL is `https://api.example.com/v2`.

## Data Model

Each response envelope wraps a `data` object with a `type` discriminant.

## Token Generation

Tokens are issued via the `/auth/token` endpoint with a POST request carrying
`client_id` and `client_secret` in the body.
