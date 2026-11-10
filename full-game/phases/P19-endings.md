# Phase 19: Endings

**Goal:** Build the final dungeon, all four endings, and everything the Colophon depends on.

**Suggested model:** Fable 5.2, medium effort. Four endings and an epilogue written from the save are what players will remember.

**Hand to Haiku 5.5:** Epilogue line tables once their wording is set, ending test scripts, and the Ledger of Endings screen. Rules are in `CLAUDE.md` under Models and helpers.

**Release:** 1.0

**Requires:** Phase 18 finished and tagged `phase-18-done`.

## Read for this phase

Read only these files in `docs/brief/`, and only the named part where one is given.

- `4b-content-and-story.md`
- `4c-content-rules.md`
- `5b-rpg-systems.md`: Quest safety
- `9-how-to-work.md`: Writing process
- `8-proof-tooling.md`: Critical-path and Ending tests

## Canon for this phase

These files are under `docs/`.

- SCOPE.md: 1.0
- voice/cast-and-budget.md: Rules and Credit budget
- WORLD_BIBLE.md
- bible/01-history-and-truth.md
- bible/03-voice-and-dialects.md
- bible/06-characters.md: the characters in the current quest
- bible/09-books.md
- script/04-act-three.md: Q18A to Q20
- script/05-endings.md
- script/08-secrets-and-small-tasks.md: Secret questlines

## Scope

The Gutter, the three final routes and their bosses, the secret Colophon chain and The Clerk, the point of no return, the Ledger of Endings, four outros, the save-driven epilogue, the two superbosses, and a writing review of the whole game

## Tasks

Do these in order, one at a time, following the task loop in `CLAUDE.md`.

- [ ] **P19-T01** Write Q18A, Q18B, and Q18C, the three final routes
- [ ] **P19-T02** Build The Gutter, the final dungeon shared by all routes, and Q19
- [ ] **P19-T03** Build the four final stretches and their final bosses: The Cantor, Lord Registrar Thale, The Corrector, and The Clerk
- [ ] **P19-T04** Write Q20, The Last Line
- [ ] **P19-T05** Finish the eight secret chains, including the hidden chain for the Colophon ending
- [ ] **P19-T06** Build the two hidden superbosses
- [ ] **P19-T07** Build the point of no return and the Ledger of Endings
- [ ] **P19-T08** Create the four outros
- [ ] **P19-T09** Build the save-driven epilogue generator
- [ ] **P19-T10** Finish any book, secret, or small task on the 1.0 list that is not yet built
- [ ] **P19-T11** Run the full writing review across all content, and fix contradictions
- [ ] **P19-T12** Lock `docs/voice/voice_lines.tsv`, get owner approval, generate the remaining lines by priority, review takes, and integrate them
- [ ] **P19-T13** Update `docs/AI_DISCLOSURE.md`
- [ ] **P19-T14** Run critical-path and ending tests, solo and in co-op, and the scope check across the whole game

## Exit gate

Every quest passes bot playthrough, critical-path and ending tests pass, and no built content references a Reserve entry

Save the evidence in `docs/evidence/P19/`.

## Next

Move to `phases/P20-version-one.md`.
