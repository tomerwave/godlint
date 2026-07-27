# Security policy

## Supported versions

Godlint is pre-alpha and no release is currently supported. Security fixes will be
published for the latest released version once releases begin.

## Reporting a vulnerability

Please do not open a public issue for a suspected vulnerability.

Use GitHub’s private vulnerability reporting for this repository when it is enabled.
If private reporting is unavailable, contact the maintainers through the email address
listed in the repository profile and include `Godlint security report` in the subject.

Please provide a clear description, reproduction steps or proof of concept, impact,
and any suggested mitigation. We will acknowledge receipt within seven days and share
status updates as the investigation progresses.

Godlint’s security boundaries include untrusted source trees, repository
configuration, external-tool execution, caches, and any future plugin mechanism.
Source code must never be uploaded by default.
