# DEFENDING EARTH — VISUAL DESIGN

Working draft 1, 2026-11-13. This is the rulebook every frame is judged against.
Names marked *(working)* are proposals, kept in one table so they are cheap to change.
Notes marked *As built* (dated) say where the game knowingly differs from a rule
or has not reached it yet. The rule itself stands.

## 1. The medium: aerial documentary footage

Every tactical frame is a still from a gyro-stabilised camera on a helicopter
orbiting the site at 300–500 m slant range. Nothing is drawn that this camera
could not have recorded.

### Lens
| Rule | Value |
|---|---|
| Focal length | A 70–200 mm zoom (full-frame equivalent). Vertical field of view 19.5° at widest, 6.9° at longest, 14° default. The old camera was 44°, which read as a toy diorama. *As built 2026-11-13:* battles use the 14° default. |
| Zoom | The mouse wheel changes focal length, not distance. The helicopter keeps its range. *As built 2026-11-13:* not yet. The lens stays at 14° and the wheel still moves the camera nearer or further. |
| Compression | Follows from the long lens: buildings at the back are nearly the same size as those at the front; verticals stay near-parallel. |
| Depth of field | Almost none visible. At 400 m and f/5.6 the whole site is sharp. Only at 200 mm does the far edge of the frame soften by about one pixel. No tilt-shift blur, ever. |
| Vignette | 0.4 stop at the corners at 70 mm, 0.2 stop at 200 mm. Optical falloff (cos⁴-shaped), not a dark frame. *As built 2026-11-13:* a fixed 0.25 stop in battles, because the focal length does not change yet. |
| Chromatic aberration | Lateral only, 1 px at the corners at 1080p, zero in the centre. |
| Stabilisation | Slow drift of ±0.05° over 4–8 s. No handheld shake. A hard hit nearby does not shake a camera 400 m away; an explosion's pressure wave may nudge it once, late (sound-speed delay). |
| Shutter | 1/100 s. Rain draws as streaks of that length. Rotor wash and fast debris get motion blur; walking soldiers do not. *As built 2026-11-13:* rain is drawn as 1/30 s streaks. At 1/100 s the streaks were so short they read as white specks from this height. Snow and dust keep 1/100 s. Motion blur is not built yet. |
| Sensor | Broadcast camera response: neutral colour, a soft shoulder into white, no teal-and-orange grade. Highlights clip; they do not glow. |

### Atmosphere
- Aerial perspective is always on. Contrast and saturation drop with distance
  along the view ray; the far side of the map is measurably flatter and bluer
  (or greyer, in humid air) than the near side.
- Haze density comes from humidity, which comes from biome, season and weather.
  Dry farmland at noon: visibility 40 km. Jungle after rain: 6 km. Fog: under 400 m.
  *As built 2026-11-13:* haze follows biome and weather (thin over desert,
  mountain and ice, thick over rain forest; rain, snow and dust add haze of
  their own). Season does not change it yet, and there is no fog weather.
- Scattering is directional. Looking toward the sun the haze is bright and warm;
  looking away it is thin and blue.
- Rain, fog and smoke are lit volumes. A flare inside smoke lights the smoke
  from within. Headlamps draw visible cones only when there is something in
  the air to scatter them (rain, fog, smoke, dust), never in clear air.
  *As built 2026-11-13:* rain, snow and dust are drawn in 3D as thin streaks,
  flakes and low blowing sheets, with extra haze added to the air. They are
  not lit volumes yet. Lamps and flares come in graphics phase 3.
- Lightning lights the whole map for 2–3 frames, overexposes, and the exposure
  takes a second to recover.
  *As built 2026-11-13:* lightning happens only in heavy rain and brightens
  the sky light for a frame or two. The full-frame flash is not wired in yet.

### Exposure (the camera operator)
- Auto-exposure, centre-weighted, with 1.5 s of lag. It never reaches a perfect
  result instantly.
