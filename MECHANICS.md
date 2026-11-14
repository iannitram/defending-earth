# DEFENDING EARTH — RULES REFERENCE

Version 0.9. This is the readable statement of the rules as the code runs
them. Every table and number here is this project's own. Where a rule is marked
**under review for 2.0** it works as written today and will be redesigned in
the 2.0 rules sheet (see `docs/ROADMAP.md`).

Code locations are given as file and function so they survive edits.

On 2026-11-13 several on-screen words were renamed (SPINE, GLOW, REACH,
BREACHER BATTERY, the far side of the Moon). This file uses the new words.
The numbers did not change. Rules that are decided in `docs/LORE.md` but not
built yet are listed in section 26, so nobody mistakes them for working rules.

## 1. Terms

| Term | Meaning |
|---|---|
| HEARTH | Joint Task Force HEARTH, the player's organisation |
| Charter Assembly | The funding body. Members are Charter states (land) and sea-lane groups (sea) |
| LANDFALL | The land-and-air campaign. Starts 2027 |
| UNDERTOW | The sea-floor campaign. Starts 2027 |
| Situation map | The world-map screen |
| Field view | The tactical screen |
| City raid | An enemy attack on a city |
| TU | Time units: what a unit spends to act during its turn |
| Issue / Adapted / enemy tool / Derived | The four design steps of equipment: what armies have today; the same thing rebuilt around a captured part (*cored* on land, *cav* at sea); the enemy's own tools; our own designs built their way |

## 2. Random numbers

`src/data.rs`, `Rng`.

* A linear congruential generator: `seed = seed × 214013 + 2531011`, output is
  15 bits of the result.
* `rand(n)` returns a whole number from 0 to n **inclusive**.
* `range(a, b)` returns a to b inclusive.
* `chance(p)` is `rand(100) < p`. Because `rand(100)` can return 100, a 100%
  chance still fails about one time in 101.

## 3. Field view: time units and movement

`src/tactical.rs`. The map is 26×26 tiles. A squad is up to 8 soldiers against
up to 8 hostiles. **All TU costs are under review for 2.0.**

| Action | Cost |
|---|---|
| Walk one tile | 4 TU |
| Diagonal step | ×1.5 (6 TU on flat ground) |
| Each level climbed | +4 TU |
| Each level descended | +2 TU |
| Polar ground | +3 TU per tile |
| Kneel | 4 TU |
| Stand up | 8 TU |
| Reload | 15 TU |
| Fire | weapon's TU% for the mode × the unit's maximum TU ÷ 100 |
| Throw a grenade | 25% of maximum TU; range 12 tiles |
| Trauma kit action | 10 TU |
| Reach action | 25 TU |

Polar ground means ice-sheet tiles (terrain 9 on the world map). Snow lying on
other terrain is drawing only and costs nothing extra.

The terrain of a mission comes from where its site is on the world map
(`src/data.rs` `terrain_at`, read from `assets/earth_data.png`). There are
seven land terrains: temperate, farmland, mountain, rain forest, desert, ice
and northern forest. Until 2026-11-13 this lookup was 180° of longitude off, so
every site got the terrain of the far side of the planet. It is fixed in
`world_index`; sites now get the terrain of where they really are.

Weather, season and time of day (`src/tactical.rs` `compute_environment`) are
picked from the site's terrain, latitude, month and local time. In 0.9 they
change only how the field looks. They do not change movement, sight or
shooting.

Inventory (`I`):

| Action | Cost |
|---|---|
| Draw a magazine from a shoulder slot | 3 TU |
| Take from the belt | 4 TU |
| Take from the backpack | 8 TU |
| Stow the weapon to the backpack | 12 TU |
| Set a grenade fuse | 14 TU |
| Trauma kit: treat wound, adrenaline, analgesic | 10 TU each |

A soldier carrying more weight than their Strength is shown as overburdened.

Start of each HEARTH turn, in order: smoke and fire tick; TU refill; each open
wound bleeds 1 HP; morale +15; panic check.

## 4. Shooting

`src/data.rs` `hit_chance`, `tu_cost`, `roll_damage`; `src/tactical.rs` `fire`.
**Under review for 2.0.**

Modes: Auto fires 3 rounds; Snap and Aimed fire 1. A mode with accuracy 0 does
not exist for that weapon.

Hit chance, all whole-number arithmetic:

1. `acc` = the shooter's firing accuracy. Kneeling: `acc × 115 ÷ 100`.
2. `health_mod` = `75 + 25 × HP ÷ max HP − 10 × open wounds`, never below 10.
3. `chance` = `(health_mod × acc ÷ 100) × weapon accuracy for the mode ÷ 100`.
4. Shooting from higher ground: +10. From lower ground: −10.
5. Clamp to 5–95.
6. The shot hits if `rand(100) < chance`. A miss only scatters the tracer.

Damage roll:

* Blast (type 2): `rand(power) + power ÷ 2`, so 50%–150% of power.
* Everything else: `rand(power × 2)`, so 0%–200% of power.

## 5. Taking damage

`src/tactical.rs` `damage_unit`.

1. `modified` = roll × damage-table percent ÷ 100 (section 6).
2. Facing is chosen from the direction of the hit relative to the way the
   target faces: front, left, right or rear.
3. `damage` = `modified` − armour on that facing, never below 0.
4. That facing's armour wears by `modified ÷ 10 + 1`.
5. Stun damage (type 5) adds to the stun total; everything else comes off HP.
6. Morale loss = `(bravery_mod × damage ÷ 10) × 100 ÷ leadership`.
7. If damage is above 0, the target is still up and `rand(10) < damage`, the
   target gains `rand(2) + 1` open wounds (maximum 6).
8. A unit is down when HP ≤ 0 or stun ≥ maximum HP. A hostile that goes down
   to stun is a live captive if the mission is won.

`bravery_mod` is `11 − bravery ÷ 10` for soldiers and 3 for every hostile.

## 6. Damage types and armour classes

`src/data.rs` `DAMAGE_MODS`, `DAMAGE_MODS_WATER`, `damage_modifier`.

| Id | On land | At sea | What it is |
|---|---|---|---|
| 0 | KIN | KIN | Bullets and bolts |
| 1 | HEAT | HEAT | Fire and burning |
| 2 | BLAST | BLAST | Explosive overpressure and fragments |
| 3 | CORE | CAV | Our Adapted weapons: core discharge on land, cavitating slug at sea |
| 4 | ARC | PRES | Enemy tools: arc on land, pressure at sea |
| 5 | STUN | STUN | Shock, and smoke |
| 6 | CUT | CUT | Blades, claws, teeth |
| 7 | ACID | ACID | Corrosive spit (reserved) |

