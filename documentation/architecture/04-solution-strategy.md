# 04. Solution Strategy

## 4.1 Technology Decisions

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Language** | Rust | Performance, memory safety, single binary distribution without runtime dependencies |
| **Templates** | MiniJinja | Jinja2-compatible syntax familiar to developers, pure Rust implementation without Python dependency |
| **CLI Framework** | clap | De-facto standard for Rust CLIs, automatic help generation, shell completion |
| **File Watching** | notify | Cross-platform file system notifications, performant incremental rebuilds |
| **Browser Automation** | headless_chrome | DevTools Protocol access for pixel-perfect screenshots and PDF generation |
| **YAML Parsing** | serde_yaml | Standard Rust serialization with strong typing |
| **Markdown Rendering** | pulldown-cmark | CommonMark-compliant, fast, pure Rust |

## 4.2 Architectural Approach

**Pipeline Architecture** - The build process flows linearly through distinct stages:

```
Input → Load Config → Render Templates → Copy Assets → Generate Thumbnails → Output
```

This approach was chosen for simplicity and debuggability - each stage can be understood and tested in isolation.

## 4.3 Approaches to Achieve Quality Goals

### 1. Developer Experience (Priority 1)

**Approach:**
- CLI-first design with intuitive commands
- Watch mode for instant feedback
- Clear error messages with file paths and line numbers
- Zero-configuration defaults with optional customization

**Implementation:**
- Use clap for self-documenting CLI
- File watcher with debouncing for smooth rebuilds
- MiniJinja error reporting with source context

### 2. Flexibility (Priority 2)

**Approach:**
- No restrictions on HTML/CSS/JS
- Template inheritance and composition
- Extensible via custom Stimulus controllers
- Asset pipeline doesn't transform (CSS preprocessors run externally)

**Implementation:**
- MiniJinja provides full Jinja2 feature set
- Direct file copying preserves developer intent
- Browser APIs accessible from generated HTML

### 3. Version Control (Priority 3)

**Approach:**
- All source files are plain text
- Generated output is reproducible
- Configuration as code (YAML)

**Implementation:**
- No binary formats in source
- Deterministic template rendering
- Thumbnails can be gitignored (regenerated on demand)

### 4. Performance (Priority 4)

**Approach:**
- Compiled binary (no interpreter overhead)
- Incremental rebuilds in watch mode
- Parallel processing where possible

**Implementation:**
- Rust's zero-cost abstractions
- Only re-render changed templates
- Concurrent thumbnail generation

### 5. Extensibility (Priority 5)

**Approach:**
- Template system allows custom layouts
- Asset pipeline supports any framework
- Speaker notes via HTML blocks

**Implementation:**
- MiniJinja macros and includes
- No opinionated CSS framework
- Minimal JavaScript core (Hotwire Stimulus)

## 4.4 Organizational Decisions

### Distribution Model

**Decision:** Single static binary distribution

**Rationale:**
- No runtime dependencies beyond Chrome (already required for presentations)
- Simple installation via package managers or direct download
- Cross-compilation for all major platforms (Linux, macOS, Windows)

### Release Strategy

**Decision:** Semantic versioning with GitHub Releases

**Rationale:**
- Clear upgrade paths
- Automated builds via CI/CD
- Binary artifacts attached to releases

---

*This document follows the [ARC42 architecture documentation template](https://docs.arc42.org/)*
