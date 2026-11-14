# The Bloat: design sheet

## What it is

An animal. A slow floater. It is one gas bladder with a thin skin.

## What it is for

It bursts into a thick choking cloud when it is shot, or when it reaches
its target. The cloud blocks sight. The player can use this too, by
bursting it over the enemy. (The cloud itself is an effect for a later
job. It is not in this model.)

## How it looks

One stretched sac, a little taller than wide, widest below the middle.
The skin is pulled tight over a dark frame, so it sits in tight pillows
between the ribs. It looks ready to pop. From above it is one round pale
amber ball with dark ribs showing through. It floats, and five limp
feelers hang under it.

## Sizes

From the scale sheet (`bloat`):

| Size | Value |
|---|---|
| Sac, across | 1.5 m (`diameter_m`) |
| Underside above the ground | 1.2 m (`underside_height_m`) |
| Feeler length | 0.8 m (`feeler_length_m`) |
| Footprint | 1 by 1 squares. It hangs above its square. |

From the job text: plug 8 cm across, patch 30 cm across, frame lattice
cells 6 cm.

Chosen as a shape, not as a size: the sac is 1.12 times as tall as it is
wide (1.68 m), and it is widest 40% of the way up. So the top of the sac
is 2.88 m above the ground and the plug stands 2.4 cm above that.

Measured from the finished model (`size.json`): 1.50 m wide, 1.51 m deep
(the patch and its stitches stand 1 cm proud at the front), 2.47 m from
the feeler tips to the top of the plug, 35,148 triangles. The feeler tips
hang about 0.44 m above the ground.

## Parts

All parts hang from the root object `bloat`, which sits on the ground at
the centre of the square.

| Object | What it is |
|---|---|
| `sac_skin` | The skin. Its origin is at the centre of the widest plane. |
| `frame_ribs` | Eight ribs from top to bottom, one ring round the widest part, and a small hub at the bottom. Inside the skin. |
| `murk_inside` | The faint cloud inside. Denser toward the bottom. |
| `veins` | The larger veins, as thin strips lying on the skin. |
| `wrong_repair_patch` | The patch. Origin at its centre on the skin. |
| `wrong_repair_seam` | The stitches. Child of the patch. |
| `feeler_1` to `feeler_5` | The feelers. Each has its origin where it joins the skin. |
| `control_plug` | The black plug on the very top. |
| `control_lens` | The amber lens in the plug. Child of the plug. |
| `rib_arcs` | Electrical life. Hidden except in the night picture. |

## Colours and surfaces

- **Skin:** amber-grey `#b89e78`. Thin and see-through: most when you
  look straight at it, least at the edge. Daylight passes through it.
  A slight sheen in the middle of each pillow, where it is tightest.
  Small creases beside the ribs. A net of hair-thin red lines in the
  skin, stronger low down. A little darker and denser at the bottom.
- **Veins:** dark red `#8c1f1a`. They start thick at the bottom and
  branch and thin toward the top. They stop at the patch.
- **Stretch marks:** three pale streaked patches (`#ddd0b4`): upper
  front left, lower front right, and one at the back.
- **Frame:** the shared SPINE material, `#14171a`, a regular lattice
  with 6 cm cells, slightly wet.
- **Feelers:** the same amber-grey, darker, going to dark brown at the
  tips.
- **Plug:** Registrar black `#0b0b0c`, twelve exact flat sides.
- **Wrong repair:** a rough oval of thick, pale, opaque skin (`#d6c8aa`,
  shown darker and leathery), 30 cm long, set askew on the front, above
  the ring. Thirty uneven slanted dark stitches hold it in.

## Lights

- **Amber lens** `#ffa01e`, emission 6. On top of the plug, facing up.
  It shows from the air, not from the side.
- **Electrical life** `#cfe4ff`, emission 4. Hair-thin broken lines along
  two of the ribs, inside the skin. Night picture only.
- No other lights. The skin does not glow.

## Dead form

Collection `dead`, root object `dead_bloat`. Burst. The frame has
collapsed and fallen to one side: the ribs lie bent on the ground, a few
still arching up to about 0.25 m, two of them kinked. The skin is one
torn sheet draped over them, split open across the middle and ripped in
from the edge in five places. The five feelers lie flat, fanned out from
the bottom hub. The plug lies on its side with the lens dark. A few small
scraps lie clear. A damp, blotchy stain lies round it all.

The skin and frame are 1.95 m across, so they fit one square. The stain
is about 2.5 m across and spills over the square's edge. 16,966
triangles. No cloud.

## Check pictures

In `renders/`: front, side, three-quarter, aerial, size, night, detail
(the lower half seen against the light), dead, and the contact sheet.

## Left for later

- Rig and motions: drift, the feelers swaying, swelling before a burst,
  the burst itself.
- The choking cloud (an effect).
- Baked textures. All colour is procedural now. The veins are real
  strips so they can be baked into a texture later.
- A lower-detail version for the game. The veins alone are 9,200
  triangles and can go once they are baked.
