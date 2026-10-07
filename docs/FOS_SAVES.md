# The original game's saves (`.fos`)

M7 research (docs/TASKS.md): the layout of Fallout: New Vegas's own save
files, what nv-rs needs from them to continue a game, and what nv-rs is
missing to take it. Branch `claude/fos-saves`.

Status: **format researched and checked** against real saves; a
read-only reader (`crates/fos`) and inspector (`nvinspect fos`) are
implemented and tested. No import into nv-rs state yet.

- Every structure below was read from the writer (and, where it matters, the
  reader) in FalloutNV.exe 1.4.0.525 (addresses are PC unless marked) and
  named from the Xbox 360 prototype's symbols where they help (Xbox PDB).
- Every structure was then checked against nine real saves made by the
  1.4.0.525 game (five named saves, the autosave and quicksave and their two
  `.bak` copies; 1.65 to 1.87 MB, all early game, plugin list `FalloutNV.esm`
  only). All nine walk from the first byte to the last with no byte left over,
  and every change form of the types marked "decoded" below decodes to its
  exact length. The saves themselves stay private.
- Community pages (UESP's save format pages) were not needed; nothing below
  is taken from them.
- The `.nvse` files next to the saves are the script extender's co-saves, a
  separate format, not covered here.

## Conventions

- Little-endian throughout. `u8/u16/u32/i32/f32/f64` as usual.
- **Pipe buffer.** Most of the file is written through `BGSSaveGameBuffer`
  (Xbox PDB), which appends a `|` (0x7C) after every value it writes
  (`00865ce0`; `00865570` with its last argument 0). Below, `x|` means "x
  then a `|` byte". Parts written straight to the file have no `|`.
- **String** `wstr`: `u16 length|` then, only if the length isn't 0, the
  characters (no terminator) and `|` (`00865e70`).
- **vsval** (variable-sized value, `BGSSaveGameBuffer::SaveVariableSizedValue`
  (Xbox PDB), `00865ff0`): the low two bits of the first byte give the size,
  0 = 1 byte, 1 = 2 bytes, 2 = 4 bytes; the value is the whole little-endian
  number shifted right by 2. Then `|`. The writer first puts a `#`
  placeholder (`00865f20`) and widens it once the count is known, so counts
  written before their items always use this encoding. Values of 2^30 or
  more can't be stored (the writer logs an error).
- **refID** (`BGSSaveLoadFormInfo` reference, `00853570` writes, `00853500`
  reads): 3 bytes, most significant first (`b0 << 16 | b1 << 8 | b2`).
  - bit 0x800000 set: a form created in game; its form id is
    `0xFF000000 | (refID & 0x7FFFFF)`.
  - otherwise: an index into the **form id array** (below), counting from 1;
    0 is "none". The writer assigns indices in the order it first meets each
    form (`00846c90`).
- Form ids inside the form id array carry the **save's** plugin index in
  their top byte. The loader maps that index to the current load order by
  plugin name, ignoring case (`00847660` builds `cFileIndexArray` (Xbox PDB);
  `00846d80` remaps); a form whose plugin is missing becomes 0 and is
  dropped ("SAVELOAD: Cannot find file %s referenced in the save game").
  `0xFF` ids are not remapped.

## File layout

Written by `008503b0` (the save driver): header `0084d4b0` (through
`00850ea0`), then `BGSSaveLoadGame::SaveGame` (Xbox PDB) `00847850`. Read by
`00850760` → version check `00850ed0` → `BGSSaveLoadGame::LoadGame` `00847df0`.

| Part | Layout | Notes |
| --- | --- | --- |
| Magic | `"FO3SAVEGAME"` (11 bytes, raw) | The FO3 name kept. |
| Header size | `u32` (raw) | Bytes of the header block that follows (140 in every file seen). |
| Header | pipe buffer, see below | |
| Screenshot | `width × height × 3` bytes, RGB, raw | Dimensions from the header. |
| Minor version | `u8` (raw) | `cCurrentMinorVersion` (Xbox PDB). The writer uses `00851110` = 0x1B (27); every change form repeats it. |
| Plugins | `u32 size` (raw), then pipe buffer: `u8 count|`, `count × wstr` | `00847590`. Size covers the pipe buffer. |
| File location table | 0x6E bytes (raw), see below | Written as zeros first, filled in at the end (`00847850`). |
| Global data table 1 | `count1 ×` global data | Types 0 to 11, ascending (`0084bb50(file, 0, 12)`). |
| Change forms | `changeFormCount ×` change form | |
| Global data table 2 | `count2 ×` global data | Type 1000 only (`0084bb50(file, 1000, 1001)`). |
| Form id array | `u32 count`, `count × u32` form id (raw) | `pFormIDMap` (Xbox PDB), `00846e00`. Element *i* is refID *i + 1*. |
| Worldspace id array | `u32 count`, `count × u32` form id (raw) | `pWorldspaceFormIDMap` (Xbox PDB), same writer. Indexed by the exterior-cell initial data (below). In every file seen: one entry, `WastelandNV`. |
| History | `u32 size` (raw), pipe buffer: `u32 count|`, `count × wstr` | `pHistory` (Xbox PDB), `0084e0d0`. Empty (count 0) in every file seen. End of file. |

### Header (pipe buffer, `0084d4b0`)

| Field | Layout | Source |
| --- | --- | --- |
| Version | `u32|` | `0066d730` = 0x30 (48). The loader refuses anything but 0x2F or 0x30 ("outdated" / "comes from a newer executable", `00850ed0`). |
| Language | 64 bytes, NUL-padded, then `|` | The game's language setting (`ENGLISH`). |
| Screenshot width, height | `u32|`, `u32|` | 512 or 432 wide, chosen by a test of the display's proportions against 0.75 in `0084d4b0`; height from the capture (`00878f60`). Every file seen: 512 × 288. |
| Save number | `u32|` | `BGSSaveLoadManager` (`011de134`) +8, incremented after a save unless `00851a20` or `00851a60` holds for its name (apparently the autosave and quicksave tests; the files agree) (`008503b0`). |
| Player name | `wstr` | Player's full name (`0055d520`). |
| Karma title | `wstr` | From level and karma (`0047e0e0`). |
| Level | `u32|` | Player level (`0087f9f0`, low 16 bits). |
| Location | `wstr` | The place shown in the load menu (`00851b40`). |
| Play time | `wstr` | `"hhh.mm.ss"` (`00851bf0`). |

### File location table (0x6E bytes)

| Offset | Field |
| --- | --- |
| 0x00 | `u32` offset of the form id array |
| 0x04 | `u32` offset of the history block (just past the worldspace id array) |
| 0x08 | `u32` offset of global data table 1 |
| 0x0C | `u32` offset of the change forms |
| 0x10 | `u32` offset of global data table 2 |
| 0x14 | `u32` count of global data table 1 (12) |
| 0x18 | `u32` count of global data table 2 (1) |
| 0x1C | `u32` count of change forms |
| 0x20 | 0x4E bytes, zero (unused) |

The loader seeks to the form id arrays first (`00846e70` twice), then reads
the global data and change forms.

## Global data

Each entry: `u32 type`, `u32 size`, `size` bytes (raw header, the data is a
pipe buffer). Types (`BGSSaveLoadGlobalData::GLOBAL_DATA` (Xbox PDB);
switch `0084bcd0`, loader `0084be40`):

| Type | Name (Xbox PDB) | Writer | Layout (as far as decoded) |
| --- | --- | --- | --- |
| 0 | `GLOBAL_DATA_MISC_STATS` | `MiscStatManager::SaveGame` `004d5fa0` | `u32 count|` (43), then `count × u32|`: the Pip-Boy's misc statistics in the order of the name table `01189280` (`004d5c80` builds them in that order), i.e. `world::stats::NAMES`. **Decoded.** |
| 1 | `GLOBAL_DATA_LOCATION` | `SaveLocationData` `0084c490` | `u32|` next created form id (`TESDataHandler::iNextID`, +0x208), refID `|` of the current worldspace (`TES::pWorldSpace`, +0x88), `i32|` `i32|` current grid x, y (`TES` +0x24, +0x28), refID `|` of the player's worldspace (or parent cell when inside), `3 × f32` player position `|`, then `LoadingMenu::SaveGame` `0078d6b0` (refID `|`, three 4-byte values `|`, `u8|`; not interpreted). **Decoded** except the loading-menu part's meaning. |
| 2 | `GLOBAL_DATA_TES` | `TES::SaveGame` `00459490` | vsval count, each refID `|` + `u16|`; then `u32 n|` and `n × n` refIDs `|` (n = 5 in every file, apparently the loaded cell grid). Not interpreted further. |
| 3 | `GLOBAL_DATA_GLOBALS` | `SaveGlobals` `0084cc90` | vsval count, each refID `|` (a `GLOB`) and `f32|` value. Constant globals (form flag 0x40) are skipped. Game time lives here (`GameHour`, `GameDaysPassed`, `GameDay`, `GameMonth`, `GameYear`, `TimeScale`). **Decoded.** |
| 4 | `GLOBAL_DATA_PROCESS_LISTS` | `ProcessLists::SaveGame` `00975840` | Four `u32|` then five vsval-counted lists. Not decoded. |
| 5 | `GLOBAL_DATA_COMBAT` | `CombatManager::SaveGame` `009932d0` | Not decoded. |
| 6 | `GLOBAL_DATA_INTERFACE` | `Interface::SaveGame` `007066d0` | Not decoded. |
| 7 | `GLOBAL_DATA_EFFECTS` | `SaveEffects` `0084cf30` | vsval count, each `f32|` `f32|` refID `|`. Not interpreted. |
| 8 | `GLOBAL_DATA_WEATHER` | `Sky::SaveGame` `0063eb70` | Four refIDs `|` (`Sky` +0x10 to +0x1C) and eleven 4-byte values `|`. nv-rs's `world::weather` already models the sky's saved state; not mapped here. |
| 9 | `GLOBAL_DATA_ACTOR_CAUSES` | `ActorCause::SaveActorCausesList` `0066e6d0` | Not decoded. |
| 10 | `GLOBAL_DATA_RADIO` | `FalloutRadio::SaveGame` `008366e0` | Not decoded. |
| 11 | `GLOBAL_DATA_AUDIO` | `FalloutAudio::SaveGame` `0082ec40` | One refID `|`. |
| 1000 | `GLOBAL_DATA_TEMP_EFFECTS` | `004534f0` (empty on PC) | Always size 0 on PC. |

## Change forms

One record per changed form (`00865a30` for a form in memory, `00866200`
for one kept as an unloaded buffer from an earlier save; both write the same
record):

| Field | Layout |
| --- | --- |
| refID | 3 bytes (above) |
| Change flags | `u32` |
| Type | `u8`: low 6 bits the save type (table below), high 2 bits the size of the length field (0: `u8`, 1: `u16`, 2: `u32`; `00845d40`) |
| Version | `u8`: the minor version when written (27 in every file seen) |
| Length | `u8`, `u16` or `u32` |
| Data | `length` bytes, a pipe buffer |

The data is, in order (`00847850`):

1. **Initial data** for references and cells (`0084eb80`; kind chosen by
   `0084e730`, `BGSSaveLoadInitialData::INITIAL_DATA_TYPE` (Xbox PDB)),
   written as one block then `|`:

   | Kind | When | Bytes |
   | --- | --- | --- |
   | 0 none | anything else | 0 |
   | 1 exterior cell, char | `CELL`, flags 0x40000000 and 0x20000000 | `u16` worldspace index (into the worldspace id array, `0084e340` reads), `i8` x, `i8` y, `u32` detach time (8) |
   | 2 exterior cell, short | `CELL`, flags 0x40000000 and 0x10000000 | `u16` worldspace index, `i16` x, `i16` y, `u32` detach time (10, `0084e3c0`) |
   | 3 interior cell | `CELL`, flag 0x40000000 only | `u32` detach time (4) |
   | 4 reference location | reference (form types 58 to 64 or 105), flags 0x2 or 0x4, not 0x8 | refID of the cell, or of the worldspace for an exterior reference; `3 × f32` position; `3 × f32` rotation (27) |
   | 5 created reference | reference with a `0xFF` form id | refID cell/worldspace, position, rotation, `u8` flags (1; \|2 per `005653d0`; \|4 when the reference is a mobile object, vtable +0xFC, and `00574900` holds), refID of the base object (the leveled base when flag 0x40000) (31) |
   | 6 moved reference | reference with flag 0x8 (`CHANGE_REFR_CELL_CHANGED`) | refID cell/worldspace, position, rotation, refID of the editor-location cell, `i16` `i16` its grid (editor x, y >> 12), zeros if it has none (34) |

2. For a reference with `CHANGE_REFR_HAVOK_MOVE` (0x4): a vsval length and
   the Havok data written by `00562de0` (only when vtable +0xF0, the
   reference test, holds; nv-rs's `GameState::havok_moved` and
   `havok_velocity` describe the pose and velocities the game keeps).
3. The form's own `SaveGame` (vtable +0x54 on PC, as in the Xbox class
   layout) for the change flags it has. `CheckSaveGame` (+0x80) runs first
   and may clear flags but writes nothing.

