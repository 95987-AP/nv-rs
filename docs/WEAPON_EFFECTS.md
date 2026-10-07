# Weapon fire effects: firing sounds, muzzle flashes, tracers (B5)

Batch `claude/b5-weapon-effects`, 2026-10-07. Code: `crates/world/src/weapon_fx.rs`
(rules), `viewer/src/weapon_fx.rs` (sounds and flashes on screen), reports from
`viewer/src/combat.rs` (the player, `FireWeapon` objects) and
`viewer/src/fighting.rs` (people). Executable: `FalloutNV.exe` 1.4.0.525. Impacts
(decals, particles, impact sounds) are B6 and `hiteffects`.

## What the game does (traced)

### Firing sound (`0083ac30`)

- Called by the weapon fire `00523150` with the weapon, the shooter, its fire node
  and "is the player". Guns only: thrown weapons (animation 10–13) take the other
  branch, and melee attacks don't go through `00523150`.
- Weapon sounds (`TESObjectWEAP`): `+0x21c` `SNAM` shoot 3D, `+0x220` `SNAM` shoot
  distant, `+0x224` `XNAM` shoot 2D, `+0x22c` `TNAM` (melee swing / dry fire),
  `+0x240`…`+0x248` the silenced set (`WMS1`, `WMS1`, `WMS2`), chosen when the
  player's weapon has mod effect 11 or 16 (`004bda70`; read for the player only).
- No 3D sound: nothing. The player with a 2D sound: the 2D sound alone (flags
  0x20181), first or third person. Everyone else: the 3D sound and the distant one
  (0x20182).
- Each starts only when the listener is within its largest distance (`SNDD` byte 1
  × 100) × a multiplier on each axis (`0082eca0`).
- Interior multipliers when the player's cell is an interior (flag 0x01) not
  behaving like an exterior (0x80; `00425fd0`, `00454b10`):
  `fWeaponInteriorNearVolumeMod` 1.0, `fWeaponInteriorFarVolumeMod` 0.1,
  `fWeaponInteriorNearAttenuationMod` 3.0, `fWeaponInteriorFarAttenuationMod` 0.75
  (exe defaults; the reads at `0083ad5b`…`0083ad88`). Outdoors all 1.
- Volumes: 3D near volume, distant far volume. Both buffers then get the **3D
  sound's** distances: smallest byte × 5, largest byte × 100 × near multiplier
  (disassembly `0083b4ba`…`0083b566`; the distant sound's bytes are read but unused,
  so when the 3D sound didn't start both are 0). A playing 3D buffer given a
  distance of 0 or less keeps 69.99 (smallest) and 1e9 (largest) (`00aefda0`), so
  that distant sound carries with no distance falloff at the far volume.
- Position: the fire node's (people), else the shooter's 3D; attached to it
  (`00ad8f20`), so it follows.
- 20 entries (0x24 bytes at `011dd9e8`: shooter, time, distance²). Scan: someone
  other than the player stops at their own entry; the oldest entry (ties to the
  later) and, for the player, the entry farthest from the listener beyond the new
  sound that isn't the player's. Oldest not playing (`00ad8ce0`) → it; else own
  entry; else (player) the farthest; else no sound.

### Loudness (the audio thread)

- `0082d400` sets a sound up from its record: static attenuation (`SNDD` i16 at 8,
  form `+0x4c`) by message 0x21, distances (byte 0 × 5, 14 when 0; byte 1 × 100,
  60 when 0), the curve (`SNDD` +12…+20, percent) through `00aeff60`:
  −2000·log10(p ÷ 100), rounded; 0 → 5000. The game sound's constructor
  (`00aeac20`) has 0, 600, 1400, 2600, 5000.
- `00aed990` every update: d = distance to the listener; if |max − min| ≤ 10 or
  d ≥ 1.1 max, attenuation 0 when d ≤ max else 10000; otherwise 10000 past max, 0
  within min, else linear along the curve at min, ¼, ½, ¾ of the way, max.
- `00aed660`: volume in DirectSound hundredths of a dB = trunc(2000·log10(volume))
  − static − distance (and two other fields not used here), clamped to
  [−10000, −1].
- DirectSound's own distance falloff is switched off (minimum distance 1e9 on the
  3D buffer, `00aec530`); it only pans.

### Muzzle flash

- Lit at each projectile's launch (`009bda10` → `009c2ff0`): `PROJ` flag 0x08 and a
  `NAM1` model; the shooter present, not dead (+0x22c), not knocked out (+0x230),
  for actors not in anim action 9 or 15–17 (`008a8870`).
- Actors keep one flash in their process (`+0x3d4`), made again when the projectile
  (model, light) changes (`009bb6d0`, `00902190`); other shooters a list
  (`00978740`).