| Class | Name | Who has it |
|---|---|---|
| 0 | Bare | Reserved |
| 1 | Soft | Soldier in combat uniform; Tender; Fathom |
| 2 | Plated | Spine Carrier, Dive Rig |
| 3 | Sealed shell | Sealed Frame, Nacre Dive Rig, Deep Frame |
| 4 | Membrane | Kite, Bloat, Wreath |
| 5 | Scaled | Lash, Wader |
| 6 | Insulated | Regent, Deacon |
| 7 | Heavy build | Maul, Mantle |
| 8 | Mineral | Cinder |
| 9 | Slick | Brooder, Silt |
| 10 | Beast hide | Gorger, Hulk |
| 11 | Machine plate | Trestle (to be removed, see section 14) |
| 12 | Drone casing | Lathe, Lamp |
| 13 | Pressure shell | Bell |

Percent of damage that gets through, **on land**:

| Type | Bare | Soft | Plat | Seal | Memb | Scal | Insu | Heav | Mine | Slik | Hide | Mach | Dron | Pres |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| KIN | 100 | 100 | 90 | 80 | 110 | 90 | 100 | 80 | 50 | 100 | 90 | 60 | 80 | 70 |
| HEAT | 120 | 110 | 90 | 50 | 150 | 100 | 60 | 90 | 20 | 130 | 110 | 40 | 70 | 80 |
| BLAST | 120 | 110 | 100 | 80 | 130 | 100 | 100 | 80 | 120 | 100 | 90 | 70 | 110 | 60 |
| CORE | 100 | 100 | 100 | 90 | 100 | 110 | 70 | 100 | 80 | 110 | 100 | 130 | 140 | 90 |
| ARC | 110 | 110 | 100 | 80 | 100 | 90 | 60 | 90 | 70 | 100 | 100 | 80 | 90 | 80 |
| STUN | 110 | 100 | 90 | 70 | 110 | 100 | 80 | 80 | 40 | 90 | 90 | 0 | 0 | 60 |
| CUT | 120 | 110 | 80 | 60 | 140 | 90 | 100 | 80 | 30 | 110 | 90 | 40 | 60 | 50 |
| ACID | 130 | 120 | 110 | 90 | 120 | 90 | 100 | 90 | 140 | 70 | 100 | 120 | 110 | 80 |

**Under water**:

| Type | Bare | Soft | Plat | Seal | Memb | Scal | Insu | Heav | Mine | Slik | Hide | Mach | Dron | Pres |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| KIN | 100 | 100 | 90 | 70 | 100 | 80 | 100 | 70 | 60 | 110 | 80 | 70 | 80 | 40 |
| HEAT | 100 | 90 | 80 | 40 | 120 | 90 | 70 | 80 | 30 | 110 | 90 | 50 | 70 | 60 |
| BLAST | 130 | 120 | 110 | 90 | 140 | 110 | 100 | 90 | 110 | 120 | 100 | 80 | 120 | 50 |
| CAV | 100 | 100 | 100 | 90 | 110 | 100 | 90 | 90 | 90 | 110 | 100 | 110 | 120 | 80 |
| PRES | 120 | 110 | 100 | 80 | 120 | 100 | 90 | 90 | 80 | 110 | 90 | 90 | 100 | 30 |
| STUN | 110 | 100 | 90 | 80 | 110 | 100 | 70 | 90 | 40 | 100 | 90 | 0 | 0 | 70 |
| CUT | 120 | 110 | 80 | 60 | 150 | 90 | 100 | 80 | 30 | 120 | 90 | 40 | 60 | 40 |
| ACID | 110 | 100 | 90 | 80 | 100 | 80 | 90 | 80 | 120 | 60 | 90 | 100 | 90 | 70 |

How to read them: values move in steps of 10 around 100. Membranes tear,
mineral bodies and machines shrug off most things except blast, core and acid,
stun does nothing to machines and drones, and soldiers always sit close to 100
so that the suit's armour values decide survival.

## 7. Soldier armour

`src/data.rs` `ARMORS`.

| # | Suit | Step | Front | Side | Rear | Under | Class | Weight | Where |
|---|---|---|---|---|---|---|---|---|---|
| 0 | COMBAT UNIFORM | Issue | 8 | 6 | 5 | 2 | 1 Soft | 0 | Land |
| 1 | SPINE CARRIER | Adapted | 46 | 30 | 36 | 20 | 2 Plated | 9 | Land |
| 2 | SEALED FRAME | Derived | 90 | 72 | 64 | 50 | 3 Sealed | 18 | Land |
| 3 | DIVE RIG | Issue | 40 | 28 | 24 | 16 | 2 Plated | 14 | Sea |
| 4 | NACRE DIVE RIG | Adapted | 84 | 60 | 56 | 40 | 3 Sealed | 17 | Sea |
| 5 | DEEP FRAME | Derived | 118 | 88 | 80 | 60 | 3 Sealed | 22 | Sea |

Land suits are not worn under water and dive suits are not worn on land: a
squad sent under water is put into dive gear with dive weapons on deployment.

## 8. Soldiers

`src/data.rs` `Stats::recruit`, `RANKS`, `evaluate_force_promotions`.

Ranks, lowest first: RECRUIT, PRIVATE, CORPORAL, SERGEANT, LIEUTENANT, CAPTAIN,
MAJOR.

A new recruit rolls:

| Attribute | Range |
|---|---|
| Time units | 48–62 |
| Stamina | 45–75 |
| Health | 32–44 |
| Bravery | 20–70 |
| Reactions | 32–58 |
| Firing accuracy | 38–66 |
| Throwing accuracy | 45–75 |
| Strength | 25–55 |
| Reach skill | 0 |
| Reach strength | 5–95 |

Promotion happens at the monthly review (**under review for 2.0**):

* A Recruit with at least one mission becomes a Private.
* With N living soldiers the force may hold N÷5 Corporals, N÷11 Sergeants,
  N÷23 Lieutenants, N÷30 Captains, and one Major once N reaches 30.
* Open places go to the highest merit: `(missions + kills) × 10` plus the sum
  of TU, health, stamina, reactions, firing, throwing, strength and bravery.

A wounded soldier is out for a number of days and returns at full health.

