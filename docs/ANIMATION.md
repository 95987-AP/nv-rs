# NPC body animation: furniture transitions and talking

Branch `claude/m2-npc-anims`, 2026-10-06. Maintainer's playtest: people
talked to "are completely still other than their face and mouth and
eyes", and Doc Mitchell "jumps" in his chair after sitting down. Private
exports (decompiles, captures, logs) in
`%USERPROFILE%\nv-re\work\npcanims-2026-10-06`. Names marked (Xbox PDB)
come from `Fallout_Release_MemDebug.pdb`; PC addresses are FalloutNV.exe
1.4.0.525. Older animation and furniture evidence: `findings\animation.md`,
`findings\furniture.md`, [FURNITURE.md](FURNITURE.md).

## The chair jump

Cause: the furniture procedures turn the actor by the marker's heading
delta (or half a turn) at the moment the entry or exit animation hands
over, and the game swaps the animations in that same frame without a
blend. nv-rs turned the heading but let the entry ease out over its
`Blend:15` (0.5 s). `Chair_ForwardEnter.kf` ends with `Bip01 NonAccum`
at 87.2°, `DynamicIdle_ChairSit.kf` holds −92.8°: the half turn the
heading takes. During the ease out the body was the entry's pose on the
new heading, so it swung half a turn and back: the jump.

| Address | What | Used for |
| --- | --- | --- |
| `004974a0` | sets `Animation` +0x120 = `cSkipNextBlend` (Xbox PDB layout matches PC: +0xe0 current sequences, +0x124 `spAnimIdle`) | `animation::Player::skip_next_blend` |
| `004949a0` | playing a group: blend 0 while the flag is set | `Player::play_from_pose` |
| `004994f0`, `00496080` | stopping a section: blend 0, deactivated at once | `Player::stop_section`, ends in `update` |
| `00491180` | the animation update clears the flag at its end | `Player::update` |
| `009213e0` case 3/8 | entry done, marker < 21: heading += delta (`00931d30`), flag, entry freed (`00498910(1,0)`) | `Sitter::update` → `skip_next_blend` |
| `00921e80` case 5/10 | exit loaded: heading −= delta, flag, exit played (`00498230`); exit done, marker < 21: flag, heading += π, `00895110`; no exit: flag, section 0 cleared | `Sitter::update` |

The viewer's `sitting::furniture_frame` hands the Sitter's flag to the
rig's `Player`. Tree idles asked by the idle clock wait for the next frame
when the flag is set (the process plays requests at a later update,
`008dab40` → `008dae00`), so a seated idle doesn't cut in unblended.

## Talking: the speaker's body in the dialogue menu

Two causes of "completely still": the viewer froze every animation while
any menu was open, and the game's own dialogue menu screen counts as one
(`Menus::game_open`); and nothing asked for talking idles.

| Address | What | Used for |
| --- | --- | --- |
| `0061e780` | response loader: `SNAM` speaker idle, `LNAM` listener idle after each `TRDT` (IDLE forms) | `dialogue::Response::speaker_idle`, `listener_idle` |
| `008a5580` | `Actor::UpdateInDialogue` (Xbox PDB): +0x7d = 1 (talking), +0x86 = `bUseEmotion` (`TRDT` byte 20, `008a5cf0`), say (+0x284 `008a20d0`) with the response's emotion and idles and a last argument 1; each later frame with a dialogue package (type 0x1c), special idle done (`004985f0`) and no request waiting (process +0x350, vfunc +0x718 = `006214b0`): idle request mode 2 | `talk_idles::Talking::say`, `between_says` |
| `008a20d0` | say: speaking emotion (+0x2d4/+0x2dc); with an idle or the last argument: request (process +0x44 = `008dab40`) for the idle (mode 3) or the tree (mode 2), forced; the listener likewise (not shown: the player) | `Talking::say` |
| `008dab40` | request: sit state 0/4/9; special idle done, idle named or forced; tree (`00600950`) only when no special idle is starting (`00498f80`); queued at +0x350 | `talk_idles::takes_request`, `Talking::queued` |
| `008dae00`, `00497f20`, `00498290` | the queued idle played: old special idle freed, new one blended from the pose, loops rolled | `sitting::play_requested` (`Player::request_special_idle`) |
| `008b1070` → `008ba600` → `008daf20` | leaving the menu: process flag 0x800; once no special idle is starting, sit state 0/4/9 frees the special idle | `Talking::menu_closed`, `sitting::free_talk_idle` |
| `005a1150` → `008a67f0` | `IsTalking`: actor +0x7d | true from the say's first frame |
| `005a4480` | `GetDialogueEmotion`: speaking emotion if +0x86, else −1 | `IdleQuestion::emotion` |
| `0059c380` | `MenuMode`: 0 any menu, else that menu | `IdleQuestion::menu` (dialogue 1009) |

