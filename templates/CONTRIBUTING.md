# Contributing to {{ project_name }}

Thank you for your interest in contributing to {{ project_name }}! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) to keep our community approachable and respectable.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

When creating a bug report, include:
- A clear and descriptive title
- Steps to reproduce the behavior
- Expected behavior
- Actual behavior
- Screenshots (if applicable)
- Environment details (OS, version, etc.)

### Suggesting Features

Feature requests are welcome! Please:
- Check if the feature has already been suggested
- Provide a clear description of the feature
- Explain why this feature would be useful
- Include examples of how it would work

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our coding standards
3. **Add tests** if applicable
4. **Update documentation** as needed
5. **Ensure tests pass** locally
6. **Submit a pull request**

#### Pull Request Guidelines

- Use a clear and descriptive title
- Reference any related issues
- Include a description of changes
- Keep changes focused and atomic

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/{{ project_name }}.git
cd {{ project_name }}

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/{{ project_name }}.git

# Install dependencies
# (Add project-specific setup instructions)

# Run tests
# (Add test commands)
```

### Coding Standards

- Follow existing code style
- Write meaningful commit messages
- Comment complex logic
- Keep functions focused and small

### Commit Messages

Use clear and meaningful commit messages:
- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and pull requests when relevant

Example:
```
feat: Add user authentication

- Implement JWT token generation
- Add login/logout endpoints
- Include password hashing

Closes #123
```

## Getting Help

- Open an issue for questions
- Join our community discussions
- Check the documentation

## Recognition

Contributors will be recognized in our README and release notes.

Thank you for contributing!