- Dusk: the operator opens up, then adds gain. Gain adds noise and drains
  colour. Full night is a noisy, low-saturation picture in which the only clean,
  coloured things are light sources and what they touch.
- Fire, flares and muzzle flash blow out to white at the core, with colour only
  at the edges. Bloom is narrow and only on values that actually clipped.
- Storm: flat, low contrast, exposure hunting slightly as cloud density changes.
- *As built 2026-11-13:* on snow the operator holds the exposure up, by up to
  half a stop at full cover (`ev_bias` in `gfx::Lens`), so snow reads white,
  not grey.
- No flat ambient fill. Shadow areas are lit by the sky (blue), by bounce from
  the ground (the ground's colour) and by nothing else.

### Surfaces
- Nothing is one colour or one roughness. Every surface carries large-scale
  variation (field patterns, drainage, wear paths) and small-scale breakup.
- No visible tiling. Ground materials are blended by world-space masks, never
  repeated on the game grid. The grid shows only as a faint overlay when the
  player needs it.
- Wetness is a state, not a material. Rain darkens albedo, lowers roughness,
  fills low ground with puddles over the turns, and dries from high ground first.
  *As built 2026-11-13:* the ground is fully wet while rain falls and dry
  otherwise. Puddles and drop rings are drawn by the ground shader, with small
  splashes where drops land. Filling over the turns and drying are not built,
  and puddles read as pale patches, not yet as mirrors of the sky.
- The battle writes onto the ground and stays: footprints in mud, crushed grass
  along walked routes, tyre and skid marks, blood, scorch, craters, ash.
- Edges are worn, straight lines are slightly off, roofs are stained, concrete
  is patched. Clean CG is a bug.
- *As built 2026-11-13:* snow and sand are states of the ground too, set
  through the shader value `surf` in `src/gfx.rs`. Snow cover is a share from
  0 to 1 that follows biome, season and snowfall. It lies on ground, roofs,
  trees and crates, patchy when thin and joined into drifts when deep. Desert
  ground is sand with slow dunes and wind ripples.

### What we refuse
Plastic default materials. Uniform tiling. Bloom on everything. Cartoon fire.
Flat ambient. Rim lighting on characters. Glowing outlines in the world (unit
selection lives in the HUD layer and is as thin as it can be).

## 2. Where the game is now (judged against the above)

Source: `--demo tactical` and the default situation map, captured 2026-11-13.

*As built 2026-11-13, later the same day:* this list is the starting point,
kept for the record. Graphics phases 1 and 2 have since fixed points 1 to 3
for battles (real sun and shadows, photographed ground grain, the long lens,
haze, 3D weather) and point 4 on the globe. One part of point 3 is left: the
old rectangle vignette and the scanlines are still drawn over every screen,
battles included (`src/main.rs`). The current state and its known flaws are in
`STATUS.md`.

1. **No light.** Tactical is one untextured, vertex-coloured mesh rebuilt each
   frame. Face brightness is a fixed multiplier; the sun direction is computed
   and never used. Shadows are dark ellipses.
2. **No surfaces.** No textures at all. Ground is flat green with a tile grid.
3. **No lens or atmosphere.** 44° field of view, linear fog capped at 40%, a
   vignette made of 112 rectangles and scanlines over the whole screen.
4. **Situation map:** the globe has a good base (albedo, atmosphere rim, clouds) but
   the sun is frozen in place and not tied to the clock, city lights are noise
   rather than real cities, and the land appears mirrored east–west.

## 3. Engine: what we keep and what we add

The engine stays: Rust, macroquad 0.4.16 on miniquad 0.4.11, OpenGL. Nothing in
this design needs a swap. Checked in the miniquad source:

| Need | Available |
|---|---|
| HDR scene buffer | RGBA16F render textures |
| Several outputs per pass | `new_render_pass_mrt`, up to 8 attachments |
| Shadow maps | Depth textures can be sampled; filtering is done in the shader |
| Environment maps | Cubemap textures load (cannot be rendered into; ours are baked, so fine) |
| Modern shaders | GLSL 3.30 works in custom materials on this GPU |
| Mipmaps | Yes, through the raw texture id |
| Instancing | Yes, through miniquad directly (for grass, rain, debris) |

Limits to design around: only one float format (RGBA16F), no hardware shadow
comparison, macroquad's batcher clamps large meshes (*as built:* `main.rs`
raises its limit to 65,535 corner points and 400,000 index entries per draw;
65,535 is the ceiling and must not be raised), and the game cannot run
without a display, so each screenshot briefly opens a window on your desktop.

