# DEFENDING EARTH — NAMES

Version 0.9. Companion to `ART_BIBLE.md` and `ROADMAP.md`.

Every name in the game and what it is for. All of them are this project's own.
The code is the authority for spelling; this file is the readable list. When a
name changes in `src/`, change it here in the same edit.

Last matched to the code on 2026-11-13. This file lists the names shown on
screen. Some labels inside the code still use older words (`lattice`,
`electrolyte`, `psi_strength`, `PsiDamper`, `PsiWing`, `DerivedBattery`, and
research ids such as `LATTICE`, `LATTICE_CARRIER`, `ELECTROLYTE`, `PSI_WING`,
`PSI_WING_DEEP` and `SEED_LOCATION`). The
player never sees those, so they are not names. Names that are decided but not
built yet are in section 12.

Several tables below are index-stable: the code stores a row number, so rows
may be renamed but not reordered, inserted or removed. Those are marked
**(fixed order)**.

## 1. Organisation, campaigns and screens

| Name | What it is |
|---|---|
| **Joint Task Force HEARTH** (`HEARTH`, formal short form `JTF HEARTH`) | The player's organisation |
| **the Charter Assembly** ("the Assembly") | The funding body |
| Charter states | The 16 funding nations of the land campaign (real countries) |
| sea-lane groups | The 13 funding members of the sea campaign (section 9) |
| SEPARATE TERMS | The status of a member that has left and made its own peace with the enemy |
| **LANDFALL** ("Operation Landfall") | The land-and-air campaign. Starts 2027 |
| **UNDERTOW** ("Operation Undertow") | The sea-floor campaign. Starts 2027 |
| SITUATION MAP | The world-map screen |
| FIELD VIEW | The tactical screen |
| CITY RAID ("raid") | An enemy attack on a city |
| RAID CRAFT | The enemy craft carrying out a city raid |
| DIVE GEAR, DIVE ARMS, divers | Underwater kit, weapons and soldiers |
| TRAUMA KIT | The healing item |
| live study | Research on a live captive |
| HEARTH ALPHA, BRAVO, CHARLIE … | Land base names, in founding order |
| HEARTH DEEP 1, 2, 3 … | Sea-floor station names, in founding order |
| the Registrar | The enemy: a cataloguing machine. First met as a broken fragment of the word. Lore only, not yet in the UI (see `LORE.md`) |

### Ranks (fixed order)

`RECRUIT`, `PRIVATE`, `CORPORAL`, `SERGEANT`, `LIEUTENANT`, `CAPTAIN`, `MAJOR`.

### Design steps

The four words used across weapons, armour, craft and base modules.

| Step | Meaning | How the name shows it |
|---|---|---|
| **Issue** | What armies and police have today | Named the way a quartermaster would list it: service rifle, machine gun, frag grenade |
| **Adapted** | The same kit rebuilt around a recovered part | *Cored* on land (a sealed core filled with Glow, in place of a magazine); *cav* at sea (a brine-driven slug that travels in its own vapour pocket) |
| **Enemy tool** | The enemy's own equipment, used as found | Named by our soldiers after workshop tools: arc stylus, cutter, torch on land; pressure awl, bore, ram at sea |
| **Derived** | Our own design, built their way | Plain functional names: breacher, implosion pod, deep frame |

## 2. Creatures

One maker's work (see `ART_BIBLE.md` section 5). Names are single words; the
live-study matching relies on that.

**The land creature names are final** (decided 2026-11-13, see `LORE.md`):
Tender, Kite, Lash, Maul, Regent, Brooder, Lathe, Gorger, Bloat, Cinder. They
are no longer placeholders. The sea names are still placeholders; the sea
campaign is on hold.

The Role column says what each kind does in the game today. Several kinds have
new rules decided in `LORE.md` that are not built yet (section 12).

### Land (kinds 0..10, fixed order)

| Kind | Name | Role |
|---|---|---|
| 0 | **Tender** | Small handler. Fragile, quick, strong-minded |
| 1 | **Kite** | Gliding membrane scout. Fast, sharp reactions, tears easily |
| 2 | **Lash** | Serpent hunter. Scaled, steady shot |
| 3 | **Maul** | Heavy infantry. Carries the arc torch |
| 4 | **Regent** | Reach commander. The only land kind that routs and takes soldiers |
| 5 | **Brooder** | Parasite breeder. Fast melee |
| 6 | **Lathe** | Hovering fabrication drone turned weapon |
| 7 | **Gorger** | Large pack beast. Melee, soft underside |
| 8 | **Trestle** | Walking weapons platform. Heaviest plate in the game. To be removed (section 12); the row number stays reserved |
| 9 | **Bloat** | Drifting spitter |
| 10 | **Cinder** | Slow mineral-bodied crawler. Not yet in any spawn pool |

