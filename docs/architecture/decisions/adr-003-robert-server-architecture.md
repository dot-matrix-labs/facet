# ADR-003: Robert Server Architecture (Local + Remote Modes)

**Date**: 2025-12-02
**Status**: Accepted
**Updated**: 2025-12-05

## Context

Initial consideration was to deprecate `robert-server` in favor of embedding all logic directly in `robert-app`. However, after further analysis, we determined that supporting both local and remote/team deployments requires maintaining the server as a separate component.

## Decision

We will **keep `robert-server`** as a core component with dual deployment modes:

1. **Local Mode**: `robert-app` spawns a local `robert-server` instance (127.0.0.1:8443) automatically for individual users
2. **Remote Mode**: `robert-server` runs as a standalone service for team deployments, cloud hosting, and headless/CI-CD use cases

Key responsibilities:
- `robert-server` provides the REST API layer and manages `robert-core` instances
- `robert-app` can connect to either a local or remote server
- `robert-cli` can start/manage servers and act as a client

## Consequences

### Pros
- Supports both individual and team use cases
- Enables cloud/remote deployments for enterprises
- Clean API boundary between UI and business logic
- Allows headless/CLI-only usage
- Better separation of concerns

### Cons
- Slightly more complex architecture than fully embedded approach
- Requires managing server lifecycle in desktop app
- Additional component to maintain and test
