# Judging rounds for job S0 (scale sheet, helper, size chart)

Each round: render all four pictures, look at every one, name the three
weakest things, fix them, render again.

## Round 1

Weakest things:

1. **The night picture was pure black.** The soldier could not be seen at
   all. The moon was about twenty times too weak.
2. **The stand-in read as a thin toy figure.** Square coat-hanger
   shoulders, spindly arms and legs, bulging hips.
3. **The ground was too bright and the square lines too loud.** The soil
   came out pale beige, far lighter than `#6b5f4e`, the stand-in was nearly
   white, and the lines read as paving slabs. A dark band also showed past
   the edge of the ground in the level view.

Also seen: on the chart two notes ran into each other, the blocks were
hard to tell from the wall, the Maul's block cast a shadow on the Bloat's,
and the top third of the chart was empty.

Changed:

- Moon strength 0.022 to 0.45, night sky a little brighter.
- Stand-in rebuilt: sloping shoulders, thicker chest, arms and legs,
  narrower hips, thicker neck.
- Day sky strength 0.11 to 0.05. Line darkness 0.42 to 0.28, and the lines
  now fade in and out with the soil. The plain soil now runs 2 km each way
  so the horizon is real.
- Chart: shorter notes, wider slots, darker blocks, blocks cast no shadow,
  chart made less tall, a white mark under each creature shows its squares.

## Round 2

Weakest things:

1. **A jagged crease ran across the stand-in's chest and armpit.** The arm
   and an extra back piece lay too close to the chest surface.
2. **The chart's name band and scale bar were black.** They sit below
   ground level, so the ground shaded them from the sun.
3. **The soil had large cloudy blotches and was too orange.** It looked
   like cloud shadow more than dry earth.

Changed:

- Stand-in: arms moved 1.5 cm out, chest a little narrower, finer skin.
- Chart: the ground on the chart casts no shadow, so the band is lit.
- Soil: the light and dark tones now keep the hue of `#6b5f4e`, the big
  patches are smaller, softer and more broken.

Every helper call was also tried in a scratch folder outside the project:
all six cameras, the footprint outline, both scale-bar directions, night
after day in one run, the contact sheet, `measure`, `write_size_json` and
the two error messages of `thing()`. All worked.

## Round 3

Weakest things:

1. **A stair-step seam was still there where the shoulders meet the chest**
   (seen in the close view). The stand-in also had 36,800 triangles, which
   is a lot for a stand-in.
2. **On the chart the flyers' blocks hung in the air with nothing to read
   their height from.**
3. **The stand-in's chest is still a plain barrel.** Left as it is: it is a
   stand-in, and it reads as a person at the game's distance.

Changed:

- Stand-in: the extra back piece removed, shoulders sunk deeper into the
  chest, more smoothing, then thinned to about 12,800 triangles. The seam
  is gone.
- Chart: a thin post under the Kite, the Bloat and the taker.

## Final check

- `soldier_standin.png`: reads as a standing person, no face, no seam.
- `ground_day.png`: squares visible but quiet, clear lit side and shadow.
- `ground_night.png`: dark, the soldier and the lines just visible.
- `size_chart.png`: all eleven names clear, no overlaps.
- The chart was rendered twice. Both files have the same SHA-256 sum.
- Stand-in measured: 1.800 m tall, 0.627 m wide over the arms, 0.303 m
  deep, 12,790 triangles. Shoulders fitted to 0.55 m.
