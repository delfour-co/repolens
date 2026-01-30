//! Template file creation

use crate::error::{ActionError, RepoLensError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Create a file from a template
pub fn create_file_from_template(
    path: &str,
    template_name: &str,
    variables: &HashMap<String, String>,
) -> Result<(), RepoLensError> {
    let template_content = get_template(template_name)?;

    // Replace variables in template
    let mut content = template_content;
    for (key, value) in variables {
        content = content.replace(&format!("{{{{ {} }}}}", key), value);
        content = content.replace(&format!("{{{{{}}}}}", key), value);
    }

    // Write the file
    let file_path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = file_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                RepoLensError::Action(ActionError::DirectoryCreate {
                    path: parent.display().to_string(),
                    source: e,
                })
            })?;
        }
    }

    fs::write(file_path, content).map_err(|e| {
        RepoLensError::Action(ActionError::FileWrite {
            path: path.to_string(),
            source: e,
        })
    })?;

    Ok(())
}

/// Get template content by name
fn get_template(name: &str) -> Result<String, RepoLensError> {
    match name {
        "LICENSE/MIT" => Ok(MIT_LICENSE.to_string()),
        "LICENSE/Apache-2.0" => Ok(APACHE_LICENSE.to_string()),
        "LICENSE/GPL-3.0" => Ok(GPL_LICENSE_HEADER.to_string()),
        "CONTRIBUTING.md" => Ok(CONTRIBUTING_TEMPLATE.to_string()),
        "CODE_OF_CONDUCT.md" => Ok(CODE_OF_CONDUCT_TEMPLATE.to_string()),
        "SECURITY.md" => Ok(SECURITY_TEMPLATE.to_string()),
        "ISSUE_TEMPLATE/bug_report.md" => Ok(BUG_REPORT_TEMPLATE.to_string()),
        "ISSUE_TEMPLATE/feature_request.md" => Ok(FEATURE_REQUEST_TEMPLATE.to_string()),
        "PULL_REQUEST_TEMPLATE/pull_request_template.md" => Ok(PULL_REQUEST_TEMPLATE.to_string()),
        _ => Err(RepoLensError::Action(ActionError::UnknownTemplate {
            name: name.to_string(),
        })),
    }
}

const MIT_LICENSE: &str = r#"MIT License

Copyright (c) {{ year }} {{ author }}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;

const APACHE_LICENSE: &str = r#"                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   Copyright {{ year }} {{ author }}

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
"#;

const GPL_LICENSE_HEADER: &str = r#"Copyright (C) {{ year }} {{ author }}

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
"#;

const CONTRIBUTING_TEMPLATE: &str = r#"# Contributing

Thank you for your interest in contributing to this project!

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Use a clear and descriptive title
- Provide steps to reproduce the issue
- Include relevant logs or screenshots

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests to ensure everything works
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style

- Follow the existing code style
- Write meaningful commit messages
- Add tests for new features
- Update documentation as needed

### Development Setup

```bash
# Clone the repository
git clone <repository-url>
cd <project>

# Install dependencies
# Add project-specific setup instructions here
```

## Questions?

Feel free to open an issue for any questions or concerns.
"#;

const CODE_OF_CONDUCT_TEMPLATE: &str = r#"# Code of Conduct

## Our Pledge

We as members, contributors, and leaders pledge to make participation in our
community a harassment-free experience for everyone, regardless of age, body
size, visible or invisible disability, ethnicity, sex characteristics, gender
identity and expression, level of experience, education, socio-economic status,
nationality, personal appearance, race, caste, color, religion, or sexual
identity and orientation.

## Our Standards

Examples of behavior that contributes to a positive environment:

* Using welcoming and inclusive language
* Being respectful of differing viewpoints and experiences
* Gracefully accepting constructive criticism
* Focusing on what is best for the community
* Showing empathy towards other community members

Examples of unacceptable behavior:

* The use of sexualized language or imagery, and sexual attention or advances
* Trolling, insulting or derogatory comments, and personal or political attacks
* Public or private harassment
* Publishing others' private information without explicit permission
* Other conduct which could reasonably be considered inappropriate

## Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be
reported to the project maintainers. All complaints will be reviewed and
investigated promptly and fairly.

## Attribution

This Code of Conduct is adapted from the [Contributor Covenant](https://www.contributor-covenant.org),
version 2.1.
"#;

const SECURITY_TEMPLATE: &str = r#"# Security Policy

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please follow these steps:

1. **Do not** open a public issue
2. Email us at [security@example.com] with details
3. Include steps to reproduce the vulnerability
4. Allow time for us to investigate and respond

## What to Include

- Type of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution**: Depends on severity and complexity

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| < 1.0   | :x:                |

## Security Best Practices

When using this project:

- Keep dependencies up to date
- Use environment variables for secrets
- Follow the principle of least privilege
- Enable security features where available

Thank you for helping keep this project secure!
"#;

const BUG_REPORT_TEMPLATE: &str = r#"---
name: Bug Report
about: Create a report to help us improve
title: ''
labels: bug
assignees: ''
---

## Description

A clear and concise description of what the bug is.

## Steps to Reproduce

1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

## Expected Behavior

A clear and concise description of what you expected to happen.

## Actual Behavior

A clear and concise description of what actually happened.

## Environment

- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
- Version: [e.g. 0.1.0]
- Rust version: [e.g. 1.70.0]

## Additional Context

Add any other context about the problem here.

## Screenshots

If applicable, add screenshots to help explain your problem.
"#;

const FEATURE_REQUEST_TEMPLATE: &str = r#"---
name: Feature Request
about: Suggest an idea for this project
title: ''
labels: enhancement
assignees: ''
---

## Problem Statement

A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

## Proposed Solution

A clear and concise description of what you want to happen.

## Alternatives Considered

A clear and concise description of any alternative solutions or features you've considered.

## Use Cases

Describe the use cases for this feature:

1. Use case 1
2. Use case 2
3. Use case 3

## Additional Context

Add any other context, mockups, or examples about the feature request here.

## Implementation Notes (Optional)

If you have ideas about how this could be implemented, please share them here.
"#;

const PULL_REQUEST_TEMPLATE: &str = r#"## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Checklist

- [ ] Code compiles without errors
- [ ] Tests pass
- [ ] Code follows project style guidelines
- [ ] Documentation updated if needed
"#;
