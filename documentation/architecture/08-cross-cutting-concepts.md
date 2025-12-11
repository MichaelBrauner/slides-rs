# 08. Cross-Cutting Concepts

## 8.1 Error Handling

**Approach:** Centralized error types with thiserror

**Implementation:**
- All error types defined in `src/error.rs` as a single `Error` enum
- User-friendly error messages via `#[error("...")]` attributes
- IO errors include file paths for context
- Warnings logged via `log::warn!` (visible at RUST_LOG=warn)
- Errors propagate with `?` operator and display at CLI entry point

**Error Categories:**

| Category | Examples | Behavior |
|----------|----------|----------|
| IO | FileRead, FileWrite, CreateDir | Include path in message |
| Config | DecksNotFound, DeckNotFound | Point to missing resource |
| Template | TemplateNotFound, TemplateRender | Include template name |
| Project | DirNotEmpty, DirExists | Clear user guidance |
| Export | PdfGeneration, PdfEncryption | Technical details |
| Browser | Browser | Include install hints |
| Import | InvalidPptx, ZipError | File format issues |
| Watch | WatcherInit, WatchPath | File system notifications |

**Rationale:**
- Consistent error presentation to users
- Clear distinction between errors (stop execution) and warnings (continue)
- Easy to add new error types as the application grows
- Automatic `Display` implementation via thiserror

## 8.2 Logging

**Approach:** Standard Rust logging with env_logger

- Default log level: `warn`
- Configurable via `RUST_LOG` environment variable
- Warnings for non-fatal issues (missing optional templates, glob errors)
- Errors displayed via stderr with exit code 1

## 8.3 Configuration

**Approach:** Convention over configuration

- Sensible defaults for all options
- YAML for human-readable configuration
- Environment-based overrides where applicable

---

*This document follows the [ARC42 architecture documentation template](https://docs.arc42.org/)*
