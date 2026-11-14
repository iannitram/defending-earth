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
# Job H3: the six portrait stages (design sheet and pictures)

## Goal

Make the six stages of a soldier's close-up portrait, from new recruit to
the thousand-yard stare, on two example heads, with a design sheet.
Folder: `source_assets/blender/soldier/portraits/`.

## The settled design

**What it is.** A soldier's portrait changes with service. The rule is
fixed: **wear builds on the same face. The face shape never changes.**
Only the skin, the eyes, grime and the light change. Each stage keeps
everything from the stage before and adds to it.

**The head.** Use `source_assets/blender/soldier/head/head_lib.py`
(`build_head`, `random_values`). Two example soldiers: seed 11 with skin
tone 2, and seed 23 with skin tone 5. Each wears the plain dark grey cap
from the head job and the collar of grey-green fatigues (`#5c6455`).
Never change a slider value between stages.

**Framing.** Head and shoulders, 768 by 960, the eyes one third down from
the top, camera level with the eyes, an 85 mm lens. Plain backdrop.
Cycles, 96 samples.

**The six stages** (each exactly as written):

1. **New.** Clean skin, even soft light from the front, pale grey
   backdrop. Eyes a little wider than neutral (lids opened about 15
   percent) and looking 6 degrees to the left of the camera, not at it.
   Mouth slightly parted. Collar neat.
2. **Blooded.** The same light. First grime: a smudge on the left cheek
   and a line of dust along the brow under the cap. Lips pressed
   together, jaw set. Eyes back to neutral width, looking at the camera.
3. **Seasoned.** Weathered skin: slightly darker and redder across the
   nose and cheeks, rougher, fine lines at the corners of the eyes.
   Eyes steady on the camera. Dust in the creases of the collar. The key
   light moves 20 degrees to the side.
4. **Worn.** Darker skin under the eyes. Cheeks look hollower through
   shading only (a darker tone under the cheekbone), not through any
   change of shape. A day's stubble as a faint shadow on the jaw (leave
   it out if the face is one that would not grow it: use it on seed 11,
   not on seed 23). Backdrop one step darker.
5. **Hard.** The lines from stage 3 are deeper and there are two more
   between the brows. Mouth flat and closed. The key light is now hard
   and from the side, 70 degrees from the camera, so half the face is in
   shadow, with very little fill.
6. **The stare.** Eyes fixed straight ahead and focused far behind the
   camera, so they look through the viewer rather than at them. Upper
   lids lowered about 10 percent. No expression at all. Colour drained:
   the whole picture's colour strength cut by 45 percent and slightly
   cooled. Backdrop dark grey.

**Event marks (one example only).** Scars are added per wound in the
game. Show one example: a pale scar 3 cm long through the outer end of
the left eyebrow, on seed 11 at stage 4. Earned-trait marks are not
designed in this job.

**Pictures.**
- `s11_stage1.png` to `s11_stage6.png` and `s23_stage1.png` to
  `s23_stage6.png`.
- `s11_stage4_scar.png`.
- `row_s11.png` and `row_s23.png`: the six stages side by side.
- `contact.png`: both rows and the scar picture.

**Files to write besides the pictures.** `sheet.md` (the six stages in
plain words, what changes at each and what never changes, with the exact
numbers used for light angle, lid opening, colour strength and so on, so
the game can copy them later), `stages.json` (the same numbers as data),
`build.py`, `render.py`, `rounds.md`.

## Files you may write

Only inside `source_assets/blender/soldier/portraits/`.

## Done looks like

All pictures exist and you have looked at every one. Put side by side,
the six stages read as one person ageing through a war: the first looks
young and unsure, the last is hard to look at. No stage changes the
shape of the face. Nothing looks like make-up or a horror mask. Three
judging rounds are in `rounds.md`.

## Must not touch

Anything outside that folder, including the head job's folder.
