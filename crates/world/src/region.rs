//! Regions (`REGN`): parts of a worldspace drawn as outlines, which
//! weathers, sounds and scripts (`IsPlayerInRegion`) go by.
//!
//! A region: `WNAM` its worldspace, then its areas, each `RPLI` (the edge
//! fall-off, u32) and `RPLD` (the outline's points, x and y f32 pairs, in
//! the worldspace's units). Checked on `vMapVictorInNovacRegion`: one area
//! of four points around Novac (x 32,347 to 50,711, y -44,569 to -24,964).

use esm::{FormId, FourCC, LoadOrder};

use crate::cell::{le_f32, le_u32};

const REGN: FourCC = FourCC::new(b"REGN");
const WNAM: FourCC = FourCC::new(b"WNAM");
const RPLD: FourCC = FourCC::new(b"RPLD");

#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub form_id: FormId,
    pub world: Option<FormId>,
    /// Each area's outline.
    pub areas: Vec<Vec<[f32; 2]>>,
}

impl Region {
    pub fn load(order: &LoadOrder, id: FormId) -> Option<Region> {
        let rr = order.get(id).filter(|r| r.entry.header.kind == REGN)?;
        let record = rr.record().ok()?;
        Some(Region {
            form_id: id,
            world: record
                .get(WNAM)
                .filter(|s| s.data.len() >= 4)
                .map(|s| rr.plugin.to_global(FormId(le_u32(&s.data, 0)))),
            areas: record
                .get_all(RPLD)
                .map(|s| {
                    s.data
                        .chunks_exact(8)
                        .map(|c| [le_f32(c, 0), le_f32(c, 4)])
                        .collect()
                })
                .collect(),
        })
    }

    /// Whether a point (x, y) is inside one of its areas (even-odd rule).
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.areas.iter().any(|area| {
            let mut inside = false;
            for (i, a) in area.iter().enumerate() {
                let b = area[(i + area.len() - 1) % area.len()];
                if (a[1] > y) != (b[1] > y) && x < (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) + a[0]
                {
                    inside = !inside;
                }
            }
            inside
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_inside_an_outline() {
        let r = Region {
            form_id: FormId(1),
            world: None,
            // An L: the square 0..20 without its top-right quarter.
            areas: vec![vec![
                [0.0, 0.0],
                [20.0, 0.0],
                [20.0, 10.0],
                [10.0, 10.0],
                [10.0, 20.0],
                [0.0, 20.0],
            ]],
        };
        assert!(r.contains(5.0, 5.0));
        assert!(r.contains(15.0, 5.0));
        assert!(r.contains(5.0, 15.0));
        assert!(!r.contains(15.0, 15.0));
        assert!(!r.contains(-1.0, 5.0));
    }
}