## 9. Morale, panic and leadership

`src/tactical.rs` `team_leadership`, `check_unit_panic`. **Under review for 2.0.**

Leadership is set by the highest rank alive on the side:

| Highest rank | Leadership |
|---|---|
| Recruit, Private | 100 |
| Corporal | 110 |
| Sergeant, Lieutenant | 115 |
| Captain | 125 |
| Major | 150 |

* When a unit dies, each teammate loses about `bravery_mod × 2` morale (at
  least 1). The side that made the kill gains `20 × leadership ÷ 100`.
* Panic check: if morale is below 50 and `rand(100) ≤ 100 − morale × 2`, roll
  `rand(3)`: 0 the unit freezes; 1 or 2 it panics and loses its TU; 3 it goes
  berserk (TU to 0, morale +20).

## 10. Reach

Reach is what a Regent does when it reaches into another mind. Until
2026-11-13 the game called it psi. Only the on-screen word changed; the rules
below are the same.

`src/tactical.rs`. Three actions, 25 TU each, against a hostile in sight.
**Under review for 2.0.**

* **READ** shows the target's HP, TU, morale, Reach strength and weapon.
* **ROUT** attacks morale.
* **TAKE** turns the target to your side until the start of the enemy turn.

Attack = `rand(55) + Reach strength × Reach skill ÷ 50 − distance ÷ 2`.

Soldier using Reach on a hostile: defence = target Reach strength + 10 for
ROUT, or + 25 for TAKE, plus a rank bonus (Corporal 10, Sergeant or
Lieutenant 15, Captain 25, Major 40). ROUT succeeds when attack beats defence and drains
`max(attack − defence, 25) + 15` morale, capped at 75, followed by a panic
check. A taken unit loses 30 morale when it is released.

Hostile using Reach on a soldier (only Regent and Deacon have the skill):
defence = Reach strength + 10 + (Reach skill ÷ 5 if skill is above 4). On success
the soldier loses `bravery_mod × 10` morale and takes a panic check. A
successful attempt costs the hostile 25 TU and is its action for the turn.
A failed attempt costs nothing and the hostile goes on to the next step in
section 13.

## 11. Reaction fire

`src/tactical.rs`. **Under review for 2.0.**

* Reaction score = `reactions × current TU ÷ maximum TU`.
* Checked each time a soldier enters a tile. Only hostiles react.
* The highest-scoring hostile whose score beats the mover's, who can see the
  mover and has the TU and ammunition for a snap shot, fires one.
* At most 2 reaction shots per step.

## 12. Explosions, smoke, fire and cover

`src/tactical.rs` `explode`, and the smoke and fire tick.

Radius in tiles:

| Kind | Radius |
|---|---|
| Blast | power ÷ 20, between 2.0 and 5.5 |
| Smoke | power ÷ 20, between 2.5 and 6.0 |
| Heat | power ÷ 30, between 2.0 and 5.0 |

Power falls off in a straight line from the centre to the edge.

Hand grenades: FRAG GRENADE power 64 blast; SMOKE GRENADE power 48.

Units in the radius:

* Smoke: stun = a damage roll on half the tile's power.
* Otherwise: damage roll × table percent ÷ 100 − under-armour ÷ 2, at least 1.
  *Known quirk:* blast against units always reads the land table, even under
  water.

Cover in the radius:

| Obstacle | Armour | HP |
|---|---|---|
| High wall | 35 | 80 |
| Low wall | 25 | 60 |
| Tree | 10 | 40 |
| Boulder | 20 | 50 |
| HEARTH craft hull | 15 | 300 |
| Enemy craft hull | 15 | 400 |
| Enemy power plant | 15 | 500 |

Damage to cover = `(tile power − armour) × 2`, at least 15. A high wall at 40
HP or less becomes a low wall; at 0 it is gone and the sightline opens.

Smoke and fire left behind: a smoke burst adds up to 14 smoke per tile, a blast
up to 8, heat up to 12 fire, each scaled by falloff. Smoke is capped at 15 per
tile.

Each turn:

* Fire burns a unit standing in it for `max(2, fire ÷ 2) − under-armour ÷ 3`
  (at least 1) and costs 8 morale. The tile gains 3 smoke and loses 1 fire.
* Smoke loses 1. Smoke above 3 spreads 1 to each of the 4 neighbouring tiles
  unless a high wall is in the way.

Sight: about 12 tiles. Trees, high walls, craft hulls and ground 2 or more
levels higher block it. So does smoke once the smoke along the line adds up to
18.

## 13. Enemy behaviour

`src/tactical.rs`. One action per hostile per turn:

1. Reach, if the kind has both Reach strength and Reach skill.
2. Melee, if its melee value is above 25 and a soldier is within 1.7 tiles.
   *Known quirk:* the strike uses the damage of weapon 0 (SERVICE PISTOL,
   22 kinetic), not the creature's own natural weapon.
3. Otherwise, with a target in sight: 40% advance up to 3 tiles; else 78% shoot
   (snap, then auto, then aimed, whichever it can afford).
4. With no target in sight: move 4 tiles toward the nearest soldier.

The mission is won when every hostile is down and lost when every soldier is.

## 14. Creatures

`src/tactical.rs` `alien_def`. Armour is front / left / right / rear / under.
"Melee" above 25 means the kind fights hand to hand.

Land:

| # | Name | HP | TU | React | Acc | Armour | Class | Reach str | Reach skill | Weapon | Melee | Floats |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | Tender | 24 | 62 | 52 | 45 | 2/2/2/1/0 | 1 | 55 | 0 | ARC STYLUS | 0 | no |
| 1 | Kite | 30 | 66 | 68 | 48 | 5/4/4/3/8 | 4 | 20 | 0 | ARC CUTTER | 0 | yes |
| 2 | Lash | 52 | 48 | 55 | 55 | 16/14/14/12/8 | 5 | 30 | 0 | ARC CUTTER | 0 | no |
| 3 | Maul | 110 | 52 | 50 | 62 | 24/20/20/16/8 | 7 | 25 | 0 | ARC TORCH | 0 | no |
| 4 | Regent | 48 | 60 | 70 | 55 | 30/26/26/22/18 | 6 | 60 | 38 | ARC CUTTER | 0 | no |
| 5 | Brooder | 84 | 96 | 66 | 100 | 30/26/26/22/14 | 9 | 40 | 0 | NATURAL WEAPON | 75 | no |
| 6 | Lathe | 105 | 58 | 58 | 76 | 32/28/28/24/20 | 12 | 95 | 0 | ARC TORCH | 0 | yes |
| 7 | Gorger | 150 | 70 | 55 | 95 | 26/24/24/20/6 | 10 | 15 | 0 | NATURAL WEAPON | 85 | no |
| 8 | Trestle | 115 | 50 | 52 | 78 | 120/100/100/70/60 | 11 | 95 | 0 | ARC TORCH | 0 | no |
| 9 | Bloat | 60 | 64 | 38 | 60 | 14/14/14/14/14 | 4 | 35 | 0 | ARC STYLUS | 0 | yes |
| 10 | Cinder | 95 | 34 | 30 | 85 | 60/48/48/40/15 | 8 | 80 | 0 | NATURAL WEAPON | 70 | no |

