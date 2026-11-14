# The Kite: design sheet

## What it is

An animal from the reed-stalkers' wetland world. A thin skin stretched on a
cross of bone, gliding high over the fight. It has no face.

## Purpose

It is the spotter for the Lash. It circles above and marks soldiers. Kill it
and the marksmen lose their eyes. (Rules are in `docs/LORE.md`.)

## The look

- A four-pointed kite shape. Wide across, short nose, longer tail.
- The four edges bow inward between the spar tips, like a stretched hide.
- The skin billows a little between the spars.
- The trailing edges are torn. There are a few long rips in the skin.
- It flies with a 6 degree bank, its own left wing down.
- A small black cage hangs under the crossing with four lenses looking down.
- Twelve long threads hang below with small pale beads along them.

## Parts

Parts that will move later are separate objects with their origin at the
joint. There is no rig.

| Object | What it is |
|---|---|
| `kite` | Root. Holds the bank and the flight height |
| `skin_left`, `skin_right` | The two halves of the skin, split along the main spar |
| `spar_main` | Bone spar, nose to tail |
| `spar_wing_left`, `spar_wing_right` | Bone cross spars, from the crossing out |
| `repair_rod` | Black rod that replaces the outer third of the right spar. Two collars and a wire winding hold it. Child of `spar_wing_right` |
| `spar_edge` | Thin edge line round the outline |
| `cage` | Black frame and clamp under the crossing |
| `lens_1` to `lens_4` | The four lens barrels. Children of `cage` |
| `thread_cage_1` to `thread_cage_7` | Threads tied to the cage. Children of `cage` |
| `thread_tail_1` to `thread_tail_5` | Threads tied to the tail |

The Kite's own left is +X. It faces -Y.

## Sizes

From `source_assets/scale/scale_sheet.json`:

| What | Size |
|---|---|
| Span | 1.8 m |
| Length | 1.5 m |
| Flight height (lowest point of the body) | 6.0 m |
| Footprint | 1 x 1 square |

The sheet's cheat applies: the Kite hangs in the air above the one square
it occupies. Its threads trail past that square.

From the job text:

| What | Size |
|---|---|
| Spar thickness | 2 cm |
| Target sign across | 25 cm |
| Cage across | 20 cm |
| Threads | 12, each 1.5 m |
| Repair rod | outer third of the right cross spar |

Measured (`size.json`): span 1.8, length 1.5, lowest body point 6.0, thread
length 1.5. Live form 29,403 triangles, dead form 28,593. The limit is
40,000.

## Colours

| Part | Colour |
|---|---|
| Skin | `#c98a2a`, thin and see-through |
| Veins and target sign | `#8c1f1a` |
| Bone spars | `#cfc6b0` (same bone as the Lash mask) |
| Cage, lens barrels, repair rod | Registrar black `#0b0b0c`, rough 0.35, metallic 1 |
| Rings on the barrels, rod collars | Etched gold `#8a6a2a` |
| Lens cores | Amber `#ffa01e` |
| Beads | Electrical life `#cfe4ff` |

The target sign is on the Kite's left wing, drawn in the skin in vein red.
It uses the same proportions as the sign on the Lash.

## Lights

- Four lens cores: amber `#ffa01e`, emission 6, behind dark smoked glass.
- Beads: `#cfe4ff`, emission 1.
- Nothing else glows. No glowing outlines.

## Dead form

Collection `dead`, root `dead_kite`. It lies belly-up on the ground. The
skin is shrunk and crumpled, the tail is kinked, and one wing is folded up
over the body. The cage is tipped and its lenses knocked out of line. The
threads lie in a tangle beside it. The lenses and beads are dark.

## Pictures

`renders/01_front` to `09_top`, plus `10_close` (a closer three-quarter view
from above, added because the soldier-height views show the Kite small and
edge-on) and `contact.png`.

## Left for later

- Rig and motion: flapping, banking, thread sway, the fall.
- Skin and vein pictures baked to textures for the game. They are shader
  nodes now.
- A lower-detail copy for the game. The threads and beads are most of the
  triangles.
- The dazing shot and the mark: how they look is not designed here.
- Thread sway in wind. The threads are frozen in one pose.