### Sea (kinds 0..8, fixed order)

| Kind | Name | Role |
|---|---|---|
| 0 | **Fathom** | Line infantry |
| 1 | **Wader** | Amphibious raider. Quick, scaled |
| 2 | **Mantle** | Armoured heavy. Carries the pressure ram |
| 3 | **Deacon** | Reach officer. The only sea kind that routs and takes soldiers |
| 4 | **Bell** | Pressure-shelled brute. Slow melee, very hard shell |
| 5 | **Silt** | Small bottom-lying ambusher. Fast melee, almost no armour |
| 6 | **Lamp** | Lure-light drone. Floats, cannot be stunned |
| 7 | **Wreath** | Drifting parasite. Fast floating melee |
| 8 | **Hulk** | Massive beast. The biggest health pool in the game |

### Reserved

**Husk** — a taken soldier or animal, still wearing what it wore. Not a row in
either table: it is a state of a soldier record, kept for the taken-soldier
mechanic and "DO IT." (see `ROADMAP.md`). Armour class 0 is held for it.

## 3. Soldier weapons (fixed order)

| # | Name | Short | Step | What it is |
|---|---|---|---|---|
| 0 | SERVICE PISTOL | PST | Issue | Sidearm. Feeds 9MM MAGAZINE |
| 1 | SERVICE RIFLE | RIF | Issue | Standard rifle. Feeds 5.56 MAGAZINE |
| 2 | HEAVY RIFLE | HVR | Issue | Anti-materiel rifle. Feeds .50 MAGAZINE |
| 3 | MACHINE GUN | MG | Issue | Squad support gun. Feeds 7.62 BELT |
| 4 | AT LAUNCHER | ATL | Issue | Shoulder launcher. Fires HE ROCKET |
| 5 | CORED PISTOL | CRP | Adapted | Service pistol frame with a sealed core. No magazine |
| 6 | CORED RIFLE | CRR | Adapted | Service rifle layout with a sealed core |
| 7 | CORED HEAVY GUN | CRH | Adapted | Heavy mount for the largest carried core |
| 8 | ARC STYLUS | AST | Enemy tool | Their small hand tool. Feeds STYLUS CELL |
| 9 | ARC CUTTER | ACT | Enemy tool | Their standard tool. Feeds CUTTER CELL |
| 10 | ARC TORCH | ATR | Enemy tool | Their heavy tool. Feeds TORCH CELL |
| 11 | FRAG GRENADE | FRG | Issue | Thrown fragmentation grenade |
| 12 | DEMO CHARGE | DMO | Issue | Thrown demolition charge |
| 13 | STUN BATON | BTN | Issue | Melee stun, for live captures |
| 14 | DIVE PISTOL | DPS | Issue | Underwater sidearm. Feeds PISTOL BOLTS |
| 15 | DIVE RIFLE | DRF | Issue | Underwater rifle. Feeds RIFLE BOLTS |
| 16 | HEAVY DIVE RIFLE | HDR | Issue | Heavy underwater rifle. Feeds HEAVY BOLTS |
| 17 | DIVE MG | DMG | Issue | Underwater support gun. Feeds BOLT BELT |
| 18 | TORPEDO TUBE | TRP | Issue | Shoulder tube. Fires MINI TORPEDO |
| 19 | DIVE CHARGE | CHG | Issue | Thrown underwater charge |
| 20 | STUN PROD | PRD | Issue | Underwater melee stun |
| 21 | DIVE KNIFE | KNF | Issue | Underwater blade |
| 22 | CAV PISTOL | CVP | Adapted | Dive pistol with a cav barrel. Feeds CAV PISTOL MAG |
| 23 | CAV RIFLE | CVR | Adapted | Dive rifle layout with a cav barrel. Feeds CAV RIFLE MAG |
| 24 | CAV HEAVY GUN | CVH | Adapted | Heavy cav mount. Feeds CAV HEAVY MAG |
| 25 | PRESSURE AWL | AWL | Enemy tool | Their small hand tool at sea. Feeds AWL CHARGE |
| 26 | PRESSURE BORE | BOR | Enemy tool | Their standard tool at sea. Feeds BORE CHARGE |
| 27 | PRESSURE RAM | RAM | Enemy tool | Their heavy tool at sea. Feeds RAM CHARGE |
| 28 | SNARE PROJECTOR | SNR | Adapted | Ranged stun launcher. Fires SNARE CHARGE |
| 29 | IMPLOSION POD | IMP | Derived | Thrown charge that collapses the water around it |
| 30 | NATURAL WEAPON | NAT | — | Claws and teeth. Land creatures only; never offered to soldiers |
| 31 | NATURAL WEAPON (DEEP) | NAT | — | The same, for sea creatures |

