# 001. Rust as Implementation Language

In the context of **building a CLI tool for generating presentations**,
facing **the need for easy distribution, fast performance, and reliable execution**,
we decided for **Rust**
to achieve **single-binary distribution and excellent performance**,
accepting **steeper learning curve and longer compile times**.

## Context

slides-rs needs to be distributed to users who may not have development environments set up. The tool should:

- Be easy to install (ideally a single download)
- Run fast enough for interactive development (watch mode)
- Work reliably across different operating systems
- Be maintainable long-term

Alternatives considered:

| Option | Pros | Cons |
|--------|------|------|
| **Rust** | Single binary, fast, safe, good ecosystem | Learning curve, compile times |
| **Go** | Single binary, fast compilation, easy to learn | Less expressive, GC pauses |
| **Node.js** | Large ecosystem, fast development | Requires Node runtime, dependency hell |
| **Python** | Rapid development, familiar | Requires Python runtime, packaging complexity |

## Decision

We chose Rust because:

1. **Single Binary Distribution**: Users download one file and it works. No runtime dependencies.
2. **Performance**: Template rendering and file I/O are fast, important for watch mode.
3. **Memory Safety**: No garbage collector, predictable performance.
4. **Ecosystem**: MiniJinja, clap, and other high-quality crates available.
5. **Cross-compilation**: Easy to build for Linux, macOS, and Windows.

## Consequences

### Positive

- Installation is trivial (download binary, run)
- Build times are consistently fast (<1s for typical projects)
- No "it works on my machine" issues due to runtime version mismatches
- Strong type system catches many errors at compile time

### Negative

- Contributors need to learn Rust
- Compile times during development are longer than interpreted languages
- Binary size is larger than Go (~15MB vs ~10MB)

## Related
- [QS-005: Single Binary Installation](../quality-scenarios/QS-005_operability_installability.md)
- [QS-006: Cross-Platform Support](../quality-scenarios/QS-006_operability_portability.md)
- [QS-007: Fast Builds](../quality-scenarios/QS-007_efficiency_performance.md)

Date: 2025-11-07
