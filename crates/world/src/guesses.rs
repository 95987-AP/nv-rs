//! A switch for behaviour that isn't traced yet ([G] in the code and the
//! topic files), off by default so the base-game routes run only what was
//! read from the game. The Dead Money contributor's untraced rules (a
//! teammate with nothing to do follows the player, teammates come along to
//! another place) need it on: the viewer turns it on with `NV_GUESSES=1`.
//! See `docs/CONTRIB_PLAYCON.md`.

use std::sync::atomic::{AtomicBool, Ordering};

static ON: AtomicBool = AtomicBool::new(false);

/// Whether untraced behaviour runs.
pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Turns untraced behaviour on or off (process-wide).
pub fn set(on: bool) {
    ON.store(on, Ordering::Relaxed);
}
