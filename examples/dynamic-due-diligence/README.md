# Dynamic Due Diligence

This example ports the useful shape from Monterro's dynamic due-diligence flow
into the hackathon repo without importing its direct provider calls.

What it demonstrates:

- Organism-seeded breadth and depth research strategies
- Converge-governed evidence collection and hypothesis extraction
- Explicit contradiction facts instead of hidden prompt-only reconciliation
- A structured final brief returned in `projection.details`

## Run It

Start the local server:

```bash
just server
```

Then execute the example request:

```bash
curl -s http://127.0.0.1:8080/v1/truths/dynamic-due-diligence/execute \
  -H 'content-type: application/json' \
  -d @examples/dynamic-due-diligence/dynamic-due-diligence.request.json
```

## Why It Exists

`evaluate-vendor` is still the baseline truth students should learn first.

`dynamic-due-diligence` is the next step up:

1. It keeps the curated `converge-pack` / `converge-kernel` and `organism-pack` / `organism-runtime` surfaces.
2. It uses an offline-first mock research corpus so the example works without live services.
3. It shows how to model provenance, gap-chasing, and contradiction handling without dropping into provider-specific code.
