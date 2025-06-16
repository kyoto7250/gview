# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

These instructions must always be followed.

## ✅ Development Commands

### Build and Test
- `cargo build` - Compile the project
- `cargo run` - Run the application (optionally with commit ID: `cargo run abc123`)
- `cargo test` - Run all unit and snapshot tests
- `cargo check` - Fast type checking without compilation

### Code Quality
- `cargo fmt` - Format code (required before submission)
- `cargo clippy` - Lint code with strict warnings (`-D warnings` in CI)
- `cargo clippy -- -D warnings` - Match CI linting standards

### Snapshot Testing (cargo insta)
- `cargo insta test` - Run snapshot tests
- `cargo insta accept` - Accept snapshot changes (never use `cargo insta review`)
- `cargo test` - Run both unit and snapshot tests

## ✅ Architecture Overview

### Application Structure
gview is a TUI application with a **message-passing architecture** and **component-based design**:

- **Main App Loop** (`src/app.rs`): Event-driven architecture with 6 core components
- **Repository Layer** (`src/repository.rs`): Git operations using `git2` crate
- **Component System** (`src/components/`): Modular UI components with focus management

### Key Components
1. **Filter Panel** - File search with fuzzy matching
2. **Filer Panel** - File list navigation
3. **Commit Viewer** - Current commit display
4. **Content Viewer** - File content with blame/line numbers
5. **Commit Modal** - Commit selection interface
6. **Help Modal** - Interactive help system

### Message Passing System
Components communicate via structured messages (`Message` enum):
- `MultipleTimes` - Repeated operations
- `Once` - Single-shot operations
- `NoAction` - No operation needed
- `Error` - Error states

### State Management
- **Shared State**: `Arc<Mutex<RepositoryInfo>>` for thread-safe Git repository access
- **Focus Management**: Tab navigation with visual feedback
- **Event Handling**: Centralized keyboard event processing

##  ✅ Work Order
Your work order is as follows:
1. Analyze the code and create an implementation plan.
2. Modify the code while writing tests.
3. Ensure the build passes successfully.
4. Once the build passes, consider whether refactoring is possible.
5. After completing the refactoring, run formatting and linting.
6. Update the README and documentation.
7. Provide a concise explanation of the changes made.


## ✅ Snapshot Testing Policy (`cargo insta`)
- Use snapshot tests **only** for full-screen TUI rendering or complete standard output/error.
- Do **not** apply snapshot tests to struct methods or logic-level units.
- Snapshot tests must capture **entire outputs**, not partial sections.
- Do **not** alter runtime behavior just to support snapshot testing.
- Do **not** write snapshot tests that merely check printed input/output or constant values
- Do **not** use `cargo insta review` because this command is interactive, so you should use `cargo insta accept` or `cargo test`.
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
