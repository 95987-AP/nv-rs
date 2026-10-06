# Physics: clutter Havok moves

Free rigid bodies (clutter, props, weapons lying about) as the game hands
them to Havok: read from the models, simulated, pushed by shots, blasts,
the player and people, carried with the Grab key, sounding when they hit
something, drawn where they are and kept where they come to rest.
Branches `claude/m2-physics` and `claude/m2-physics-2`, 2026-10-06.
Private exports (decompiles, logs, frames):
`%USERPROFILE%\nv-re\work\physics-2026-10-06` and `…\physics2-2026-10-06`.

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
`OnLoad SetDestroyed 1`, `OnHitWith` counts `VCG02.nTargetCount` (once per
bottle, weapon out, animation types 4–8), Sunny's `SayTo`,
`SetObjectiveCompleted VCG02 10` at 3 and `SunnyREF.evp`; `OnActivate`
(after the quest) `SetDestroyed 0`.

`SetDestroyed 1` is why the game shows no prompt on them: a destroyed
reference (form flag 0x800000, `00477ba0`) under the crosshair gets no
activate prompt unless it is an actor (`00775a00`, the HUD's crosshair
update, clears the prompt tiles), and activating one returns at once
(`005180b0`). The crosshair still finds them, so the Grab key works on them.

## Traced

