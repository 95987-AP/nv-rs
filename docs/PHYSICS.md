# Physics: clutter Havok moves

Free rigid bodies (clutter, props, weapons lying about) as the game hands
them to Havok: read from the models, simulated, pushed by shots, blasts and
the player, drawn where they are and kept where they come to rest. Branch
`claude/m2-physics`, 2026-10-06. Private exports (decompiles, logs,
frames): `%USERPROFILE%\nv-re\work\physics-2026-10-06`.

Status words: **traced** (read in FalloutNV.exe 1.4.0.525 or the game's
data), **implemented**, **tested** (generated regressions), **verified
live** (viewer run on installed data), **not compared** (nothing here has
been checked against the running original game).

## The VCG02 bottles (Back in the Saddle)

`VCG02Bottle` (MISC `0010A1F6`, "Sunset Sarsaparilla Bottle") is ordinary
Havok clutter: model `clutter\junk\SSBottle02.NIF`, one
`bhkConvexVerticesShape` hull (18 corners, 14 faces, convex radius 0.1) on
a `bhkRigidBody` on layer 10 (props): mass 1, linear/angular damping
0.1/0.05, friction 0.5, restitution 0.4, most speeds 1068 Havok units/s and
31.57 rad/s, motion system 4 (box inertia), quality 3 (debris). No `DEST`
data: the bottles don't break, they get knocked off. The seven placed
bottles (`0010A202`–`0010A209`, not `206`) follow `VCG02BottleMarkerREF`
(initially disabled) and stand on the fence rail at z 8438.1; their record
flags are 0 (no "Don't Havok Settle"). Their script `VCG02TargetSCRIPT`:
`OnLoad SetDestroyed 1` (they can't be taken), `OnHitWith` counts
`VCG02.nTargetCount` (once per bottle, weapon out, animation types 4–8),
Sunny's `SayTo`, `SetObjectiveCompleted VCG02 10` at 3 and `SunnyREF.evp`;
`OnActivate` (after the quest) `SetDestroyed 0`. Scripts are the shots'
owners' path (`combat`); the knock-off is the physics below.

## Traced

| What | Where | Rule |
| --- | --- | --- |
| Rigid body block | `bhkRigidBody` (236 bytes, checked on the bottle) | inertia rows 116, centre 164, mass 180, damping 184/188, friction 192, restitution 196, most speeds 200/204, penetration depth 208, motion 212, deactivator 213, solver deactivation 214, quality 215 (`nif::RigidBodyInfo`) |
| Step clock | `00c66760` (`iUpdateType` 0 from `[HAVOK]`, `0044fb20`) | fixed steps of `fMaxTime` (0.016, `Fallout_default.ini`) × the time multiplier at `011ac3a0` (1); steps = the accumulated time ÷ step rounded, at most 3; the rest (can be negative) carried, held to one step; under half a step waits (`physics::rigid::Clock`) |
| Gravity | `00f4b550` | 98.1 Havok units/s² (existing `physics::GRAVITY`) |
| Gameplay impulse scaling | `TESHavokUtilities::ScaleGameplayImpulseForce` (Xbox PDB), `0062b520` | × `fGameplayImpulseMult{Biped (layers 8, 29) 0.2, Prop (10) 0.1, Trap (14) 0.15, DebrisLarge (20) 1, Clutter (else) 1}`; × mass ÷ `fGameplayImpulseMinMass` (5) when lighter; × `fGameplayImpulseScale` (150) |
| Shot push | `Projectile::ApplyImpactForce` (Xbox PDB), `009c2e80`, from `Projectile::ProcessImpacts` `009c1b70` | normalized projectile velocity (`+0x104`) × PROJ impact force (`+0x94`, `00644930`) scaled as above, applied at the impact point to bodies whose motion type is below 4 (`00517630`); not for projectiles that explode on impact |
| Explosion push | `Explosion::PushRigidBody` (Xbox PDB), `009b0920`; `Explosion::ApplyForces` `009afef0` | EXPL force (`+0x74`) > 0; direction explosion → body, normalized, z + `fExplosionForceClutterUpBias` (0.5) for biped/dead-biped layers only (`00624070`); × `fExplosionSourceRefMult` for the source's own body (then uncapped); scaled as above; min `fExplosionMaxImpulse` (8000); linear × `fExplosionForceMultLinear` at the centre, angular random(−1..1)³ × force × `fExplosionForceMultAngular`; bodies that move or are large debris; "push source only" (flag 0x20) |
| Moved references | `0083fef0` names `CHANGE_REFR_HAVOK_MOVE` | moved objects are saved where they are |
| Grab (Z key) | `0095f930`, `00960520` | `fZKey…` settings read, not implemented |
| Physics damage | `0062be90`, `006238b0` | `fPhysicsDamage…`, not implemented |

Unused finds: `0081f000`/`008d0020` are another knock path (base 1750 or
the object's `+0x98`, `fProjectileKnockMult…`, `fProjectileCollisionImpulseScale`)
reached from `00810aa0`/`008133f0`/`008cba30`, not the bullet path.

## Implemented

- `nif::RigidBodyInfo` on every `CollisionPart` (values in the model's
  space and game units).
- `physics::rigid`: `RigidWorld`, `Clock` (`00c66760`), bodies with their
  hulls/spheres/capsules, contacts against the collider (body corners
  against faces, world edges reaching into hulls), against each other and
  against walkers (`Mover`), friction and restitution, damping, speed
  limits, sleeping and waking, Havok-unit impulses. `physics::impulses`:
  the three translated pushes. `Collider` triangles carry their body's
  friction and restitution (`Surface`).
- `preview::CellScene::dynamic_bodies` (one moving body per reference;
  those parts left out of the static collider), `ViewerScene::bodies`.
- `world::GameState::havok_moved`, saved as `havokmove`.
- Viewer `clutter`: registers spawned bodies (at their saved pose when
  moved), settles them as their place loads unless flagged "Don't Havok
  Settle" (0x20000000, xEdit's name), keeps their triangles in
  `CellCollision` under the reference and moves them, moves their drawing,
  takes shot pushes (`hiteffects::HitReports::shot_on_world`) and blasts
  (`explosives::explode`), the player as a pusher, writes moved poses into
  the state every frame they move.

This solver's own (labelled in code; Havok's internals aren't in the
game's code we traced): XPBD substeps (8) and passes (4); contact
generation (no edge-against-edge between bodies); friction and restitution
combined as √(a·b) (Havok's default `hkpMaterial`); surfaces without a body
use Havok's default 0.5/0.4; sleeping after 1 s under 2 units/s and 0.3
rad/s; overlap recovery at most 0.05 units per correction; walkers as
unstoppable capsules reaching 2 units out (`hkpCharacterProxy` untraced);
a body's velocity isn't saved (a mid-air save resumes from rest).

## Tested

`nif` scene `reads_collision_shapes_in_game_units` (body values); `physics`
`impulses::tests` (3), `rigid::tests` (clock, fall and rest, shot off a
rail, stacking and walker push, deltas, impulse units, surfaces through
`extend`, rail end under a body, six-sided bottle on a 3-unit rail at the
origin and at Goodsprings' coordinates); `preview` collision
`moving_clutter_is_a_body_and_left_out_of_the_collider`; `world`
`save::tests::havok_moved_objects_keep_their_pose_through_a_save`; viewer
`clutter::tests` (Bevy-space deltas, shot queue).

## Verified live

Release viewer, installed data, `nv-viewer <Data> WastelandNV --at
-68232.9,4900,8400,0,8.2 --walk --weapon WeapNVVarmintRifle --run
"StartQuest VCG02" --run "VCG02BottleMarkerREF.Enable"`, driven by a
PowerShell script (user32 `SetCursorPos`/`mouse_event`/`keybd_event`,
frames with `CopyFromScreen`; private `drive.ps1`):

- The square loads with its clutter as bodies ("13 placed objects Havok
  moves"); placed clutter settles and sleeps within about a second
  (barrels, crates, tumbleweeds, the bottles, which rise 1.4 units out of
  the rail's shells onto it and stay standing).
- A left click with the crosshair on bottle `0010A208` (149 units):
  `Hit 0010A208`, glass impact, "the shot pushes 0010A208: impact force 3
  → impulse 9.0 (layer 10, mass 1)", Sunny's `SayTo` from `OnHitWith`. The
  bottle tips backward off the rail and comes to rest on the ground behind
  the fence at (−68220.1, 5076.4, 8386.4) (run p1; frames before, during
  and after).
- F5 then F9: the save holds `havokmove 0010A208 …`; after the reload the
  bottle is back on the ground where it lay (35.5 s: rest at −68220.0,
  5076.5, 8386.4), not on the rail.

Not driven live: explosions on clutter (unit-tested only), the player
pushing clutter, other bottles, leaving and re-entering the squares.

## Not compared / gaps

- Nothing compared with the original game: how far bottles fly, how they
  tumble, settle heights, rest times.
- Havok's solver, contact manifolds, deactivation and penetration recovery
  are not reproduced (above). Models with several moving bodies stay
  static. Inertia under a reference's scale isn't traced (scaled by s²,
  mass kept).
- Not done: Z-key grab, physics damage from flying objects, NPCs pushing
  clutter, explosions' source-reference rule (no body counts as the
  source's), the phantom's exact overlap (centre within the radius),
  ragdolls taking shot/blast pushes (they use `physics::ragdoll`).
- Seen, owned elsewhere: a scripted object's shot test uses its placed
  bounds, so after the bottle falls the place it stood still takes hits
  (`combat`); the fence's walking collision stops shots from behind it
  (shots use the walking collider, not the projectile layer); the bottles
  show "E) Take" though `SetDestroyed 1` should block it.

**Next action:** record a bottle shot off the fence in the original game
(fall direction, distance, rest time) and compare.
