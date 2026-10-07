//! Flee packages (`PKDT` type 10): the `FLEE_NON_COMBAT` procedure
//! (`008ddac0`, high-process vfunc +0x7e4), the only procedure of their
//! list (32, `011a3eac`).
//!
//! Whom they flee from is the package's target (`PTDT`: process vfunc
//! +0x8c `0091aca0` → +0x810 `0091ab30` → `0091ae00`: a specific reference,
//! or the linked reference); where they flee to is the package location's
//! reference (+0x90 `00924080` → +0x814 `00923e40`: near a reference, near
//! the linked reference, or the nearest object of a kind for kinds 4 and 5).
//! "In a cell", "near the current location" and "near the editor location"
//! give no reference, so a flee package with such a location and no target
//! (Goodsprings' `GSSettlerAAMHidePackage`, `EasyPeteHideInSaloonPackage`)
//! or with neither (`GoodspringsFleePackage`) ends its procedure at once:
//! the person stops what they were doing and stands. With one, the actor
//! starts the engine's flee package (actor vfunc +0x410 `00897de0`,
//! `InitiateFlee` (Xbox PDB), type 0x16) from the target, to the
//! reference.

use esm::{FormId, LoadOrder};

use super::{linked_ref, Package};
use crate::scripting::GameState;

/// What the flee procedure does this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FleeStep {
    /// Nothing to flee from or to: the procedure is over (process vfunc
    /// +0x288 with 1: on to `DONE`).
    Done,
    /// Far enough from the target (or at the place): stop if still moving
    /// (`stop` true, vfunc +0x294). Once stopped the procedure is over only
    /// when `finish` (package flags 0x4 or 0x2); otherwise it stays, and
    /// runs again if the distance changes.
    Safe { stop: bool, finish: bool },
    /// Flee: the engine's flee package from `from`, to `to`, with the
    /// target distance (`-1` without a target) and the location radius.
    Run {
        from: Option<FormId>,
        to: Option<FormId>,
        distance: f32,
        radius: f32,
    },
}

/// Package general flags tested by the flee procedure (`0067efd0` 0x4,
/// `008b1ff0` 0x2). Named "must complete" and "must reach location" by the
/// editor's terms (inferred; the code only tests the bits).
pub const FINISH_FLAGS: u32 = 0x4 | 0x2;

/// The flee procedure's choice. `from`/`to` are the resolved references
/// with their distance from the fleeing actor now; `target_distance` the
/// package target's value (`PTDT` i32 at 8) when it has a target;
/// `radius` the location's radius (`00676280`); `moving` whether the
/// actor's mover is still on a path (`008b3bb0` false).
// Translated from 008ddac0 (decompiled, FalloutNV.exe 1.4.0.525).
pub fn step(
    from: Option<(FormId, f32)>,
    to: Option<(FormId, f32)>,
    target_distance: Option<i32>,
    radius: f32,
    flags: u32,
    moving: bool,
) -> FleeStep {
    if from.is_none() && to.is_none() {
        return FleeStep::Done;
    }
    // The target's value as a float, -1.0 (`01012054`) without a target.
    let distance = target_distance.map_or(-1.0, |d| d as f32);
    let safe = match to {
        None => from.is_some_and(|(_, d)| distance <= d),
        Some((_, d)) => d <= radius,
    };
    if safe {
        return FleeStep::Safe {
            stop: moving,
            finish: !moving && flags & FINISH_FLAGS != 0,
        };
    }
    FleeStep::Run {
        from: from.map(|f| f.0),
        to: to.map(|t| t.0),
        distance,
        radius,
    }
}

/// Whom a flee package flees from: its target (`PTDT`), a specific
/// reference (kind 0) or the actor's linked reference (kind 3). Object and
/// object-type targets (kinds 1, 2: a search, `0091ae00`) aren't resolved
/// here (`None`). Dead targets are skipped (the target's vfunc +0x22c,
/// `IsDead` (Xbox PDB, slot +0x228 there)).
pub fn flee_from(
    order: &LoadOrder,
    state: &GameState,
    actor: FormId,
    package: &Package,
) -> Option<FormId> {
    let (kind, form, _) = package.target?;
    let who = match kind {
        0 => Some(form).filter(|f| f.0 != 0)?,
        3 => linked_ref(order, actor)?,
        _ => return None,
    };
    (!state.dead.contains(&who)).then_some(who)
}

/// Where a flee package flees to: its location's reference (`00923e40`):
/// near a reference (kind 0), near the linked reference (6). The nearest
/// object (kinds 4, 5) is a search not done here; the others give none.
pub fn flee_to(order: &LoadOrder, actor: FormId, package: &Package) -> Option<FormId> {
    let loc = package.location?;
    match loc.kind {
        0 => Some(loc.form).filter(|f| f.0 != 0),
        6 => linked_ref(order, actor),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_to_flee_from_or_to_ends_at_once() {
        assert_eq!(
            step(None, None, None, 0.0, 0x0C20_3002, false),
            FleeStep::Done
        );
        assert_eq!(step(None, None, Some(500), 0.0, 0, true), FleeStep::Done);
    }

    #[test]
    fn fleeing_from_a_target_until_its_distance() {
        let t = FormId(0x10);
        // Nearer than the target's value: run.
        assert_eq!(
            step(Some((t, 300.0)), None, Some(1000), 0.0, 0, false),
            FleeStep::Run {
                from: Some(t),
                to: None,
                distance: 1000.0,
                radius: 0.0
            }
        );
        // At or beyond it: safe; stop if moving, finish only with the flags.
        assert_eq!(
            step(Some((t, 1000.0)), None, Some(1000), 0.0, 0, true),
            FleeStep::Safe {
                stop: true,
                finish: false
            }
        );
        assert_eq!(
            step(Some((t, 1200.0)), None, Some(1000), 0.0, 0, false),
            FleeStep::Safe {
                stop: false,
                finish: false
            }
        );
        assert_eq!(
            step(Some((t, 1200.0)), None, Some(1000), 0.0, 0x4, false),
            FleeStep::Safe {
                stop: false,
                finish: true
            }
        );
        assert_eq!(
            step(Some((t, 1200.0)), None, Some(1000), 0.0, 0x2, false),
            FleeStep::Safe {
                stop: false,
                finish: true
            }
        );
    }

    #[test]
    fn fleeing_to_a_place_until_within_its_radius() {
        let place = FormId(0x20);
        let t = FormId(0x10);
        // A place: its radius decides, not the target's distance.
        assert_eq!(
            step(
                Some((t, 5000.0)),
                Some((place, 300.0)),
                Some(100),
                256.0,
                0,
                false
            ),
            FleeStep::Run {
                from: Some(t),
                to: Some(place),
                distance: 100.0,
                radius: 256.0
            }
        );
        assert_eq!(
            step(None, Some((place, 256.0)), None, 256.0, 0x2, false),
            FleeStep::Safe {
                stop: false,
                finish: true
            }
        );
    }
}
