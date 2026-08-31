# Security Policy

## Reporting a vulnerability

Please report suspected security vulnerabilities privately through GitHub Security Advisories / private vulnerability reporting for this repository.

Use the repository's **Security** tab and choose **Report a vulnerability**. Do not open a public issue with vulnerability details.

If private vulnerability reporting is not visible, open a public issue that says only that you need a private security reporting channel. Do not include exploit details, affected secrets, proof-of-concept code, or other sensitive information in the public issue.

## What to include

When possible, include:

- The affected SDK crate version, commit, or release tag.
- The affected crate, module, function, helper, generated service client, or configuration path.
- A description of the vulnerability and expected impact.
- Reproduction steps or proof-of-concept details.
- Any known mitigations or workarounds.

## Scope

Security reports may apply to this Rust SDK when behavior could expose data, weaken authentication, mishandle credentials, bypass TLS expectations, retry unsafe operations, leak sensitive metadata, misrepresent API authorization semantics, or introduce unsafe async/runtime behavior.

Vulnerabilities in the daemon API contract, daemon implementation, or other SDKs may be redirected to the corresponding MycelDB repository after initial triage.
