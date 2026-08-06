# Security Policy

## Supported Versions

Banna is preparing its first public release. Until stable releases exist, only
the latest release and the current `develop` branch are considered for security
fixes. After `1.0.0`, this table will identify supported release lines
explicitly.

## Reporting A Vulnerability

Do not report a suspected vulnerability in a public issue, discussion, pull
request, or generated-project example. Use GitHub's private security-advisory
reporting for the repository:

<https://github.com/furkancemalcaliskan/banna/security/advisories/new>

Include affected versions, impact, reproduction steps or a minimal proof of
concept, and any known mitigation. Remove unrelated credentials and personal
data. You should receive an acknowledgement within seven days; validation,
remediation, release, and disclosure timing will be coordinated privately.

## Security Scope

Banna writes files into a user-selected project and may invoke local project
tooling, package managers, and EF Core migrations. Only use it with trusted
source trees and dependencies. Start with `--dry-run`, review generated diffs,
and never run migrations against a database you do not intend to modify.

Reports about path traversal, writes outside the selected project, dry-run side
effects, command or prompt injection, secret exposure, unsafe archive contents,
dependency compromise, or malicious embedded templates are in scope.

Security reports are accepted in good faith. Please avoid privacy violations,
service disruption, destructive testing, and accessing data that is not yours.

