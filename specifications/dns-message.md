# DNS message parsing boundary

## Intent and users

A draft, stateless wire-message parser contract for applications consuming
untrusted DNS bytes. This is a contribution-flow specimen, not a resolver or an
implemented parser. Human DNS and security review is still needed.

## Standards and existing behavior

- [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035.html), sections 2.3.4 and
  4.1.1–4.1.4: message fields, names and compression.
- [RFC 3597](https://www.rfc-editor.org/rfc/rfc3597.html), section 3: opaque unknown RDATA.
- [RFC 9267](https://www.rfc-editor.org/rfc/rfc9267.html), sections 2, 3, 5 and 6:
  pointer, length and count validation failure patterns. This is informational
  guidance, not an additional Internet standard.
- [Hickory DNS](https://github.com/hickory-dns/hickory-dns) is an existing Rust
  implementation to inspect before choosing extension or replacement. No
  compatibility comparison or implementation evaluation has run for this draft.

## Scope and non-goals

Parse a complete in-memory DNS message. Return structured fields or a bounded
error. Exclude transport framing, network I/O, recursive resolution, caching,
DNSSEC verification and interpretation of arbitrary RDATA types. Trailing bytes
are an open compatibility decision, not silently accepted behavior.

## Requirements

### DNS-001

The parser MUST reject messages shorter than the 12-byte header before reading
fields. Test every shorter length and a structurally valid minimal message.

### DNS-002

The parser MUST bounds-check section counts, field offsets and RDLENGTH before
reading or allocating. Truncated or inconsistent fields return an error, not a
partial successful message. Exercise truncation at each boundary and extreme counts.

### DNS-003

Name decoding MUST enforce label and expanded-name limits, reject out-of-range
compression targets and terminate on pointer cycles. Test legal compressed names,
self/two-node cycles, missing terminators and boundary lengths. Reserved label
encodings require an explicit unsupported-encoding error in this initial scope.

### DNS-004

Unknown RR types MUST retain their RDATA bytes unchanged as opaque data. Compare
input slices byte-for-byte for empty and nonempty unknown records; do not interpret
pointer-like bytes in opaque RDATA as compression instructions.

### DNS-005

The parser MUST enforce caller-supplied input, record and name-decoding work
budgets before exceeding them, returning a distinct resource-limit error. Defaults
and whether records include questions remain open decisions. Exercise each budget
at the boundary and one unit above it; no wall-clock performance claim is made.

### DNS-006

Parsing MUST leave the input unchanged and perform no network, disk or process
operations. Repeated parsing with the same input and budgets MUST yield equivalent
results. Verify before/after bytes and use an API with no I/O capabilities.

## Security boundaries

All message bytes and counts are hostile. Parser output does not authenticate
an answer. Bound allocation and traversal independently of advertised counts.
An independent verifier must review the malformed-input corpus and budgets.

## Data lifecycle

No persistent state is created. Input ownership and returned view lifetimes must
be explicit in a future Rust API. Any later persistent cache requires a separate
versioned storage and migration contract.

## Performance

No benchmarks have run. Before accepting numeric targets, select representative
small, large, compressed and adversarial messages; record CPU, toolchain, allocator,
peak allocation, throughput and latency methodology. Compare a pinned existing
implementation only on equivalent supported behavior. Budget semantics come first.

## Acceptance and evidence

[evidence plan](../evidence/dns-message.json) maps the six requirements to planned
positive and negative checks. All checks are not-run. Structural validation of
this document does not validate an implementation. A future separate verifier
must reproduce runtime results against an exact source revision.

## Open decisions

- DNS expert: compatibility policy for trailing bytes and uncommon label encodings.
- Security/domain reviewers: proposed budget defaults and counting semantics.
- Maintainer: whether this is a useful reusable example of the initial spec contract.