| What | Where | Rule |
| --- | --- | --- |
| Rigid body block | `bhkRigidBody` (236 bytes, checked on the bottle) | inertia rows 116, centre 164, mass 180, damping 184/188, friction 192, restitution 196, most speeds 200/204, penetration depth 208, motion 212, deactivator 213, solver deactivation 214, quality 215, constraint count 228 (`nif::RigidBodyInfo`) |
| Step clock | `00c66760` (`iUpdateType` 0 from `[HAVOK]`, `0044fb20`) | fixed steps of `fMaxTime` (0.016, `Fallout_default.ini`) × the time multiplier at `011ac3a0` (1); steps = the accumulated time ÷ step rounded, at most 3; the rest (can be negative) carried, held to one step; under half a step waits (`physics::rigid::Clock`) |
| Gravity | `00f4b550` | 98.1 Havok units/s² (existing `physics::GRAVITY`) |
| Havok world settings | `00c681c0` over Havok's defaults `00c90b80` | collision tolerance 0.1, contact resting velocity FLT_MAX, simulation type from `iSimType`, expected step `fMaxTime`; kept from Havok: solver 4 iterations, tau 0.6, damping 1, deactivation reference distance 0.02 Havok units |
| Collision filter (layers) | `bhkCollisionFilter`: table built by `00c828f0` (with `00c827f0`, `00c82870`), asked by `00c84740` (bodies `00c84880`, casts `00c84930`) | 43 × 64-bit rows; bodies of different groups touch when the row of the first has the second's layer; "no collision" flag 0x4000; one ragdoll's bones by a part table (`physics::layers`). Row 30 (character) equals the walking layers worked out earlier; row 6 (projectile) lacks 3 (`TRANSPARENT`), 7, 16, 21, 22, 31, 35, 42, which the character's has, and has 8 (biped), 23, 29 (dead biped), 37, which it lacks |
| Contact materials | `00cfd800` (the contact manager's new point) | friction √(f₁f₂) (entity `+0x90`), restitution √(r₁r₂) (`+0x94`) kept as a byte ×128 rounded |
| Land body | `00621f60` | fixed body, friction `[Landscape] fLandFriction` (2.5), Havok's default restitution 0.4 (`00c8f510`) |
| Gameplay impulse scaling | `TESHavokUtilities::ScaleGameplayImpulseForce` (Xbox PDB), `0062b520` | × `fGameplayImpulseMult{Biped (layers 8, 29) 0.2, Prop (10) 0.1, Trap (14) 0.15, DebrisLarge (20) 1, Clutter (else) 1}`; × mass ÷ `fGameplayImpulseMinMass` (5) when lighter; × `fGameplayImpulseScale` (150) |
| Shot push | `Projectile::ApplyImpactForce` (Xbox PDB), `009c2e80`, from `Projectile::ProcessImpacts` `009c1b70` | normalized projectile velocity (`+0x104`) × PROJ impact force (`+0x94`, `00644930`) scaled as above, applied at the impact point to bodies whose motion type is below 4 (`00517630`); not for projectiles that explode on impact |
| Explosion push | `Explosion::PushRigidBody` (Xbox PDB), `009b0920`; `Explosion::ApplyForces` `009afef0` | EXPL force (`+0x74`) > 0; direction explosion → body, normalized, z + `fExplosionForceClutterUpBias` (0.5) for biped/dead-biped layers only (`00624070`); × `fExplosionSourceRefMult` for the source's own body (then uncapped); scaled as above; min `fExplosionMaxImpulse` (8000); linear × `fExplosionForceMultLinear` at the centre, angular random(−1..1)³ × force × `fExplosionForceMultAngular`; bodies that move or are large debris; "push source only" (flag 0x20) |
| Grab (Z) | `PlayerCharacter::HandlePhysicsGrab`, `::CreateMouseSpring`, `::UpdateMouseSpring`, `::DestroyMouseSpring` (Xbox PDB): `0095f6c0`, `0095f930`, `00960520`, `00961280`; spring `hkpMouseSpringAction::applyAction` `00cbb1e0` | see `physics::grab`: control 27 ("Grab", Z) toggles; crosshair reference whose body moves and weighs ≤ `fGrabMaxWeightWalking` (100); held at the picked point's distance, ≥ controller radius + 5; target = eye + view × distance cut by a cast; lets go past 96 units, or past `fZKeyMaxContactDistance` (10) against a body > `fZKeyMaxContactMassRatio` (4) × as heavy; spring `fZKeySpringDamping` 0.5, `…Elasticity` 0.2 (trap layer × 0.1), `fZKeyObjectDamping` 0.75, `fZKeyMaxForce` 750 (trap × 0.5); the spring: velocities × object damping, impulse −K⁻¹(damping·v + elasticity/dt·error) at the point, at most dt·mass·force. No throw: no setting or code path found |
| Contact listener | `FOCollisionListener::contactPointAddedCallback` (Xbox PDB), `00623cb0` | for each contact point added: sound when |projected velocity| × 7 ≥ `fMinSoundVel` (10) and ≥ 1; then physics damage |
| Impact sounds | `ImpactMixer::PlayCollisionSound` (Xbox PDB), `00837550`; picker `00839e00` | each side's sound by its Havok material, its mass (5 without a moving body) and the speed (`fCollisionSoundHeavyThreshold` picks H/L sets; `f…MediumMassMin`/`LargeMassMin`, glass/wood-for-water 6/15, grass 6/70; below 1e-4 the "Static" sounds); nothing for skin on skin; one per material pair `(a+1)(b+1)` per `iCollisionSoundTimeDelta`; static attenuation (1 − min(speed/400, 1)) × 3000 hundredths of a dB; command 0x42 with 1 or (4500 − attenuation)/3000 (read as the frequency) |
| Physics damage | `FOCollisionListener::StoreObjectDamage` `006238b0`, `0062be90`, `DealObjectDamage` `00623640` | |projected velocity| (Havok units) ≥ `fPhysicsDamageSpeedMin` (150); a side takes it if its reference has destructible data (flag 0x1000000), isn't destroyed and neither side is a projectile; damage by the other side's mass (fixed: 10000): < 10 none, < 50 1 above 500, < 100 5 above 350, else 10 above 150, × (1 + 0.0001 × speed); dealt to the reference later (`+0x144`, the object damage virtual) |
| Character contacts | `00c711d0` with `fMoveLimitMass` (95, `011b0128`) | the controller's contact callback treats bodies ≥ 95 apart (keeps the surface velocity only for lighter ones); read as: walkers push only lighter bodies |
| Saved Havok data | `TESObjectREFR::SaveHavokDataForCollisionObject` / `LoadHavokData…` (Xbox PDB): `00563220`, `00563380` | per body: position and rotation, a flags byte (bit 1 active, bit 2 keyframed), then for an active body its linear and angular velocity; loading sets both and activates it |
| Moved references | `0083fef0` names `CHANGE_REFR_HAVOK_MOVE` (flag 4) | moved objects are saved where they are |

Unused finds: `0081f000`/`008d0020` are another knock path (base 1750 or
the object's `+0x98`, `fProjectileKnockMult…`, `fProjectileCollisionImpulseScale`)
reached from `00810aa0`/`008133f0`/`008cba30`, not the bullet path. Mode 3
of the mouse spring is telekinesis (`fMagicTelekinesis…`).

## Implemented

- `nif::RigidBodyInfo` on every `CollisionPart` (values in the model's
  space and game units), with its constraint count.
- `physics::layers`: the collision filter's table and word test
  (translated). `Collider` triangles carry their body's layer;
  `Collider::raycast_layer` casts as a layer does.
- `physics::rigid`: `RigidWorld`, `Clock`, bodies with their
  hulls/spheres/capsules, contacts against the collider, each other and
  walkers (`Mover`; only bodies under `MOVE_LIMIT_MASS`), friction and
  restitution combined as the game does, damping, speed limits, sleeping
  and waking, Havok-unit impulses, `Spring` (the mouse spring),
  `ContactEvent`s (a pair beginning to touch, with its closing speed),
  `set_velocity`. `physics::impulses`: the three pushes. `physics::grab`:
  the grab's rules. `physics::contacts`: sound choice, attenuation, the
  pair key, physics damage. `physics::LAND_SURFACE` on the terrain.
- `preview::CellScene::dynamic_bodies` (one moving body per reference;
  those parts left out of the static collider); models with several
  moving bodies or constrained bodies stay solid and are listed at load
  (`unsimulated_bodies`).
- `world::GameState::havok_moved` (`havokmove`) and `havok_velocity`
  (`havokvel`) in saves.
- Viewer `clutter`: bodies registered (at their saved pose and velocity;
  once the collider has ground under them), settled, moved in
  `CellCollision` and drawn; shot pushes and blasts;
  the player and every moving `Walker` push (and `clutter::actor_walks`
  for actor controllers to report their controller); the Grab control
  (`grab_held`); contact sounds through `SoundRequests` with the material
  pair gate; physics damage worked out and logged for destructible
  references. `combat`: shots cast on the projectile layer; scripted
  objects with bodies are met by their triangles where they are now.
  `scripts`: no prompt on destroyed references; bodies picked where they
  are. `walk`/`hud`: only doors are taken for swinging doors.
  `controls`: the Grab binding.

This solver's own (labelled in code; Havok's solver isn't translated):
XPBD substeps (8) and passes (4); contact generation (no edge-against-edge
between bodies); sleeping after 1 s under 2 units/s and 0.3 rad/s (Havok's
deactivation uses its reference distance and frame counters, not
translated); overlap recovery at most 0.05 units per correction; walkers
as unstoppable capsules reaching 2 units out (`hkpCharacterProxy`'s
surface interactions aren't translated); a contact "added" is a pair that
starts touching (Havok adds and removes single points).

## Tested

`nif` scene `reads_collision_shapes_in_game_units`; `physics`
`layers::tests` (3), `grab::tests` (2), `contacts::tests` (3),
`impulses::tests` (3), `rigid::tests` (clock, fall and rest, shot off a
rail, stacking and walker push, the move limit, the player walking into
a body whose triangles are in the collider, deltas, impulse units,
surfaces through `extend`, material combination, landing contact event,
grab spring carry/release/removal, rail end, bottle at the origin and at
Goodsprings' coordinates), `lib` `shots_pass_a_transparent_fence_that_stops_walkers`;
`preview` collision `moving_clutter_is_a_body_and_left_out_of_the_collider`;
`world` `save::tests::havok_moved_objects_keep_their_pose_through_a_save`
(with velocities); viewer `clutter::tests`, `controls::tests`.

## Verified live

See the 2026-10-06 runs below (release viewer, installed data, input
posted to the viewer's own window, F12 reports and logs in
`physics2-2026-10-06\<run>`).

Command: `nv-viewer <Data> WastelandNV --at <x,y,z,heading,pitch> --walk
--weapon <W> --run "StartQuest VCG02" --run "VCG02BottleMarkerREF.Enable"`;
driver `drive2.ps1` (`PostMessage` of keys and clicks to the viewer's
window only).

- **Moved bodies are hit where they are** (a1, at −68232.9, 4900,
  pitch 8.2, varmint rifle): the first shot, `Hit 0010A208 at 149 units`,
  push 9.0, `PHYBottleH` as it lands at 331 units/s, rest at (−68220.2,
  5078.8, 8386.7); the second shot at the same spot: "hit nothing but a
  wall" (before, the bottle's placed bounds took the hit).
- **No prompt on the destroyed bottles** (a2, F12 report 001 with the
  crosshair on 0010A208): no Info panel at all. The old "E) Take" came
  from the swinging-door pick taking any owner of collider triangles
  (clutter bodies too) for a door; it now takes doors only.
- **Z grab** (a2, at −68232.9, 4990, pitch 20): "Grabbed 0010A208
  (VCG02Bottle) 63 units away"; walking back one second (S held) carries
  it off the rail and over the fence in front of the player (report 003);
  Z again: "Let go", it drops and bounces (`PHYBottleH` 293 and 103,
  `PHYBottleL` 57 units/s) and rests at (−68242.9, 4856.6, 8360.1).
- **Explosions** (a6, a8, `WeapNVDynamite` thrown at 1200 units/s, 2.5 s
  fuse, in front of the fence): all seven bottles fly (contacts up to
  1985 units/s) and come to rest around the square.
- **Saved velocities** (a6, a8): F5 0.3 s after the blast saves
  `havokvel` for the 13 bodies still moving; F9 sets them going again and
  they come to rest near where they did before the load.
- **Walking into clutter** (a7, at −66018.6, 4650 heading north, W held
  3 s): tumbleweed 00178A80 (mass 2.5) is pushed ahead from y 4770.9 to
  5529.6.
- **Contact sounds**: every landing logs its two sounds by material
  (`PHYBottleH`/`L`, `CStoneMedium`, `CWoodLarge`, `PHYBabyRattle`…) with
  the game's attenuation and frequency (not applied to playback).
- **The wire mesh under the rail** (a3, pitch 48 through it): the shot
  goes through, but the unfiltered cast does too: the mesh has no
  collision triangles here, so the earlier "the fence stops shots from
  behind" was its rail or posts, which projectiles meet in the game too
  (layer 6 touches static layers). Which layer the rail is on wasn't
  established live.
- **Joined bodies**: the loaded squares list "2 placed objects with joined
  or several moving bodies, kept solid": `MaizeWitheredGroup2b` ×2; no
  constrained bodies (hanging signs, chains) load around the square.

Not seen live: people pushing clutter, physics damage (nothing
destructible was struck hard enough), the grab's contact release.
Seen, not fixed: bottle 0010A204 tips off the rail while settling (in the
previous batch's runs too); four burnt fence pickets south of the square
(001788DA, 001788DB, 001788E6, 001788E9) fall through the ground while
settling (cause not found; the viewer now waits for ground under a body
before simulating it, which didn't change them).

## Not compared / gaps

- Nothing compared with the original game: how far bottles fly, how they
  tumble, settle heights, rest times, grab feel, sound choice and volume.
- Havok's solver, contact manifolds, deactivation and penetration
  recovery are not reproduced (above). Constraints aren't simulated:
  joined or constrained bodies stay solid. Inertia under a reference's
  scale isn't traced (scaled by s², mass kept).
- Grab: actors/ragdolls (the complex spring and helper), letting go when
  standing on the held body, the keep-out near the player, the
  hold-Activate grab, `fGrabMaxWeightRunning`.
- Sounds play without the game's attenuation and frequency (the viewer's
  sounds aren't placed or scaled); shell casings' sounds; terrain has no
  Havok material here, so its side is silent.
- Physics damage isn't dealt: nv-rs has no destructible objects yet (the
  amount is logged). Living actors' bones are on the biped layer, which
  clutter's layers don't touch, so flying clutter doesn't hurt people.
- Shots meet bodies on layers in the walking collider only: living
  bones (8) and dead bodies (29) aren't in it (people are met by their
  own capsules in `combat`).
- Explosions' source-reference rule (no body counts as the source's), the
  phantom's exact overlap (centre within the radius), ragdolls taking
  shot/blast pushes (they use `physics::ragdoll`).
- The bullet cast's layer is taken as 6 (`PROJECTILE`) from its name.

**Next action:** record a bottle shot off the fence and a Z-grab carry in
the original game and compare.