- Flash object (`009bacb0`): shown, light on, fresh, rest (`+4`), time left (`+8`,
  the `PROJ` duration `DATA` 44). Lighting (`009bb690`) only when rest ≤ 0. Update
  (`009bb080`): light follows shown; rest −= dt; shown: left −= dt, at ≤ 0 hidden,
  rest = 0.1 s, left = duration. dt ÷ the time multiplier in V.A.T.S. or under
  Turbo.
- Placed at the fire node (actors vtable +0x20c; others `ProjectileNode` /
  `##ProjectileNode`, `00525700`); the player in first person at the first-person
  model's node (`009bb240`). Its light is the `PROJ` muzzle flash light (`DATA` 20,
  form `+0x74`, `009bb7f0`).

### Tracers

- Missile set-up `009b7cc0`: tracer = `rand % 100` < trunc(chance × 100)
  (`004fdf60`, chance `DATA` 24). Hitscan off in V.A.T.S. A hitscan non-tracer gets
  projectile flag 0x2, so no model is loaded (`009bda10`): ordinary bullets show
  nothing in flight. Tracers fade over the `PROJ` fade duration (`009becc0`).
- Vanilla: only `MinigunTracerProjectile` (0.3) and `AutoTracerProjectile` (0.5)
  have a chance; the 9mm, .357, varmint rifle and shotgun projectiles have 0.

### Melee swings

- `00899200` at the swing's hit: with no target found, the weapon's `TNAM` plays at
  the attacker (flags 0x102, its own distances), attached to it. A hit plays the
  impact sounds instead (`hiteffects`). The machete has no `TNAM`.

## Implemented

- `world::weapon_fx`: `WeaponSounds`, `SoundLevels` (distances, curve),
  `curve_millibels`, `distance_attenuation`, `millibels` / `amplitude`, `audible`,
  `FireSoundMods`, `plan_fire_sounds`, `FireSoundSlots`, `ProjectileEffects`
  (muzzle flash, tracer roll, model shown), `MuzzleFlash`, `shot_projectile`.
- Viewer `weapon_fx`: shots report (player's guns, people's guns, `FireWeapon`
  objects); the planned sounds play at the game's volume and are re-attenuated
  every frame from the camera's distance, following the shooter; the 20 entries are
  kept. Muzzle flashes: the `NAM1` model spawned once per shooter, placed every
  frame at the held weapon's `ProjectileNode` (first-person model in first person,
  the third-person body or the person otherwise), billboards facing the camera,
  shown and hidden by the game's clock. Melee swings that meet no one play `TNAM`.
- Changed: melee attacks no longer play the weapon's `SNAM` (the game doesn't); the
  player's 2D sound replaces the 3D one for the player.
- `NV_SHOT_ON_FLASH=player|npc`: the `--screenshot` waits for a flash on screen
  (test aid). The log prints each firing sound (`firing sound <EDID> (2D|3D|distant)
  from <shooter> at <d> units: <dB>`) and each flash lit (`muzzle flash <model> on
  <shooter> at <x,y,z>`).
- Tests: 9 in `world::weapon_fx` (generated records), 1 in the viewer's
  `weapon_fx` (loudness by distance).

## Seen live (release viewer, 2026-10-07; private outputs `nv-re\work\b5\live`)

- Goodsprings, `--weapon WeapNV9mmPistol`, the left button pressed every 0.4 s by
  `--key-at`: the flash shows at the pistol's muzzle in first person and, after
  `--key-at 3 f`, at the third-person body's pistol; the log has
  `WPN9mmFire2D (2D) … −10.1 dB` per shot (the sound's static attenuation).
- Ghost Town Gunfight (the acceptance route's commands): 108 flashes lit on the
  gangers (handgun, rifle flashes) at their weapons' nodes; the picture shows a
  revolver ganger's flash (its glow; the flash's planes point at the camera). The
  log has the gangers' 3D sounds fading with distance (.357 3D −10.6 dB at 102
  units, −28.6 dB at 1510) and distant sounds past the 3D range.
- Heard: not checked by ear (no audio capture in these runs).

## Not done

- Panning (DirectSound places 3D sounds left/right; Bevy plays them centred).
- The flash's light (the viewer's lights are fixed per surface at load), the flash
  model's animations (drawn as it stands) and particles (left out).
- Flashes for `FireWeapon` objects; Turbo / V.A.T.S. time scales on the flash.
- Projectiles in flight and tracers on screen (the viewer's shots are rays); the
  tracer's placement while it fades isn't traced.
- Thrown weapons still play their `SNAM` when thrown (pre-existing; `00523150`
  doesn't).
- The anim-action check (9, 15–17) on lighting a flash.
- Nothing compared with the original game.
