# Player movement and the character controller

Topic file for the player's walking, running, sneaking and jumping, and
the character controller's ground and air states. Read from
`FalloutNV.exe` 1.4.0.525 (unpacked image) with Ghidra; names marked
(Xbox PDB) come from the Xbox 360 prototype symbols (ADR-0002). Earlier
controller facts (shape, step, slope, gravity, fall damage) are in
`CLAUDE_REFERENCE.md`/`ENGINE_REFERENCE.md` and the private
`findings/physics.md`; they are not repeated here.

Status words: **implemented** (code exists), **tested** (regression test),
**compared** (checked against the running original game). Nothing in this
batch has been compared.

## Speed (`world::locomotion`)

| Address | What | Status |
| --- | --- | --- |
| `00885a50` | `Actor::GetWalkSpeed` (Xbox PDB): gathers the inputs and calls `00647d10`; × a factor from actor vfunc +0x428 when it returns an object (the player's `00acbb70` returns none) | implemented, tested |
| `00885bf0` | `Actor::GetRunSpeed` (Xbox PDB): `00647f00`, the same vfunc +0x428 factor, then perk entry point 42 (Modify Run Speed) | implemented, tested (perk entry through the existing perk code) |
| `00647d10` | The formula, translated in `locomotion::walk_speed` | implemented, tested |
| `00647f00` | Running = walking × `fMoveRunMult` | implemented, tested |
| `00514410` | Armour class from the biped model's general flags (`ARMO` +0x70 + 8 = `BMDT` byte 4): 0x80 heavy (`004c0bd0`), else 0x08 medium (`00514450`) | implemented, tested |
| `00891b90` | The armour asked about: the item equipped in biped slot 2 (upper body) | implemented |
| `00646cb0`, `00446390` | Weapon animation type at form +0xf4; types 2, 5, 6, 8, 9 pass the two-handed test | implemented, tested |

The formula (`00647d10`): SpeedMult (AV 21) × 0.01 (double constant
`01031148`) × `fMoveBaseSpeed` × legs, × `fMoveNoWeaponMult` when the
weapon is away and actor vfunc +0x218 is true (people `008d0360` → 1,
creatures `0047c850` → 0), × (1 − armour penalty − drawn weapon penalty),
× `fMoveSneakMult` sneaking (movement flags 0x400 without 0x800,
`004997b0`); negative → 0.

- Legs: 1.0, `fMoveOneCrippledLegSpeedMult` with one crippled,
  `fMoveTwoCrippledLegsSpeedMult` with two (`00646800`: condition ≤ 0).
  IgnoreCrippledLimbs (AV 72) resets it to 1 only when actor vfunc +0x358
  is false; for the player that vfunc is `00954cc0`, over-encumbered. So
  an over-encumbered player with IgnoreCrippledLimbs is still slowed.
  (`body_parts::leg_speed_mult`, used by AI code, does not have this
  exception; not changed here.)
- Weapon penalty, drawn only: animation types 5 and 6 `fMove2HRPenalty`
  (0.1), 8 and 9 `fMove2HBigPenalty` (data 0.1), 2 (two-handed melee) and
  7 (energy rifles) none.
- Armour penalty: `fMoveHeavyArmorPenalty` (data 0.15) or
  `fMoveMediumArmorPenalty` (data 0.075).
- `FalloutNV.esm` values: base 77, run × 4, sneak × 0.57; exe defaults
  for `fMoveNoWeaponMult` 1.1 and `fMove2HRPenalty` 0.1.

Deviations fixed: the viewer had hard-coded 77/4/0.57, no weapon-away
× 1.1, no armour or weapon penalties, and applied the run perk to walking
too. Float order follows the exe as single-precision steps; the tests
declare CPU tier B assuming PC=24 (the D3D thread's control word), not
confirmed for this caller and not run on `nv-call`.

## Running and jumping allowed (`0093e860`)

The player's control handler (an aligned-stack prologue at `0093e860`;
Ghidra had no function there). Keyboard path: with Always Run (control
10) on, running unless Run (control 9) is held; off, running while it is
held; in both cases not when process vfunc +0x3e4 `GetAnimAction` (Xbox
PDB) is 7, the player is over-encumbered (vfunc +0x358), or process vfunc
+0x404 `GetIronSights` (Xbox PDB) is set. Then running is cleared when the
grabbed object's weight (player +0x640) exceeds `fGrabMaxWeightRunning`
(50). Each direction key sets its flag and walk (0x100) unless running
(0x200) is set. Jump (control 12) is ignored while over-encumbered.

Implemented: `may_run` (over-encumbered, iron sights, grabbed weight),
`may_jump`. Tested. The viewer passes no iron sights and no grabbed weight
(neither exists there yet); anim action 7 is not modelled.

## Jump and air control (`physics::Character::update_controlled`)

| Address | What | Status |
| --- | --- | --- |
| `00930640` | Jump height `fJumpHeightMin` × `GetScale` (`00567400`), × `fJumpSwimmingMult` swimming | implemented, tested (player scale taken as 1) |
| `00884aa0` | Player vfunc +0x254: `00930640` then the jump sound | sound not done here |
| `00cd4280` | Jumping state: vertical speed √(2 \|g × mult\| h); horizontal velocity **replaced** by the support's velocity (`01267e30`, zero for static ground) | implemented, tested |
| `00cd3fb0` | In-air state: the movement input's gain = air control × 0.3 + 0 (`011b0140`, `01267bbc`), or 1 with controller flags 0x1800; max change 2000 Havok/s (`01013970`); vertical component restored, gravity added | gain implemented, tested; flags 0x1800 not traced |
| `0087d6c0` | Actor vfunc +0x2e4, air control: 1.0 for everyone | implemented |
| `00cd4800` | On-ground state: gain 1, max change 500 Havok/s (`01013d84`) | implemented |
| `00c73170` | The controller's per-frame movement ("TtCharacter movement"): desired velocity = the move delta ÷ its dt (z kept only flying or swimming) | used: one controller update per frame, assumed because `PlayerMover::Update` (`009e9e50`) hands the frame's dt and delta to player vfunc +0x250; not confirmed by a recording |

Deviations fixed: a jump kept the run-up's horizontal velocity and the
player had full control in the air. Now the jump starts with no horizontal
velocity on still ground and the velocity closes 0.3 of the gap to the
wanted one each frame (frame-rate dependent, if the controller is
integrated once per frame as assumed above). The player may not jump while
over-encumbered. The earlier finding that the jump keeps the horizontal
velocity was wrong: `00cd4280` adds the jump vector to the support's
velocity (`01267e30` holds zeros).

Unresolved, labelled in code:
- Whether the in-air state also runs in the frame the jump starts
  (`00c6cba0`); here it doesn't.
- The wanted velocity in the air: in the game it is the PlayerMover's move
  delta (`009e9e50`, `PlayerMover::Update` (Xbox PDB)), whose length comes
  from the current movement animation's root motion (`00494390`) and whose
  direction comes from the keys. Here it is the key direction at the
  formula speed, on the ground and in the air.
- Slopes: on the ground the game builds the velocity in the support's
  plane (`00d6aef0` with the support normal) and Havok's proxy solver
  slides it; here the horizontal velocity is used unchanged and the
  capsule is stepped and snapped down.
- Backward and sideways speeds: the game's speed is the movement
  animation's root speed × (formula speed ÷ the Forward group's root
  speed) (`00895110`, `world::animation::movement_rate`), so backward and
  strafe speeds follow each animation's own root speed. Here every
  direction moves at the formula speed.