The idle tree's own `DialogueIdles` (`GetCurrentAIProcedure` 4 OR
`MenuMode 1009`) / `TalkToPlayer` / `DialogTalking` branches and the
seated `ChairDialogueIdles` → `ChairTalk` then pick the gestures by
`IsTalking`, `GetDialogueEmotion`, `IsLastIdlePlayed` and
`GetRandomPercent`. `actors::animate_actors` now keeps animating while the
only menu up is the game's dialogue menu (class 1009); `ai` still holds
everyone but the speaker.

## Implemented and tested

- `world::animation`: `cSkipNextBlend`; unit test
  `skip_next_blend_switches_at_once_for_one_update`.
- `world::furniture::Sitter::skip_next_blend` at the four transitions;
  `world/tests/furniture_blend.rs` runs Sitter + Player on generated
  sequences shaped like Doc's chair: the body's facing changes by at most
  5° a frame (60 fps) with the flag, over 90° without it.
- `world::talk_idles` (new): say, between-says, request gate, menu close;
  unit tests. `world::idles`: `MenuMode`, `GetDialogueEmotion`,
  `IdleClock::played`; test on a generated dialogue branch.
- `world::dialogue::Response`: `use_emotion`, `speaker_idle`,
  `listener_idle`.
- Viewer: `sitting::dialogue_frame`, `dialogue_over`, `free_talk_idle`;
  `ai::move_actors` calls them for the menu speaker; `dialogue::Talk::said`;
  `menus::Menus::only_game_menus`; `actors::animate_actors` menu rule
  (`menu_stops_animation`, viewer test
  `only_the_dialogue_menu_lets_the_speaker_animate`).

