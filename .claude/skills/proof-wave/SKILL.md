---
name: proof-wave
description: "Manage proof waves in context/waves: find active architecture/quality work, show status, advance pulses, and close waves."
tags: [proof, wave, quality, architecture, execution, planning]
---

# proof-wave

Use this skill when the user asks for a wave, quality rail, architecture review,
roadmap, milestone execution, or to continue from the active proof wave.

## Source Of Truth

- Wave index: `context/waves/PHASES.md`
- Active wave: first row with `status: active`
- Active wave card: `context/waves/{active}/WAVE.md`
- Pulse plans: `context/waves/{active}/pulses/`
- Fork contexts: `context/waves/{active}/forks/`
- Review panels: `context/waves/{active}/panels/`
- Narrative release history: `CHANGELOG.md`

## Status Procedure

1. Read `context/waves/PHASES.md`.
2. Resolve the first `active` wave directory.
3. Read `context/waves/{active}/WAVE.md`.
4. List pulses in order.
5. Report the active wave, completed pulses, next todo pulse, validation gates,
   and carry-forwards.

## Next Procedure

1. Resolve the next `todo` pulse unless the user names a pulse.
2. Read the pulse file completely.
3. Run the pulse's pre-implementation scout commands.
4. Implement the deliverables using existing proof conventions.
5. Update documentation and pulse checkboxes.
6. Update `WAVE.md` pulse status.
7. Run validation commands from the pulse.
8. Report files changed, gates run, and carry-forwards.

## Close Procedure

Close only when every pulse in `WAVE.md` is done or explicitly deferred:

1. Write `CLOSE.md`.
2. Update `context/waves/PHASES.md` status.
3. Bridge significant shipped behavior into `CHANGELOG.md`; do not rewrite
   historical release notes.
4. Run final validation.

## Rules

- A wave is an execution rail, not a public release milestone.
- A pulse is the smallest committable quality or architecture improvement unit.
- Do not mark a pulse done unless its validation ran or the blocker is written.
- Backfilled waves cite existing release notes, commits, or artifacts rather than
  pretending the plan existed before the work.