The save type → form type table is `011a2428` (55 entries, set up by
`BGSSaveLoadFormInfo::InitSaveGameFormTypes` (Xbox PDB) `008535e0`; form
type names from `01187004`). Every change form in the nine files had the
record type its form has in `FalloutNV.esm`.

### Change flags and data by type

Change flag names are `CHANGE_TYPE` (Xbox PDB). Bit 0x1
(`CHANGE_FORM_FLAGS`) always adds `u32|` form flags first (`00484d60`,
`TESForm::SaveGame`).

| Save type | Form | `SaveGame` (PC) | Flags → data (in written order) | Status |
| --- | --- | --- | --- | --- |
| 9 | `QUST` | `0060e810` | 0x2 `QUEST_FLAGS`: `u8|` (`QuestFlag`: 0x01 enabled/running, 0x02 completed, 0x04 allow repeats, 0x08 allow repeated stages, 0x10 starts enabled, 0x20 shown in the HUD, 0x40 failed). 0x4 `QUEST_SCRIPT_DELAY`: `f32|`. 0x80000000 `QUEST_STAGES`: vsval stage count; per stage `u8 index|`, `u8 done|`, vsval item count, per item `u8 index|`, `u8 has log date|`, if so a `Date` (`u16` day of year, `u16` year) `|`. 0x40000000 `QUEST_SCRIPT`: script locals (below). 0x20000000 `QUEST_OBJECTIVES`: vsval count; per objective `u32 index|`, `u32 state|` (`QUEST_OBJECTIVE_STATE`: 0 dormant, 1 displayed, 2 completed, 3 completed and displayed). | **Decoded** |
| 7 | `CELL` | `00555630` | 0x2 `CELL_FLAGS`: `u8|` (cell flags & 0x60 \| flags2). 0x80000000 `CELL_SEENDATA`: exterior 32 bytes `|` (`SeenData` `0087a0c0`); interior vsval count, per part two `u8|` and 32 bytes `|` (`IntSeenData` `0087a4e0`). 0x4 `CELL_FULLNAME`: `wstr`. 0x8 `CELL_OWNERSHIP`: refID `|`. 0x40000000 `CELL_DETACHTIME`, 0x20000000 / 0x10000000: initial data only. | **Decoded** |
| 8 | `INFO` | `00484d60` | Flags only: 0x80000000 `TOPIC_SAIDONCE` has no data. | **Decoded** |
| 10, 11 | `NPC_`, `CREA` | `00608f00`, `005fb330` | Actor base (`005f1f30`): 0x2 `ACTOR_BASE_DATA`: 24 bytes `|` (the `ACBS` data, level included); 0x10 `ACTOR_BASE_SPELLLIST`: two vsval-counted refID lists; 0x4 `ACTOR_BASE_ATTRIBUTES`: 7 bytes `|` (S.P.E.C.I.A.L.); 0x8 `ACTOR_BASE_AIDATA`: 20 bytes `|`; 0x20 `ACTOR_BASE_FULLNAME`: `wstr`. `NPC_` then: 0x200 `NPC_SKILLS`: 28 bytes `|`; 0x400 `NPC_CLASS`: refID `|`; 0x2000000 `NPC_RACE`: two refIDs `|`; 0x800 `NPC_FACE`: face data (`00608f00`, not decoded); 0x1000000 `NPC_GENDER`: `u8|`. | **Decoded** but `NPC_FACE` and `CREA` |
| 34 | `FACT` | `005fd690` | 0x4 `FACTION_REACTIONS`: vsval count, per entry refID `|`, `i32|` modifier, `i32|` group reaction (`0048c9d0`). 0x2 `FACTION_FLAGS`: `u32|`. 0x80000000 `FACTION_CRIME_COUNTS`: `i32|` major, `i32|` minor (`iMajorCrime`, `iMinorCrime`). | **Decoded** |
| 33 | `CLAS` | `005f7150` | 0x2 `CLASS_TAG_SKILLS`: `4 × i32|` actor values (−1 unused). | **Decoded** |
| 50 | `CHAL` | `005f5780` | Always `i32|` `iProgress`, `i32|` `nProgressFlags` (`CHALLENGE_PROGRESS`). | **Decoded** |
| 43 | `REPU` | `00616660` | Always `f32|` fame (`fPositiveValue`), `f32|` infamy (`fNegativeValue`). | Read from code; no file has one yet |
| 37 | `FLST` | `00590080` | 0x80000000 `FORM_LIST_ADDED_FORM`: `u32 count|` then `count` refIDs `|` (the forms added in game). | Read from code |
| 38 to 40 | `LVLC`, `LVLN`, `LVLI` | `0050adb0` | 0x80000000 `LEVELED_LIST_ADDED_OBJECT`. | Located |
| 16, 22, 28, 27, 47, 53 | `BOOK`, `MISC`, `KEYM`, `AMMO`, `CHIP`, `CMNY` | `00515340`, `0051b0d0`, `00504450` | 0x2 `BASE_OBJECT_VALUE`: `u32|` (`0048ea40`); `BOOK` 0x20 `BOOK_TEACHES_SKILL`: `u8|`. | Read from code |
| 15, 17, 21, 26, 42 | `ARMO`, `CLOT`, `LIGH`, `WEAP`, `IMOD` | `005144c0`, `0050e5c0`, `005230f0`, `0051a600` | Not looked into. | Located |
| 12, 25, 13, 14 | `ACTI`, `FURN`, `TACT`, `TERM` | `00511860`, `004ff800`, `00501b90` | `TACT` 0x800000 `TALKING_ACTIVATOR_SPEAKER`. | Located |
| 32 | `ECZN` | `00526410` | 0x2 `ENCOUNTER_ZONE_FLAGS`, 0x80000000 `ENCOUNTER_ZONE_GAME_DATA`. | Located |
| 35 | `PACK` | `006797c0` | 0x80000000 `PACKAGE_NEVER_RUN`, 0x40000000 `PACKAGE_WAITING`. | Located |
| 41 | `WATR` | `005806e0` | 0x80000000 `WATER_REMAPPED`. | Located |
| 31 | `NOTE` | `00484d60` | Flags only: 0x80000000 `NOTE_READ`. | Read from code |
| 18 to 20, 23, 24, 29, 30, 36, 44 to 46, 48, 49, 51, 52, 54 | `CONT`, `DOOR`, `INGR`, `STAT`, `MSTT`, `ALCH`, `IDLM`, `NAVM`, `PCBE`, `RCPE`, `RCCT`, `CSNO`, `LSCT`, `AMEF`, `CCRD`, `CDCK` | `00484d60` or small | Form flags (`LSCT`, `CCRD`: `0059add0`). | Located |
| 0 | `REFR` | `00562230` | 0x10 `REFR_SCALE`: `f32|`. Extra data list (`ExtraDataList::SaveGame` `00426a30`) when any of 0xA4021C40 is set (0xA4061840 for actors): 0x40 `EXTRA_OWNERSHIP`, 0x400 `OBJECT_EXTRA_ITEM_DATA`, 0x800 `OBJECT_EXTRA_AMMO`, 0x1000 `OBJECT_EXTRA_LOCK`, 0x20000 `DOOR_EXTRA_TELEPORT`, 0x4000000 `EXTRA_ACTIVATING_CHILDREN`, 0x20000000 `EXTRA_ENCOUNTER_ZONE`, 0x80000000 `EXTRA_GAME_ONLY` (actors: 0x800 `ACTOR_EXTRA_PACKAGE_DATA`, 0x1000 `MERCHANT_CONTAINER`, 0x20000 `DISMEMBERED_LIMBS`, 0x40000 `LEVELED_ACTOR`). 0x20 `REFR_INVENTORY` or 0x8000000 `REFR_LEVELED_INVENTORY`: inventory (`004d4090`, `InventoryChanges::SaveGame`). 0x10000000 `REFR_ANIMATION` (not actors): vsval length and animation data (`00563650`). 0x2 `REFR_MOVE`, 0x4 `REFR_HAVOK_MOVE`, 0x8 `REFR_CELL_CHANGED` are initial data. 0x400000 / 0x800000 `OBJECT_OPEN_DEFAULT_STATE` / `OBJECT_OPEN_STATE`, 0x200000 `OBJECT_EMPTY` and 0x40000000 `EXTRA_CREATED_ONLY` are outside this function's masks; where (if anywhere) they add data is not traced. | Header and initial data decoded; extra data, inventory, animation not decoded |
| 1, 2 | `ACHR`, `ACRE` | `008d33a0`, `008aaf40` | `Actor::SaveGame` (`008aaf40`) starts with `MobileObject::SaveGame` (`00932880`): `u8|` process level (0xFF none), the `REFR` data above, twelve `u8`/`u32` fields `|`, two refIDs `|`, then the process's own `SaveGame` (process vtable +0x54C). The actor part then writes a fixed run of fields `|` and three refIDs `|` whatever the flags, then 0x400 `ACTOR_LIFESTATE`: `u8|`; 0x80000 `ACTOR_DISPOSITION_MODIFIERS`: vsval count, per entry refID `|`, 4 bytes `|`; 0x800000 `ACTOR_PERMANENT_MODIFIERS` and 0x400000 `ACTOR_OVERRIDE_MODIFIERS`: a modifier list each (`00937b50`); then the `SaveGame` of the object at +0x190. `Character` (`008d33a0`) adds two `u8|`; the player has `PlayerCharacter::SaveGame` `009590f0` (4.8 KB; its Xbox counterpart calls `CharacterProgression::SaveGame`). | Initial data decoded; the rest not decoded |
| 3 to 6 | `PMIS`, `PGRE`, `PBEA`, `PFLA` | `009baaa0`, `00979bb0`, `009b3eb0` | Projectiles in flight. | Located |