Frame plan (tactical):
1. Shadow pass: sun or moon, one 4096² map fitted to the 26×26 field.
2. Scene pass into HDR: static pieces with baked AO / bent normal / bounce
   atlases, lit by the current sky state; dynamic objects lit by the same sky
   probe plus contact shadows.
3. Dynamic lights: up to 4 shadow-casting (nearest flares, fires, flashlights),
   up to 24 without shadows, culled per tile.
4. Volumes: smoke, fog, rain, lit by 1–3.
5. Lens: exposure, shoulder, vignette, chromatic aberration, sensor noise,
   narrow bloom on clipped values.
6. HUD on top at native resolution, untouched by the lens.

Budget: 16.6 ms at 1920×1080. If over, the scene buffer drops to 1600×900 and
is upscaled before the HUD; effects are cut only after that.

Blender is the bake tool (AO, bent normals, bounce, sky states). Blender 5.2 is
installed (Flatpak) but was not running when this was written; baking needs it
open with the MCP add-on started.

## 4. Light and time

- One sun position function drives everything: tactical sky, weather choice and
  the situation map terminator. Inputs: date, time, latitude, longitude. The
  terminator tilts with the season.
  *As built 2026-11-13:* the sun drives the battle sky and the globe. Weather
  is picked from the site's biome, season and hour, with a dice roll. The
  biome comes from the world map (`assets/earth_data.png`), which holds seven
  land biomes: temperate, farmland, mountain, rain forest, desert, ice and
  northern forest. Season follows the month and the hemisphere. The tropics
  count as summer all year, and the HUD there says WET SEASON or DRY SEASON.
- Nine baked sky states per biome (dawn, morning, noon, afternoon, golden hour,
  dusk, night, overcast, storm). The game blends the two nearest as turns pass.
  A mission that starts at dusk ends in the dark.
  *As built 2026-11-13:* one set of sky states for all biomes, not one per
  biome. Dawn, morning, noon, afternoon, golden hour and dusk are measured
  from Blender renders. Night, overcast and storm are still set by hand. The
  sky is fixed for the length of a battle; it does not move on with the turns
  yet.
- Night is dark. Moon phase follows the date; a new moon under cloud is nearly
  black and the squad's lights are the picture.
- Suit lights switch on when scene luminance at the soldier falls below a
  threshold, staggered by a few frames per soldier, not all at once.

### Situation map
- Day and night follow the game clock and date. The terminator is a soft band
  with a thin warm edge, as seen from orbit.
- City lights come from real night-light data (NASA Black Marble, public
  domain), not noise. Each city switches on as civil twilight reaches it, large
  cities first and brighter, with a few minutes of spread so a coast lights up
  as a wave.
- A site's light on the globe always agrees with its light on the battlefield.
- Fix the east–west mirroring before anything else.
- *As built 2026-11-13:* the mirroring is fixed, the globe's sun follows the
  campaign clock, and real city lights come on as night passes, big cities
  first.

## 5. The world's design line

### Humans: present day, then pulled forward
The war starts with what exists today, filmed as it is. Research into recovered
material changes both what the gear does and how it looks, in three steps that
must read as one family:

