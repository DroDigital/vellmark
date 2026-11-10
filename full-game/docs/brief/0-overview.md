# Vellmark overview

You are building **VELLMARK**, a first-person open-world fantasy RPG with seamless drop-in local co-op, a custom renderer, and a look no shipped game has built itself around: a realistic, naturally lit world in the manner of Skyrim, in which every surface is secretly written in ASCII characters too small to see. From any distance it is a real place. Walk up to a wall and it turns out to be made of thousands of tiny letters, and you can read them.

The game takes structural inspiration from Skyrim and similar open-world RPGs: go anywhere, skills that grow through use, dungeons, factions, books, towns with routines. All names, lore, quests, characters, and assets must be original. Use nothing from The Elder Scrolls or any other existing franchise.

The game stands on three pillars, **Read**, **Rewrite**, and **Wander**, defined in `0a-pillars-and-releases.md`. Every system must serve one of them. That file also defines the cost tiers and the release plan, and `docs/SCOPE.md` records what is built in which release.

The files in `docs/brief/` are the source of truth for design. The phase files in `phases/` are the source of truth for build order. Read only the brief files your current phase lists.

**If two rules conflict,** use this priority: legal and license rules, my direct instructions, save integrity and stability, the three pillars, art direction, performance budgets, then content counts. Log the conflict and your resolution in `docs/DECISIONS.md`.

## Where each section lives

| Section | Files |
|---|---|
| 0 | `0a-pillars-and-releases.md` |
| 1 | `1a-core-constraints.md`, `1b-storefront-and-release.md` |
| 1.1 | `1.1a-licenses-and-ai-gate.md`, `1.1b-asset-catalog.md`, `1.1c-libraries-and-import.md` |
| 2 | `2-art-direction.md`, `2a-signature-visuals.md` |
| 3 | `3a-renderer-core.md`, `3b-renderer-surfaces.md` |
| 4 | `4a-world.md`, `4b-content-and-story.md`, `4c-content-rules.md` |
| 5 | `5a-core-gameplay.md`, `5b-rpg-systems.md`, `5c-enemies-and-bosses.md`, `5d-loot.md`, `5e-ui-and-accessibility.md`, `5f-coop.md`, `5g-input.md`, `5h-signature-mechanics.md`, `5i-physics-and-destruction.md` |
| 6 | `6-audio.md` |
| 7 | `7-characters-and-animation.md` |
| 8 | `8-proof-tooling.md` |
| 9 | `9-how-to-work.md` |
| 10 | `10-definition-of-done.md` |

The build order is Phases P00 to P20 in `phases/`. Where a brief file says M0, it means Phase 00.
