# Contributing to Superset Portable

First off, thanks for taking the time to contribute! 🎉

## How to Contribute

### Reporting Bugs
1.  Check the [Issue Tracker](../../issues) to see if the bug has already been reported.
2.  Open a new issue, describing the bug, steps to reproduce, and expected behavior.

### Suggesting Enhancements
1.  Open a new issue with a clear title and description.
2.  Explain why this enhancement would be useful.

### Pull Requests
1.  Fork the repository.
2.  Create a new branch (`git checkout -b feature/amazing-feature`).
3.  Commit your changes (`git commit -m 'feat: Add amazing feature'`).
4.  Push to the branch (`git push origin feature/amazing-feature`).
5.  Open a Pull Request.

## Development Setup

### Prerequisites
- Windows 10/11
- Rust (latest stable)
- Python 3.8+ (for development scripts)

### Build
```powershell
# Build launcher
cargo build --release

# Create release package
python build_release.py
```

## Style Guide
- **Rust**: Follow standard Rust formatting (`cargo fmt`).
- **Python**: PEP 8.
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat: ...`, `fix: ...`).

Thank you for your contributions!