| Step | Reads as | Shapes | Materials |
|---|---|---|---|
| 1. Issue | Today's military and police kit | Chunky, bolted, strapped, rails and pouches | Nylon, anodised aluminium, painted steel, scuffed polymer |
| 2. Adapted | Human kit rebuilt around recovered parts | The same silhouettes with foreign cores: sealed housings, cable runs, heat sinks, field repairs | Above, plus dull grown-metal inserts that do not match anything around them |
| 3. Derived | Human design that has absorbed the foreign method | Fewer parts, continuous surfaces, no fasteners, weight where balance wants it | Grown metal as structure, ceramic skins, soft-sealed joints |

Rules: every later item keeps one recognisable feature of its ancestor (a grip
angle, a stock line, a helmet brow). Nothing becomes glossy, chrome or neon.
Step 3 is quieter than step 1, not louder.

The same three steps apply to base modules, craft and soldier armour.

### The enemy: the Registrar and its reissues
Story in `LORE.md`. A machine built to catalogue life; it "preserved" its
makers by reading them completely, which destroyed them, and spread. One seed came down on Mars, has been consuming the
planet, and rebuilds living species from genomes held in its memory. What
lands on Earth are those rebuilt species: creatures above, a machine below.

Visual language, shared by every enemy so they read as one maker's work:
- **Grown on a frame.** Flesh over a printed lattice. Where skin is thin or
  torn the lattice shows: regular, dark, slightly wet.
- **Wrong repairs.** The machine patches its creatures the way it would patch
  equipment: seams, plugs, a limb replaced with something that works but is
  not the same species.
- **Electrical life.** Faint, irregular discharge at joints and along the
  spine, visible mainly at night. It is a symptom, not decoration, and it
  flickers out when the creature dies.
- **No faces to read.** Eyes are sensors or missing. Expression comes from
  posture and motion only.
- **Injuries** show as on soldiers, located where the hit landed, and expose
  lattice rather than bone.

Working roster by battlefield role. Each name fills one slot in the creature
table; slot order and the full list of names are in `NAMES.md`.

| Surface | Role | | Underwater | Role |
|---|---|---|---|---|
| Tender | Small handler, psi-capable | | Fathom | Line infantry |
| Kite | Gliding membrane scout | | Mantle | Armoured heavy |
| Lash | Serpent hunter; real body-wave motion on the ground | | Deacon | Psi officer |
| Maul | Heavy infantry | | Bell | Pressure-shelled brute |
| Regent | Psi commander | | Wader | Amphibious raider |
| Brooder | Parasite breeder: slimy, wet, reflective, catches every light | | Lamp | Lure-light drone |
| Husk | A taken soldier or animal, still wearing what it wore | | Wreath | Drifting parasite |
| Lathe | Hovering fabrication drone turned weapon | | Hulk | Massive beast |
| Gorger | Large pack beast | | Silt | Small bottom-lying ambusher |
| Trestle | Walking weapons platform | | | |
| Bloat | Drifting spitter | | | |
| Cinder | Slow mineral-bodied crawler | | | |

The final site: a hive that looks biological and is not. Ribbed, resin-like
surfaces that are actually conductor and insulation; light that travels along
the walls as current; symmetry that is too exact; spaces sized for no body.

### Soldiers
- Appearance is generated once from a seed at hiring and stored as a small
  record. Kit, rank, wear, repairs, wounds, burns and scars are layers on top.
  The record is self-contained so the same soldier, or their husk, can later be
  shown in another player's game.
- At gameplay range a soldier is 30–50 px tall. Realism goes into silhouette,
  load carriage, gait, weapon handling and material response, not faces.
- The designer view is lit like a photo-ID booth or a field portrait, slightly
  stylised in the face to stay out of the uncanny valley.

### Base
Chunky, serviced, lived-in hardware: cable trays, floor markings, spill
stains, tape labels. Modules are prebuilt with baked light. Destructible
interior pieces have broken variants and debris baked under the same light.

## 6. How a frame is judged

A frozen frame must pass as a still from aerial news footage. For each phase:
at least three rounds of screenshots at different moments, each judged by
naming the three weakest things, fixing them and re-shooting. Frame time is
measured every round; under 50 fps, speed comes before features.