Sea:

| # | Name | HP | TU | React | Acc | Armour | Class | Reach str | Reach skill | Weapon | Melee | Floats |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | Fathom | 38 | 50 | 50 | 50 | 8/6/6/4/2 | 1 | 25 | 0 | PRESSURE AWL | 0 | no |
| 1 | Wader | 46 | 60 | 52 | 54 | 14/10/10/8/6 | 5 | 20 | 0 | PRESSURE BORE | 0 | no |
| 2 | Mantle | 98 | 54 | 48 | 66 | 30/24/24/18/10 | 7 | 35 | 0 | PRESSURE RAM | 0 | no |
| 3 | Deacon | 70 | 62 | 72 | 60 | 22/20/20/18/12 | 6 | 55 | 42 | PRESSURE BORE | 0 | no |
| 4 | Bell | 90 | 44 | 40 | 90 | 70/60/60/50/20 | 13 | 70 | 0 | NATURAL WEAPON (DEEP) | 80 | no |
| 5 | Silt | 30 | 72 | 60 | 85 | 6/5/5/4/10 | 9 | 30 | 0 | NATURAL WEAPON (DEEP) | 65 | no |
| 6 | Lamp | 100 | 66 | 48 | 82 | 28/22/22/18/12 | 12 | 95 | 0 | PRESSURE RAM | 0 | yes |
| 7 | Wreath | 78 | 88 | 74 | 95 | 18/16/16/14/10 | 4 | 85 | 0 | NATURAL WEAPON (DEEP) | 85 | yes |
| 8 | Hulk | 165 | 58 | 50 | 88 | 80/70/70/60/12 | 10 | 90 | 0 | PRESSURE RAM | 0 | no |

Which kinds appear depends on how far the campaign has gone
(`src/main.rs` `roll_alien`). Cinder is in no spawn pool yet.

The land table shows what the code runs today. `docs/LORE.md` has since
decided changes that are not built: the Trestle is dropped as a unit (it is
still in the code and can still appear), and the Maul, Brooder, Lathe, Kite,
Bloat, Cinder and Gorger get rules of their own. See section 26.

## 15. Weapons

`src/data.rs` `SURFACE_ITEMS`, `WATER_ITEMS` and the weapon list in
`GameData::load`. Accuracy and TU% come from the weapon; damage, type and
magazine size come from its ammunition when it has any. A dash in a mode
column means the mode does not exist; a dash under Clip means the weapon does
not use magazines.

Land:

| Weapon | Step | Dmg | Type | Acc auto/snap/aimed | TU% auto/snap/aimed | Clip | Wt | Ammunition |
|---|---|---|---|---|---|---|---|---|
| SERVICE PISTOL | Issue | 22 | KIN | —/58/74 | —/16/28 | 15 | 3 | 9MM MAGAZINE |
| SERVICE RIFLE | Issue | 27 | KIN | 42/65/100 | 40/22/65 | 30 | 5 | 5.56 MAGAZINE |
| HEAVY RIFLE | Issue | 62 | KIN | —/55/115 | —/34/78 | 10 | 15 | .50 MAGAZINE |
| MACHINE GUN | Issue | 34 | KIN | 48/52/78 | 38/30/72 | 50 | 14 | 7.62 BELT |
| AT LAUNCHER | Issue | 90 | BLAST | —/50/108 | —/42/74 | 1 | 10 | HE ROCKET |
| CORED PISTOL | Adapted | 42 | CORE | 30/62/82 | 26/18/46 | — | 3 | none |
| CORED RIFLE | Adapted | 56 | CORE | 44/64/98 | 32/24/52 | — | 5 | none |
| CORED HEAVY GUN | Adapted | 78 | CORE | —/48/88 | —/32/72 | — | 12 | none |
| ARC STYLUS | enemy tool | 48 | ARC | 45/62/82 | 28/20/48 | 24 | 3 | STYLUS CELL |
| ARC CUTTER | enemy tool | 72 | ARC | 52/78/96 | 34/28/58 | 30 | 7 | CUTTER CELL |
| ARC TORCH | enemy tool | 104 | ARC | 46/70/108 | 36/30/62 | 36 | 11 | TORCH CELL |
| FRAG GRENADE | Issue | 64 | BLAST | —/65/85 | —/24/50 | — | 1 | — |
| DEMO CHARGE | Issue | 105 | BLAST | —/55/75 | —/30/55 | — | 3 | — |
| STUN BATON | Issue | 60 | STUN | —/100/— | —/28/— | — | 2 | — |
| NATURAL WEAPON | creatures only | 58 | CUT | —/90/— | —/22/— | — | 0 | — |

Sea:

