# Contributing to mintcarbon-backend

First off, thank you for considering contributing to mintcarbon! It's people like you that make mintcarbon such a great platform.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. (Link to your Code of Conduct here, if applicable).

## How Can I Contribute?

### Reporting Bugs

- **Check if the bug has already been reported** by searching on GitHub under Issues.
- If you can't find an open issue addressing the problem, **open a new one**.
- Use a clear and descriptive title.
- Describe the exact steps which reproduce the problem in as many details as possible.

### Suggesting Enhancements

- **Check if the enhancement has already been suggested**.
- Open a new issue and clearly describe the suggested enhancement and why it would be useful.

### Pull Requests

- Fork the repository and create your branch from `main`.
- If you've added code that should be tested, add tests.
- If you've changed APIs, update the documentation.
- Ensure the test suite passes.
- Make sure your code follows the existing style (run `cargo fmt` and `cargo clippy`).

## Development Process

1. **Setup**: Follow the instructions in the [README.md](README.md) to set up your development environment.
2. **Branching**: Use descriptive branch names (e.g., `feature/add-kyc-provider`, `fix/login-error`).
3. **Commits**: Write clear, concise commit messages.
4. **Testing**: Run `cargo test` before submitting your PR.
5. **Linting**: Run `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`.

## Style Guide

- We use standard Rust formatting. Please run `cargo fmt` before committing.
- We aim for high test coverage. Please include tests for new functionality.
- Use descriptive variable and function names.
- Document public APIs using doc comments (`///`).

## Questions?

Feel free to open an issue with the "question" label.
