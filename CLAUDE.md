# claude.md

## ✅ Snapshot Testing Policy (`cargo insta`)

- Use snapshot tests **only** for full-screen TUI rendering or complete standard output/error.
- Do **not** apply snapshot tests to struct methods or logic-level units.
- Snapshot tests must capture **entire outputs**, not partial sections.
- Do **not** alter runtime behavior just to support snapshot testing.
- Do **not** write snapshot tests that merely check printed input/output or constant values.
- Snapshot tests are meant to detect visual or full-output regressions, not internal logic.
- Remove any snapshot tests that violate these rules.

## ✅ Unit Testing Policy

- Test struct methods and logic using **traditional unit tests**, not snapshots.
- Ensure **all condition branches are covered**.
- Add tests directly into the **same file** as the tested code.
- Avoid test-side effects—**mock external dependencies** if necessary.
- Do **not rely on printed values** or superficial assertions.

## ✅ Security and Safety

- **Never embed sensitive data** like `API_KEY`, file paths, personal user data, or internal directories in code or tests.
- Do **not delete data** without explicit user confirmation.
- **Do not use `panic!`** for error handling—handle all errors gracefully.
- Follow **security best practices** at all times.
- Never simplify code at the expense of introducing security vulnerabilities.

## ✅ General Guidelines

- Always run `cargo fmt` and `cargo clippy` before submitting code.
- If a task fails repeatedly, **summarize what cannot be done and request further instructions**.
- Unless explicitly instructed, **do not change existing behavior**.
- Follow **Rust idioms and best practices**.
- Account for **edge cases** such as empty inputs, missing files, permission errors, etc.

## ✅ Coding Style

- Write all code comments in **English**.
- Avoid obvious or redundant comments.
- Prefer **one concise comment per method** to clarify intent when necessary.
- Prioritize **simplicity, speed, and predictability** over clever abstractions.

## ✅ Testing Philosophy

- Snapshot tests are for **detecting full UI regressions**, not partial behavior.
- Logic and flow must be tested using **unit or integration tests**, not snapshots.
- **All functionality must be test-covered** without modifying production behavior.
