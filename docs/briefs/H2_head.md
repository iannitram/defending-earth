# Rules for every job (read all of this first)

You are one run in a larger piece of work on the game Defending Earth
(Rust, macroquad). The project folder is `/home/deck/dev/Defending Earth`.
A lead has already made every design decision. You carry the brief out.

- **Do not ask questions.** Nobody is there to answer. If the brief does not
  cover a choice that changes what the result looks like, what it is called,
  or how a rule works: **stop**, change nothing further, and end with a
  report that starts with the line `UNDECIDED:` and states the question.
  Do not guess. Small technical choices (variable names, mesh density, how
  to call a Blender function) are yours.
- **Write only the files listed under "Files you may write".** Read anything
  in the project. Never write anywhere else in the project.
- **Never commit, push, stash, reset or otherwise touch git.**
- **Never launch the game** (`defending_earth`, `lightlab`). The game opens
  a window on the owner's screen. Only the lead launches it.
- **Never use a sub-agent, the Agent tool or a workflow.** Do the work
  yourself in this run.
- **Never use the Blender MCP tools or connect to an open Blender.** Blender
  runs only in the background from the shell (see the Blender section if
  your job has one).
- **Original work only.** Never name, copy or allude to an existing
  commercial game, film or real military unit. No real flags, badges,
  camouflage patterns, maker's names or real people's names or faces.
- **The repository is public.** Never write personal details, keys or
  passwords into any file.
- Plain words in every document you write: short sentences, no jargon
  without a one-line explanation. Times are written like 7am and 4:45pm.
- Pictures in `reference/` and `docs/lore_refs/` are idea references only.
  You may look at them. Build from the description in this brief, never
  copy a picture.

## How to finish

End with a short plain report, in this order:
1. `DONE`, `PARTLY DONE` or `UNDECIDED:` on the first line.
2. Every file you wrote, one per line.
3. What you checked and the result (commands run, numbers measured).
4. The three weakest things in the result, honestly.
5. Anything the lead must decide.
# Blender rules for this job

- Blender is the Flatpak build, version 5.3. There is no `blender` command.
  Run it only like this, from the project folder:
  `flatpak run org.blender.Blender -b --python <script.py> -- <arguments>`
  It can read and write anywhere under `/home/deck`.
- The Python API may differ from what you remember. When a call fails, look
  it up by introspection inside Blender (`dir()`, `bl_rna.properties`),
  do not guess twice.
- **Everything is built by your script.** One script, `build.py`, in your
  folder, builds the whole model from nothing and saves `model.blend`. A
  second script, `render.py`, opens `model.blend` and writes the check
  pictures. Running both again must give the same result.
- **Sizes come from the scale sheet**, never typed into your script. Read
  `source_assets/scale/scale_sheet.json` through the helper
  `source_assets/scale/scale_lib.py` (add its folder to `sys.path`). If a
  size you need is not in the sheet, that is an `UNDECIDED:` stop. (The
  job that makes the scale sheet is the one exception.)
