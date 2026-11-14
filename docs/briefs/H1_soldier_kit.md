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
# Job H1: the soldier's body and issue kit (design sheet, model, check pictures)

## Goal

Build the starting soldier, body and kit, as a Blender model at true
size, with its design sheet and check pictures. Folder:
`source_assets/blender/soldier/body_kit/`. Sheet name: `soldier`.

## The settled design

**What it is.** The soldier the player starts with. Everything the player
starts with looks present-day: what armies have now, nothing near-future.
It is our own design of an ordinary type. One plain multinational kit,
the same in every biome. No flag, no real camouflage pattern, no real
maker's name or badge, no unit patch.

**Body.** Use the CC0 base mesh in `source_assets/mpfb2_cc0/3dobjs/`
(load the mesh file directly with your own script; there is no add-on
and you must not install one). Average adult build, 1.8 m tall standing
in boots and helmet (scale the body so the top of the helmet is at
1.8 m). Use the shape files in `source_assets/mpfb2_cc0/targets/` only
if you need them to get an average, neutral, adult build. If the base
mesh cannot be loaded or posed without the add-on, stop with
`UNDECIDED:` and say exactly what is missing.

**Pose.** Standing relaxed and alert, weight on both feet, feet 0.3 m
apart, rifle held low across the body in both hands, muzzle down to the
left, head level. This is a model pose, not an animation.

**Kit** (all cloth is matte, a little worn, dusty at the knees and
elbows; nothing glossy):
- *Fatigues:* jacket and trousers in one plain muted grey-green
  (`#5c6455`), no pattern. Trousers bloused into the boots, sleeves down.
- *Plate carrier:* a slightly darker green-grey (`#4b5245`), front and
  back plates, shoulder straps, a row of webbing loops across the front.
- *Pouches:* three magazine pouches in a row on the front of the
  carrier, one square medical pouch on the left hip, one small utility
  pouch on the right hip, a radio pouch high on the back of the left
  shoulder with a short stub aerial.
- *Helmet:* an ordinary modern helmet shape with a cloth cover in the
  fatigue colour, a little wrinkled, a plain strap under the chin. No
  goggles, no visor. A short rail stub on each side, empty.
- *Pack:* a small pack of about 25 litres on the back, same green-grey
  as the carrier, with a rolled mat strapped under it.
- *Belt:* a wide belt with a plain canteen on the right rear.
- *Gloves:* dark grey. *Knee pads:* dark grey. *Boots:* dark brown-black
  (`#2a2622`), scuffed.
- *Face:* the lower face is plain skin with no features sculpted beyond
  what the base mesh has; the head and face are designed in another job.
  Keep the face in shadow under the helmet brim in the pictures.
- *Rifle:* a **placeholder**: a plain dark grey rifle-shaped block of the
  right length (0.85 m) with a stock, grip and barrel, clearly simple.
  Weapons are designed in a later phase. Say so in the sheet.

**Size targets.** Height 1.8 m. Width with kit 0.70 m. Depth with pack
about 0.45 m.

**Pictures.** The standard list, with these changes:
- `06_night.png` is replaced by `06_back.png`: straight from behind, day
  light.
- `07_detail.png`: chest, carrier and pouches, close.
- `08_dead.png` is replaced by `08_aerial_squad.png`: six copies standing
  in loose order on six squares, from the aerial camera, to judge how a
  squad reads from the game's view.
- `05_size.png`: beside the grey soldier stand-in from the scale helper
  and a door outline 1.0 m wide by 2.1 m tall.

## Files you may write

Only inside `source_assets/blender/soldier/body_kit/`: `build.py`,
`render.py`, `model.blend`, `sheet.md`, `size.json`, `rounds.md`, and
pictures under `renders/`. Triangle limit: 30,000.

## Done looks like

All pictures and the contact sheet exist. From the aerial camera the
figure reads as a soldier by its silhouette: helmet, pack, carrier,
rifle line. Sizes match within 5 percent. Three judging rounds are in
`rounds.md`.

## Must not touch

Anything outside that folder, including `source_assets/mpfb2_cc0/`.
