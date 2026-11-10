# Section 5: physics, ragdolls, destruction

Vellmark's world has weight. Things fall, swing, break, and stay where they land. All of it runs on `rapier3d` inside `sim`, at the fixed timestep, and all of it has a budget, because the minimum machine is an Apple M1 with 8 GB of memory.

## Physics

- **Loose props** are rigid bodies: crates, barrels, bottles, books, tools, weapons on the ground, and anything a player drops or throws. Players can pick up, carry, and throw them.
- **Hanging and jointed things** use joints: shop signs, chains, lanterns, rope bridges, portcullises, and cart wheels.
- **Words change physics.** `HEAVY` and `LIGHT` change a body's real mass, so a `LIGHT` boulder can be thrown and a `HEAVY` crate holds a pressure plate. `COLD` water bears weight as ice. `WHOLE` and `BROKEN` drive destruction, below.
- **Sleeping.** Bodies at rest cost nothing. A prop becomes active only when touched, struck, or written on.
- **Persistence.** A moved prop's resting place is saved, as Section 4 already requires. Only resting transforms are saved, never motion.
- **Co-op.** Both players act on the same simulation. Physics stays at full detail around each of them.

## Ragdolls

- A slain creature becomes a ragdoll built from its rig family, with joint limits tuned per family, and inherits the momentum of the blow that killed it.
- After a few seconds at rest, the body dissolves into its letters, which spill and settle as a pool of characters, as the canon describes. This is also how bodies are cleaned up.
- **Hit reactions.** A heavy hit blends a partial ragdoll into the animation for a moment, so limbs give way where they were struck and then recover.
- The amorphous rig family does not use ragdolls. It sheds and scatters characters instead.
- The player's own death is not a ragdoll in their own view. Their partner sees one.

## Destruction

Destruction is authored, pre-computed, and tied to the world's own rules. There is no free-form destruction of terrain or of whole buildings.

- **Breakable props:** crates, barrels, pottery, furniture, fences, shutters, and light doors. The bake tool pre-fractures each one into pieces. Enough force, fire, or the word `BROKEN` swaps the prop for its pieces, which fall under physics.
- **Marked structures:** walls, pillars, bridges, floors, and gates that a location's data marks as breakable, each with an authored broken state and a purpose: a new route, a trap, a shortcut, a boss arena that changes. Hero dungeons use several. Kit-built dungeons may use one or two.
- **`WHOLE` repairs.** Writing `WHOLE` on a broken thing pulls its pieces back along the paths they fell. A bridge reassembles under the player's feet. This is the signature of destruction in Vellmark and must look exact.
- **Things break along their text.** Fracture lines follow the lines of writing on a surface where they can, and small debris crumbles into loose characters within seconds.
- **Saved as a word.** A destructible's state is its Inscription, `WHOLE` or `BROKEN`, plus nothing else. Loading rebuilds the broken state from the authored one.
- **Never a dead end.** A quest-critical structure is Sealed or has an authored result for both states, and the content validator checks that no broken state blocks a required path.
- **Enemies use it.** Some species and bosses break marked structures themselves, as listed in the bestiary.

## Budgets

These are limits for the minimum machine and scale up with presets. The physics soak test enforces them.

| Thing | Limit on the minimum preset |
|---|---|
| Active rigid bodies near each player | 150 |
| Active ragdolls in the world | 6 |
| Loose fracture pieces before the oldest crumble into characters | 200 |
| Physics step, worst case | 2 ms |
