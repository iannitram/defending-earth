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
# Job C10: the builder (the Lathe) in four forms (design sheet, model, check pictures)

## Goal

Build the builder as a Blender model at true size, in four forms and a
wreck, with its design sheet and check pictures. Folder:
`source_assets/blender/creatures/builder/`. Sheet name: `builder`. Its
on-screen name is LATHE.

## The settled design

**What it is.** A machine, not a fighter. It mends other machines, breaks
wreckage down into parts, raises cover for the enemy, and, left alone
beside a pool of the taker's fluid, builds a new taker. One of a pair
with the taker: one takes apart, one builds. At its centre is a sphere,
the enemy machine's sign for making.

**The same parts in every form.** Build the parts once and place them
differently for each form. Nothing is added or removed between forms.
- *Core:* a black, exact sphere 0.5 m across, perfectly smooth, with
  amber light only in three hairline seams round it like lines of
  latitude. It never glows as a whole.
- *Blade legs:* six tall curved blades of dark Registrar metal, each
  2.6 m long, 0.35 m wide at the widest and 3 cm thick, curved like a
  long thin leaf or a scythe, pointed at the foot. Fine dull gold
  measuring circles and lines etched on the outer face, amber light in
  one seam along each inner edge.
- *Collar:* a thin black ring 0.7 m across round the core that the tops
  of the blades hang from, without touching it. Gaps are even and exact.
- *Beam:* in the working forms, a hair-thin amber beam from the bottom of
  the core straight down to the work. Thin and steady, not a glowing
  tube.

**Four forms, each a collection in the one file** (sizes from the scale
sheet):
1. *Travel:* closed and armoured. The six blades close together round
   the core into a ball-like bud 1.8 m across, edges meeting, the core
   hidden. It stands on the points of three blades, which reach down
   0.4 m below the bud; the other three clasp over the top.
2. *Repair:* blades spread wide, feet planted on a circle 3.6 m across
   (inside two squares by two), core lowered to 0.6 m above the ground
   and exposed. Under it a broken taker plate (a flat black triangle
   0.75 m long, cracked in two) with the beam on the crack.
3. *Make cover:* four blades planted, two reaching forward and down to a
   low wall it is raising 1.5 m in front of it: black exact blocks, each
   0.5 by 0.25 by 0.25 m, stacked into a wall 2.0 m wide and 1.0 m high
   with the top row half finished. Core at 1.2 m, exposed, beam on the
   last block.
4. *Build a taker:* the blades stand close in a cage 1.6 m across round a
   half-built taker: a black exact cube 0.4 m across with seven of its
   twelve triangular plates hanging in place round it and the other five
   lying stacked on the ground. Core directly above at 2.0 m, exposed,
   beam on the cube. Beside the cage a pool of the Glow (yellow-green,
   shared colour) 1.2 m across.

**No living signs.** No flesh, no frame, no electrical arcs, no wrong
repairs. Exact and clean.

**Wreck.** Shut down: the six blades fallen outward and lying flat like
the petals of a dead flower, the collar dropped, the sphere resting in
the middle whole, its seams dark. The sphere is what the player
recovers.

**Pictures.** The standard list for form 2 (repair, the widest), with
these changes:
- `06_night.png`: form 4 at night, lit by the seams, the beam and the
  Glow pool.
- `07_detail.png`: the exposed core, collar and blade tops in form 2.
- `08_dead.png`: the wreck.
- Add `09_travel.png`, `10_make_cover.png`, `11_build_taker.png` (each
  three-quarter, day) and `12_forms_row.png`: all four forms in a row in
  side view with no perspective, with the soldier stand-in.
Write `size.json` for the repair form.

## Files you may write

Only inside `source_assets/blender/creatures/builder/`: `build.py`,
`render.py`, `model.blend`, `sheet.md`, `size.json`, `rounds.md`, and
pictures under `renders/`.

## Done looks like

All pictures and the contact sheet exist. The same six blades, collar and
sphere are in every form. The sphere is black, exact and smooth in every
picture, and can never be mistaken for the taker's cube. The repair form
fits two squares by two. Three judging rounds are in `rounds.md`.

## Must not touch

Anything outside that folder.
