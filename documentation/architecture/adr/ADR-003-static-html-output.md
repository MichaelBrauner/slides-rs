# 003. Static HTML Output (No Server)

In the context of **delivering presentations to end users**,
facing **the need for maximum portability and simplicity**,
we decided for **static HTML files**
to achieve **zero-dependency viewing**,
accepting **no dynamic features without JavaScript**.

## Context

The generated presentation needs to be viewable by end users. Options:

- Static HTML files that can be opened directly in a browser
- Single-page application (SPA) that requires a build step
- Server-rendered application that requires a running server

Alternatives considered:

| Option | Pros | Cons |
|--------|------|------|
| **Static HTML** | Open in any browser, share as files | No server-side features |
| **SPA (React/Vue)** | Rich interactivity | Requires bundler, larger output |
| **Server (Express/etc)** | Dynamic features | Requires running server |
| **Electron wrapper** | Native app feel | Huge binary, overkill |

## Decision

We chose static HTML because:

1. **Universal Viewing**: Double-click to open, no server needed.
2. **Easy Sharing**: Send files via email, USB, or any file transfer.
3. **Offline Support**: Works without internet connection.
4. **Hosting Simplicity**: Any static file host works (GitHub Pages, S3, etc.).
5. **Predictable**: What you build is what you get.

## Consequences

### Positive

- Presentations work anywhere a browser exists
- No server to configure, deploy, or maintain
- Can be viewed offline
- Easy to archive (just zip the output folder)
- Fast loading (no JavaScript framework overhead)

### Negative

- Real-time collaboration requires external tools
- No server-side analytics
- Some features require JavaScript (presenter sync)

## Related
- [QS-002: Customizable Styles](../quality-scenarios/QS-002_flexibility_customizability.md)
- [QS-006: Cross-Platform Support](../quality-scenarios/QS-006_operability_portability.md)
- [QS-008: Git and AI Friendly](../quality-scenarios/QS-008_suitable_interoperability.md)

Date: 2025-11-07
