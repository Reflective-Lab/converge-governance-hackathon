---
tags: [domain, landscape, positioning]
---
# Procurement & Vendor-Decision Landscape

Where this project sits relative to commercial procurement and spend-management software. Useful for pitch positioning and for keeping scope honest.

## What this project is

[[Domain/Vendor Selection]] defines the scope: **governed AI vendor selection**. Multi-agent convergence on a defensible *sourcing decision* — compliance + cost + capability + risk → recommendation with audit trail. This is the **decision layer** of procurement.

It is not spend management. It is not an intake form. It is not an enterprise sourcing suite.

## The four layers

| Layer | What it does | Example tools |
|---|---|---|
| **Intake & orchestration** | Capture procurement requests, route approvals, kick off workflows | Zip |
| **Decision layer** *(this project)* | Governed multi-agent convergence on which vendor to onboard, with auditable reasoning | — (unclaimed) |
| **Enterprise procurement / sourcing** | RFP/RFQ, supplier scorecards, contract lifecycle, sourcing events | Coupa, SAP Ariba |
| **Spend management & operational vendor mgmt** | Corporate cards, AP, expense, vendor spend visibility, renewals, duplicate-SaaS detection | Ramp, Brex, Airbase, Navan |

## How they relate to this project

- **Ramp / Brex / Airbase** — *downstream*. Once a vendor is selected, these systems hold the vendor record, run approvals, and track ongoing spend. Ramp's Buyer acquisition pushes it slightly upstream into negotiation, but still post-selection.
- **Coupa / Ariba** — *overlapping*. They include sourcing and supplier scorecards, but the heavy enterprise-procurement-suite framing is broad and convention-based. This project's claim is **governance by construction** — Cedar policy + multi-agent convergence + honest stopping — not bolted-on workflow.
- **Zip** — *upstream*. Zip is the request/intake layer; it routes to whoever decides. This project is what happens *after* the intake — the decision itself.

## Positioning takeaway

Governed selection — the auditable decision step between "we need a vendor" (Zip) and "we now manage this vendor" (Ramp) — is largely unclaimed. Enterprise suites cover it by convention; spend-management tools don't pretend to cover it. That gap is the wedge.

See also: [[Domain/Vendor Selection]], [[Presentations/Team Pitch - Governed Vendor Selection]]
