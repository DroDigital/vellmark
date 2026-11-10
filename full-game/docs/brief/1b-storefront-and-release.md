# Section 1: storefronts and release

**Storefront features.**

- `--features steam`: Steam Cloud, achievements, rich presence, Remote Play Together invites, the on-screen keyboard, and timeline marks for Steam's game recording, as set out under Spectacle in `5a-core-gameplay.md`.
- `--features itch`: DRM-free and SDK-free, with local saves and local achievement tracking. It must never require an account.
- Everything in the game must work the same with either storefront, and no feature may depend on Steam being available beyond the Steam features above.
- The Steam app ID comes from configuration. Use a placeholder until I provide one.
- `.itch.toml` declares the launch action in each itch build.

**Staged release.** The game is released in stages, as set out in `0a-pillars-and-releases.md`: Release 1 as paid early access on Steam and itch.io, then one update per Quire, then 1.0. Every stage needs my approval before anything goes live. The store page states exactly what the current release contains, says that the game is unfinished until 1.0, and gives no dates. Saves from every earlier release must load.

**How the game is presented.** Everything a stranger sees first sells the power and saves the letters for the turn. This applies to store pages, the demo page, trailers, screenshots, key art, and update notes.

- **The line:** "An open-world RPG where you rewrite reality with a word." Variations must keep the verb and the promise.
- **The first two sentences of any page** never contain ASCII, text-based, typography, reading, or letters. They say what the player does: open, burn, calm, mend, and fight with a blade in one hand and a word in the other.
- **The first screenshot** is a word changing something large. The second is a Redactor over a town. The split frame that shows the world as text comes third, as the surprise.
- **A trailer** opens on a cast, within its first three seconds, and shows in this order: a word obeyed at scale, blade and word in a fight, a Redactor striking out a street, loot, two players, and only then the push in through the letters. It ends on the town writing itself back.
- **Key art** is a Redactor above Scrivenhold with a lone figure below, one hand lit by a word.
- **Tags and genre:** open-world RPG, action RPG, magic, local co-op. Never "text-based", which means something else on every store.
- The interface's plain words are used throughout. The world's own terms appear only in quoted lines.
- Nothing here may promise what a release does not contain, and every image is captured from the game.

**Demo.** Once the Hook Gate has passed, the hook room is offered to strangers as a free demo. Its purpose is to learn early, from people who owe the game nothing, whether reading and rewriting are fun.

- The demo is the Phase 02 room and arena and nothing else, for one or two players. Its page says so: one room and one fight, an early prototype, about twenty minutes.
- It needs a title card, a controls screen, volume, sensitivity, and invert settings, an end card, and a way back to the start. The end card carries a feedback link and a wishlist link, both supplied by the owner.
- It goes to itch.io first, as a private page. A Steam demo needs a Steam app and a public store page, so it is the owner's decision and is not assumed.
- It ships for Windows and Linux, and for macOS only if the owner has supplied signing credentials, since an unsigned macOS build is blocked for most people.
- It collects nothing. There is no telemetry, as everywhere in the game. Feedback arrives only through the owner, who pastes it into `docs/DEMO_FEEDBACK.md`.
- Each batch of feedback is sorted in that file into three lists: fix now in the hook room, carry to a named later phase, and set aside with a reason. Anything that changes the design goes through `docs/PLAN_CHANGES.md` like any other change.
- The demo passes the license audit and carries its notices and AI disclosure before it is offered.
- Publishing the demo needs the owner's explicit approval, like every release. Work on Phase 03 does not wait for it.
- After the Feel Gate, the owner may ask for the demo to be refreshed once with the Phase 06 combat. It is not otherwise kept up to date.

**Release artifacts.**

- Steam: one depot per OS, uploaded through SteamPipe build scripts.
- itch.io: one `butler` channel per OS, containing a Windows zip, a zipped macOS `.app`, and a Linux tarball.
- macOS builds must be signed and notarized. Steam requires this. Include the entitlements Steam documents (`com.apple.security.cs.disable-library-validation` and `com.apple.security.cs.allow-dyld-environment-variables`), and do not enable the App Sandbox, which Steam does not support.
- Windows code signing is optional. Add it as a script step for itch downloads.
- All credentials come from environment variables, with the steps documented. Without credentials, unsigned local builds must still complete.
- Upload scripts may push only to a private Steam beta branch or a private itch page. **Never set a build live or publish a store page without my explicit approval.**
