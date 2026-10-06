# Dead Money coverage matrix

Every quest (`QUST`) in `DeadMoney.esm`, what it needs from the engine, and how far it has been
played in the nv-rs viewer. Written from the plugin's records with nvinspect and from what the
earlier Dead Money work recorded (`docs/DEAD_MONEY.md` on the Dead Money branches); no engine
changes. Names and behaviour are described in my own words; no game text is quoted, and nothing
decompiled is included.

**57 quests**: 4 PLAYED, 17 PARTIAL, 36 NOT PLAYED.

## How to read it

* **PLAYED**: reached its end in the viewer, with its stages moved by the game's own scripts,
  triggers and dialogue (a test character may set the start).
* **PARTIAL**: some of it was played; the note says how far and what was helped along with a console
  line (`--run "SetStage ..."`).
* **NOT PLAYED**: not tried; the note says what blocks it.
* **[G]** marks a guess (nothing in the data or the code settles it). **[C]** marks something to check
  against the original game. Both are things for the maintainer to confirm.
* "Needs" is the systems the quest's scripts and objectives point at. It comes from reading the quest
  record and its scripts' function names; it is not a trace of every call, so a quest may use a system
  not listed.
* Stage counts are the numbers in the quest record (`INDX`). Where an Elijah bark quest lists many
  numbered lines, they are spoken lines in the record's objective list, not journal objectives.
* Evidence for the PLAYED and PARTIAL rows: the maintainer's bunker launch test (narrator voice,
  slides, arrival in the Villa, the hologram talking), and the headless and viewer runs recorded in
  `docs/DEAD_MONEY.md` ("D3", "Playing Act 1", "Act 2 groundwork"). Those runs used `--run` console
  lines for what the player's hands would do, which is why most of Act 1 is PARTIAL and not PLAYED.
