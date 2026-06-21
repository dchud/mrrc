# Security Policy

## Supported Versions

mrrc is pre-1.0. Only the latest released version receives security fixes;
older releases are not patched.

| Version | Supported |
|---------|-----------|
| latest release | yes |
| earlier releases | no |

## Reporting a Vulnerability

Please report vulnerabilities privately through GitHub's security advisory
form: <https://github.com/dchud/mrrc/security/advisories/new>. Do **not**
open a public issue for security problems.

If you cannot use GitHub, email the maintainer at
<daniel.chudnov@gmail.com> with "mrrc security" in the subject line.

Include in your report:

- A description of the vulnerability and its impact
- A minimal reproducer (sample MARC input if the issue is parser-related)
- The mrrc version and platform affected

## What to Expect

- Acknowledgement of your report within 7 days
- An assessment and remediation plan, coordinated with you, before any
  public disclosure
- Credit in the release notes for the fix, unless you prefer otherwise

## Scope Notes

mrrc parses untrusted binary input (ISO 2709, MARCXML, JSON, and other
formats), so memory safety and panic behavior on malformed input are in
scope — the parsers are fuzzed continuously, and crashes, hangs, or
out-of-memory conditions triggered by crafted records are treated as
security reports.
