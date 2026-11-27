# Review and evidence boundaries

Humans and agents define requirements together. Experts provide intent, domain
constraints, representative examples and non-goals. Reusable decisions let
agents proceed without repeating the same questions.

Escalate conflicting standards, ambiguous intent, changed trust boundaries,
compatibility or data guarantees, changed acceptance criteria and high-impact
operational decisions. Use the decision template: one question, sources,
alternatives, recommendation, impacted requirements and requested decision.
Silence does not approve a consequential change.

Routine formatting and structural validation are automated. Maintainers review
merges and can delegate bounded areas to qualified reviewers as participation
grows. Initially repository ownership is not proof of domain expertise. Track
the domain reviewer and the question they decided instead of claiming a universal
expert approval. Builder and verifier roles should be separate; a different model
name alone does not make evidence independent.

Evidence states are explicit: proposed, measured, independently reproduced,
reviewed, or formally proved for a named property. They are not interchangeable.
Specs and evidence reference exact revisions. Performance targets include workload,
hardware, resource limits and methodology; never infer performance from language.

The PR gate reads candidate documents using the validator and schemas on the
base commit. It does not run candidate scripts or install candidate dependencies.
Validator, schema, workflow and acceptance changes require review. Their own
candidate test execution is a separate maintainer action in an isolated runner.
There is no automatic merge or release. No production credentials are needed.

The bootstrap gate validates structure, local links and traceability, not truth
of a benchmark or absence of vulnerabilities. Phase one must define independent
unit/property, integration, conformance, fuzzing, static analysis and appropriate
formal checks. QEMU and finite real-hardware gates apply where behavior needs
them; no simulation stands for a completed hardware check.

Code and data version separately. Stateful specs need migrations across skipped
versions, interruption recovery, backup/restore and explicit downgrade behavior.
Returning stale devices require remediation isolation before normal operation.