## 4. Other items

Both item tables hold 80 fixed slots; unused slots are named `RESERVED`.

| Name | Table | What it is |
|---|---|---|
| SMOKE GRENADE | Land | Thrown smoke |
| TRIP MINE | Land | Placed charge set off by movement |
| HANDHELD RADAR | Land | Short-range movement detector |
| HANDHELD SONAR | Sea | The same, underwater |
| TRAUMA KIT | Both | Healing item |
| `<NAME>` CORPSE | Both | A dead creature brought back for study, one item per kind |
| LATHE WRECK, TRESTLE WRECK, LAMP WRECK | Both | The same for the three machine kinds |
| LIVE `<NAME>` | Both | A live captive, one item per kind |
| DISABLED LATHE, DISABLED TRESTLE, DISABLED LAMP | Both | The same for the three machine kinds |
| ALIEN CORPSE | Stores | The generic body item a won mission adds to base stores |
| LIVE ALIEN CAPTIVE | Stores | The generic market entry for a held captive |

Soldier pack slots in the field view also show `SMOKE CHARGE` (the pack label
for a carried smoke grenade) and `EMPTY COMPARTMENT`.

## 5. Materials and fuel

| Name | Short | Campaign | What it is |
|---|---|---|---|
| SPINE | — | LANDFALL | The printed frame under every creature and hull, cut from their wrecks and their dead. Structure material |
| GLOW | — | LANDFALL | The shining fluid every enemy machine runs on, gathered from the pools they leave. Power material and Adapted craft fuel |
| NACRE PLATE | NACRE | UNDERTOW | The layered shell the sea-going builds grow in place of Spine. Structure material |
| LIVE BRINE | BRINE | UNDERTOW | The sea match for Glow; stays charged only while cold and dark. Power material and Adapted craft fuel |
| JET FUEL | — | LANDFALL | Fuel for Issue aircraft |
| DIESEL-ELECTRIC | — | UNDERTOW | Power for Issue submarines |

Blueprint costs name the two materials generically as *structure* and *power*.

In the game today a won mission simply adds a set amount of both materials to
the base. Cutting Spine from wrecks and collecting Glow from pools is decided
in `LORE.md` but not built yet. In the stores list at a base of the other
campaign, the four show as SPINE (LAND), GLOW (LAND), NACRE (SEA) and
BRINE (SEA).

## 6. Soldier armour (fixed order)

| # | Name | Step | What it is |
|---|---|---|---|
| 0 | COMBAT UNIFORM | Issue | Fatigues, helmet and webbing. What a soldier arrives in |
| 1 | SPINE CARRIER | Adapted | Plate carrier with cut Spine plates |
| 2 | SEALED FRAME | Derived | Powered sealed shell on a Spine frame, driven by Glow |
| 3 | DIVE RIG | Issue | Chest plate and twin rebreather bottles |
| 4 | NACRE DIVE RIG | Adapted | The dive rig with nacre plate over it |
| 5 | DEEP FRAME | Derived | Powered pressure suit grown in nacre |

DEEP FRAME THRUST PACK is the workshop upgrade that adds a guided thrust pack
and trim fins to the Deep Frame.

## 7. Damage types and armour classes (fixed ids)

| Id | Land label | Sea label | What it is |
|---|---|---|---|
| 0 | KIN | KIN | Bullets and bolts |
| 1 | HEAT | HEAT | Fire and burning |
| 2 | BLAST | BLAST | Explosive overpressure and fragments |
| 3 | CORE | CAV | Adapted weapons: core discharge on land, cavitating slug at sea |
| 4 | ARC | PRES | Enemy tools: arc on land, pressure at sea |
| 5 | STUN | STUN | Shock, and smoke |
| 6 | CUT | CUT | Blades, claws, teeth |
| 7 | ACID | ACID | Corrosive spit (reserved) |

