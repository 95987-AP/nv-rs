//! The player's control bindings as the game reads them: the INI's
//! `[Controls]` (`FalloutPrefs.ini` keeps them), one value per control,
//! eight hex digits: the keyboard's DirectInput key number in the second
//! byte from the left, the mouse button in the third (0 left, 1 right, 2
//! middle; FF none), the controller's in the last. `Block=00380110`: Left
//! Alt or the right mouse button. When a control isn't in the INI, the
//! default given here is this install's value (the exe's own defaults
//! aren't traced).

use bevy::prelude::*;

/// A control's keyboard key and mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Binding {
    pub key: Option<KeyCode>,
    pub mouse: Option<MouseButton>,
}

impl Binding {
    /// From the INI's value.
    pub fn from_value(value: u32) -> Binding {
        Binding {
            key: crate::lockpick::scan_code_key((value >> 16) & 0xFF),
            mouse: match (value >> 8) & 0xFF {
                0 => Some(MouseButton::Left),
                1 => Some(MouseButton::Right),
                2 => Some(MouseButton::Middle),
                _ => None,
            },
        }
    }

    pub fn pressed(&self, keys: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> bool {
        self.key.is_some_and(|k| keys.pressed(k)) || self.mouse.is_some_and(|m| mouse.pressed(m))
    }

    pub fn just_pressed(
        &self,
        keys: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.key.is_some_and(|k| keys.just_pressed(k))
            || self.mouse.is_some_and(|m| mouse.just_pressed(m))
    }
}

/// The controls the player's actions here read.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Controls {
    /// Control 6, "Block" in the INI: aim (iron sights) or block.
    pub aim: Binding,
    /// Control 8, "Crouch/Sneak".
    pub sneak: Binding,
    /// Control 10, "Always Run": toggles running by default.
    pub always_run: Binding,
    /// Control 11, "Auto Move".
    pub auto_move: Binding,
    /// `bAlwaysRunByDefault` (the player's +0x651 at the start).
    pub always_run_default: bool,
}

impl Default for Controls {
    fn default() -> Self {
        Controls {
            aim: Binding::from_value(0x0038_0110),
            sneak: Binding::from_value(0x001D_FF08),
            always_run: Binding::from_value(0x003A_FFFF),
            auto_move: Binding::from_value(0x0010_FFFF),
            always_run_default: true,
        }
    }
}

impl Controls {
    /// From the game's INI files, this install's values where they're
    /// silent.
    pub fn read(settings: &assets::IniSettings) -> Controls {
        let base = Controls::default();
        let binding = |name: &str, default: Binding| {
            settings
                .get("Controls", name)
                .and_then(|v| u32::from_str_radix(v.trim(), 16).ok())
                .map_or(default, Binding::from_value)
        };
        Controls {
            aim: binding("Block", base.aim),
            sneak: binding("Crouch/Sneak", base.sneak),
            always_run: binding("Always Run", base.always_run),
            auto_move: binding("Auto Move", base.auto_move),
            always_run_default: settings
                .get("Controls", "bAlwaysRunByDefault")
                .map_or(base.always_run_default, |v| v.trim() != "0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_are_read_as_the_ini_writes_them() {
        // This install's Block: Left Alt and the right mouse button.
        let aim = Binding::from_value(0x0038_0110);
        assert_eq!(aim.key, Some(KeyCode::AltLeft));
        assert_eq!(aim.mouse, Some(MouseButton::Right));
        let sneak = Binding::from_value(0x001D_FF08);
        assert_eq!((sneak.key, sneak.mouse), (Some(KeyCode::ControlLeft), None));
        assert_eq!(
            Binding::from_value(0x003A_FFFF).key,
            Some(KeyCode::CapsLock)
        );
        let mut ini = assets::IniSettings::default();
        ini.add("[Controls]\nCrouch/Sneak=002EFF08\nbAlwaysRunByDefault=0\n");
        let c = Controls::read(&ini);
        assert_eq!(c.sneak.key, Some(KeyCode::KeyC));
        assert!(!c.always_run_default);
        assert_eq!(c.aim, Controls::default().aim);
    }
}
