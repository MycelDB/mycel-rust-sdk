# Third-party notices and dependency license inventory

This repository is licensed under the Apache License, Version 2.0. The project also uses third-party open source dependencies. This document records the current license inventory and the release expectations for preserving upstream notices.

Tracked issue: MycelDB/mycel-rust-sdk#2

## Inventory

The generated inventory is committed at:

```text
docs/legal/dependency-license-inventory.tsv
```

It includes package name, resolved version, dependency ecosystem, scope, declared or detected license, detected license files when available, and upstream source/resolution metadata.

Current detected license summary:

- `(MIT OR Apache-2.0) AND Unicode-3.0`: 1
- `Apache-2.0`: 5
- `Apache-2.0 / MIT`: 1
- `Apache-2.0 AND ISC`: 1
- `Apache-2.0 OR ISC OR MIT`: 3
- `Apache-2.0 OR MIT`: 11
- `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`: 3
- `BSD-2-Clause OR Apache-2.0 OR MIT`: 2
- `BSD-3-Clause`: 1
- `ISC`: 2
- `MIT`: 37
- `MIT AND BSD-3-Clause`: 1
- `MIT OR Apache-2.0`: 87
- `MIT OR Apache-2.0 OR LGPL-2.1-or-later`: 1
- `Unlicense OR MIT`: 2

## Covered ecosystems

- Rust crate dependencies from `Cargo.lock` via `cargo metadata --locked`.

## Notice handling

- Keep upstream copyright, license, and NOTICE files intact in source checkouts and vendored/generated material.
- Do not remove license headers from generated code or copied third-party source.
- Binary, Docker, SDK, and desktop distributions should include this repository's `LICENSE` and link to or include this third-party notice document.
- If a dependency declares a license outside the existing allowlist, review it before release and update this document with any required attribution or redistribution notes.
- This inventory is a release-hygiene aid and is not legal advice.

## Regeneration

Run `python3 scripts/generate-license-inventory.py` from the repository root. The script uses `cargo metadata --locked` and the committed `Cargo.lock`; it does not require external Python packages.

After regenerating, review changes to `docs/legal/dependency-license-inventory.tsv`, update the summary above if needed, and run the normal repository validation before opening a release PR.

## Distribution guidance

Crate/source releases should keep this notice document and inventory in the repository and release source archives.
