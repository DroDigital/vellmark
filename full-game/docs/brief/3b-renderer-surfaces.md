# Section 3: surfaces, effects, damage, magic (3.8, 3.9, 3.13)

### 3.8 Beyond opaque surfaces

Each of these looks natural from a distance and is written in fine print up close.

- **Grass:** real instanced blades that sway, bend around the player, and are culled by compute shader (budget target: over 300,000 visible). Up close, each blade is a line of characters.
- **Trees:** generated trunks whose bark carries text, and leaf cards whose leaves are characters at arm's length, with conventional levels of detail and impostors at distance.
- **Water:** physically shaded, flowing along a flow map, with reflections, refraction, and shoreline foam. At the water's edge, ripples resolve into `~ - =` and foam into `.` and `,`.
- **Sky:** a physically based atmosphere, layered procedural clouds, and a real star field. The moon looks like a moon whose dark seas faintly suggest an `@`. It turns crimson on rare story nights and loses detail as the Erasure advances. In close reading, the stars are characters, the moon is one enormous `@` built of smaller ones, and the aurora is curtains of falling `! | :`.
- **Particles:** snow, embers, and smoke look natural at distance. A flake that drifts close to the eye is a `*`, an ember is a `'`. Magic is always flying words.
- **Fog:** volumetric and lit, with the god rays of a dawn valley in full colour.

### 3.9 Damage

Each character near a player has a persistent, low-resolution damage mask, pooled to a fixed budget. Hits knock real letters off the body as particles, and those letters remain missing until healed. On death, a body falls as a ragdoll and then dissolves into a pool of characters on the ground, as set out in Section 5i.

### 3.13 Magic effects

Emendation is where the world's writing shows itself, so its effects are the richest in the game. Every word's effect is assembled from the same five parts, tuned per word, so 35 words do not mean 35 separate systems.

1. **The charge.** Gold characters gather at the casting hand, more and larger as the charge builds, lighting the hand and whatever is near it.
2. **The flight.** The word travels as a stream of GPU-simulated glyph particles with a bright head and a fading trail. It is a real light source and casts moving light on the world it passes.
3. **The impact.** A burst shaped by the word's class, with heat haze or refraction where it suits.
4. **The rewrite wave.** From the point of impact, the target's own fine print visibly rewrites itself outward across its surface and then sinks back out of sight. The wave is drawn in the surface shader from a few parameters per impact, so it costs almost nothing. This is the signature of magic in Vellmark.
5. **The lasting state.** While the word holds, the surface shows it: frost that creeps as `*` and `.`, embers that crawl along the letters of a `BURNING` door, a `HIDDEN` thing whose letters thin to nothing.

- Each class has its own colour of light, sound, and motion: Temperature flickers and creeps, Weight drops or lifts, Mind drifts toward the eyes, Integrity cracks or knits, Concealment thins.
- A typo's effect is its word's effect going visibly wrong halfway through the flight.
- Sorts change the delivery: `!` bursts into a ring, `,` forks, `...` hangs in the air as three points until it lands.
- Particles are simulated in compute, sorted cheaply, lit by the scene, and capped per view. On the Low preset, counts halve and the rewrite wave is kept.
- Effects must read clearly in split-screen and must respect the photosensitivity option.
