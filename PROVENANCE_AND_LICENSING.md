# Provenance and licensing boundary

This reviewer package contains mixed technical surfaces: formal descriptions,
Rust implementation code, tests, benchmark material, and integration material.
The available record does not establish that every file was independently
authored by one person or that every component may be relicensed under a single
repository-wide license.

The package is therefore published as a curated review surface, not as a blanket
license grant. Upstream-derived or borrowed material retains its applicable
license and attribution requirements. Original work may receive an explicit
license only after file-level provenance and redistribution rights are confirmed.

This distinction does not invalidate the technical evidence. It keeps source
rights separate from proof status, SMT verification, silicon testing, and
performance measurement.

## Metadata consistency (RC-06)

The package states its licence in two places: the `LICENSE` file and `Cargo.toml`. Both must name
the same licence, and the copyright holder in `LICENSE` must be the author declared in
`Cargo.toml`. At `review-v0.1.26` both say Apache-2.0 and both name Sean Christopher Southwick.

Nothing enforces this agreement. `validate_release.sh` does not read `Cargo.toml` and does not
compare it with `LICENSE`, so the two can be changed apart while the gate still passes. The
contract records this as RC-06 at Implemented: None, and the adversarial plan has no case for it
because there is no behaviour to test yet. Until a check exists, agreement between the two files
is a current fact about the tree, not a property the release path maintains.
