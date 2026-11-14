# DEFENDING EARTH

A squad-tactics game built with Rust and macroquad. You run **Joint Task Force
HEARTH**: watch a rotating Earth for unidentified craft, send interceptors after
them, land a squad at the crash site and fight it out turn by turn, then bring
what you find home to study and build from.

There are two campaigns, both starting in 2027 and chosen from the campaign
screen:

* **LANDFALL** — land and air. Bases on land, funded by the 16 Charter states.
  Ends at the Seed, on the far side of the Moon.
* **UNDERTOW** — sea floor and trenches. Sea-floor stations, funded by 13
  sea-lane groups. Dive gear only. Ends at the Root, Verity Deep, South Pacific.

This is version 0.9, a development build. Graphics phases 1 and 2 (real
lighting, then weather, biomes and seasons) were finished on 2026-11-13; phase
3 (flashlights, flares, fire) is next. Where the work stands is in
`docs/STATUS.md`. The long-term plan (a server-hosted multiplayer strategy
game) is in `docs/ROADMAP.md`; the look of the game is defined in
`docs/ART_BIBLE.md`; the story is in `docs/LORE.md`; every name in the game is
listed in `docs/NAMES.md`; the code is explained in `docs/CODE_MAP.md`.

## Run

```
cargo run --release
```

The window is 1920×1080 physical (1280×720 UI units).

Command-line flags:

* `--screenshot <file.png>` draws the situation map, saves the window after a
  few frames and exits. Combined with `--demo`, it saves that screen instead.
* `--self-test <file.json>` runs a scripted field check (move, fire, HUD) and
  writes its results.
* `--demo title|map|campaign|base|facilities|research|workshop|market|fleet|finances|launch|assembly|editor|tactical|water|debrief|dogfight`
  opens one screen directly for inspection (`equip` is the same as `base`,
  `interception` the same as `dogfight`, `map` is the situation map). A `sea-`
  prefix, as in `sea-market`, shows the screen in the underwater campaign.
* `--ai-test` plays a mission with no input, prints a one-line summary and exits.
* `--dump-map` writes the world land and biome map to `shots/worldmap.png` and exits.

Debug flags, for looking at one scene. They work with `--demo`:

* `--clock "2027-06-21 8am"` sets the date and time. In a battle it is the
  local sun time at the site; on the globe it is UTC (world clock time).
* `--weather clear|rain|snow|dust|storm|blizzard|duststorm` forces the weather.
  The last three are the heavy versions.
* `--site <lat,lon>` puts the battle at a real place, for example
  `--site 64,-150`. The biome (the kind of country) and the season come from
  the world map at that place.
* `--biome <n>` forces a biome by number: 0 temperate, 1 farmland, 5 mountain,
  6 rain forest, 7 desert, 9 ice, 11 northern forest.
* `--seed <n>` picks which random battlefield is built.
* `--view wide|tele|low` picks the camera for the picture.
* `--t <seconds>` waits that long before `--screenshot` saves.
* `--frametime` shows how long each frame takes. With `--screenshot` it also
  prints the average when the picture is saved.

Example: `cargo run --release -- --clock "2027-01-15 3pm" --weather clear --site 30,82 --demo tactical`
(a mountain site in winter).

The lighting test scene is a separate small program:
`cargo run --release --example lightlab`.

## Folders

* `src/` — the game's code.
* `examples/lightlab.rs` — the lighting test scene.
* `assets/` — files the game ships with: the Earth maps, fonts, ground
  textures and the sky colour table.
* `source_assets/` — the original downloads and the scripts that turn them
  into `assets/` (`blender/`, `polyhaven/`, `nasa_black_marble/`).
* `docs/` — status, roadmap, code map, art bible, lore and names.
* `checks/` — results from `--self-test`.
* `shots/` — screenshots.
* `MECHANICS.md`, `mechanics.json` — the rules reference.

## Controls

**Situation map.** Drag to rotate the globe, wheel to zoom, click a base to
open it, click a crash site or city raid and press LAUNCH MISSION. Time-speed
buttons run along the top.