- Units are metres. Z is up. The model stands on the ground at Z = 0, its
  footprint centred on the origin, facing −Y (toward Blender's front view).
- Geometry: meshes made by script (bmesh, primitives, skin and subdivision
  modifiers, curves, metaballs turned into mesh). Materials: Principled
  BSDF with procedural textures (noise, Voronoi, wave, gradients). No
  downloaded textures, no image files. Baking waits for a later job.
- Size limit: at most 40,000 triangles for one creature or figure after
  modifiers are applied, unless the brief gives another number. In the game
  it is seen 30 to 80 pixels tall, so the silhouette and the large shapes
  matter most; fine detail goes into the material.
- Every part gets a clear object name. Parts that will move later (limbs,
  jaws, plates, lenses) are separate objects parented in a sensible tree,
  with their origin at the joint. No rig, no animation.

## Shared colours (use these exact values)

| Thing | Colour | Surface |
|---|---|---|
| The frame under enemy flesh (the material SPINE) | `#14171a` | regular lattice with 6 cm cells, slightly wet, roughness 0.3 |
| Registrar hardware (spike, rings, cores, plates) | `#0b0b0c` | exact flat faces, sharp edges, roughness 0.35, metallic 1 |
| Etched measuring circles on Registrar plates | `#8a6a2a` | thin lines only, dull gold, never glowing |
| Amber lens or seam light (Registrar only) | `#ffa01e` | emission strength 6 |
| Electrical life (living creatures only) | `#cfe4ff` | tiny thin arcs at joints, emission 4; shown only in the night picture |
| The Glow (the taker's fluid) | `#c8e61e` | emission 5, wet |
| Magma and fire | `#ff5a0c`, white-hot `#fff2d0` at the deepest point | emission 8 to 30 |

Amber belongs only to Registrar hardware. Yellow-green belongs only to the
Glow. Orange belongs only to fire, the Maul and Cinders. Never mix them.
Nothing is glossy plastic, chrome or neon. No glowing outlines. No faces:
no eyes, mouths or expressions on any enemy.

## Check pictures (write these into your folder's `renders/`)

Render with Cycles on the CPU, 1600 by 900, 64 samples, denoising on, view
transform AgX. Keep any one picture under about three minutes; lower the
samples before lowering the size. Use `scale_lib.py` for the ground, the
light and the soldier stand-in so every job's pictures match:
- Ground: `scale_lib.make_ground()` (dry soil with faint lines every 2 m
  that show the battlefield squares).
- Day light: `scale_lib.day_light()`. Night: `scale_lib.night_light()`.
- Soldier stand-in, 1.8 m: `scale_lib.add_soldier(x, y)`.

Pictures every model job must write, unless the brief changes the list:
1. `01_front.png`, `02_side.png`, `03_threequarter.png`: day light, camera
   at the model's mid height, the whole model in frame.
2. `04_aerial.png`: the game's view. Camera 45 m away, looking down at 40
   degrees, 135 mm lens, day light. The footprint squares are drawn on the
   ground in a thin white line.
3. `05_size.png`: straight side view, no perspective (orthographic), with
   the soldier stand-in beside the model and a 1 m scale bar.
4. `06_night.png`: three-quarter view, night light, so only the model's
   own lights and a little moonlight show.
5. `07_detail.png`: a close view of the part named in the brief.
6. `08_dead.png`: the dead or wrecked form, three-quarter view, day light.
7. `contact.png`: all of the above on one sheet with their names.

## Judging (you do this before you finish)

Work in at least three rounds. In each round: render, **open and look at
every picture** with the Read tool, judge it as a harsh art director would
judge a still from aerial news footage, write down the three weakest
things, fix them, render again. Keep the notes in `rounds.md` in your
folder: per round, the three weakest things and what you changed. Typical
failures to look for: a shape that reads as a toy or as smooth plastic;
parts that float or pass through each other; sizes that do not match the
sheet; a silhouette that is unclear from the aerial view; lights that are
too even or too bright; anything that looks like a face.

## Other files every model job writes in its folder

- `sheet.md`: the design sheet in plain words. What it is, what it is for,
  how it looks, its parts, its sizes (quoted from the scale sheet), its
  colours, its lights, its dead form, and what was left for later (rig,
  motions, baked textures).
- `size.json`: measured from the finished model by your script:
  `{"name": ..., "width_m": ..., "depth_m": ..., "height_m": ...,
  "footprint_squares": [w, d], "triangles": ...}`. Width is along X, depth
  along Y.

## Added after the trial run (these override anything above that differs)

- **Starting Blender.** The Flatpak starts in its own folder and cannot
  read `/tmp`. Always give the script's full path, and keep every file
  you need inside your own folder in the project:
  `flatpak run org.blender.Blender -b --python "/home/deck/dev/Defending Earth/<your folder>/<script>.py"`.
  Delete any `__pycache__` folder Blender leaves behind.
- **The helper library exists and is finished:**
  `source_assets/scale/scale_lib.py`. Read its `README.md` first and use
  its calls as they are. Do not edit anything in `source_assets/scale/`.
  `make_ground` takes `footprint=(w, d)` so the square lines sit on the
  edges of your model's footprint.
- **Sun side.** "Front left" means the viewer's left when looking at the
  model's front: the sun is over the −X, −Y quarter.
- **Heights of things in the air.** `flight_height_m`, `hover_height_m`
  and `underside_height_m` in the sheet all mean the height of the
  lowest point of the body above the ground (hanging feelers and the
  like do not count). Build the model in place at that height.
- **Sizes the sheet does not give** (a thickness, a standing height) come
  from your job's own text below. If your job's text does not give one
  either, choose nothing: stop with `UNDECIDED:`.
# Job H2: the head built from sliders (design sheet, model, check pictures)

## Goal

Build the soldier's head as a Blender model that is shaped by a fixed
list of sliders, with its design sheet and check pictures. Folder:
`source_assets/blender/soldier/head/`.

## The settled design

**What it is.** In the game a soldier's head is built once from slider
values, and the soldier's close-up portrait is a picture of that head.
This job makes the head, the slider list and the proof that each slider
works. It does not make the game's head builder.

**Base.** The CC0 base mesh in `source_assets/mpfb2_cc0/3dobjs/` and the
CC0 shape files in `source_assets/mpfb2_cc0/targets/`. Load them
directly with your own script (a shape file is a list of vertex numbers
and offsets). There is no add-on and you must not install one. Keep the
head, neck and the top of the shoulders; cut the rest away.

**The slider list is fixed: 24 face sliders in six groups of four, plus
age and weight.** Each slider runs from −1 to +1 with 0 as the neutral
face.

| Group | Slider | −1 | +1 |
|---|---|---|---|
| Head shape | Head width | narrow | wide |
| Head shape | Head height | short | long |
| Head shape | Head depth | shallow front to back | deep |
| Head shape | Head taper | wider at the jaw | wider at the brow |
| Brow and eyes | Brow height | low | high |
| Brow and eyes | Brow ridge | flat | heavy |
| Brow and eyes | Eye size | small | large |
| Brow and eyes | Eye spacing | close | wide apart |
| Nose | Nose length | short | long |
| Nose | Nose width | narrow | wide |
| Nose | Nose bridge | low | high |
| Nose | Nose tip | turned down | turned up |
| Mouth and lips | Mouth width | narrow | wide |
| Mouth and lips | Mouth height | low on the face | high on the face |
| Mouth and lips | Upper lip | thin | full |
| Mouth and lips | Lower lip | thin | full |
| Jaw and chin | Jaw width | narrow | wide |
| Jaw and chin | Jaw angle | soft | square |
| Jaw and chin | Chin length | short | long |
| Jaw and chin | Chin forward | set back | forward |
| Cheeks and ears | Cheekbones | flat | high and strong |
| Cheeks and ears | Cheek fullness | hollow | full |
| Cheeks and ears | Ear size | small | large |
| Cheeks and ears | Ear angle | flat to the head | standing out |
| Whole head | Age | 20 | 55 |
| Whole head | Weight | lean | heavy |

For each slider, pick the shape file or pair of shape files in
`targets/` that does this and nothing else, and record the file names in
`sliders.json`. Choosing which file fits a slider is your job. **If no
shape file fits a slider, do not invent a substitute and do not drop the
slider: list it in your report under `UNDECIDED:` with the nearest files
you found**, and carry on with the others. There is no sex slider and no
ethnicity slider: the 26 sliders and the skin tone are the whole range.
Limit each slider so that no extreme, alone or combined with any other
extreme, gives a broken or monstrous face.

**Skin.** A procedural skin material: six tones from light to dark
(`#e8c9b0`, `#d4a888`, `#b98a66`, `#96694a`, `#6f4a33`, `#4a3122`),
with subsurface scattering, slight redness at the nose, cheeks and ears,
fine pore-scale roughness, lips a little darker. Eyes: simple eyeballs
with five iris colours (dark brown, brown, hazel, grey, blue). Slightly
stylised to stay out of the uncanny valley: clean forms, no photographic
blemishes.

**Hair.** Not in this job. Every head is shown with a plain close-fitting
dark grey cloth cap so the skull shape still reads. Say in the sheet
that hair styles are a later job.

**No real people.** Every example face comes from random slider values
from a numbered seed. Never shape a face after a real person.

**Pictures** (photo-booth light: one soft key from the front left, weak
fill, plain mid-grey backdrop; head and shoulders; 768 by 960; Cycles,
96 samples):
- `01_neutral_front.png`, `02_neutral_side.png`,
  `03_neutral_threequarter.png`: all sliders at 0, skin tone 3.
- `04_sliders_group1.png` to `09_sliders_group6.png`: one sheet per
  group; each row is one slider shown at −1, 0 and +1, front view, with
  the slider's name beside it.
- `10_age_weight.png`: age and weight at −1, 0, +1.
- `11_twelve_faces.png`: twelve random faces from seeds 1 to 12 with the
  skin tones spread evenly, on one sheet, seed number under each.
- `12_extremes.png`: six faces from seeds 101 to 106 with every slider at
  either −1 or +1, to prove that nothing breaks.
- `contact.png`: everything on one sheet.

**Files to write besides the pictures.**
- `sheet.md`: the design sheet. It must contain the slider table above
  with one more column, "What you see", written after looking at the
  result, so the owner can check the list.
- `sliders.json`: for each slider its name, group, the shape file or
  files used, and the limit applied at each end.
- `head_lib.py`: `build_head(values, skin_tone, iris, seed=None)` builds
  a head object from a dictionary of slider values, so later jobs (the
  portraits, the dress uniform) can make the same head. `random_values
  (seed)` returns the values for a seed.
- `build.py`, `render.py`, `model.blend` (the neutral head), `rounds.md`.

## Files you may write

Only inside `source_assets/blender/soldier/head/`.

## Done looks like

All pictures exist and you have looked at every one. Every slider visibly
does what its name says and nothing else. The twelve faces look like
twelve different plausible people, and none looks like a doll or a
corpse. Three judging rounds are in `rounds.md`.

## Must not touch

Anything outside that folder, including `source_assets/mpfb2_cc0/`.
