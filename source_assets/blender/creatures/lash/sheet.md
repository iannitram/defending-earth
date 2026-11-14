# Lash: design sheet

## What it is

A long, many-legged alien marksman. It lies low on seven pairs of legs,
lifts its front end, and aims a very long rifle held in two arms.

## Purpose

A long-range shooter. It should read at once as: low body, tall knees, a
raised front, a long gun on a bipod.

## The look

- A segmented trunk, low to the ground, with a blunt tail.
- Leg knees stand higher than the back.
- The front two segments rise up to carry the mask and arms.
- A pale skull-like mask with no eye holes and no mouth. A red target sign
  is painted on it.
- A black halo of four arcs with short spikes, bolted over the brow.
- Long spines lean back along the trunk. Red sacs sit between the plates.
  Weed hangs from the body.
- Bells and prayer strips hang from the rifle and the front legs.
- Two legs are wrong repairs: straight black struts where legs should be.

## Parts

All parts are children of the root object `lash`.

- Trunk: nine segments, each its own object.
- Legs: seven pairs. Each leg has an upper and a lower part, with the
  origin at the joint.
- Wrong repair: the third right leg and the fifth left leg are straight
  black Registrar struts with flat pads and visible plugs.
- Head: mask, halo (four arcs with spikes), two amber lenses.
- Arms: two, each with upper arm, forearm and hand.
- Shoulder plate on the left shoulder, with the target sign.
- Back: fourteen spines, eight red sacs, weed strands.
- Rifle: body, barbs, long scope, bipod of two crossed staves.
- Devotion items: six brass bells and eight prayer strips with red marks.
- Night arcs along the spines (`arcs_spines`), hidden except at night.
- Dead form: a second set under the root `dead_lash_dead`, hidden from
  renders unless asked for.

## Sizes

From the scale sheet, entry `lash`:

| Thing | Scale sheet | Model |
| --- | --- | --- |
| Footprint | 1 by 2 squares | stance fits; rifle and bipod reach past the front |
| Height | 1.7 m | 1.7 m |
| Width (stance) | 1.9 m | 1.9 m |
| Trunk length | 3.5 m | 3.502 m |
| Back height | 0.7 m | 0.7 m |
| Rifle length | 2.6 m | 2.6 m |

Other sizes, from the job text:

- Trunk width 0.45 m. Arms 1.0 m. Mask 0.3 m. Halo 0.5 m.
- Spines 0.6 to 1.1 m. Sacs 8 to 12 cm. Bipod staves 1.2 m.
- Prayer strips 25 cm.

Measured extras:

- Length with the rifle: 5.135 m. Without the rifle: 3.67 m (the spines
  reach past the tail).
- Triangles: 30,730 (limit 40,000). Dead form: 30,230.

## Colours

| Part | Colour |
| --- | --- |
| Hide | `#b89a94`, with thin red lines |
| Sacs and red signs | `#8c1f1a` |
| Weed | `#3c4630` |
| Bells | `#9c7a32` |
| Prayer strips | `#d8d2c2` |
| Registrar struts, halo | `#0b0b0c` |
| Amber lenses | `#ffa01e` |
| Night arcs | `#cfe4ff` |

Chosen by the modeller, not in the job text:

| Part | Colour |
| --- | --- |
| Mask and shoulder plate (bone) | `#cfc6b0` |
| Rifle metal | `#26282a` |
| Bipod staves | `#4a3d2c` |
| Cord | `#30291f` |

## Lights

- Two amber lenses on the halo, strength 6. They sit one above the other
  on a tab over the mask, so they do not read as eyes.
- Pale blue arcs along the spines, strength 4. Night picture only.
- Nothing else glows.

## Dead form

It lies on its side with the trunk curled. The legs fold in over the
belly. The rifle lies beside it. The mask faces up. The lenses are dark
and there are no arcs.

## Left for later

- No rig and no motion. Moving parts are separate objects with the origin
  at the joint.
- No game-ready textures. All surfaces are procedural.
- The dead legs could curl tighter.
- The hide is still pale in strong sun.
- No lower-detail version for distance.
