# Explosives: thrown weapons, grenades and explosions

M2 blocker for Ghost Town Gunfight (`VMS16`, objective 40: Easy Pete hands
out `WeapNVDynamite`). Branch `claude/m2-explosives`, 2026-10-06.

Sources: FalloutNV.exe 1.4.0.525 (unpacked image, Ghidra project copy
`ghidra_8`; private exports in `%USERPROFILE%\nv-re\work\explosives-2026-10-06`),
class names from the Xbox 360 prototype symbols (Xbox PDB), records read with
`nvinspect` from `FalloutNV.esm`. Earlier notes: `findings\hits.md` §2,
`findings\combat_ai.md` §4.1, §5.5.

Status words: **implemented** (code exists), **tested** (generated
regressions), **not compared** (nothing here has been checked against the
running original game yet).

## Records

| Record | Values (FalloutNV.esm) |
| --- | --- |
| `WeapNVDynamite` `000BA0F3` | animation type 10 (grenade), skill Explosives, range 500–1024, min spread 0, 0.405 attacks/s, projectile `NVDynamiteFragProjectile`, no ammunition |
| `NVDynamiteFragProjectile` `000E3F03` | flags 0x0806 (explosion, alt. trigger, rotation), lobber, gravity 1, speed 1200, range 10000, timer 2.5 s, proximity 0, explosion `NVDynamiteExplosion`, impact force 0.5, countdown sound `0017242D`, bounciness 1 |
| `NVDynamiteExplosion` `000E397F` | force 90, damage 75, radius 750, flags 0x49 (radius in units, knock down by formula, ignore image space swap), image space radius 1500, model `Effects\ExplosionGrenadeFrag.NIF`, two sounds, impact set `00093BA4` |

Settings used (data = set by `FalloutNV.esm`, exe = built-in default):
`fGrenadeRestitution` data 0.01, `fGrenadeFriction` data 5,
`fDamageGunWeapCondBase/Mult` data 0.66/0.34, `fDamageSkillBase/Mult` data
0.5/0.5, `fBSUnitsPerFoot` exe 22, `fGrenadeAgeMax` exe 90,
`fExplosionLOSBuffer` exe 6, `fThrowingStrengthPenalty` exe 0.05,
`fGrenadeHighArcSpeedPercentage` exe 0.67, `fGrenadeThrowHitFractionThreshold`
exe 0.8, `iBallisticProjectilePathPickSegments` exe 4,
`fCombatSplashDamageMaxSpeed/MinRadius/MinDamage` exe 3000/50/20.

## Traced behaviour

Addresses are PC function entries; Xbox names where the prototype has them.

- **Record layouts.** `PROJ` `DATA` at form `+0x60`: timer `+0x80`
  (`00644790`), proximity `+0x7c` (`0045cd80`), speed `+0x68`
  (`0069ef80`), bounciness `+0xb0` (`006d2c20`). Flag tests: 0x02
  explosion (`004fd360`), 0x04 alt. trigger (`00975300`), 0x400 detonates
  (`005de080`). `EXPL` `DATA` at `+0x74`: damage `+0x78` (`006a78f0`),
  radius `+0x7c`, flags `+0x88` (`00477950`), image space radius `+0x8c`.
- **Radius units** (`BGSExplosion::GetRadiusBSUnits`, `00477900`): the
  stored radius × `fBSUnitsPerFoot` unless flag 0x01 is set (also for the
  image space radius `009ac550` and radiation radius `009ad5a0`).
- **Throwing** (`00523150`): animation types 10–13 fire from the actor's
  hand node and use up the weapon (the count/unequip step at the end;
  the per-shot decrement through vfunc `+0x3f0` is inferred). Speed
  (`009669c0`, `009bca60`): the record's speed; animation type 13 ×
  (1 + `fThrowingStrengthPenalty` × (STR − Str req)); perk entry 59;
  the AI's high arc × 0.67 (`009cb830` → `00966a00`).
- **Grenade object** (`GrenadeProjectile`, vtable `0108f674`; constructor
  `009b4020`, made for type 2 by `009bd5c0`/`009bca60`). Initialize
  (`009bda10`): the fuse `+0xe4` = the record's timer unless a mine
  (`009bdf80`). Havok setup (`009be0a0`): restitution = bounciness ×
  `fGrenadeRestitution` (the setting alone if bounciness ≤ 0), friction
  `fGrenadeFriction`, when those settings are non-zero.
- **Update** (`GrenadeProjectile::UpdateProjectile`, `009b41b0`): past
  `fGrenadeAgeMax` it is removed (`009bc8f0`); then the explosion check
  (`009c3190`): alt. trigger → fuse −= frame time, explode at ≤ 0 (mines
  with no fuse wait for proximity, `009c39e0`); otherwise explode once the
  impact list is non-empty unless "detonates". Impact-triggered grenades
  explode at the impact point (flag 0x2000 set by `009b40d0`), others at
  their position.
- **Spawn** (`Explosion::SpawnExplosion`, `009ac9c0`; constructor
  `009ac200`; `Explosion::Initialize`, `009acd60`): radius = record radius
  × player perk entry 72 "Adjust Explosion Radius" (asked about the
  weapon) when the projectile's shooter is the player.
