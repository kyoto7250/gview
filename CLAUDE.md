# claude.md

## ✅ Snapshot Testing Policy (`cargo insta`)

- Use snapshot tests **only** for TUI rendering output or full standard output/error.
- Do **not** use snapshot tests for struct method logic or state verification.
- Snapshot tests must cover the **entire output**, not just partial sections.
- **Do not modify behavior** just for the purpose of making it snapshot-testable.
- Avoid snapshot tests that only verify printed output.
- Do **not** write tests that only check constant values or trivial behaviors.

## ✅ Unit Testing Policy

- Use standard unit tests for logic inside functions and methods.
- Be mindful to **cover all conditional branches** in logic.
- **Add tests within the same file** as the code under test, not in a separate test module or file.
- Avoid side effects in tests—**use mocks where needed** to isolate behavior.

## ✅ Security and Quality

- **Do not intentionally introduce security issues.**
- Follow **security best practices** at all times.
- Avoid using `panic`; **do not terminate the program abruptly**.
- Implement **explicit error handling** for all fallible operations.

## ✅ General Guidelines

- Always run `cargo fmt` and `cargo clippy` before completing any task.
- If a task fails after multiple attempts, **summarize the failure reason and ask for instructions**.
- **Do not alter existing behavior** unless explicitly instructed.
- Follow **Rust idioms and best practices**.
- Consider edge cases, such as **empty input, missing files, or permission errors**.

## ✅ Coding Style

- Write all comments in **English**.
- Avoid obvious comments; **keep one concise comment per method** that conveys intent.
- Prioritize **simplicity, speed, and predictability** over cleverness or abstraction.