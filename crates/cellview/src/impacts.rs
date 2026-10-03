//! Impact effect models (`world::impacts`): how long the game plays one.
//!
//! **How long** ([`animation_time`], the game's `00689840`, used because
//! the game makes these effects with flag 4, `00689310`): the longest
//! (stop − start) of the time controllers on the model's nodes and shapes,
//! looked for through every node's children; none gives `None` (the game's
//! −1), and the effect then lasts what it was given
//! (`world::impacts::effect_lifetime`). The effects themselves aren't drawn
//! yet: that needs their controllers, billboard nodes and particles run as
//! the game runs them.

fn le_i32(b: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn le_f32(b: &[u8], at: usize) -> Option<f32> {
    Some(f32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The controllers an object has: an `NiObjectNET`'s controller (after its
/// name and extra data list), then each controller's next (a
/// `NiTimeController` starts with it: next (4), flags (2), frequency,
/// phase, start time, stop time).
fn controller_times(nif: &nif::Nif, object: usize, longest: &mut Option<f32>) {
    let bytes = nif.block_bytes(object);
    let Some(count) = le_i32(bytes, 4) else {
        return;
    };
    let Ok(count) = usize::try_from(count) else {
        return;
    };
    let mut next = le_i32(bytes, 8 + 4 * count).unwrap_or(-1);
    let mut steps = 0;
    while let Ok(c) = usize::try_from(next) {
        if c >= nif.blocks().len() || steps > 64 {
            break;
        }
        steps += 1;
        let b = nif.block_bytes(c);
        let (Some(start), Some(stop)) = (le_f32(b, 14), le_f32(b, 18)) else {
            break;
        };
        let t = stop - start;
        if t.is_finite() {
            *longest = Some(longest.map_or(t, |l| l.max(t)));
        }
        next = le_i32(b, 0).unwrap_or(-1);
    }
}

/// How long a model animates (see the module notes).
pub fn animation_time(nif: &nif::Nif) -> Option<f32> {
    fn visit(nif: &nif::Nif, index: usize, depth: usize, longest: &mut Option<f32>) {
        if depth > 64 || index >= nif.blocks().len() {
            return;
        }
        controller_times(nif, index, longest);
        if let Ok(nif::Block::Node(node)) = nif.block(index) {
            for child in node.children {
                if let Ok(c) = usize::try_from(child) {
                    visit(nif, c, depth + 1, longest);
                }
            }
        }
    }
    let mut longest = None;
    for &root in nif.roots() {
        if let Ok(r) = usize::try_from(root) {
            visit(nif, r, 0, &mut longest);
        }
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A model whose root node carries the controllers playing `times`
    /// but the last, which is on its child shape.
    fn animated_nif(times: &[(f32, f32)]) -> nif::Nif {
        nif::Nif::parse(testdata::impacts::animated_effect_nif(times)).unwrap()
    }

    #[test]
    fn an_effect_lasts_its_longest_controller() {
        // Root 0.5–2.0 (1.5 s), the shape 0–3.25: the shape's, found
        // through the root's children.
        let nif = animated_nif(&[(0.5, 2.0), (0.0, 3.25)]);
        assert_eq!(animation_time(&nif), Some(3.25));
        // A chain on the root: 1, then 4 (the longest), then the shape 2.
        let chain = animated_nif(&[(0.0, 1.0), (1.0, 5.0), (0.0, 2.0)]);
        assert_eq!(animation_time(&chain), Some(4.0));
        let still = animated_nif(&[]);
        assert_eq!(animation_time(&still), None);
    }
}