- Camera follow: the camera still sits at `cellview::EYE_HEIGHT` above the
  feet; the game's first-person camera node was not traced in this batch.

## Settings

| Setting | Exe | Data | Read by |
| --- | --- | --- | --- |
| `fMoveBaseSpeed` | 85 | 77 | `00647d10` |
| `fMoveRunMult` | 4 | 4 | `00647f00` |
| `fMoveSneakMult` | 0.6 | 0.57 | `00647d10` |
| `fMoveNoWeaponMult` | 1.1 | – | `00647d10` |
| `fMoveHeavyArmorPenalty` | 0.2 | 0.15 | `00647d10` |
| `fMoveMediumArmorPenalty` | 0.1 | 0.075 | `00647d10` |
| `fMove2HRPenalty` | 0.1 | – | `00647d10` |
| `fMove2HBigPenalty` | 0.15 | 0.1 | `00647d10` |
| `fJumpHeightMin` | 64 | – | `00930640` |
| `fJumpSwimmingMult` | 2 | 2 | `00930640` |
| `fGrabMaxWeightRunning` | 50 | – | `0093e860` |
| `fJumpMoveMult`, `fJumpMoveBase` (GMST) | 0.3, 0 | – | no reader; the controller uses its own statics |

## Checks and handoff

Branch `claude/m1-movement`. Tests: `world` `locomotion::tests` (5),
`physics` `a_running_jump_starts_from_standing_and_steers_three_tenths_a_frame`.
Not compared against the original game. Next action: record the player's
speed and jump arc in the original (holstered/drawn rifle, heavy armour,
over-encumbered, running jump distance) with `nv-probe` on `00885bf0` and
the controller's velocity, and compare with these rules.
