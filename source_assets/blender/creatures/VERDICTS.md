# Creature models, first round: the lead's verdicts

Judged 2026-11-13 by the lead, by eye, from each creature's
`renders/contact.png`. The owner agreed with every verdict the same day.
All sizes measure inside the scale sheet's 5 percent allowance
(`source_assets/scale/scale_sheet.json`).

These are shape and base-surface models only. Surfaces use Blender's
built-in patterns. No image textures, no baking, no rig, no motion yet.

Only `renders/contact.png` is kept in git. Run a creature's `render.py`
in a background Blender to make the other pictures again.

| Creature | Verdict | What works | Fix in the next pass |
|---|---|---|---|
| Regent | Accepted | Tall, thin, strange; fits one square | 6 cm legs may vanish at game distance; dead form is a pile of poles; gold marks on the cloth only show close up |
| Tender | Accepted | Small and frail beside the Regent | Head plate reads a little like a bird's beak; dead form is a tangle at distance; cloth strips hard to see from the front |
| Lash | Accepted with notes | Long low body, rifle and bipod read | Hide is pale and plastic in sun; the hanging grass and weed is thin; dead form reads as bent sticks; front view is crowded |
| Gorger | Accepted | 10 m body, head-out form, faint buried sign | Plates look puffy, like stuffed cloth; dead form looks like the live one; dust wisp nearly invisible in a still |
| Kite | Accepted with notes | Thin veined skin on spars, see-through from above; lens cage and bead threads | Flat diamond with crossed spars looks close to a toy kite; skin goes flat at night; cage reads as a black box from far off; a stray blue patch shows on the skin in `07_detail.png` |
| Bloat | Accepted with notes | Reads at once as a floating gas sac; dead form is a collapsed skin | Stitched patch can read as an eye; murk inside is faint by day; dead skin looks stiff, not wet |
| Maul | Accepted with notes | Strong outline of stone columns; controlled, wild and dead forms | Spike and collar read as a sword hilt; its two lenses read as a pair of eyes; crystals wash out to salmon pink by day; dead form reads as a fallen wall |
| Cinder | Accepted, the strongest | Cracked black crust over a glowing core, clear at all three sizes | Glow barely lights the ground at night; fire goes salmon pink where the core is bare; dead form only passable; large size is 1.05 m against 1.0 m |

## Choices the lead made where a brief was silent

- Hover, flight and underside heights mean the lowest point of the body.
- "Front left" sun means the -X, -Y quarter.
- Tender: the dead dome is clear grey, not solid dark (a dark dome hid the lobe).
- Regent ring: six places, five arcs present, one hanging loose.
- Gorger: the sign's faintness is kept as built.
- Cinder: most of the core is covered by a darker cooling skin so the fire
  stays orange. The black scorch patch and a visible ground glow work
  against each other; which wins is for the glow job (G4).

## Waiting on the owner

- Maul: does the 3.5 m height include the spike (it now stands to 4.42 m)?
  Does a dead Maul keep the whole spike or a stump? Both are built
  (`08_dead.png`, `08b_dead_wild.png`).
- Which creatures carry moss, grass or other growth. Only the Lash was
  briefed with plant matter. This is lore and comes before the texture pass.
- Forms larger than their squares: the wild Maul (4.01 m wide), the dead
  Maul (about 3 by 3 squares), the Bloat's dead stain (2.5 m), the Kite's
  threads (0.7 m past its square).
- Story chance: should some soldiers spot the buried Gorger's faint sign
  better than others? Nothing was added.