**Base.** Tabs along the top (number keys also switch tabs); Esc goes back.

**Interception.** 1 Shadow (hold range), 2 Long Shot, 3 Press, 4 Close In,
Esc Break Off. Space or Enter continues when the engagement is over.

**Field view.**

| Input | Action |
|---|---|
| LMB | Select a soldier, or move to a tile |
| RMB | Fire at a target (heavy weapons can also attack ground and walls) |
| 1 / 2 / 3 | Auto / Snap / Aimed shot |
| Tab | Next soldier |
| K | Kneel or stand |
| R | Reload |
| H | Trauma kit |
| G | Frag grenade at the hovered tile |
| V or Shift+G | Smoke grenade |
| I | Inventory |
| P | Reach panel |
| `[` / `]` or PageDown / PageUp | View level down / up |
| Q / E | Rotate the camera |
| WASD or arrows | Pan |
| Wheel | Zoom |
| Space | End turn |
| Esc | Help, or close the open panel |

## What is in it

* **Situation map.** A rotatable, zoomable Earth with animated water, city
  lights and a lit atmosphere. Contacts, crash sites, city raids, interception,
  a monthly Assembly review, land bases and sea-floor stations.
* **Bases.** A 6×6 module grid, roster and hiring, armoury, market, research,
  workshop, holding for live captives, hangars and craft.
* **Interception.** Real-time radar engagement with four attack stances and a
  break-off, weapon reloads, return fire and crash-site generation.
* **Field view.** A 26×26 battlefield drawn in 3D, lit by the real sun for
  the mission's place, date and time, with shadows. It has time units, three firing
  modes, armour facings and armour wear, kneeling, reloading, grenades, smoke
  and fire that linger and spread, walls and cover that break, reaction fire,
  morale and panic, Reach (READ, ROUT, TAKE), a slot inventory with carry weight,
  and trauma kits.
* **Soldiers.** Seven ranks from Recruit to Major, an appearance editor, a
  loadout screen with full weapon readouts, wounds and recovery time.
* **Weather and atmosphere** chosen from the mission's place and season, and
  drawn in 3D: rain with splashes and puddles, snow that falls, drifts and
  lies on the ground, blowing dust, and the heavy versions (rain storm,
  blizzard, dust storm). Seven kinds of land: temperate, farmland, mountain,
  rain forest, desert, ice and northern forest. Underwater missions have
  caustics (moving light patterns) and a cold deep-water grade.
* **After-action report** with hostiles down, losses, shots, accuracy, loot,
  score and the operation log.
* **Sound** synthesised at runtime in `src/audio.rs`.

## Art and data

All rules, names and numbers are this project's own and live in the source:
the item tables, creature rows, damage tables and catalogs are in `src/data.rs`
and `src/tactical.rs`. `MECHANICS.md` is the readable rules reference.

The Earth geography is Natural Earth public-domain data; the city lights come
from NASA's public-domain Earth at Night map; the ground textures come from
Poly Haven (CC0, free for any use). Soldiers, creatures, effects and UI are
drawn by code at runtime. See `CREDITS.md`.

## Known limits

* No save slot yet: each launch starts a fresh campaign.
* Buildings have no multi-level interiors yet.
* The enemy AI is simple: advance, shoot, strike in melee, and use Reach where
  the kind has it.
* Night battles are close to black until flashlights exist (graphics phase 3).
* Trees, buildings, crates and soldiers are still simple boxes.
* Overcast, storm and night skies are still set by hand, so storm and dust
  scenes look flat.
* The Moon is not drawn; the LANDFALL finale's marker sits on Earth. The
  finale is still an ordinary battle: low gravity, the lunar
  ground and the ending vote are decided in `docs/LORE.md` but not built.
* Much of the story in `docs/LORE.md` is decided but not built yet. The list
  is in `docs/STATUS.md`. That includes the soldier pool: there is no page
  yet for making your own soldiers.
* Several rules (hit chance, time-unit costs, reaction fire, promotion,
  funding) are marked "under review for 2.0" in `MECHANICS.md` and will change.
