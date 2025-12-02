# Tests

Test suite for slides-rs.

## Structure

```
tests/
├── common/           # Shared utilities
├── functional/       # End-to-End CLI tests
├── integration/      # Module integration tests
└── fixtures/         # Test data
```

## Running Tests

```bash
# All tests
cargo test

# Specific test suite
cargo test --test functional
cargo test --test integration
```

---

*Documentation will be expanded as tests are written and reviewed.*