| Weapon | Step | Dmg | Type | Acc auto/snap/aimed | TU% auto/snap/aimed | Clip | Wt | Ammunition |
|---|---|---|---|---|---|---|---|---|
| DIVE PISTOL | Issue | 20 | KIN | —/54/70 | —/17/30 | 5 | 3 | PISTOL BOLTS |
| DIVE RIFLE | Issue | 30 | KIN | 36/58/92 | 38/24/66 | 26 | 6 | RIFLE BOLTS |
| HEAVY DIVE RIFLE | Issue | 54 | KIN | —/52/105 | —/35/76 | 8 | 14 | HEAVY BOLTS |
| DIVE MG | Issue | 36 | KIN | 44/48/72 | 40/30/70 | 40 | 15 | BOLT BELT |
| TORPEDO TUBE | Issue | 96 | BLAST | —/48/100 | —/44/76 | 1 | 12 | MINI TORPEDO |
| DIVE CHARGE | Issue | 70 | BLAST | —/60/80 | —/26/52 | — | 2 | — |
| STUN PROD | Issue | 66 | STUN | —/100/— | —/30/— | — | 2 | — |
| DIVE KNIFE | Issue | 24 | CUT | —/110/— | —/14/— | — | 1 | — |
| CAV PISTOL | Adapted | 44 | CAV | 28/60/80 | 28/19/44 | 12 | 4 | CAV PISTOL MAG |
| CAV RIFLE | Adapted | 60 | CAV | 40/62/96 | 34/25/54 | 18 | 7 | CAV RIFLE MAG |
| CAV HEAVY GUN | Adapted | 82 | CAV | —/46/90 | —/33/74 | 10 | 15 | CAV HEAVY MAG |
| PRESSURE AWL | enemy tool | 50 | PRES | —/60/84 | —/21/50 | 20 | 3 | AWL CHARGE |
| PRESSURE BORE | enemy tool | 76 | PRES | 40/74/98 | 36/29/60 | 24 | 8 | BORE CHARGE |
| PRESSURE RAM | enemy tool | 112 | PRES | —/66/104 | —/32/64 | 16 | 12 | RAM CHARGE |
| SNARE PROJECTOR | Adapted | 90 | STUN | —/55/95 | —/40/70 | 1 | 11 | SNARE CHARGE |
| IMPLOSION POD | Derived | 100 | BLAST | —/62/82 | —/26/52 | — | 2 | — |
| NATURAL WEAPON (DEEP) | creatures only | 62 | CUT | —/90/— | —/22/— | — | 0 | — |

Weight is the weapon plus one load of ammunition.

Other field items: SMOKE GRENADE (power 48), TRAUMA KIT (treat wound: +12 HP
and one wound closed; adrenaline: clears stun; analgesic: +25 morale), TRIP
MINE, HANDHELD RADAR and HANDHELD SONAR (the last three are stocked but have no
field rule yet).

## 16. Situation map and contacts

`src/main.rs`.

* A new contact appears every 600–1,600 game minutes.
* A site that is not answered before it expires costs 30 score.
* Enemy craft classes: SPORE (light scout), BURR (collector), THORN (city
  raider), RHIZOME (undersea runner), TAPROOT (capital craft).
* Research THE FARSIDE SIGNAL (still `SEED_LOCATION` inside the code) puts
  the lunar far side assault on the map; VERITY DEEP puts the Verity Deep
  assault on the map. The first needs an
  Albatross, the second an Emperor.

Score:

| Event | Score |
|---|---|
| Mission won | +100, +20 per hostile down, −15 per soldier lost |
| Mission lost | −60 |
| Site went cold | −30 |
| Live study completed | +150 |

## 17. Interception

`src/dogfight.rs`. Real time. Distances are in km.

| Key | Stance | Holds at | Fires |
|---|---|---|---|
| 1 | SHADOW | 65 | no |
| 2 | LONG SHOT | longest weapon range (60 at most) | yes |
| 3 | PRESS | 22 | yes |
| 4 | CLOSE IN | 7.5 | yes |
| Esc | BREAK OFF | opens to 85; contact is broken at 75 | no |

Closing speed scales with the craft's speed ÷ 2000, between 0.6 and 2.5.

Our shot: `half = accuracy ÷ 2`; effective accuracy =
`half + (half ÷ divisor) × 3`, clamped 5–100, where the divisor is 4 for LONG
SHOT, 3 for PRESS and 2 for CLOSE IN. It hits if `rand(100)` is below that.
Damage = `(rand(damage) + damage) ÷ 2`, at least 1. A contact brought to 0 by
a single hit of 140 or more breaks up and leaves nothing; otherwise it comes
down and a crash site is marked.

Their shot hits when `rand(100) > 40`, for `rand(damage)`, at least 10.

Reload time by weapon group, in seconds: gun pod and bolt cannon 0.35; cored
lance and cav cannon 1.8; arc and pressure projectors 2.0; short-range missile
and light torpedo 2.6; long-range missile and heavy torpedo 3.4; breachers 4.0.

Enemy craft:

| Class | Hull | Range | Damage | Accuracy | Weapon | Reload | Speed |
|---|---|---|---|---|---|---|---|
| SPORE | 110 | 14 | 22 | 50 | ARC NEEDLE | 2.4 | 3000 |
| BURR | 460 | 22 | 46 | 60 | ARC LANCE | 2.1 | 2600 |
| THORN | 1100 | 32 | 80 | 68 | ARC VOLLEY | 1.9 | 3600 |
| RHIZOME | 600 | 25 | 55 | 62 | PRESSURE LANCE | 2.1 | 3200 |
| TAPROOT | 2600 | 42 | 130 | 76 | MAIN DISCHARGE | 1.6 | 4400 |

Craft weapons (`src/data.rs` `CRAFT_WEAPONS`). Ammo "∞" means it does not run out.

| Weapon | Where | Step | Range | Damage | Accuracy | Ammo | Cost |
|---|---|---|---|---|---|---|---|
| SHORT-RANGE MISSILE | Land | Issue | 28 | 65 | 72 | 6 | $18,000 |
| LONG-RANGE MISSILE | Land | Issue | 58 | 95 | 78 | 4 | $24,000 |
| GUN POD (20MM) | Land | Issue | 9 | 12 | 30 | 250 | $11,000 |
| CORED LANCE | Land | Adapted | 24 | 75 | 40 | ∞ | $170,000 |
| ARC PROJECTOR | Land | enemy tool | 48 | 130 | 82 | 80 | $240,000 |
| BREACHER MISSILE | Land | Derived | 62 | 215 | 95 | 2 | $310,000 |
| LIGHT TORPEDO | Sea | Issue | 30 | 62 | 68 | 8 | $17,000 |
| HEAVY TORPEDO | Sea | Issue | 52 | 105 | 76 | 4 | $25,000 |
| BOLT CANNON | Sea | Issue | 7 | 14 | 28 | 240 | $12,000 |
| CAV CANNON | Sea | Adapted | 22 | 85 | 42 | 60 | $175,000 |
| PRESSURE PROJECTOR | Sea | enemy tool | 50 | 140 | 80 | 90 | $250,000 |
| BREACHER TORPEDO | Sea | Derived | 58 | 205 | 94 | 2 | $320,000 |

## 18. Craft

`src/data.rs` `CRAFT_CATALOG`. Land craft are named for land birds, sea craft
for diving seabirds.