| Class | Name | Who has it |
|---|---|---|
| 0 | Bare | Reserved: civilians, animals, Husks |
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
| 11 | Machine plate | Trestle |
| 12 | Drone casing | Lathe, Lamp |
| 13 | Pressure shell | Bell |

## 8. Craft

### Enemy craft classes

Reporting names are botanical, in keeping with the Seed and the Root. On the
map and in interception each shows as `<NAME>-CLASS UFO` in the air or
`<NAME>-CLASS USO` at sea.

| Class | Role | Weapon |
|---|---|---|
| SPORE | Light scout | ARC NEEDLE |
| BURR | Collector: takes people and livestock | ARC LANCE |
| THORN | City raider | ARC VOLLEY |
| RHIZOME | Undersea runner (sea only) | PRESSURE LANCE |
| TAPROOT | Capital craft | MAIN DISCHARGE |

### Player craft

Named after birds: land birds and long-range fliers for aircraft, diving
seabirds for submarines.

| Name | Campaign | Step | Role |
|---|---|---|---|
| HERON | LANDFALL | Issue | Twin-rotor transport helicopter. Slow, long-legged, lands anywhere |
| SWIFT | LANDFALL | Issue | Twin-engine interceptor with two hardpoints |
| PEREGRINE | LANDFALL | Adapted | Interceptor: a human airframe around a recovered power plant |
| IBIS | LANDFALL | Adapted | Assault transport. Carries a squad and one weapon |
| ALBATROSS | LANDFALL | Derived | Long-range transport. The only craft that can reach the far side of the Moon |
| GANNET | UNDERTOW | Issue | Transport submersible with a lock-out chamber for a full dive team |
| SKUA | UNDERTOW | Issue | Small attack submarine with two torpedo stations |
| CORMORANT | UNDERTOW | Adapted | Attack sub: a human hull around a recovered brine drive |
| RAZORBILL | UNDERTOW | Adapted | Assault sub. Carries a dive team and one weapon |
| EMPEROR | UNDERTOW | Derived | Deep transport. The only craft rated for Verity Deep |

### Craft weapons

| Name | Campaign | Step | What it is |
|---|---|---|---|
| SHORT-RANGE MISSILE | LANDFALL | Issue | Heat-seeking missile. Six to a rail set |
| LONG-RANGE MISSILE | LANDFALL | Issue | Radar-guided missile. Reaches further, carries fewer |
| GUN POD (20MM) | LANDFALL | Issue | Rotary cannon. Short reach, deep magazine |
| CORED LANCE | LANDFALL | Adapted | A sealed core behind a long barrel. Never needs rearming |
| ARC PROJECTOR | LANDFALL | Enemy tool | An enemy arc tool scaled up and bolted to a human pylon |
| BREACHER MISSILE | LANDFALL | Derived | Two rounds, each built to open a capital craft |
| LIGHT TORPEDO | UNDERTOW | Issue | Homing torpedo. Eight to a load |
| HEAVY TORPEDO | UNDERTOW | Issue | Wire-guided torpedo. Slower to reload, harder hit |
| BOLT CANNON | UNDERTOW | Issue | Rapid-fire bolt gun for knife-fight range |
| CAV CANNON | UNDERTOW | Adapted | Brine-driven slugs that travel in their own vapour pocket |
| PRESSURE PROJECTOR | UNDERTOW | Enemy tool | An enemy pressure tool scaled up and mounted in a human turret |
| BREACHER TORPEDO | UNDERTOW | Derived | Two rounds, each built to open a capital hull |

## 9. Base modules

