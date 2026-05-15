---
name: proof-plan
description: "Create proof wave or pulse plans with mission, artifacts, governing roles, gates, and non-goals."
tags: [proof, plan, wave, pulse, gates]
---

# proof-plan

Use this skill when drafting a proof quality wave, architecture refactor,
coverage sweep, docs contract repair, or execution plan.

## Wave Card Minimum

Write wave cards to `context/waves/YYYY-MM-DD-wave-slug/WAVE.md` with:

- frontmatter: `wave`, `date_open`, `status`, optional `source_request`
- mission
- claim boundary
- inputs
- pulse status table
- validation gates
- done criteria
- non-goals
- closeout/lessons when complete

## Pulse Plan Format

Write pulse plans to `context/waves/{active}/pulses/NN+slug.md` with:

- frontmatter: `wave`, `pulse`, `date`, `status`, `depends_on`,
  `governing_roles`
- mission
- scope inventory
- pre-implementation scout
- deliverables checklist
- validation gates
- non-goals
- evidence/commits when backfilling completed work

## Planning Rules

- Prefer one committable outcome per pulse.
- Name source artifacts and generated artifacts explicitly.
- Include review roles when the pulse changes doctrine, schema semantics,
  diagnostic contracts, or release claims.
- Put gates in the plan before implementation.
- Backfilled pulses cite commits, release notes, or artifacts rather than
  pretending they were planned before the work.