| Craft | Where | Step | Speed | Fuel | Fuel type | Seats | Mounts | Hull | Cost | Upkeep / month |
|---|---|---|---|---|---|---|---|---|---|---|
| HERON | Land | Issue | 700 | 1600 | JET FUEL | 12 | 0 | 140 | $550,000 | $36,000 |
| SWIFT | Land | Issue | 1900 | 900 | JET FUEL | 0 | 2 | 110 | $650,000 | $12,000 |
| PEREGRINE | Land | Adapted | 3800 | 24 | GLOW | 0 | 2 | 240 | $1,300,000 | $18,000 |
| IBIS | Land | Adapted | 2900 | 32 | GLOW | 10 | 1 | 280 | $1,500,000 | $24,000 |
| ALBATROSS | Land | Derived | 5000 | 64 | GLOW | 20 | 2 | 480 | $2,800,000 | $34,000 |
| GANNET | Sea | Issue | 720 | 1500 | DIESEL-ELECTRIC | 12 | 0 | 150 | $550,000 | $36,000 |
| SKUA | Sea | Issue | 2200 | 850 | DIESEL-ELECTRIC | 0 | 2 | 125 | $650,000 | $12,000 |
| CORMORANT | Sea | Adapted | 4100 | 28 | LIVE BRINE | 0 | 2 | 260 | $1,400,000 | $18,000 |
| RAZORBILL | Sea | Adapted | 3600 | 40 | LIVE BRINE | 10 | 1 | 320 | $1,600,000 | $24,000 |
| EMPEROR | Sea | Derived | 5000 | 64 | LIVE BRINE | 20 | 2 | 520 | $2,900,000 | $38,000 |

The HERON is a twin-rotor transport helicopter with a rear ramp, and the
SWIFT is a jet fighter: ordinary present-day types, our own designs. The
ALBATROSS is the only craft that can reach the far side of the Moon.

A LANDFALL campaign starts with SWIFT-1 (short-range and long-range missiles)
and HERON-1. An UNDERTOW campaign starts with SKUA-1 (light and heavy
torpedoes) and GANNET-1.

## 19. Bases

`src/data.rs` `FACILITY_CATALOG`. A base is a 6×6 grid. Every module must
connect back to the entry, which cannot be dismantled.

| Module (land / sea) | Step | Cost | Build days | Upkeep / month | Gives |
|---|---|---|---|---|---|
| ENTRY SHAFT / MOON POOL | Issue | $250,000 | 2 | $5,000 | Way in and out |
| BARRACKS / BERTHING | Issue | $350,000 | 14 | $8,000 | Room for 48 people |
| RESEARCH LAB / WET LAB | Issue | $700,000 | 28 | $28,000 | 40 scientists |
| MACHINE SHOP | Issue | $750,000 | 28 | $32,000 | 40 engineers |
| SEARCH RADAR / SEARCH SONAR | Issue | $450,000 | 14 | $9,000 | Detection to 750 nm |
| LONG-RANGE RADAR / TOWED ARRAY | Issue | $850,000 | 21 | $17,000 | Detection to 1,500 nm |
| SAM BATTERY / TORPEDO BATTERY | Issue | $250,000 | 14 | $6,000 | Defence 450 |
| SUPPLY BAY | Issue | $200,000 | 7 | $4,000 | 60 units of storage |
| HOLDING BLOCK / HOLDING TANK | Issue | $450,000 | 21 | $14,000 | 8 live captives |
| CORED BATTERY / CAV BATTERY | Adapted | $450,000 | 21 | $9,000 | Defence 650 |
| ARC BATTERY / PRESSURE BATTERY | enemy tool | $650,000 | 28 | $13,000 | Defence 950 |
| BREACHER BATTERY | Derived | $900,000 | 35 | $18,000 | Defence 1300 |
| FARADAY SCREEN / BAFFLE SCREEN | Adapted | $1,100,000 | 35 | $16,000 | Defence 350 |
| REACH DAMPER | Adapted | $1,250,000 | 35 | $18,000 | Hides the base from scouts |
| REACH WING | Adapted | $800,000 | 21 | $16,000 | Reach testing and training |
| SIGINT STATION | Adapted | $1,500,000 | 28 | $32,000 | Detection to 2,600 nm with class, crew and heading |
| HANGAR / WET DOCK | Issue | $250,000 | 21 | $22,000 | One craft; takes 2×2 |

Live captives beyond holding capacity (8 per finished holding module,
`src/main.rs` `holding_cap`) are lost.

## 20. Materials

| Material | Short | Campaign | What it is |
|---|---|---|---|
| SPINE | SPINE | LANDFALL | The printed frame under every creature and hull, cut from wrecks and bodies |
| GLOW | GLOW | LANDFALL | The glowing charged fluid the frame runs on |
| NACRE PLATE | NACRE | UNDERTOW | The layered shell sea-going builds grow |
| LIVE BRINE | BRINE | UNDERTOW | Their power fluid; live only while cold and dark |

On land the structure material is SPINE and the power material is GLOW. Cored
weapons are filled with Glow, and Adapted and Derived craft burn it as fuel.

A campaign starts with 30 structure material and 15 power material. A won
mission brings back 25 structure and 10 power material, plus enemy tools and
bodies. (**Counts under review for 2.0.**) These are flat amounts today. The
decided rule, not built yet, is that loot comes from the battlefield (section
26). Full stores names for the materials are in `docs/NAMES.md` section 5.

## 21. Research

`src/data.rs` `RESEARCH_SURFACE`, `RESEARCH_WATER`. Each scientist on a topic
adds 24 points per day.

LANDFALL:

| Topic | Points | Needs | Opens |
|---|---|---|---|
| CORED WEAPONS | 140 | — | Cored pistol, rifle, heavy gun topics |
| CORED PISTOL | 90 | Cored Weapons | Cored Pistol |
| CORED RIFLE | 180 | Cored Weapons | Cored Rifle |
| CORED HEAVY GUN | 240 | Cored Weapons | Cored Heavy Gun |
| CORED LANCE | 320 | Cored Heavy Gun | Aircraft weapon: Cored Lance |
| SPINE | 220 | — | Spine Carrier, enemy craft studies |
| SPINE CARRIER | 160 | Spine | Spine Carrier |
| SEALED FRAME | 320 | Spine Carrier, Glow | Sealed Frame |
| ARC TOOLS | 380 | — | Arc Stylus, Arc Cutter, Arc Torch |
| GLOW | 260 | — | Sealed Frame, enemy power plant |
| ENEMY POWER PLANT | 340 | Glow | Enemy airframe |
| ENEMY GUIDANCE | 310 | Spine | SIGINT Station, enemy airframe |
| ENEMY AIRFRAME | 440 | Enemy Power Plant, Enemy Guidance | Peregrine |
| SIGINT STATION | 420 | Enemy Guidance | SIGINT Station module |
| REACH STUDIES | 360 | — | Reach Wing module |
| THE FARSIDE SIGNAL | 520 | Enemy Airframe | Far side of the Moon objective, Albatross |
| LIVE STUDY: TENDER | 140 | a live Tender | Tender file |
| LIVE STUDY: LASH | 170 | a live Lash | Lash file |
| LIVE STUDY: MAUL | 190 | a live Maul | Maul file |
| LIVE STUDY: REGENT | 310 | a live Regent | Regent file |

UNDERTOW:

| Topic | Points | Needs | Opens |
|---|---|---|---|
| NACRE PLATE | 230 | — | Nacre Dive Rig, enemy sea guidance |
| NACRE DIVE RIG | 190 | Nacre | Nacre Dive Rig |
| DEEP FRAME | 330 | Nacre Dive Rig, Live Brine | Deep Frame |
| DEEP FRAME THRUST PACK | 410 | Deep Frame, Enemy Sea Guidance | Deep Frame Thrust Pack |
| CAV WEAPONS | 170 | — | Cav pistol, rifle, heavy gun |
| CAV RIFLE | 140 | Cav Weapons | Cav Rifle |
| CAV HEAVY GUN | 230 | Cav Weapons | Cav Heavy Gun |
| PRESSURE TOOLS | 400 | — | Pressure Awl, Bore, Ram |
| LIVE BRINE | 260 | — | Deep Frame, brine drive |
| ENEMY SEA GUIDANCE | 310 | Nacre | SIGINT Station, enemy pressure hull |
| BRINE DRIVE | 370 | Live Brine | Enemy pressure hull |
| ENEMY PRESSURE HULL | 470 | Brine Drive, Enemy Sea Guidance | Cormorant |
| SIGINT STATION | 420 | Enemy Sea Guidance | SIGINT Station module |
| REACH STUDIES | 370 | — | Reach Wing module |
| THE ROOT SIGNAL | 460 | Enemy Pressure Hull | Verity Deep study |
| VERITY DEEP | 560 | The Root Signal, Enemy Pressure Hull | Verity Deep objective, Emperor |
| LIVE STUDY: FATHOM | 150 | a live Fathom | Fathom file |
| LIVE STUDY: WADER | 170 | a live Wader | Wader file |
| LIVE STUDY: MANTLE | 230 | a live Mantle | Mantle file |
| LIVE STUDY: DEACON | 330 | a live Deacon | Deacon file |

A live study uses up the captive.

## 22. Workshop

`src/data.rs` `WORKSHOP_BLUEPRINTS`. Each engineer on a job adds 24 hours per
day. "Structure" and "power" are the campaign's two materials.

| Blueprint | Campaign | Needs research | Engineer hours | Cost | Structure | Power |
|---|---|---|---|---|---|---|
| SPINE CARRIER | LANDFALL | Spine Carrier | 700 | $20,000 | 4 | 0 |
| SEALED FRAME | LANDFALL | Sealed Frame | 1,100 | $48,000 | 6 | 4 |
| CORED PISTOL | LANDFALL | Cored Pistol | 250 | $9,000 | 1 | 0 |
| CORED RIFLE | LANDFALL | Cored Rifle | 450 | $18,000 | 2 | 0 |
| CORED HEAVY GUN | LANDFALL | Cored Heavy Gun | 750 | $36,000 | 4 | 0 |
| CORED LANCE | LANDFALL | Cored Lance | 1,100 | $170,000 | 6 | 0 |
| PEREGRINE INTERCEPTOR | LANDFALL | Enemy Airframe | 4,200 | $1,300,000 | 22 | 14 |
| ALBATROSS TRANSPORT | LANDFALL | The Farside Signal | 8,500 | $2,800,000 | 42 | 32 |
| NACRE DIVE RIG | UNDERTOW | Nacre Dive Rig | 750 | $22,000 | 4 | 0 |
| DEEP FRAME | UNDERTOW | Deep Frame | 1,150 | $50,000 | 6 | 4 |
| DEEP FRAME THRUST PACK | UNDERTOW | Deep Frame Thrust Pack | 1,500 | $62,000 | 8 | 7 |
| CAV RIFLE | UNDERTOW | Cav Rifle | 450 | $19,000 | 2 | 0 |
| CORMORANT ATTACK SUB | UNDERTOW | Enemy Pressure Hull | 4,400 | $1,400,000 | 24 | 14 |
| EMPEROR TRANSPORT SUB | UNDERTOW | Verity Deep | 9,200 | $2,900,000 | 46 | 36 |

## 23. Money

`src/main.rs` `monthly_tick`; `src/base.rs` market and sale lists.
**The funding rules are under review for 2.0.**

* Starting fund: $6,000,000.
* Hiring: soldier $40,000, scientist $60,000, engineer $50,000.
* Monthly salaries: soldier $20,000, scientist $30,000, engineer $25,000.
* Monthly upkeep: every module and every craft, from the tables above.

Each month, for each funding state or group:

* If HEARTH activity there beat alien activity by more than 30, or
  satisfaction is 80 or more: funding rises by 5–21%, satisfaction +8,
  panic −15.
* If alien activity led by more than 30, or panic is above 50: funding falls by
  5–21%, satisfaction −15, panic +18.
* Otherwise no change. Funding never drops below $40,000 while the member
  stays.
* A member with satisfaction below 15 and panic of 70 or more signs SEPARATE
  TERMS and pays nothing from then on.

Assembly rating from the running score: above 500 EXCELLENT, above 0 GOOD,
above −300 SATISFACTORY, above −600 POOR, otherwise TERRIBLE. Debt beyond
$1,000,000 draws a deficit notice.

Market prices:

| Item | Price |
|---|---|
| SERVICE RIFLE | $2,600 |
| 5.56 MAGAZINE | $300 |
| HEAVY RIFLE | $7,200 |
| DIVE RIFLE | $2,900 |
| RIFLE BOLTS | $340 |
| HEAVY DIVE RIFLE | $7,600 |
| FRAG GRENADE | $250 |
| DIVE CHARGE | $280 |
| TRAUMA KIT | $800 |
| HANDHELD RADAR / HANDHELD SONAR | $1,500 |
| SWIFT INTERCEPTOR / SKUA ATTACK SUB | $650,000 |
| HERON TRANSPORT / GANNET TRANSPORT SUB | $550,000 |