| Land name | Sea name | What it is |
|---|---|---|
| ENTRY SHAFT | MOON POOL | Shaft and lock to the outside. Every other module must connect back to it |
| BARRACKS | BERTHING | Bunks, mess and showers for soldiers, scientists and engineers |
| RESEARCH LAB | WET LAB | Bench space for scientists |
| MACHINE SHOP | MACHINE SHOP | Floor space and tooling for engineers |
| SEARCH RADAR | SEARCH SONAR | Rotating search set |
| LONG-RANGE RADAR | TOWED ARRAY | Large fixed array with twice the reach |
| SAM BATTERY | TORPEDO BATTERY | Issue point-defence battery |
| SUPPLY BAY | SUPPLY BAY | Racked storage for kit and recovered material |
| HOLDING BLOCK | HOLDING TANK | Sealed cells for live captives awaiting study |
| CORED BATTERY | CAV BATTERY | Adapted point defence. Fast and accurate |
| ARC BATTERY | PRESSURE BATTERY | Enemy tools on a human mount. Hits hard, runs hot |
| BREACHER BATTERY | BREACHER BATTERY | Our own design, built their way. The last line, and the heaviest |
| FARADAY SCREEN | BAFFLE SCREEN | Bleeds off incoming fire and slows the attackers' approach |
| REACH DAMPER | REACH DAMPER | Keeps the base quiet to whatever the enemy listens with |
| REACH WING | REACH WING | Testing and training rooms for soldiers who show Reach ability |
| SIGINT STATION | SIGINT STATION | Reads the enemy's traffic: every contact in range, with class, crew and heading |
| HANGAR | WET DOCK | 2×2 bay for one craft, with fuel, arming and a way out |

Sea-lane groups (the 13 UNDERTOW funding members, fixed order): NORTH ATLANTIC
COMPACT, EUROPEAN SHELF GROUP, EAST ASIA SEA LANES, CORAL SEA GROUP, SOUTH
ATLANTIC COMPACT, GULF OF GUINEA GROUP, ARABIAN SEA GROUP, RED SEA AUTHORITY,
NORTH PACIFIC COMPACT, NORDIC AND BALTIC GROUP, ARCTIC ROUTE AUTHORITY, TASMAN
GROUP, CARIBBEAN AND CANAL AUTHORITY. The first four are also drawn as regions
on the situation map and are matched to their outlines by name.

## 10. Research

Categories: Weapons, Armor, Materials, Propulsion, Craft, Facilities,
Reach, Story, Live Study.

"Reach" is this game's word for one mind pushing on another. A Regent reaches
a soldier. In the field view the button reads REACH (key P).

### LANDFALL

| Topic | Category | Leads to |
|---|---|---|
| CORED WEAPONS | Weapons | Cored Pistol, Cored Rifle, Cored Heavy Gun |
| CORED PISTOL | Weapons | Armoury: Cored Pistol |
| CORED RIFLE | Weapons | Armoury: Cored Rifle |
| CORED HEAVY GUN | Weapons | Armoury: Cored Heavy Gun |
| CORED LANCE | Craft | Aircraft weapon: Cored Lance |
| SPINE | Materials | Spine Carrier, enemy craft studies |
| SPINE CARRIER | Armor | Workshop: Spine Carrier |
| SEALED FRAME | Armor | Workshop: Sealed Frame |
| ARC TOOLS | Weapons | Arc Stylus, Arc Cutter, Arc Torch |
| GLOW | Propulsion | Sealed Frame, enemy power plant study |
| ENEMY POWER PLANT | Propulsion | Enemy airframe study |
| ENEMY GUIDANCE | Propulsion | SIGINT Station, enemy airframe study |
| ENEMY AIRFRAME | Craft | Workshop: Peregrine |
| SIGINT STATION | Facilities | Facility: SIGINT Station |
| REACH STUDIES | Reach | Facility: Reach Wing |
| THE FARSIDE SIGNAL | Story | Objective: the far side of the Moon |
| LIVE STUDY: TENDER / LASH / MAUL / REGENT | Live Study | That creature's file |

### UNDERTOW

| Topic | Category | Leads to |
|---|---|---|
| NACRE PLATE | Materials | Nacre Dive Rig, enemy sea guidance study |
| NACRE DIVE RIG | Armor | Workshop: Nacre Dive Rig |
| DEEP FRAME | Armor | Workshop: Deep Frame |
| DEEP FRAME THRUST PACK | Armor | Workshop: Deep Frame Thrust Pack |
| CAV WEAPONS | Weapons | Cav Pistol, Cav Rifle, Cav Heavy Gun |
| CAV RIFLE | Weapons | Workshop: Cav Rifle |
| CAV HEAVY GUN | Weapons | Armoury: Cav Heavy Gun |
| PRESSURE TOOLS | Weapons | Pressure Awl, Pressure Bore, Pressure Ram |
| LIVE BRINE | Propulsion | Deep Frame, brine drive study |
| ENEMY SEA GUIDANCE | Propulsion | SIGINT Station, enemy pressure hull study |
| BRINE DRIVE | Propulsion | Enemy pressure hull study |
| ENEMY PRESSURE HULL | Craft | Workshop: Cormorant |
| SIGINT STATION | Facilities | Facility: SIGINT Station |
| REACH STUDIES | Reach | Facility: Reach Wing |
| THE ROOT SIGNAL | Story | Verity Deep study |
| VERITY DEEP | Story | Objective: Verity Deep |
| LIVE STUDY: FATHOM / WADER / MANTLE / DEACON | Live Study | That creature's file |

