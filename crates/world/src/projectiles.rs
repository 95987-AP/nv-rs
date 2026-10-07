//! Guns' projectiles: whether a shot strikes at once or flies there, and
//! a flying one's way (read from FalloutNV.exe 1.4.0.525 with Ghidra;
//! class names from the Xbox 360 prototype's symbols, marked (Xbox PDB);
//! the energy weapons' evidence is in `docs/ENERGY_WEAPONS.md`). Thrown
//! weapons and grenades are [`crate::explosions`].
//!
//! - A projectile's kind (`PROJ` `DATA` type, [`crate::explosions::proj_type`])
//!   picks its class: beams make a `BeamProjectile`, missiles a
//!   `MissileProjectile` (Xbox PDB). Each class's `Initialize` sets the
//!   projectile's run-time flag 0x1, which `Projectile::Initialize`
//!   (`009bda10` → `009bec90`) turns into a cast along the whole range as
//!   it's made: the hit is decided then (see [`delivery`]).
//! - A missile without it (the plasma bolts) moves each frame by its speed
//!   (`009bf300`: `009669c0`'s speed × the frame's seconds, along its
//!   heading) and is checked for what it struck (`009c3190`); past
//!   `fArrowAgeMax` seconds, or past its range with nothing struck, it's
//!   removed (`MissileProjectile::UpdateProjectile`, `009b8030`). See
//!   [`Missile`].

use esm::LoadOrder;

use crate::explosions::{proj_flags, proj_type, ProjectileRecord};

/// How a projectile's shot meets what it's fired at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Decided as it's fired, by a cast along its range: beams (the
    /// lasers: `BeamProjectile::Initialize`, `00979280`, sets run-time
    /// flag 0x1 always) and missiles with the record's "hitscan" flag
    /// (bullets: `MissileProjectile::Initialize`, `009b7cc0`, sets it
    /// from `009a7f80`, the record's flag 0x1).
    AtOnce,
    /// Flies at its speed and strikes what it meets on the way: missiles
    /// without "hitscan" (the plasma bolts, `PlasmaProjectile` flags
    /// 0x20C).
    Flies,
    /// Thrown or lobbed (lobbers, `crate::explosions`).
    Lobbed,
    /// Flames and continuous beams: their classes' `Initialize` isn't read
    /// yet (treated as decided at once by the callers).
    NotTraced,
}

/// How a projectile is delivered (see [`Delivery`]). `009b7cc0` leaves
/// flag 0x1 off for a missile whose `0044ddc0` is 4, whatever the record
/// says (what that value is isn't traced: not modelled).
pub fn delivery(p: &ProjectileRecord) -> Delivery {
    match p.kind {
        proj_type::BEAM => Delivery::AtOnce,
        proj_type::MISSILE if p.flags & proj_flags::HITSCAN != 0 => Delivery::AtOnce,
        proj_type::MISSILE => Delivery::Flies,
        proj_type::LOBBER => Delivery::Lobbed,
        _ => Delivery::NotTraced,
    }
}

/// `fArrowAgeMax` (the exe's 90 s; no plugin sets it): how long a missile
/// may fly (`009b8030`).
pub fn age_max(order: &LoadOrder) -> f32 {
    crate::scripting::game_setting(order, "fArrowAgeMax").unwrap_or(90.0)
}

/// A missile in flight (`MissileProjectile`, Xbox PDB): where it is, its
/// heading (unit), its speed (units a second, as launched:
/// [`crate::explosions::launch_speed`]), its range (the record's, the
/// projectile's `+0xd4`, `009a7c40`), how far it has gone (`+0x110`) and
/// for how long (`+0xd8`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Missile {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub speed: f32,
    pub range: f32,
    pub travelled: f32,
    pub age: f32,
}

impl Missile {
    pub fn launch(position: [f32; 3], direction: [f32; 3], speed: f32, range: f32) -> Missile {
        Missile {
            position,
            direction,
            speed,
            range,
            travelled: 0.0,
            age: 0.0,
        }
    }

    /// How far it goes this frame: its speed × the frame's seconds
    /// (`009bf300`), straight along its heading. Gravity on a missile
    /// (record gravity > 0 without "hitscan": run-time flag 0x40 in
    /// `009b7cc0`) isn't traced; the energy weapons' bolts have none.
    pub fn step_length(&self, seconds: f32) -> f32 {
        self.speed * seconds.max(0.0)
    }

    /// Moves it `length` along its heading, `seconds` older.
    pub fn advance(&mut self, length: f32, seconds: f32) {
        for k in 0..3 {
            self.position[k] += self.direction[k] * length;
        }
        self.travelled += length;
        self.age += seconds.max(0.0);
    }

    /// Whether it's done with (`009b8030`, after the frame's impacts):
    /// older than `age_max` ([`age_max`]), or past its range with nothing
    /// struck. The frame that takes it past its range still flies whole.
    pub fn spent(&self, age_max: f32) -> bool {
        self.age > age_max || self.travelled > self.range
    }

    /// How long it takes to go `distance` (seconds).
    pub fn time_to(&self, distance: f32) -> f32 {
        if self.speed > 0.0 {
            distance / self.speed
        } else {
            f32::INFINITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use esm::FormId;

    fn record(flags: u16, kind: u16, speed: f32) -> ProjectileRecord {
        let mut d = flags.to_le_bytes().to_vec();
        d.extend(kind.to_le_bytes());
        for v in [0.0f32, speed, 10000.0] {
            d.extend(v.to_le_bytes());
        }
        d.resize(84, 0);
        ProjectileRecord::parse(FormId(1), &d, |_| None).unwrap()
    }

    #[test]
    fn beams_and_hitscan_missiles_strike_at_once_plasma_flies() {
        // `BeamLaserProjectile` (0x8C, beam), a 9mm bullet (0x01 hitscan,
        // missile), `PlasmaProjectile` (0x20C, missile), dynamite (lobber).
        assert_eq!(delivery(&record(0x8C, 4, 10000.0)), Delivery::AtOnce);
        assert_eq!(delivery(&record(0x01, 1, 23680.0)), Delivery::AtOnce);
        assert_eq!(delivery(&record(0x20C, 1, 7500.0)), Delivery::Flies);
        assert_eq!(delivery(&record(0x806, 2, 1200.0)), Delivery::Lobbed);
        assert_eq!(delivery(&record(0x8D, 8, 12000.0)), Delivery::NotTraced);
    }

    #[test]
    fn a_plasma_bolt_flies_its_range_then_is_spent() {
        // The plasma pistol's bolt: 7500 units a second, range 10000.
        let mut m = Missile::launch([0.0; 3], [0.0, 1.0, 0.0], 7500.0, 10000.0);
        assert!((m.time_to(1500.0) - 0.2).abs() < 1e-6);
        let mut frames = 0;
        while !m.spent(90.0) {
            let l = m.step_length(0.125);
            assert_eq!(l, 937.5);
            m.advance(l, 0.125);
            frames += 1;
        }
        // Ten frames go 9375; the eleventh, flown whole, takes it past.
        assert_eq!(frames, 11);
        assert_eq!(m.position[1], 10312.5);
        // A slow one is spent by age.
        let mut slow = Missile::launch([0.0; 3], [1.0, 0.0, 0.0], 10.0, 10000.0);
        slow.advance(slow.step_length(91.0), 91.0);
        assert!(slow.spent(90.0) && slow.travelled < slow.range);
    }
}