- **Targets** (`Explosion::InitHavok`, `009ae280`: a phantom sphere of the
  radius) and **damage** (`Explosion::ProcessTargets`, `009b00a0`): per
  target, share = 1 − (d/R)² inside R (`00647920`, d from the explosion to
  the target's position); damage = `Explosion::GetDamage` (`009b0f80` with
  `006479e0`) × share; actors must pass `Explosion::RunLOSPick`
  (`009b1810`); the hit is built by `009b5770` (attacker = the explosion's
  actor cause, weapon, flag 0x2000: armour `009b5a30`, critical
  `009b7060`, multiplier `009b73d0`) and handled by `0089a760`.
- **GetDamage**: (`fDamageGunWeapCondBase` + `…Mult`) × record damage;
  with an actor cause and weapon × (`fDamageSkillBase` + `fDamageSkillMult`
  × skill × 0.01).
- **Line of sight** (`009b1810`): skipped with flag 0x10. A cast from the
  explosion to the target; each hit blocks unless (component 2 of a vector
  built from the hit, read here as the ray's z, < −0.98 and the hit is
  within `fExplosionLOSBuffer`) or it belongs to the source. Actors with
  3D get six more points ±2 × a radius (`0084d030`, `+0xc`) on each axis.
- **Knockdown** (`009b0d70`, `00646400`), **limb targets** (`009afcc0`,
  `009b1720`), **forces** (`009b0920`, `fExplosionMaxImpulse`), **shudder**
  and image space (`009b00a0` tail, `009acd60`): read, not implemented.
- **AI throws** (`CombatProcedureAttackGrenade`, `009cafe0`; aim
  `009cb6e0` → `009a8bf0` → `009a8df0` → `009a8f00`): aim mode (`009a8bf0`):
  2 for a high arc of a falling projectile, 4 (feet, `009a8460`) for
  splash damage, else 1 (half height); pitch (`009a7c60`): the low/high
  root of the ballistic equation, 45° when out of reach; aim point
  z = tan(pitch) × x + origin z. Arc choice (`009a7670`): low first, then
  high; a path test (`009a6e90`, `iBallisticProjectilePathPickSegments`)
  must not be blocked before `fGrenadeThrowHitFractionThreshold`.
- **Band** (`009a9180`): an exploding projectile keeps the minimum range
  at least the explosion radius.

## Implemented

Core (`crates/world`):

- `world::explosions` (new): `ProjectileRecord`, `ExplosionRecord`,
  `falloff`, `base_damage`/`damage_with`, `blast_radius`, `launch_speed`,
  `is_thrown`, `FlightSettings`, `Flight` (launch, step, fuse, expiry),
  `los_clear`, `blast_targets`, `aim_mode`, `aim_height`, `launch_pitch`,
  `aim_point`, `arc_clear`. Translated functions carry their address.
- `Runner::explosion_hit` (`world::scripting`): `OnHit`/`OnHitWith`,
  armour, critical, health, `OnDeath`, crime and fighting back.
- `combat_ai::ranged_band_blast`; perk entries 59 and 72.

Viewer:

- `explosives` (new): throw queue, launch (AI arc choice against the
  cell collision, using up the weapon), flight through the collision,
  explosion sounds, hits on everyone alive in range and sight, hit reports.
- `combat`: the player's attack throws weapons of types 10–13.
- `fighting`: people holding one keep out of its blast and throw it.

## Tested (generated inputs, no game files)

`world` unit tests `explosions::tests` (15: record parsing, radius units,
falloff, damage, restitution, fuse/landing/rest, impact grenades, expiry,
line of sight incl. the floor buffer and offset points, targets, launch
pitch, aim point, aim mode, arc test); `combat_ai::tests::
dynamite_keeps_its_thrower_out_of_the_blast`; integration test
`scripting::explosions_hurt_through_the_hit_path`; viewer
`explosives::tests` (normals, queue).

## Not compared / gaps (labelled in code)

- Nothing compared with the original game yet (fuse timing, throw arcs,
  damage at distance, who is shielded).
- The grenade is a point with a simple contact response, not Havok's rigid
  body (shape, spin, the surface's friction, the model's own restitution
  when a setting is 0). It collides with the world only, not with actors.
- The LOS vector's identity in `009b1810` and `fExplosionLOSBufferDistance`
  (24)'s use are not resolved; the target point is its position (feet);
  bodies use the people's controller radius for the offsets.
- Throw origin (eye for the player, 60 units up for people), immediate
  release (no throw animation timing), target height 128 for the aim.
- Not done: models (projectile, explosion), light, image space, decals,
  camera shake, the countdown sound, forces on bodies (forces on clutter:
  [PHYSICS.md](PHYSICS.md), `009b0920`), knockdowns,
  limb damage from blasts, mines/proximity, damage to destructible
  objects, the planner switching NPCs to grenades (only the weapon in hand
  is thrown), `CheckExplosionAttack` (`00992720`, friends in the blast).
- Explosion hit reports carry the thrower's weapon, so the hit sounds are
  the weapon's impact set (which set the game uses for blasts isn't
  traced).

Next action: run the Goodsprings gunfight with the player and a settler
holding `WeapNVDynamite`, and compare fuse, arc and damage with the
original game.
