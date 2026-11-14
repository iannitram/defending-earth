# Gorger: design sheet

## What it is

A burrowing worm, 10 m long. It travels under the ground and comes up
to bite. The Registrar controls it with hardware bolted to its armour.

## What the model is for

A true-size model to judge the look and the size, and to make the game
pictures from later. It has no rig (the inner skeleton used to move a
model) and no motions yet.

## The four forms

All four are in `model.blend`, each in its own collection (a named
group of objects).

| Collection | Form | Ground it takes |
|---|---|---|
| `model` | On the surface. Lies in a shallow S. Jaws half open. | 1 by 5 squares |
| `head_out` | The first 3.0 m rising straight out of the ground. Jaws fully open. A ring of thrown soil and clods at the base. | 1 by 1 square |
| `sign` | Buried. Only the sign on the ground shows. | none |
| `dead` | As on the surface, but jaws slack, feelers limp, lenses dark, one plate cracked through. | 1 by 5 squares |

## The look

- **Body.** Thickest a third of the way back, thin at the tail.
- **Armour.** 28 overlapping plates over the back and sides. Bone grey,
  matte, chipped, with soil in the gaps.
- **Flesh.** Dark red and damp. It shows between the plates and along
  the belly. Dark frame ribs run across the belly.
- **Jaws.** Six curved jaws that open like a flower. The inside is dark
  red with rows of barbs.
- **Feelers.** Ten, swept back from behind the jaws.
- **No face.** No eyes, no mouth shape other than the jaws.
- **Control hardware.** A black housing with five amber lenses, behind
  the head. Beside it, a broken piece of the Registrar's black ring,
  bolted across the plates.
- **The wrong repair.** Three plates in a row, half way along, are flat
  black Registrar plate with thin etched lines. They do not match.
- **At night only.** Thin pale sparks jump between the feelers.

## The buried sign

A 2 m patch of ground raised 4 cm. Fine cracks. The soil is a little
lighter and looser. One thin wisp of dust, about 0.4 m tall, leans with
the wind.

It is meant to be faint. There is no rim and no outline. Picture
`12_sign_compare` is the same view as `10_sign_aerial` with a thin red
circle drawn round the sign, for checking only.

## Sizes

From `scale_sheet.json`, read by the build script:

| What | Size |
|---|---|
| Length | 10 m |
| Body thickness, a third back | 1.1 m |
| Tail thickness | 0.5 m |
| Height out of ground (head out) | 3.0 m |
| Sign patch | 2.0 m |
| Sign heave | 0.04 m |
| Ring arc radius | half the Taker's ring diameter (0.7 m) |

From the job text: jaws 0.9 m, feelers 1.2 m, ring arc 0.6 m long, wisp
0.4 m, 28 plates, 6 jaws, 10 feelers, 5 lenses.

Measured from the model:

| Form | Width | Depth | Height | Triangles |
|---|---|---|---|---|
| Surface | 1.849 m | 9.985 m | 1.534 m | 29,892 |
| Head out | 1.877 m | 1.626 m | 3.0 m | 13,232 |
| Dead | 1.983 m | 10.005 m | 1.118 m | 29,552 |
| Sign | 2.0 m | 2.0 m | 0.043 m | 4,230 |

Length along the body, jaw tip to tail tip: 10.000 m. The triangle
limit for the surface form is 60,000.

## Colours

| Part | Colour | Finish |
|---|---|---|
| Plates | `#a39c8c` | matte, chipped |
| Flesh | `#5a1c1c` | damp |
| Frame ribs | `#14171a` | SPINE lattice, 6 cm cells, roughness 0.3 |
| Registrar plate, housing, ring arc | `#0b0b0c` | roughness 0.35, metal |
| Etched lines | `#8a6a2a` | thin, never glowing |
| Lenses | `#ffa01e` | glow strength 6 |
| Sparks | `#cfe4ff` | glow strength 4, night picture only |

## Lights

The five amber lenses are the only lights on the live animal. They are
dark on the dead form. The sparks are hidden in every picture except
`06_night`.

## Choices made in this job

These were small model choices the brief left open. Change any of them
in `build.py`.

- The outer faces of the jaws are armour coloured. Barbs are pale.
- Feelers are a pale horn colour.
- The head is 0.65 of the body thickness.
- Plates are 3.2 cm thick and wrap 115 degrees each side of the back.
- Jaws half open is 30 degrees. Fully open is 60 degrees, so the open
  mouth stays inside one square.
- The top of the head-out form is set to exactly 3.0 m.
- The repaired plates are numbers 12 to 14. The cracked plate on the
  dead form is number 6.
- In `09_head_out` the soldier stands 1.5 m away, centre to centre.
- The wind in the sign pictures blows toward +X.

## Left for later

- Rig and motions: burrowing, rising, the bite, the jaws opening, the
  feelers, the dust wisp drifting, the sparks.
- Baked textures (surface pictures saved to files for the game).
- A lower-detail version for the game, if needed.

## Files

- `build.py`: builds all four forms, writes the size files, saves
  `model.blend`.
- `render.py`: makes the pictures in `renders/`.
- `size.json`, `size_surface.json`, `size_head_out.json`: measured sizes.
- `rounds.md`: the judging notes.
