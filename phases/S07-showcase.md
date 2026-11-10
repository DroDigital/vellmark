# Phase S07: Showcase

**Goal:** The last tenth: smoothness, polish, and the pictures that sell it.

**Suggested model:** Opus 5.5, high effort.

**Hand to Haiku 5.5:** Credits and third-party notices, the store text draft for the owner to edit, the playtest checklist, and collecting evidence. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `docs/TECH.md`
- `docs/SHOWCASE.md`: Done means
- `full-game/docs/brief/5e-ui-and-accessibility.md`: Photo mode and Accessibility
- `full-game/docs/brief/1b-storefront-and-release.md`: How the game is presented and Demo

## In this phase

The performance pass, photo mode, accessibility basics, packaging for three platforms, and the demo page kit.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [ ] **S07-T01** [Render] Run the performance run across the whole tour on the owner's Mac, and fix every pass over its share and every frame over 33 ms
- [ ] **S07-T02** [Render] Tune the governor until its steps cannot be seen, and tune Low and High
- [ ] **S07-T03** [Render] Build photo mode: the free camera, the player's character with poses, text view, the split frame, and saving to PNG
- [ ] **S07-T04** [Shell] Add accessibility basics: subtitles, letter size, interface scale, reduced motion, reduced screen response, the photosensitivity option, and full remapping
- [ ] **S07-T05** [Shell] Finish the settings menu, the pause menu, and a clean first launch that picks a preset by itself
- [ ] **S07-T06** [Sim] Run a soak of four hours of bot play and fix every crash, leak, and stuck state
- [ ] **S07-T07** [Shell] Package for Windows, macOS, and Linux, run the license audit, and generate the notices and the AI disclosure
- [ ] **S07-T08** [Shell] Capture the store kit in photo mode as the brief says the game is presented: key art, eight screenshots with three split frames, and the shots for a sixty second trailer
- [ ] **S07-T09** [Shell] Write the demo page text, and push a private itch build for the owner to publish
- [ ] **S07-T10** [Shell] Give a stranger's first-play checklist to the owner, collect what they report, and fix what it finds
- [ ] **S07-T11** [Shell] Run the Showcase Gate, save the evidence, and write `docs/CONTINUE.md`

## Exit gate

Every line under Done means in `docs/SHOWCASE.md` is true, with its evidence in `docs/evidence/S07/`.

## Owner gate

**Stop. Give the owner the builds, the performance report, and the store kit.**

The owner checks:

- You played the whole tour without thinking about the build
- It is smooth everywhere, on your Mac
- You took a screenshot because you wanted to
- Someone you showed it to asked when they can play more
- Nothing in it is a placeholder

**It passes when** the owner is ready to show it to strangers. Publish nothing without the owner saying so.

When it passes, move the full-game plan to the root as `docs/CONTINUE.md` describes, and start with split-screen co-op.