Checks: root `cargo test --workspace` (only `repo_hygiene` fails, on the
worktree's `.git` pointer file), clippy and fmt clean; viewer build,
tests, clippy and fmt clean.

## Verified live (installed data, release viewer)

- Chair: `GSDocMitchellHouse --at 2200,2180,7360,60,10` (Doc walks to his
  chair and sits, entry 2.9 s → seated 4.6 s). Screen captured at ~63 fps
  (`burst.ps1`, `montage.ps1`). With the fix the seated pose is steady from
  the moment he settles; with the flag disabled (a temporary local
  switch, not committed) he visibly rises and turns as the entry ends and
  drops back over about half a second: the reported jump.
- Talking, standing: `GSProspectorSaloonInterior --talk` (Sunny Smiles,
  dialogue menu): `DialogNoBodyShiftTalkSubtle` on her line, then
  `DialogNoBodyShiftListenA` at 14.1 s when the topics came up. Picture
  motion (64-px thumbnails, mean absolute difference between samples,
  8–20 s): 0.00 before (frozen), 0.96 after.
- Talking, seated: `GSDocMitchellHouse --stage VCG01 27 --at
  2200,2180,7360,60,10 --run-at 8 "DocMitchellREF.StartConversation
  player"`: seated Doc gets `SitChairTalkPlayerC` from `ChairTalk`; after
  the Goodbye the idle is freed and the relax idles resume.


## Batch 2: the whole picking, weapons, sections, lines outside the menu

Branch `claude/m2-npc-anims-2`, 2026-10-06. Private exports in
`%USERPROFILE%\nv-re\work\npcanims2-2026-10-06`.

### What plays, picked as the game picks it

| Address | What | Used for |
| --- | --- | --- |
| `005f2370`, `005f2400`, `005f23c0` | group id: group, weapon kind (bits 8–11), movement kind (12–14), power armour (15) | `animation::groups::id` |
| `005f38d0`, `01197794`, `011977a4` | a file's kinds from its name: `PA`, `Sneak`/`Swim`/`Fly`, `H2H` … `1LM` | `groups::file_kinds` |
| `005f3a20` | the group is the sequence's name | `AnimSet::add` |
| `00447330`, `008b73f0`, `008b78c0` | a 3D loads every `.kf` of its skeleton folder and `Locomotion\`, people also `Locomotion\Male` or `\Female` (`\Child`, `\Hurt` not done) | viewer `anim_library` |
| `0048f450` (`AnimSequenceMultiple`, Xbox PDB) | several files for one id: one drawn at random | `AnimSet::file` |
| `00495740` | the lookup and its fallbacks (iron sights → plain; 2HM → 1HM; other kinds → 1HP; → no kind; run → walk; movement kind → none; → the kinds' idle) | `AnimSet::lookup` |
| `0118a838` | weapon animation type → weapon kind (mines: type 11 `1MD`, 12 `1LM`; `world::actor::first_person_kind` had them swapped, fixed) | `groups::weapon_kind` |
| `00895110` (`Actor::PickAnimations`, Xbox PDB) | the movement/idle/turn group with the kinds (weapon kind while drawn, or drawing/putting away; sneak), speed < 1 → idle, the rate = speed ÷ whole units a second of the kinds' `Forward`/`FastForward` (`00494300`: single file only), the idle re-picked on a movement-kind change, the walk stopped when a non-movement group is chosen unless still blending in | `animation::pick::Picker::pick` |
| `00895110` cases 0/1, `00491180` kinds 3/4 | `Equip`/`Unequip` start when the process wants the weapon out and it isn't drawn (or the reverse); the weapon is in hand / put away once the `Attach`/`Detach` key is passed; at `end` the section stops | `Picker::pick`, `GroupData::attach` |
| `00888070` → `008b28c0` | weapon drawn, nothing in the weapon section: the aim (with kinds) | `Picker::pick` |
| `004994f0`, `00496080` | stopping the weapon section with the weapon drawn stops only up/down and cross-fades into the aim; 0x14/0x15 fan out | `Player::stop_section`, `Player::weapon_drawn`, `Picker::ended` |
| `00888070` (combat turn scale), Xbox PDB `Actor` | actor +0x104 is `bInCombat` | `Frame::in_combat` |
| `008a6970`, `008a6840` | `IsWeaponOut` = process `GetWantWeaponDrawn` (vtable +0x44c, Xbox PDB names, PC slots = PDB), `SetWeaponOut` | `ActorRig::want_drawn` |
| `008eeec0`, `009da7c0`, `0067a460` | weapon out: in combat the combat's equip action draws; out of combat a running package with flag 0x800000 ("weapon drawn") or `GetAlert` draws, otherwise a drawn, wanted weapon is put away (not the player's) | `pick::want_weapon_out`, `package_draws` |
| `00888b50`, `0067a4f0`, actor +0x125 (`bForceSneak`, Xbox PDB) | an NPC sneaks when forced or its package has flag 0x20000 | `pick::npc_sneaks` |
| `009e2aa0` (`DetailedActorPathHandler`, Xbox PDB) | walking a path while facing a point: within 1° the mover's (forward); > 135° back (if the 3D has `Backward`); ≤ 45° forward; else left/right by the sign; after 0.25 s | `pick::facing_direction` (viewer: fighters stepping while they move) |
| `00888070` → `004955c0(0x14, 0xe0, −1)` | a 3D with a plain `Death` group takes its first frame before the ragdoll (robots: `mtdeath.kf`) | `ActorRig::go_limp` |

### Idles in their sections

`00497f20` → `00498290` play an `IDLE`'s `SpecialIdle` sequence in its
record's section (`00494740` with the section): 0 the base loop, 1 or
0x14 the movement slot, 0x15 the weapon slot (upper body: `HeadUpGreet`,
`2hrLoiter`, the NVS fidgets), 7 the special idle. Base/movement-section
idles set anim action 0xd (`00496fe0`), which keeps the walk from taking
the section (`00895110`). Implemented: `Player::play_in`,
`play_idle_in`, `request_idle_in`, `free_idle_in`, `cut_section`;
`Picker::idle_played`; the viewer plays tree idles, requested and
scripted idles in their sections (`ActorRig::idle_section`,
`overlay_section`).

### Lines said outside the menu, listeners, hits

| Address | What | Used for |
| --- | --- | --- |
| `008a20d0` (`SpeakSoundFunction`, Xbox PDB slot +0x280 = PC +0x284) | every say: the speaker asks for the response's `SNAM` idle, or the tree when the caller forces it (forced unless in combat); a listener who is an actor with a process asks for the `LNAM` idle, or the tree unless its running package has idles (+0x34), always forced; the player listener takes none (`008dab40` clears it) | `talk_idles::say_requests` |
| `008dbe30` (`ProcessGreet`) | greetings, chatter, Say To: force unless the current package has idles or the run-once package is type 0x1a | `greet_forces_tree` |
| `009ee0a0`, `009edd80`, `008b2170`, `008b19c0`, `00935480` | conversations: force as made (script `StartConversation` yes; package conversations unless the package has idles); with no line said, both ask the tree forced | `conversation_forces_tree`, `BETWEEN_LINES` |
| `008dade0` | a running package with flag 0x1000000 refuses requests | `package_refuses_idles` |
| `005a5330` | `IsGreetingPlayer`: process greeting flag (+0x30c) and the player as target | `IdleQuestion::greeting_player` (flag taken as set while a GREET line to the player is said: its lifetime isn't traced) |
| `0089a760`, `005a3c30`, `011df760` | a hit that doesn't kill, with `bPlayHitLocationIdles`, a known part and `IgnoreCrippledLimbs` (AV 72) ≤ 0: the tree is asked at once with `GetHitLocation` set (the `HitReactionIdles` branch: its idles need the part's condition at 0, i.e. crippled) | viewer `sitting::idle_requests`, `GameState::hits_taken`, `IdleQuestion::hit_location` |

Viewer: `chatter::Lines::started` (each response begun),
`Lines::say_in_conversation`; `sitting::idle_requests` (a system after
`say_lines`) and `sitting::request_idle` (the menu, lines and hits share
it).

### The player's third-person body

Same picking (`player_body` sets the flags: keys' directions, sneak, the
Ready state as `want_drawn`, weapon kind, attack group); the furniture
procedure's `skip_next_blend` reaches it through `PlayerSeat`; the body
rebuilt for lighting/clothes/weapon keeps its animation state (before,
every lighting change restarted the animations).

## Verified live (installed data, release viewer)

- Gunfight (`docs/GOODSPRINGS_ROUTE.md` command, logs and frames in the
  private folder): the gangers on `GSPGTravelPackage` (flags 0x00801004,
  weapon drawn) and the settlers on their gunfight packages draw at the
  start (`Equip` 0x0418/0x0518/0x0218/0x0318, in hand 0.25–0.5 s later at
  `Attach`), Settler04 puts his away (`Unequip` 0x0419) when his ambush
  package ends; attacks with the weapons' groups (0x0432 `1hpattackright`,
  0x0544 `2hrattack8`), reloads (0x05bd), hit reactions from the tree
  (`1stPHitLegLeft`, section 1, after a leg was crippled); frames show a
  pistol settler in the 1hp aim and a disarmed ganger in the unarmed
  stance.
- The player's body in third person (keys injected, `drive.ps1`):
  drawing fists (`Equip` 0x0118, in hand at 0.13 s) and the body
  dropping into the unarmed stance, walking, backing up, sneaking
  (crouched), strafing; putting them away (`Unequip` 0x0119).
- Greetings outside the menu reach the request (the tree is asked; for
  that settler `GreetIdles`' `HighDisposition` failed, so nothing played).
- Saloon: Sunny's menu idles still play (`DialogNoBodyShiftTalkSubtle`).
- Animation sets load once per skeleton folder (1029 files, 0.5 s).

## Implemented, tested (generated data)

`world::animation::groups` (ids, kinds, lookup, draws), `pick` (walk,
sneak, back-up rates; draw/put away by the attach keys; aim after an
attack; an idle holding the movement section; sneak/weapon/facing
rules), `talk_idles` say requests; viewer actor tests on a test library.

## Not compared with the original game

None of this is compared side by side: draw/holster timing, the walk
groups with weapons and sneaking, the strafe choice, greetings and
conversation gestures, hit reactions.

## Gaps (not implemented, not substituted)

- Weapon up/down sections (aim pitch weights, `009295c0`); jumping;
  swimming; `Hurt\`, `Child\`, `Toddler\` locomotion; power armour flags
  (`008ba3e0`/`008ba410`) read as false; the per-weapon-kind precached
  lists (`00600700`).
- The combat equip action's own timing (`009da7c0`: drawing once the
  combat weapon is in hand) is taken as "in combat → wanted"; the
  stealthy-combat sneak (`00566950`/`009549a0`).
- Attack-speed perk entry and melee rate (`005e58f0`, `00646020`):
  weapon rate 1.
- The strafe/back choice is applied to fighters only; whether other
  procedures walk facing a point isn't traced.
- `0089a760`'s other hit-reaction conditions; the GREET flag's lifetime;
  the conversation's between-lines asks (the viewer's conversation timing
  isn't the game's update cadence, so they aren't made).
- Stagger (no character files; creatures' not wired), knockdown
  (`GetKnockedState`), `Recoil`/`BlockHit`.
- Replay delays counting in menu mode; `008dab40`'s `00437bf0`.