### Workshop blueprints

LANDFALL: SPINE CARRIER, SEALED FRAME, CORED PISTOL, CORED RIFLE, CORED HEAVY
GUN, CORED LANCE, PEREGRINE INTERCEPTOR, ALBATROSS TRANSPORT.

UNDERTOW: NACRE DIVE RIG, DEEP FRAME, DEEP FRAME THRUST PACK, CAV RIFLE,
CORMORANT ATTACK SUB, EMPEROR TRANSPORT SUB.

## 11. The two final sites

| | LANDFALL | UNDERTOW |
|---|---|---|
| Place | **The far side of the Moon** — a real place: the half of the Moon that always faces away from Earth. Moved here from Mars on 2026-11-13 | **Verity Deep** — our invention: a hadal basin (the very deepest kind of sea floor) in the South Pacific near the point furthest from any land, about 47°S 126°W, named in the fiction for the survey ship that first sounded it |
| Enemy installation | **the Seed** — the Registrar's crashed seed | **the Root** — what the Seed put down on Earth |
| Why there | It is the side of the Moon we never see. The fight is on the dark half, so the Registrar's own lights are what light the ground | The deepest, quietest water on Earth, with nobody near it |
| Found by research | THE FARSIDE SIGNAL | THE ROOT SIGNAL, then VERITY DEEP |
| Craft needed | ALBATROSS | EMPEROR |
| Map label | LUNAR FAR SIDE // FINAL ASSAULT | VERITY DEEP // FINAL ASSAULT |
| Win screen | THE MOON IS SILENT | THE DEEP IS SILENT |

## 12. Decided, not yet in the game

These names are settled in `LORE.md` but are not in the code yet. Do not look
for them on screen. When one is built, move it into its table above.

| Name | Kind | What it is |
|---|---|---|
| SLAG | Material | What is left of a dead Maul. Limited and valuable. The fuller the Maul was when it died, the more there is. Needed for the best heat and blast protection and a late close-range magma weapon |
| MESS HALL | Base module | Feeds the soldiers, and later the refugees |
| SCRUB ROOM | Base module | Cleans the Registrar's hidden copy out of what we build from enemy parts |
| REFUGEE QUARTERS | Base module | Where freed aliens live |
| OVERWATCH | Battle command | Planned, not yet in the game. A button that puts a unit on guard. It takes reaction shots in the other side's turn, each paid from its remaining time units. Aliens use the same rule |
| SHOTGUN | Soldier weapon | Starting weapon for close range. It should go at the end of the weapon list, so no row number moves |
| KNIFE | Soldier weapon | Starting blade for close range. Also to be added at the end |
| refugees | People | Freed aliens who can serve as soldiers: Regent, Tender, Lash. `LORE.md` also calls them "defectors"; it means the same people |
| BRAVE, COLD, EMPATH, PROTECTIVE, RECKLESS, STEADY, SUPERSTITIOUS, LONER | Soldier traits (strengths) | The eight starter traits. All eight are strengths, each with its own built-in catch. Every recruit gets one strength and one fear, both hidden until something in battle shows them |
| FIXED DICE | New-campaign option | The save keeps where the dice are, so reloading and repeating an action gives the same result. Locked for the whole campaign |
| IRON MAN MODE | New-campaign option | One save slot, saved by the game after every decision. A finished campaign's save is kept to look at, not to play on. Locked for the whole campaign |
| SEED | New-campaign page | The number every roll of the campaign starts from. Shown on the page; the player can type one in |
| ROLL LOG | Debrief | The list of every rule roll in the battle (who, what, odds, result). Shown only after the battle |

### Soldiers: strengths and fears (third session, 2026-11-13)

The rule: **the eight starter traits are strengths, and fears are flaws.**
A recruit gets one of each. Both are dealt at random and stay hidden until
something in battle shows them.

