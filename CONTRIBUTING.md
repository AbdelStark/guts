# Contributing to Guts

Thank you for your interest in contributing to Guts! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up the development environment**:
   ```bash
   # Install Rust (1.85+)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Clone and build
   git clone https://github.com/YOUR_USERNAME/guts.git
   cd guts
   cargo build --workspace
   ```

## Development Workflow

### Branch Naming

- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions/changes

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(repo): add branch creation support
fix(p2p): resolve connection timeout issue
docs(readme): update installation instructions
```

### Code Style

- Run `cargo fmt --all` before committing
- Run `cargo clippy --workspace --all-targets -- -D warnings`
- All public items must have documentation
- Write tests for new functionality

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p guts-core

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Pull Request Process

1. **Create a branch** from `main`
2. **Make your changes** with clear commits
3. **Update documentation** if needed
4. **Add tests** for new functionality
5. **Run all checks**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
6. **Open a Pull Request** with a clear description

### PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New code has tests
- [ ] Documentation is updated
- [ ] Commit messages follow convention
- [ ] PR description explains the changes

## Architecture

### Crate Organization

| Crate | Purpose |
|-------|---------|
| `guts-core` | Core types, traits, errors |
| `guts-identity` | Cryptographic identity |
| `guts-storage` | Content-addressed storage |
| `guts-repo` | Git operations |
| `guts-protocol` | Network protocol |
| `guts-consensus` | BFT consensus |
| `guts-p2p` | P2P networking |
| `guts-api` | HTTP/gRPC API |
| `guts-node` | Node binary |
| `guts-cli` | CLI binary |

### Design Principles

1. **Modularity**: Each crate has a single responsibility
2. **Safety**: No unsafe code unless absolutely necessary
3. **Performance**: Profile before optimizing
4. **Documentation**: All public APIs documented

## Reporting Issues

### Bug Reports

Include:
- Guts version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs

### Feature Requests

Include:
- Clear description of the feature
- Use case / motivation
- Possible implementation approach

## Getting Help

- Open a GitHub issue for bugs/features
- Check existing issues before creating new ones
- Read the [documentation](docs/)

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
