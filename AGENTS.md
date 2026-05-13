# Crusty Project Guidelines

Instructions for AI agents working on this codebase.

## Code Style

- Prefer smaller functions. When a function exceeds 30 lines, consider splitting it into multiple functions with meaningful names.
- Prefer smaller files. When a file exceeds 300 lines, consider splitting it into multiple files with meaningful names that are easy to understand.
- Prefer functional iterator pipelines (`map`, `filter`, `sum`, `collect`) for transforming and aggregating collections when it keeps the code readable.
- Prefer imperative style (`for` loops, explicit mutation) when accumulating into mutable state or when a functional version would require an awkward `fold`.
- In the domain layer, prefer newtypes (e.g., `struct PartyId(String)`) over raw primitives like `String` or `u64` to provide abstraction and type safety. Plain primitives are acceptable for genuinely opaque text such as descriptions or error messages.

## Error Handling

- Use `thiserror` for domain errors. Do not allow implementation-specific errors in the domain.
- Use `anyhow` at the top level, for errors that we only report but never match.
- Be careful with swallowing errors without surfacing/bubbling them up. Swallowing can be acceptable during error handling in parsing to avoid nested errors.
- For errors that should never happen (programming errors), it is okay to use functions that panic (e.g., `expect`). Provide context. They act as invariants and keep return types simpler.