**Script locals** (`ScriptLocals` (Xbox PDB), `005a9db0`; read by
`005a9f20`): vsval count; per variable `u32 id|` and either `f64|` (a
number) or, when the id has bit 0x80000000, a refID `|` (a reference
variable; the id is the low 31 bits); then `u8 has event list|` and, if so,
8 bytes `|`; then `u8|` (flag 0x1000), which the loader reads only when the
form's version is over 0x14 (20). This is the one versioned field met in the
decoded types; the other loaders read the version-27 layout written above.

On loading a quest, its current stage is not read from the file: it is set
to the highest stage marked done (`0060d670`).

### What the nine files hold

Per save (first named save as the example): 3,898 change forms, of which
2,430 `ACHR`, 657 `ACRE`, 390 `REFR`, 301 `QUST`, 82 `INFO`, 31 `CELL`, 3
`NPC_`, 2 `CHAL`, 1 `CLAS`, 1 `FACT`; 8,225 form ids; 172 globals; 43 misc
stats. Other saves add `PMIS`/`PGRE` (projectiles in flight). All 301 quest
change forms have `QUEST_SCRIPT`; 6 or 7 have stages and 3 have objectives.

## Scope: what nv-rs needs to continue a game

What the import has to fill in nv-rs (`GameState` in
`crates/world/src/scripting.rs`, saved by `crates/world/src/save.rs`), where
it is in a `.fos`, and how far this research got.

