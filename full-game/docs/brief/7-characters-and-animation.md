# Section 7: characters and animation

## 7. Characters and animation

- A character assembler combines the approved base bodies, hair, and outfit parts, with parametric changes to body shape, face, and armour layers, into skinned meshes on one shared skeleton.
- A creature pipeline starts from approved monster and animal models where they fit, then reshapes and combines them so species do not look stock, and builds the rest parametrically. Every species uses one of 6 shared rig families, with procedural locomotion tuned per species: biped, quadruped, multi-legged, serpent, winged, and amorphous.
- First-person arms and weapons are a separate view-model system. Full-body animation libraries do not cover this, so build procedural arm IK and code-authored swing, block, draw, and recoil curves for each of the four movesets in Section 5, which have one attack chain and one heavy attack each, plus the shield bash. Spend the effort saved on the writing gestures for a word in either hand, in both, and onto the held weapon, with a visible charge, that must read clearly from a partner's view. A player's partner sees that player's full-body animation.
- Very distant characters may animate at a reduced rate to save performance, never visibly.
- Animation combines the approved libraries, retargeted to the shared skeleton, with code-authored keyframes for anything missing, IK foot placement, procedural look-at, and layered upper and lower body. Prioritize weight and timing.
- Faces must hold up in dialogue close-ups. Use natural proportions with slightly simplified features, glyph flow following facial form up close, a jaw and eyelid rig, and expression shapes. If a face looks wrong, favour stylization (hoods, helms, weathered masks) over uncanny detail.
- **Procedural layers do most of the work.** On top of a small clip set, add foot placement on slopes and stairs, leaning into turns, look-at for head and eyes, breathing, additive hit reactions, and weapon sway. These make a few clips feel like many.
- **Secondary motion.** Cloaks, hair, pouches, scabbards, and tails move on spring chains with simple collision, simulated only near a player.
- **Inertial blending.** Transitions between clips carry the old pose's momentum into the new clip and evaluate only one clip at a time, which is smoother and cheaper than crossfading.
- **Ragdolls and partial ragdolls** follow Section 5i.
- **Skinning on the GPU,** with animation update rates and bone counts reduced by distance and for characters no view can see.