| Name | Kind | What it is |
|---|---|---|
| strength | Term | One of the eight starter traits in the row above. The good half of a recruit, with a catch built in |
| fear | Term | A recruit's flaw. It only bites where the feared thing is present, so it can stay hidden for years |
| THE DARK | Fear (land) | Afraid of the dark |
| FIRE | Fear (land) | Afraid of fire |
| TIGHT SPACES | Fear (land) | Afraid of being closed in |
| THE GLOW | Fear (land) | Afraid of Glow, the enemy's shining fluid |
| WATER | Fear (sea only) | Afraid of water. Does nothing on land, so it stays asleep until the sea campaign exists |
| SHARKS | Fear (sea only) | Afraid of sharks. Sea only, the same as WATER |
| portrait stages | Term | The six steps a soldier's close-up face goes through (applied to the soldier's built head since the sixth session), from new recruit to the thousand-yard stare. Events add marks on top: scars from wounds, and changes that come with earned traits |

The older spellings AFRAID OF WATER and AFRAID OF SHARKS are replaced by
WATER and SHARKS. `LORE.md` writes the six fears in plain words ("the dark,
fire..."). The capital letters here are the working screen spelling, still to
settle with the user. What each fear does to a soldier in battle is not
designed yet.

### The soldier pool (third session)

| Name | Kind | What it is |
|---|---|---|
| SOLDIER POOL ("the pool"; `LORE.md` says "the soldier pool", the capitals are a working screen spelling) | Start-page screen | A page reached from the game's starting screen. The player makes soldiers there once, in a character designer (sixth session, 2026-11-13; it replaces "a look and a picture"). A photo is optional. They are then pulled at random into every campaign started afterwards |
| pool soldier | Term | A soldier made in the pool. Mixed at random into the recruits offered during a campaign. Its strength and fear are dealt fresh in every campaign and hidden like anyone else's |
| "start with me" | Tick box on the pool page | Guarantees that pool soldier a place in the starting squad |
| pool picture | Term | Replaced in the sixth session (2026-11-13): a pool soldier no longer has a stored picture. Its 3D head is built from saved slider values and the portrait is a picture of that head. The six portrait stages and event marks apply to the head. Any photo is read once and thrown away. Names and slider values stay on the player's machine. They are never put in the public repository (the project's shared online folder) |

### Ribbons and citations (sixth session, 2026-11-13)

Ribbon and citation names beyond the four below are to settle. The capitals
are a working screen spelling. `LORE.md` writes them in plain words.

| Name | Kind | What it is |
|---|---|---|
| ribbon | Term (name to settle) | What a soldier earns by doing one kind of thing again and again. Ten to start: rifle, shotgun, knife, explosives, medic, scout, overwatch, Reach, survivor, creatures. Five levels each. Ribbons replace jobs: there are no classes. Names and designs of each ribbon are to settle |
| ribbon board | Term (name to settle) | The board on the dress uniform where a soldier's ribbons are worn. A higher level replaces the lower one |
| citation | Term (name to settle) | A hidden ability a soldier earns from a mix of ribbons. It stays hidden until one of the player's soldiers earns it, then shows in a book of citations (name to settle) with how it was earned, and stays known in every later campaign |
| RANGER | Citation | Rifle plus scout. Sees further, and gets one overwatch shot each turn that costs no time units. Exact effects to settle |
| BREACHER | Citation | Shotgun plus explosives. The first shot after going through a door or a blown wall costs no time units. Exact effects to settle. Shares a word with BREACHER MISSILE, BREACHER TORPEDO and BREACHER BATTERY; to settle whether that is a clash |
| GUARDIAN | Citation | Medic plus overwatch. A free reaction shot at anything that attacks a soldier they healed this battle. Exact effects to settle |
| GHOST | Citation | Knife plus scout. Enemies that have not spotted them cannot take reaction shots at them, and they leave no footprints. Exact effects to settle |

### Creatures (third session)

| Name | Kind | What it is |
|---|---|---|
| control spike ("the spike") | Enemy hardware | A piece of Registrar hardware driven into a Maul's molten core. It is what controls the Maul. Destroy it and the Maul goes wild |
| wild | Creature state | A creature nothing controls. A wild Maul attacks whoever is nearest, the enemy included. Cinders are always wild: they eat and set fires, and go for the enemy's dead as readily as ours |
| Gorger signs | Term | Shifting ground and dust over a buried Gorger. A careful soldier can spot them from a few tiles away. The HANDHELD RADAR finds a buried Gorger reliably |
| human worker | Enemy kind (no screen name yet) | The Registrar's copies of real people it has read. First appears after it has read one of the player's soldiers. Early batches carry no weapon: they run at the squad, cling and call out in broken recorded lines. Late batches hide among civilians and use human weapons |
| batch | Term | One run of human workers. Each batch is closer to a real person than the last |

