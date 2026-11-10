# Phase S04: Bodies

**Goal:** People and creatures that look real from across the green, are written in letters up close, and fight well.

**Suggested model:** Opus 5.5, high effort.

**Hand to Haiku 5.5:** Annotation tables from the bible, animation clip lists, sound assignment tables, and ragdoll joint tables once one family is tuned. Rules are in `CLAUDE.md` under Models and helpers.

## Read first

- `full-game/docs/brief/7-characters-and-animation.md`
- `full-game/docs/brief/5c-enemies-and-bosses.md`: Enemies and Annotated enemies
- `full-game/docs/brief/3b-renderer-surfaces.md`: 3.9
- `full-game/docs/brief/2a-signature-visuals.md`: Typographic body language
- `full-game/docs/brief/5i-physics-and-destruction.md`: Ragdolls
- `docs/SHOWCASE.md`: The people and The creatures
- `full-game/docs/bible/07-bestiary-and-bosses.md`: the four species and Annotated creatures

## In this phase

The character assembler, animation, faces, the creature pipeline for three rig families, enemy AI, the four creatures with annotations, damage masks, ragdolls, and body language.

## Tasks

Do the tasks of one lane in order, following the task loop in `CLAUDE.md`. Lanes run side by side as `docs/CLOUD.md` sets out. A task marked **after** waits for the task it names.

- [ ] **S04-T01** [Content] Build the character assembler from the approved bodies and outfit parts onto one shared skeleton, with anchored letters from the rest pose
- [ ] **S04-T02** [Render] Add GPU skinning, with animation rate and bone count reduced by distance
- [ ] **S04-T03** [Sim] Build the animation runtime: retargeted clips, inertial blending, foot placement, look-at, breathing, and layered upper and lower body
- [ ] **S04-T04** [Content] Build faces that hold up in a dialogue close-up, with a jaw and eyelid rig and a few expressions
- [ ] **S04-T05** [Render] Add generated truth: clothing that carries a person's real biography and routine, rebuilt only when it changes
- [ ] **S04-T06** [Content] Build the creature pipeline for the biped, quadruped, and amorphous rig families, with procedural locomotion
- [ ] **S04-T07** [Sim] Build enemy AI: perception, telegraphs, flanking, retreat, calling for help, and reactions to words on themselves, their allies, and the place
- [ ] **S04-T08** [Content] Build the Blot Hound and the Runt that follows, with pack behaviour and the hound's own answer to `CALM`
- [ ] **S04-T09** [Content] Build the Shambling Entry and the Bone Clerk, with the Clerk's summoned paperwork
- [ ] **S04-T10** [Content] Rebuild the Inkling on the real pipeline, with splitting
- [ ] **S04-T11** [Sim] Add the four annotations on every creature that supports them, with their names, letters, and the stagger when the word is replaced
- [ ] **S04-T12** [Render] Add damage masks, letters knocked loose by hits, and typographic body language with each species' loud and quiet sets
- [ ] **S04-T13** [Sim] Add ragdolls for bipeds and quadrupeds, partial ragdoll hit reactions, and bodies that dissolve into pools of characters
- [ ] **S04-T14** [Render] Build Obelus as a flying form with its strike effect: the black rule and what it leaves behind
- [ ] **S04-T15** [Shell] Add creature voices, footsteps by surface, and combat sounds
- [ ] **S04-T16** [Shell] Extend the scripted run and the combat simulation to all four creatures, run the exit gate, and save the evidence

## Exit gate

Each creature can be beaten by blade and word, by steel alone, and by words alone. A person and a creature both pass the glyph-off comparison at ten metres and are legible at arm's length. Six creatures and four people on screen hold the frame budget on the owner's Mac.
