# <Component> specification

Copy this to `specifications/<slug>.md` and add a matching `<slug>.json`
using `specification.json`. Metadata is JSON; prose is Markdown. The checked
JSON Schema is the initial machine contract. Propose format changes in F002.

## Intent and users

## Standards and existing behavior

Name exact standards/sections and existing implementations. Separate requirements
from observed quirks and new proposals. Cite sources and state unresolved conflicts.

## Scope and non-goals

## Requirements

### EXAMPLE-001

State one externally observable MUST/MUST NOT requirement. Explain rationale and
acceptance evidence. Keep IDs stable; do not reuse an ID for different behavior.

## Security boundaries

Actors, assets, capabilities, untrusted inputs, resource limits and failure modes.

## Data lifecycle

Persistent formats, versioning, migrations, skipped upgrades, backup, restore and
downgrade behavior. Explicitly explain when the component has no persistent state.

## Performance

Workloads, hardware, latency/throughput and memory budgets, measurement procedure
and baseline. Targets are proposals until measured; do not invent benchmark results.

## Acceptance and evidence

Map every requirement to a planned check. Name the separate verifier and exact
revision once measured. Include negative cases and limitations.

## Open decisions

One clear question per decision packet; name the needed expertise.