### Refugees and their kit (third session)

| Name | Kind | What it is |
|---|---|---|
| harness | Kit for refugees | Refugees cannot wear human armour. The workshop makes each kind (Regent, Tender, Lash) its own harness. The three harnesses have no names of their own yet. Note: the battle screen already uses the word for a soldier's SHOULDER HARNESS and BELT HARNESS (ammunition straps), so the refugee harness may need a clearer name. To settle |

What each refugee does as a soldier: a Regent uses Reach and cannot use
human weapons. A Tender is small and frail, a scout that senses Reach and
enemy machines. A Lash is the best marksman the player can have, but its
first shot at any target costs extra time, because it must ask the target's
pardon.

### Signpost missions (third session)

A signpost mission is a fixed story mission that marks a turn in the
campaign. Each is set off by something the player has done, so players meet
them in different orders and may not see them all. `LORE.md` writes the
titles in plain words ("The same face"); the capitals are a working screen
spelling.

| Name | Set off by | What it is |
|---|---|---|
| THE SAME FACE | A soldier the Registrar was able to read: abducted, mind-taken, or left on the field when the squad withdrew. The exact moment that sets it off is still to settle | Every enemy on the map is a copy of that one soldier: deformed, unfinished, with stats close to the original's. It copies the highest-ranked soldier it was able to read. A soldier killed and brought home cannot be copied |
| FIRST WORDS | The first live capture | In the holding block the captive starts speaking the Registrar's first gibberish, built from words the player's own staff have said |
| THE SPLIT VOTE | First studying a Regent | In the next battle a Regent hesitates, lets a soldier live, and is recycled in front of the squad |
| ROLL CALL | A late batch of human workers appearing | A rescue mission where the "civilians" call the soldiers by name, in the voices of people the player lost |
| THE OPEN DOOR | Infection risk running high for the first time | The first battle that starts inside the player's own base. It teaches scrubbing |

### Running the base (third session)

| Name | Kind | What it is |
|---|---|---|
| hunger | Base rule | What happens when the MESS HALL cannot feed everyone. Unfed soldiers lose morale and recover more slowly. Unfed refugees weaken, then leave. Nobody starves to death on screen. Gorger corpses stock the mess for refugees |
| infection risk | Base rule | How likely the Registrar's hidden copy, carried in equipment built from enemy parts, is to act inside a base. Low risk brings sabotage. High risk brings a battle that starts inside the base with no warning, enemies stepping out of the workshop and stores. The SCRUB ROOM pushes the risk back down, never to zero. Refugees add no risk: once freed they are clean |
| sabotage | Base event | What low infection risk causes: a module goes dark, research is lost, or a craft is grounded |
| scrubbing | Term | Cleaning the Registrar's copy out of what we build. Done in the SCRUB ROOM. It costs power and resources |

The exact numbers for hunger and infection are left for later. They will be
set with the test bench, then checked by playing, once the systems exist.

### Changes to names already in the tables above

- **Trestle** (land kind 8) is to be removed as a unit. It was folded into the
  Brooder. Its row number, and the TRESTLE WRECK and DISABLED TRESTLE item
  slots, should be kept as `RESERVED` so nothing else moves. Today the Trestle
  is still in the game.
- **Brooder** keeps its name but becomes "the taker": a hovering machine that
  spits Glow. **Lathe** becomes "the builder". Their Role cells will change
  when the new rules are built.
- **Maul** will lose the arc torch and attack with its own magma. It carries
  a control spike (see the creature table above). **Cinder** becomes the
  Maul's young: a glowing cube-shaped rock that rolls around, wild, and grows
  as it eats. It only becomes a full Maul between battles, never during one.
- **HANDHELD RADAR** gets its first job in battle: finding buried Gorgers.
  Today it can be bought but does nothing in a battle.
- **Kite** becomes a spotter that marks soldiers for the Lash; its only shot
  dazes. **Bloat** becomes a floater that bursts into a choking cloud.
  **Gorger** becomes a buried ambusher, and food for refugees.
