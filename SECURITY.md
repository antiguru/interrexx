# Security

## Reporting a vulnerability

Use **Report a vulnerability** on the repository’s [Security Advisories
page](https://github.com/antiguru/interrexx/security/advisories) if GitHub
offers it. Otherwise, open a public issue asking only for a private security
contact. Keep the vulnerability details, logs, and attachments in the private
channel. [GitHub’s reporting guide][report-privately] describes this process.

In the private report, include the exact interrexx version or commit, the
operating system, the affected component, and the observed impact. Continue
the discussion in the private channel.

## Running Rexx programs

Interrexx provides no sandbox. A Rexx program runs with the authority of its
interpreter process, including access the operating system grants to commands,
files, and sockets. It can load native libraries through `RXFUNCADD` and
`::REQUIRES ... LIBRARY`. In an embedded interpreter, that authority is the
host process’s.

The [Rexx security manager](spec/src/environment/security.md) provides
checkpoints for selected operations. Use operating-system isolation as the
security boundary for untrusted programs and restrict the isolated process’s
filesystem, network, and resource access. [`REXX_HEAP_LIMIT`][heap-limit]
accounts for selected Rexx object allocations. Apply process-level limits
when you need to contain total memory use.

## What to report

Report suspected memory-safety defects in source execution, compiled-image
loading, and the [C API](spec/src/native/index.md). Also report failures of
access controls on the [`rxapi` service](docs/design/rxapi.md), such as an
unauthorized user gaining access to another user’s API service.

Interrexx has no published support window or security backport policy. If
you already know whether the issue affects the current `main` branch, include
that information with the affected version.

[report-privately]: https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/report-privately
[heap-limit]: docs/developing.md#running-oorexx-and-interrexx
