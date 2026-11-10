# Section 9: how to work

## 9. How to work

The starter `CLAUDE.md`, `docs/STATE.md`, `docs/PLAN.md`, `docs/PLAN_CHANGES.md`, and phase files already exist. Keep the plan current as described in `CLAUDE.md`. In Phase 00, create `docs/ARCHITECTURE.md` and `docs/DECISIONS.md`. The canon already exists in `docs/WORLD_BIBLE.md`, `docs/bible/`, and `docs/script/`. Initialize git and commit at each meaningful step. Never commit credentials.

**Session protocol.** This project will span many sessions with no shared memory.

- Start each session by reading `CLAUDE.md` and `docs/STATE.md`, then only the docs and sections your task touches.
- Work on one small, testable task from the roadmap at a time.
- End each session with passing tests, a commit, and an updated `docs/STATE.md` covering what changed, what is broken, and the exact next step.
- Keep files small enough to read cheaply, and record what you learn in docs in place of relying on memory.

**Writing process.** `docs/WORLD_BIBLE.md` indexes the canon in `docs/bible/` and `docs/script/`: timeline, geography, factions, characters, voice guide, dialects, and the full story. Check every new piece of writing against it.

- Write content in small batches by location or questline.
- Review each batch in a fresh context against the bible, looking for contradictions, cliché, and repetition.
- A lint step flags reused names, repeated sentence openings, overused phrases, and quests that share the same structure.

**Scope.** `docs/SCOPE.md` records the build tier of every catalogue entry and what each release contains. Read its section for the current phase before building content. Apply the pillar test in `0a-pillars-and-releases.md` before building or expanding any system. Never change a tier without recording it.

Keep the architecture simple and data-oriented. Avoid speculative abstractions. Write small, testable systems and keep game data out of code.

When a decision is ambiguous, make the best call, log it in `DECISIONS.md`, and keep going. Stop for me only at the owner gates in the phase files, for AI asset approval, downloads, paid assets, or credentials, before publishing anything, or when blocked by something irreversible. While waiting on me, continue any work that does not depend on the answer.

Critique your own screenshots honestly against section 2. Iterate until they are beautiful, not merely functional.