| nv-rs state | In the `.fos` | Status |
| --- | --- | --- |
| Game time (`GameHour`, `GameDaysPassed`, …) and other globals (`globals`) | Global data 3 | Decoded |
| Quest stages done, current stage (`stages`, `stages_done`) | `QUST` 0x80000000; current = highest done | Decoded |
| Running / completed / failed quests (`running`, `completed`, `failed`) | `QUST` 0x2 flags 0x01, 0x02, 0x40 | Decoded |
| Objectives (`objectives`) | `QUST` 0x20000000 (state 2 or 3 = completed) | Decoded |
| Quest script variables (`variables`) | `QUST` 0x40000000; the ids are the script's local variable indices (`SCRO`/`SLSD`), names come from the quest's script | Decoded (values); naming needs the script record |
| Quest delays (`quest_delays`) | `QUST` 0x4 | Decoded |
| Topics said once (`said`) | `INFO` 0x80000000 | Decoded |
| Misc statistics (`misc_stats`) | Global data 0 | Decoded |
| Player cell, worldspace, position (`player_cell`, `player_world`, `player_position`) | The player's `ACHR` (form id 0x14) initial data (kind 4 or 6), and global data 1 | Decoded |
| Player name, sex, level, S.P.E.C.I.A.L. (`player_name`, `player_female`, `player_level`) | `NPC_` 0x7 (the player's base): 0x20 full name, 0x1000000 gender, 0x2 base data (level), 0x4 attributes | Decoded |
| Tag skills (`tag_skills`) | `CLAS` `PlayerClass` 0x2 | Decoded |
| Faction crimes (`faction_crimes`), reactions (`faction_relations`) | `FACT` 0x80000000 (major, minor: nv-rs keeps (minor, major)), 0x4 | Decoded |
| Reputations (`reputations`) | `REPU` (fame, infamy) | Layout read from code, untested (no file has one) |
| Challenges (`more` challenge progress) | `CHAL` | Decoded |
| Local map fog (`seen`) | `CELL` 0x80000000 | Decoded (bits not mapped onto `local_map::Seen` yet) |
| Weather (`weather`) | Global data 8 | Located (nv-rs already reads `0063e9f0`'s format for its own save) |
| References moved (`positions`, `spaces`), Havok-moved (`havok_moved`) | Initial data 4 / 6, Havok block | Decoded (initial data); Havok block located |
| References disabled (`disabled`), deleted | The form flags word of `CHANGE_FORM_FLAGS` | Decoded (the word); which bit means disabled or deleted at run time is not traced yet |
| Scale (`scales`) | `REFR` 0x10 | Decoded |
| Inventory and container contents (`items`, `stocked`), equipped (`equipped`), weapon condition (`weapon_health`) | `REFR`/`ACHR` 0x20 / 0x8000000 (`InventoryChanges::SaveGame`, `ItemChange::SaveGame`) | **Not decoded** |
| Locks (`locks`, `broken_locks`), door teleports, ownership (`set_by_scripts`), open state | Extra data list (`00426a30`) | **Not decoded** |
| Dead, health lost, actor values (`dead`, `damage`, `actor_values`, `value_damage`) | `ACHR`/`ACRE` `Actor::SaveGame`: life state, modifier lists | **Not decoded** |
| Perks and ranks, skill points, experience (`perks`, `perk_ranks`, `skill_points`), map markers found (`discovered`, `map_markers`), active quest, hot keys, karma, radio (`radio`) | `PlayerCharacter::SaveGame` `009590f0`; global data 10 (radio) | **Not decoded** |
| Active effects (`active_effects`) | Global data 7 and actors' process data | **Not decoded** |
| Packages, combat, AI process state | Process data in `ACHR`/`ACRE`, global data 4, 5 | **Not decoded**; nv-rs re-evaluates packages on load anyway |
| Created references (`more` made references) | Initial data 5 (base object, place); their own extra data | Initial data decoded |
| Form lists and leveled lists changed by scripts (`set_by_scripts`) | `FLST`, `LVLx` 0x80000000 | Located |

What nv-rs is missing to take a `.fos` at all:

- **Form id remapping.** The save's plugin list must be matched against
  nv-rs's load order (by name), and the 0xFF created forms need a home:
  nv-rs's created references would have to adopt the save's ids or keep a
  map.
- **Script variable names.** nv-rs keeps quest and reference variables by
  name (`Locals`); the save keeps them by index, so the import needs the
  owning script's `SLSD`/`SCVR` list.
- **State nv-rs doesn't model.** Interface state, combat manager, actor
  process levels and AI procedures, temporary effects, loaded cell grid:
  the import would drop these and let nv-rs rebuild them, as its own loader
  does.
- **Unknown flags per bit.** Form flags bits for disabled/deleted, `CELL`
  flags2, `ACBS` and AI data byte meanings must be checked against the code
  that reads them before they are applied.

## Next steps

1. `ExtraDataList::SaveGame` (`00426a30`) and `InventoryChanges::SaveGame`
   (`004d4090`, `ItemChange::SaveGame` (Xbox PDB)): containers, locks,
   ownership, inventories. With them every `REFR` change form decodes to its
   length (the inspector's check).
2. `Actor::SaveGame` (`008aaf40`), `MobileObject::SaveGame` (`00932880`),
   the process `SaveGame`s and `PlayerCharacter::SaveGame` (`009590f0`).
3. An importer from the decoded parts into `GameState` (quests, globals,
   stats, player place, faction crimes, challenges), checked by loading a
   real save in the viewer and comparing with the original game.

## Reader and inspector

`crates/fos` (no dependencies) reads a save read-only: `fos::Save::parse`
splits it into the parts above and fails unless every part ends where the
location table says and the history block ends the file, so a parsed save
accounts for every byte. `fos::decode` reads quests (stages, log dates,
script locals, objectives), globals, misc statistics, the location data,
initial data and the Havok block of references, cells (both seen data
layouts), topics, notes, actor bases (not `NPC_FACE`), factions, classes,
challenges and reputations; `decode::coverage` says whether a change form
decodes to exactly its length, is not decoded yet, or fails. Tests build
synthetic saves with the test-only writer `fos::write` (feature `write`);
no real save is in the repository.

`nvinspect fos <SAVE> [PLUGIN]` prints the header, plugins, the location
table, the global data, the change forms counted by type and by change
flag (with the Xbox names) and how many decode exactly, the quests with
stages or objectives (current stage, stages done, objectives), the
globals, the misc statistics, the player's place, and the file's bytes
part by part. With a plugin (`FalloutNV.esm`) forms from it get their
editor IDs. It exits with an error if any decodable change form doesn't
decode to its length.

Run on all nine real saves (2026-10-07): every one parses with every byte
accounted for, no change form fails, and between 438 and 732 change forms
per save decode exactly (all quests, cells, topics, actor bases, factions,
classes and challenges, and the references with no extra data, inventory
or animation); the rest are references, actors and projectiles whose
extra data, inventory and actor state aren't decoded yet.
