# ADR-039: Channel Adapters

Status: Implemented

## Context

RLMX agents communicate primarily through internal domain events and the MCP protocol. To reach users on external platforms (Telegram, WhatsApp, Microsoft Teams, Discord), the system needs a pluggable channel adapter layer that normalizes message formats and abstracts platform-specific APIs.

## Decision

Create `rlmx-channels` (Rust crate) and `@aix/channels` (TypeScript package) implementing a channel adapter registry with the following design:

### Architecture

```
                    ChannelRegistry
                    /    |    |    \
            Telegram WhatsApp Teams Discord  (stub adapters)
```

### Core Types

- **ChannelId**: Newtype over UUID identifying a channel instance
- **ChannelMessage**: Normalized message with id, channel, sender, content, attachments, reply_to, timestamp
- **ChannelConfig**: Channel type + credentials map + enabled flag
- **ChannelAdapter trait**: `send`, `receive`, `health_check`, `channel_type`

### Adapter Registry

`ChannelRegistry` holds named adapters and provides:
- `register(name, adapter)` — add a platform adapter
- `send(name, msg)` — route message to named adapter
- `list()` — enumerate registered adapters
- `health(name)` — check adapter connectivity
- `remove(name)` — deregister an adapter

### Stub Implementations

Initial adapters for Telegram, WhatsApp, Teams, and Discord are stubs that:
- Accept configuration (API tokens, webhook URLs)
- Return success on `send` (log via `tracing`)
- Return a placeholder message on `receive`
- Report healthy if credentials are configured

Real platform integrations will replace stubs when API keys are available.

### Domain Events

- `ChannelMessageReceived` — emitted when an adapter receives a message
- `ChannelMessageSent` — emitted when an adapter sends a message

These events integrate with the trigger registry (ADR-038) for event-driven routing.

## Consequences

- Uniform message interface across all platforms
- Adding new platforms requires only implementing the `ChannelAdapter` trait
- Stub adapters allow testing the full pipeline without external API keys
- Channel events flow through the domain event bus for trigger matching
- Credentials are stored in `ChannelConfig` — never hardcoded
