# Security policy

## Supported versions

Opsis hasn't had a release yet. Security fixes go to the `main` branch.

## Reporting a vulnerability

Please report vulnerabilities privately, through GitHub's **Report a vulnerability** form on the repository's **Security** tab (github.com/bismitpanda/opsis/security) or by email to [contact@bismitpanda.com](mailto:contact@bismitpanda.com). Don't open a public issue for them.

Include what you found, how to reproduce it, and what it lets someone do. Reports are answered as soon as they can be, and confirmed issues are fixed on `main`; say if you'd like to be credited.

## What's in scope

Opsis is a desktop app that works offline. The parts that are most worth a look:

- **The Local API** (off by default): plain HTTP on a Unix socket in the app's data folder (mode 0600), or a named pipe on Windows. Anything that lets another user reach it, or lets a request do more than the API documents, is in scope.
- **The editor's file server:** HTTP on 127.0.0.1 behind a token made at each launch. Reading files outside the open project, the app's public files and the font library is in scope.
- **Saved sign-ins:** sealed with AES-256-GCM under a key kept in the OS keyring. Ways to read them without that key are in scope.
- **Recording and preview browsers:** pages they load are untrusted. Anything that lets a page reach the app's commands, files or other pages is in scope.
- **Downloads:** the engine and fonts download only when you ask, and engine downloads are checked against pinned sha256 sums and sizes.

Out of scope: problems in the websites you record, and issues that need someone to already control your user account.
