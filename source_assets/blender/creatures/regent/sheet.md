# The Regent: design sheet

Sheet name: `regent`. Job C01.

## What it is

A tall, thin creature on four stilt legs. Its brain sits in a glass dome
on top. A broken black ring of Registrar hardware hangs behind the dome.

## Purpose

This is the true-size source model. Later jobs make the game model, the
rig and the motion from it. The pictures in `renders/` are for checking
the look and the size.

## The look

- Very tall and narrow. The feet stand wide, the body is slight.
- No face. A pale bone blade hangs forward and down where a face would
  be. Five small amber lenses sit in a row near its lower end.
- The dome holds a mound of about 40 soft lobes that glow faintly.
- The ring is black, square-edged and plainly made by someone else. One
  arc is missing. One hangs loose.
- Old cloth panels with dull gold diagrams hang from the shoulders and
  hips.
- One leg has been replaced below the knee by a black strut.

## Parts

Every moving part is its own object with its origin at its joint. There
is no rig. The creature faces -Y. Its own left is +X.

| Part | Objects |
|---|---|
| Body | `torso`, `neck`, `head` |
| Dome | `dome_glass`, `dome_lobes`, `lobe_01` to `lobe_40` |
| Head plate | `head_plate` (with the five lenses) |
| Ring | `ring_mount`, `ring_frame`, `ring_arc_1` to `ring_arc_5`, `ring_cable_1` to `ring_cable_3` |
| Legs | `leg_FL`, `leg_FR`, `leg_RR`, each as `_thigh`, `_shin`, `_foot` |
| Strut leg | `leg_RL_thigh`, `leg_RL_strut` (rear leg on its own left) |
| Arms | `arm_UL`, `arm_UR`, `arm_LL`, `arm_LR`, each as `_upper`, `_fore`, `_claws` |
| Shoulder plates | `plate_shoulder_*`, `plate_shoulder_low_*` |
| Cloth | `cloth_shoulder_L`, `cloth_shoulder_R`, `cloth_hip_front`, `cloth_hip_back` |
| Night only | `electrical_arcs` (hidden except in the night picture) |

The living model is in the collection `model` under the empty `regent`.
The dead form is in the collection `dead` under `regent_dead`.

## Sizes

From the scale sheet (`scale_sheet.json`, entry `regent`):

| | Sheet | Model |
|---|---|---|
| Footprint | 1 by 1 square | 1 by 1 square |
| Height | 3.6 m | 3.600 m |
| Width | 1.8 m | 1.801 m |
| Depth | 1.8 m | 1.801 m |

From the job text:

| Part | Size |
|---|---|
| Dome | 0.55 m across, a little taller than a half ball |
| Lobes | 40, each 4 to 7 cm |
| Head plate | 0.5 m long |
| Ring | 0.9 m across |
| Torso | 1.1 m long, about 0.35 m wide, from 2.0 m to 3.1 m, leaning 15 degrees forward |
| Legs | about 6 cm thick, knees near 2.0 m |
| Feet | three toes, at the corners of a 1.5 m square |
| Arms | 1.4 m long, three claws of 25 cm |
| Cloth panels | 1.6 m by 0.25 m |
| Dead form | 2.90 m by 1.98 m, 1.03 m high |

Triangles: 30,330 living, 28,890 dead. The limit is 40,000.

## Colours

All materials are made in Blender. No picture files.

| Thing | Colour | Notes |
|---|---|---|
| Flesh | `#3a2430` | roughness 0.3 |
| Bone plates | `#cfc6b4` | fine pale cracks |
| Lattice (Spine) | `#14171a` | 6 cm cells, roughness 0.3, where plates meet |
| Registrar hardware | `#0b0b0c` | roughness 0.35, full metal |
| Etched circles | `#8a6a2a` | on the ring hub and strut |
| Cloth | `#b8ad98` | gold diagrams `#a8842c` |
| Dome glass | clear | |
| Lobes | pink flesh tint | glow `#ffd9c8` |

## Lights

| Light | Colour | Strength | Where |
|---|---|---|---|
| Amber | `#ffa01e` | 6 | Registrar parts only: one seam on each ring arc, the five lenses, the seam on the strut |
| Lobe glow | `#ffd9c8` | 2 | the lobes in the dome |
| Electrical life | `#cfe4ff` | 4 | small arcs at the joints, night picture only |

There are no glowing outlines and no face.

## Dead form

Fallen on its right side. Legs folded, the front shins crossed over the
rear legs. The dome is dark. The lobes are grey and smaller. The amber
seams and lenses are unlit. The ring arcs lie on the ground by the dome.
It fits inside 2 by 3 m.

## Left for later

- A rig and motion. Parts are split and their origins are at the joints.
- The lower-detail game model and its baked pictures.
- Cloth that moves.
- Flicker for the electrical arcs and the lobe glow.
- Damage states between living and dead.
- Sound.

## Choices made by the builder

These were small build choices. The lead may want to check them.

- "Five arcs, one missing" was read as six places on the ring with five
  arcs present. One of the five hangs loose.
- The arcs sit on a thin black hoop held by a short fork behind the neck,
  so they do not float.
- The five lenses are on the outer face of the head plate, near its lower
  end.
- The creature's left is +X, so the strut is the rear leg at +X, +Y.
- Small bone caps were added on the knees.
- The lobes have a pink flesh base colour under the `#ffd9c8` glow.