Sale prices:

| Item | Price |
|---|---|
| SPINE | $6,000 |
| GLOW | $5,500 |
| NACRE PLATE | $7,000 |
| LIVE BRINE | $6,500 |
| LIVE ALIEN CAPTIVE | $45,000 |
| ARC TORCH | $160,000 |
| ARC CUTTER | $115,000 |
| CUTTER CELL | $8,000 |
| PRESSURE RAM | $175,000 |
| PRESSURE BORE | $130,000 |
| ALIEN CORPSE | $18,000 |
| SERVICE RIFLE | $1,300 |
| DIVE RIFLE | $1,450 |
| Anything else | $700 |

## 24. Sea-lane groups

The 13 funding members of UNDERTOW: North Atlantic Compact, European Shelf
Group, East Asia Sea Lanes, Coral Sea Group, South Atlantic Compact, Gulf of
Guinea Group, Arabian Sea Group, Red Sea Authority, North Pacific Compact,
Nordic and Baltic Group, Arctic Route Authority, Tasman Group, Caribbean and
Canal Authority. The 16 Charter states of LANDFALL are real countries.

## 25. Known quirks in 0.9

* A melee creature's adjacent strike deals weapon 0's damage, not its own.
* Blast against units always reads the land damage table.
* The weapon panel shows a fallback ammunition label for weapons that use none.
* The Moon is not drawn; the LANDFALL finale marker sits on Earth, and the
  finale is fought as an ordinary battle.
* A 100% chance can still fail (section 2).

## 26. Decided, not yet in the rules

Decided with the user in `docs/LORE.md` on 2026-11-13. None of this runs in
the game yet. The build order is the game systems track in `docs/ROADMAP.md`.

* **Materials.** A third land material, SLAG, from Maul remains. Glow is
  collected from the pools the Brooder's fluid leaves. Spine is cut from
  wrecks and bodies.
* **Weapons.** Two more starting weapons for close range: SHOTGUN and KNIFE.
* **Base modules.** MESS HALL, SCRUB ROOM and REFUGEE QUARTERS.
* **Creatures.**
  * The Trestle is removed as a unit. Its job is folded into the Brooder.
  * Maul: an animal with no weapon. It blasts magma, eats bodies and dropped
    kit, and its corpse gives more SLAG the fuller it was. A Registrar
    control spike sits in its core. Destroy the spike and the Maul turns
    wild and attacks whoever is nearest, the enemy included.
  * Brooder (the taker): a hovering machine that spits glowing fluid. One
    hit melts a civilian; a soldier's armour takes several. It leaves pools
    that hurt.
  * Lathe (the builder): repairs machines, makes cover and can build one new
    Brooder. Both machines are heavily armoured while moving and exposed
    while working.
  * Kite: marks a soldier for the Lash, and its shot dazes (half TU and no
    reaction fire for one turn).
  * Bloat: bursts into a cloud that blocks sight and chokes.
  * Cinder: the Maul's young. Wild: nothing controls it, and it eats the
    enemy's dead as readily as ours. Sets fires, eats and grows bigger and
    stronger. It becomes a full Maul only between battles, never during one.
  * Gorger: a buried ambusher. Its meat feeds refugees. A buried one shows
    shifting ground and dust that a careful soldier can spot from a few
    tiles away. The HANDHELD RADAR finds one reliably.
* **Soldiers.** Each recruit has one strength and one fear, hidden until
  something in battle shows it.
  * The eight strengths are BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS,
    STEADY, SUPERSTITIOUS and LONER. Each has a built-in catch.
  * The fears are the dark, fire, tight spaces, the Glow, water and sharks.
    A fear only bites where the thing is present. Water and sharks do
    nothing on land.
  * More traits are earned in service.
  * Portraits have six stages, from new recruit to the thousand-yard stare,
    plus marks from events: scars from wounds, and changes that come with
    earned traits.
* **The soldier pool.** A page reached from the starting screen where the
  player makes soldiers once (name, look and one picture). They are mixed at
  random into the recruits of every later campaign. A "start with me" tick
  guarantees a place in the starting squad. Strength and fear are dealt
  fresh each campaign and hidden like anyone else's. Pool names and pictures
  stay on the player's machine and are never put in the public repository.
* **Refugees.** Freed Regents, Tenders and Lashes can serve as soldiers.
  Once freed they are clean: no relapse, and no added infection risk.
  * Regent: uses Reach. Cannot use human weapons.
  * Tender: small and frail. A scout that senses Reach and enemy machines.
  * Lash: the best marksman the player can have. Its first shot at any
    target costs extra time.
  * None can wear human armour. The workshop makes each kind its own
    harness.
* **Mess hall and hunger.** Soldiers must be fed. Gorger corpses stock the
  mess for refugees. Unfed soldiers lose morale and recover more slowly.
  Unfed refugees weaken, then leave. Nobody starves to death on screen.
* **Infection tax and scrubbing.** Alien-derived equipment raises a risk
  inside the base; scrubbing lowers it and never brings it to zero.
  * Low risk brings sabotage: a module goes dark, research is lost, or a
    craft is grounded.
  * High risk brings a battle that starts inside the base with no warning.
  * The SCRUB ROOM pushes the risk back down.
* **The human worker.** It first appears after the Registrar has read one of
  the player's soldiers. Early batches carry no weapon: they run at the
  squad, cling and call out in broken recorded lines ("duck calls"). Late
  batches hide among civilians, speak clearly and use human weapons.
* **Signpost missions.** Fixed story moments set off by what the player has
  done. Five so far: THE SAME FACE, FIRST WORDS, THE SPLIT VOTE, ROLL CALL
  and THE OPEN DOOR.
* **The Moon finale.** Low gravity, ground made from real lunar surface maps,
  a fight on the dark side lit by the enemy's own lights, and an ending
  decided by the freed Tenders' vote. There is no pass mark: every living
  freed Tender brought to the Moon weakens the Seed's defences.
* **Numbers.** None of the figures for the above are set. They are set by
  playing once the systems exist.
* **Multiplayer.** A player who loses may join the Registrar's side or be
  wiped out.
