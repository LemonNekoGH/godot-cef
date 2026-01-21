# Security Policy

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| > 1.0  | :white_check_mark: |
| < 1.0 | :x:               |

We recommend always using the latest release to ensure you have the most recent security patches.

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue in Godot CEF, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Security Advisories** (Preferred): Use [GitHub's private vulnerability reporting](https://github.com/dsh0416/godot-cef/security/advisories/new) to submit a confidential report.

2. **Email**: Contact the maintainer directly at security concerns related to this project.

### What to Include

When reporting a vulnerability, please include:

- A clear description of the vulnerability
- Steps to reproduce the issue
- Affected versions
- Potential impact of the vulnerability
- Any possible mitigations you've identified

### Response Timeline

- **Initial Response**: We aim to acknowledge receipt of your report within 48 hours.
- **Status Update**: We will provide a more detailed response within 7 days, including our assessment and planned timeline.
- **Resolution**: We strive to resolve critical vulnerabilities within 30 days, depending on complexity.

### Disclosure Policy

- We follow coordinated disclosure practices.
- We will work with you to understand and resolve the issue.
- Once a fix is available, we will publicly acknowledge your contribution (unless you prefer to remain anonymous).

## Security Considerations

### Chromium Embedded Framework (CEF)

Godot CEF embeds Chromium via CEF. Security of the underlying browser engine is dependent on the CEF version used:

- We regularly update to newer CEF versions to incorporate upstream Chromium security fixes.
- Check your CEF version against [CEF releases](https://cef-builds.spotifycdn.com/index.html) for known vulnerabilities.

### Web Content Security

When using Godot CEF in your applications:

- **Trusted Content Only**: Only load web content from sources you trust.
- **JavaScript Execution**: Be aware that `eval()` executes arbitrary JavaScript. Validate any dynamic content.
- **IPC Messages**: Sanitize and validate all messages received via `ipc_message` signals before processing.
- **Local File Access**: The `res://` protocol provides access to your Godot project files. Be cautious when loading user-provided URLs.

### Recommended Practices

1. **Keep Updated**: Always use the latest version of Godot CEF.
2. **Content Security Policy**: When loading your own HTML, implement appropriate CSP headers.
3. **Input Validation**: Validate all data passed between GDScript and JavaScript.
4. **Minimize Privileges**: Only enable features you need (e.g., only enable DevTools in development).

## Security Updates

Security updates will be announced through:

- [GitHub Releases](https://github.com/dsh0416/godot-cef/releases)
- [GitHub Security Advisories](https://github.com/dsh0416/godot-cef/security/advisories)

## Acknowledgments

We appreciate the security research community and thank everyone who has responsibly disclosed vulnerabilities to help keep Godot CEF secure.