* All 199 script functions Dead Money calls are carried out (nvinspect), so none of the NOT PLAYED
  rows is blocked on a missing function; the blockers are places not visited, features not built
  (radio station playback, casino games, the hacking game, the Cloud's look) or untested behaviour.

## Matrix

### Story quests

The quests with stages and objectives the player sees.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| Sierra Madre Grand Opening! (`NVDLC01MQ00`) | 10, 100 | radio, trigger volume, gas and knock-out, fade, slideshow, inventory removal, door lock | **PLAYED** | From stage 10, which the test character gives (the 30 second wait after Radio New Vegas comes on is not played). The bunker trigger sets stage 100 itself. Confirmed by the maintainer's bunker launch: the narrator spoke, the slides changed, the Villa was reached. |
| MQ01 Master Quest (`NVDLC01MQ01`) | 5, 100 | companions, radio stations, holograms, toxic damage, gear | **PARTIAL** | Both stages were reached and its end messages ran, but the three recruits that finish it were helped along with console stage lines in the runs recorded. Its start-up work (enabling the radio stations, the hologram vendors, the holorifle) ran without errors but wasn't looked at one by one. |
| Find Collar 8: Dog (`NVDLC01MQ01a`) | 10, 20, 30, 40, 100, 255 | dialogue, Pip-Boy note (voice tape), basement trigger, companion recruit | **PARTIAL** | Played: Dog's greeting moves the quest to the basement stage; picking up the tape moves it on. Not played: the tape in the Pip-Boy (the code path is unit-tested only), the recruit stage (set by a console line). Dog's cage key and the later Dog/God switch are in other quests. |
| Find Collar 14: Dean Domino (`NVDLC01MQ01b`) | 10, 20, 100, 255 | dense world district, sitting in a chair (explosive), dialogue, companion recruit, ghost-people ambush enabled afterwards | **PARTIAL** | Played: the district loads (after the graphics fix), the player sits in his chair and his real greeting plays. The recruit stage was set by a console line in the recorded runs. [C] Whether the original needs anything between the greeting and the recruit isn't known here. |
| Find Collar 12: Christine (`NVDLC01MQ01c`) | 10, 20, 30, 40, 100, 255 | dialogue (no voice, stage directions), companion recruit, clinic basement terminal and power, Auto-Doc wing, shielded speakers | **PARTIAL** | Played: her silent conversation, stage 100, the crew notice, her following the player. Not played: the clinic basement (switching off the shielded speakers' power at a terminal) and the Auto-Doc wing. [C] Whether the normal route needs them before the conversation can be had isn't checked. |
| Return to the Fountain (`NVDLC01MQ01d`) | 10, 100 | dialogue | **PARTIAL** | Started by the master quest when all three are recruited and finished by Elijah's Act 2 greeting, which worked; but the recruits before it were helped along (see MQ01a to c). |
| Trigger the Gala Event (`NVDLC01MQ02`) | 10, 50, 100, 255 | companions in place, climbing (ladder, bell tower), control panel, elevator, timed door, traps, radio, autosave, teleport | **NOT PLAYED** | Blocked by MQ02a to c, which aren't finished, and by the climb to the bell tower (ladders and stairs in the viewer are untried [G]). The three companions must be in place first. Objectives: reach the control panel at the top of the bell tower and activate it; then get to the main gate and into the casino. |
| Fires in the Sky (`NVDLC01MQ02a`) | 5, 10, 30, 40, 100, 255 | companion escort and wait, terminal, items (harvester remains), Villa ghost people | **PARTIAL** | Started: Elijah's Act 2 greeting sets stage 5/10 and the first objectives show. Not played: escorting Dog to the substation, telling him to wait (dialogue), the two slabs of remains, the optional terminal that locks him in. |
| Strike Up the Band (`NVDLC01MQ02b`) | 10, 30, 40, 100, 255 | companion escort and wait, security holograms, switches, rooftop, items | **PARTIAL** | Started as MQ02a. Not played: escorting Dean to the rooftop, turning on both security holograms, his waiting. Holograms as enemies are untried (see HoloSupport). |
| Mixed Signals (`NVDLC01MQ02c`) | 10, 20, 30, 50, 60, 70, 100, 255 | companion escort and wait, electrical box repair, elevator, remote terminal | **PARTIAL** | Started as MQ02a. Not played: the switching station, repairing the electrical box, the elevator, the remote maintenance terminal. |
| Heist of the Centuries (`NVDLC01MQ03`) | 10, 20, 30, 40, 50, 100, 255 | casino interior, receptionist terminal and music sequence puzzle, vault with locks and voice code, elevators, sneaking, a boss fight, collar explosion timer, music, fade to credits | **NOT PLAYED** | Blocked by Act 2 and the Gala. Never tried: loading the casino interior, the vault, the music-sequence terminal, the fight with Elijah, the optional sneak-out. Repeating stages allowed (flag 0x08). |
| Put the Beast Down (`NVDLC01MQ03a`) | 5, 10, 20, 30, 40, 50, 60, 100 | casino, key, stealth (not being spotted by Dog), gas valves, dialogue or combat, collar timer | **NOT PLAYED** | Needs the casino and its electrical closet first. [G] The detection rules for 'not spotted' use nv-rs's own detection, unverified for this. |
| Curtain Call at the Tampico (`NVDLC01MQ03b`) | 10, 15, 20, 30, 40, 50, 60, 100 | theatre interior, security holograms, holotape projector, key, combat or dialogue, collar timer | **NOT PLAYED** | Blocked by the casino and by holograms as enemies being untried. |
| Last Luxuries (`NVDLC01MQ03c`) | 10, 20, 30, 100 | executive suites, combat with a former companion, collar timer | **NOT PLAYED** | Blocked by reaching the suites. A quest script ends it if Christine dies by other means. |
| Wake Up the Sierra Madre (`NVDLC01MQ03d`) | 10, 20, 100 | casino electrical closet, power switch, security holograms | **NOT PLAYED** | The smallest of the casino quests; blocked only by reaching the casino. |

### The opening and the ending

Scripted quests that carry the player between the bunker, the Villa and the credits.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| (no name) (`NVDLC01Intro`) | none | fade, slideshow textures, narrator voice, music, Pip-Boy radio off | **PLAYED** | Played in the bunker launch; confirmed by the maintainer (narrator voice, slide changes). |
| (no name) (`NVDLC01VillaTransitionTimer`) | none | fade to black, move to the Villa, music, sound | **PLAYED** | Played in the bunker launch: it moves the player to the Villa. |
| (no name) (`NVDLC01FountainStartSequence`) | 10, 20 | script packages on the hologram, fade in, controls back on, hologram dialogue | **PLAYED** | Played: the wake-up and Elijah's hologram starting to talk (the maintainer watched the replies appear). |
| (no name) (`NVDLC01BunkerTransitionTimer`) | none | fade, move to the bunker, music, radio station for Elijah | **NOT PLAYED** | Belongs to the ending, never reached. |
| (no name) (`NVDLC01FadeToCreditsTimer`) | none | fade, kill actor, autosave, music | **NOT PLAYED** | Belongs to the ending, never reached. |
| (no name) (`NVDLC01Ending`) | none | slideshow of endings [G] | **NOT PLAYED** | Never reached. [G] What it plays isn't known beyond its name. |

### Elijah's voice

Quests that hold Elijah's radio lines; their 'objectives' are mostly spoken lines, not journal objectives.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| Elijah Signal Established (`NVDLC01ElijahBarkTown`) | 27 stages (1 to 255) | radio on the collar, trigger volumes in the Villa, the Cloud, speakers, holograms, vending machines, the Gala route | **PARTIAL** | Played: his first lines at the fountain after the wake-up. Not heard: the place-triggered lines further on (the Cloud, the speakers, the clinic, the gala route, the casino gate) and the closing line when the player disobeys. |
| Elijah Signal Re-Established (`NVDLC01ElijahBarkCasino`) | none (about 60 numbered lines) | radio in the casino, locks, holograms, vault, kitchen gas, collar timers | **NOT PLAYED** | Casino side; never reached. |
| Lobby Casino Support Quest (`NVDLC01ElijahBarkLobby`) | none | casino lobby trigger | **NOT PLAYED** | Casino side; never reached. |
| (no name) (`NVDLC01ElijahCollarBark`) | none | collar radio conversation (StartRadioConversation), timer | **PARTIAL** | The collar's radio interference messages worked in the Villa; the radio conversation playback itself isn't built (see the Radio section of DEAD_MONEY.md). |

### Companions and dialogue

Dialogue holders and timers for the companions.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| DLC01 Followers (`NVDLC01Followers`) | none | companions, follower faction | **PARTIAL** | Christine followed the player after recruit and companions came into view at the fountain; whether Dog and Dean follow as the original does hasn't been watched. [G] The follow distance (200 units) is a guess. |
| Dog/God Dialogue (`NVDLC01DogDialogue`) | none | dialogue, perks that switch Dog and God, sneaking, cage key knowledge | **PARTIAL** | Greeting and tape talk worked; the Dog/God perk switch and his later lines are untried. |
| Dean Dialogue (`NVDLC01DeanDialogue`) | none | dialogue, perk, holograms | **PARTIAL** | The first greeting played for real once the player could sit; later lines untried. |
| Christine Dialogue (`NVDLC01ChristineDialogue`) | none | dialogue, elevator, vault, collar disarm | **PARTIAL** | The recruit conversation played; the vault and collar-disarm lines belong to Act 3. |
| Father Elijah Dialogue (`NVDLC01ElijahDialogue`) | none | dialogue, casino barks | **PARTIAL** | The Act 1 and Act 2 greetings and replies played; his casino lines haven't. |
| (no name) (`NVDLC01DogBarkTimer`) | none | 30 second timer, companion idle lines | **NOT PLAYED** | Never watched. |
| (no name) (`NVDLC01DeanBarkTimer`) | none | 30 second timer, companion idle lines | **NOT PLAYED** | Never watched. |
| (no name) (`NVDLC01ChristineBarkTimer`) | none | 30 second timer | **NOT PLAYED** | Never watched. |
| (no name) (`NVDLC01ChristineTerminalFX`) | none | vault password terminal, elevator unlock, conversation | **NOT PLAYED** | Act 3; never reached. |
| (no name) (`NVDLC01NewVegasFollowerFireQuest`) | none | dismissing the player's Mojave companions on arrival | **NOT PLAYED** | The test character has no companions, so it never had anything to do. |
| (no name) (`NVDLC01StarletDialogue`) | none | hologram dialogue, bark counter | **NOT PLAYED** | Starlet's hologram dialogue; not tried. |

### Collars and radio

The bomb collar and the stations the collars and speakers play.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| Bomb Collar Quest (`NVDLC01BombCollarQuest`) | none | collar timers, speakers, beeps, rumble, explosion, kill, companions | **PARTIAL** | The beeping and the radio interference messages worked in the Villa. Not played: the countdown to the explosion and a death by it, the Christine collar message, the casino collar rules. |
| Bomb Collar Audio (`NVDLC01RadioSpeakers`) | none | radio station | **NOT PLAYED** | Radio playback isn't built (stations' conversations aren't played). |
| Dog's Collar Radio (`NVDLC01RadioDog`) | none | radio station | **NOT PLAYED** | As above. |
| Dean's Collar Radio (`NVDLC01RadioDeanQuest`) | none | radio station | **NOT PLAYED** | As above. |
| (no name) (`NVDLC01RadioDeanMusic`) | none | radio station music | **NOT PLAYED** | No script; its lines are dialogue only [G]. |
| Christine's Collar Radio (`NVDLC01RadioChristine`) | none | radio station | **NOT PLAYED** | As above. |
| Elijah's Collar Radio (`NVDLC01RadioElijah`) | none | radio station, elevator, vault | **NOT PLAYED** | As above. |
| Vera Radio Dialogue (`NVDLC01RadioCasino`) | none | radio station, music sequence | **NOT PLAYED** | As above; belongs to the casino. |
| DLC01 Ambient Music (`NVDLC01RadioAMBMusic`) | none | radio station, music | **NOT PLAYED** | As above. |

### World systems and support

Quests that hold a system's rules.

| Quest (editor ID) | Stages | Needs | Status | Notes |
| --- | --- | --- | --- | --- |
| Script Functionality for Toxic Clouds (`NVDLC01ToxicCloud`) | none | the Cloud, damage over time, vision effect, companion warnings | **NOT PLAYED** | Never examined in the viewer. The functions it calls are carried out; whether the damage and effect look right isn't checked. |
| Script Functionality for Global Toxic Damage (`NVDLC01GlobalToxicDamage`) | none | the Cloud, timer | **NOT PLAYED** | As above. |
| Scripting Support for Hologram Features (`NVDLC01HoloSupport`) | none | security holograms, enabling and disabling them | **PARTIAL** | Started by the fountain sequence and ran in the Villa without errors; the holograms as armed enemies were not played. |
| Hologram Vendor Quest (`NVDLC01HoloVendor`) | none | hologram vending machines, recipe menu, first-use message | **NOT PLAYED** | The recipe menu now exists (crafting branch) and opens with --open-menu; the machine's own activation was not played. |
| NVDLC01WTSupport (`NVDLC01WTSupport`) | none | terminal use flags, hologram flags, the Gala lockdown | **NOT PLAYED** | Never reached. |
| (no name) (`NVDLC01Challenges`) | none | challenges (achievement-like counters), alert state | **NOT PLAYED** | Counters exist in nv-rs; none was played through. |
| NVDLC01CasinoComps (`NVDLC01CasinoComps`) | none | casino winnings level, chips, casino games | **NOT PLAYED** | The casino games (slots, blackjack, roulette) aren't built. |
| Chip Reward Quest (`NVDLC01ChipReward`) | none | chips, casino comps, timers | **NOT PLAYED** | Needs the casino comps. |
| (no name) (`NVDLC01ClinicAutoDocQuest`) | none | a 4 second fade, a sound, controls back on | **NOT PLAYED** | The short scene after the Auto-Doc; part of MQ01c's missing leg. |
| (no name) (`NVDLC01GalaEventQuest`) | none | timers, enabling things, Dean's topic | **NOT PLAYED** | Needs MQ02. |
| DLC01 Enemy Test (`NVDLC01EnemyTest`) | 10, 11, 20, 21, 22, 30, 31, 32, 50, 51 | teleports, keys, followers, enabling ambush groups | **NOT PLAYED** | [G] Looks like a developer test quest (it moves the player between test spots and enables trappers); not part of the story. |
| (no name) (`NVDLC01ArcadeGoodbye`) | none | dialogue only [G] | **NOT PLAYED** | No script. [G] Presumably Arcade's farewell line when dismissed on arrival. |

## Next quests to play

Of the quests not played at all, the three that look best to take next (a judgement [G], by size and by
what they unblock, not a measured effort):

1. **`NVDLC01MQ01c`'s missing leg with `NVDLC01ClinicAutoDocQuest`.** Finishing Act 1's last open
   piece: the clinic basement terminal that cuts the shielded speakers' power, the Auto-Doc wing and the
   short scene after it. Small, uses terminals and the companion recruit already in place, and it turns
   MQ01c from PARTIAL into PLAYED.
2. **`NVDLC01MQ03d` Wake Up the Sierra Madre.** Three stages, one place (the casino's electrical
   closet). The cheapest way to find out whether the casino interior loads and plays, which every other
   MQ03 quest needs. Start it with a test character placed at the casino (like the bunker entry
   character), since reaching it normally needs the whole Gala.
3. **`NVDLC01MQ03c` Last Luxuries.** Four stages, one fight against Christine in the executive suites
   with her collar's timer. A good test of combat with a former companion and the collar explosion,
   which the Villa runs have not exercised.

Not on the list on purpose: MQ02 (needs MQ02a to c first), MQ03 itself and MQ03a/b (large, and need
the casino first), and the radio quests (the station playback is a feature to build, not a quest to
play).
