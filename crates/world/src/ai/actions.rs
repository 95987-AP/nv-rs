//! Package begin/end/change actions, read from `POBA`, `POEA`, `POCA`.
//!
//! FalloutNV.exe's package loader (`00673a40`) sends each marker to
//! `0067dd20`: `INAM` is the idle, the embedded script follows, and `TNAM`
//! is the topic and terminates the action. The three actions are independent.
//! For example, VCG01PlayerSection1 begins with the situp idle but changes
//! with the bedsit idle. These records describe actions; dispatch timing and
//! player camera animation are not implemented by this reader.

use esm::{FormId, Record, Subrecord};

/// One package action. Empty actions in game files have null INAM/TNAM and
/// an empty script header; retaining that header does not imply a script runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageAction {
    pub idle: Option<FormId>,
    pub topic: Option<FormId>,
    /// Embedded script fields in file order. SCRO form references are
    /// remapped to the load order; SCRV variable indices and bytecode remain
    /// unchanged. Keep headers, locals and compiled code even without SCTX.
    pub script: Vec<Subrecord>,
}

impl PackageAction {
    pub fn source(&self) -> Option<String> {
        self.script
            .iter()
            .find(|s| s.kind.as_bytes() == b"SCTX")
            .map(Subrecord::zstring)
    }

    pub fn bytecode(&self) -> &[u8] {
        self.script
            .iter()
            .find(|s| s.kind.as_bytes() == b"SCDA")
            .map_or(&[], |s| s.data.as_slice())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageActions {
    pub begin: PackageAction,
    pub end: PackageAction,
    pub change: PackageAction,
}

impl PackageActions {
    pub(super) fn read(record: &Record, global: impl Fn(FormId) -> FormId) -> Self {
        let mut actions = Self::default();
        let mut active = None;
        let form = |data: &[u8]| {
            let raw = FormId(u32::from_le_bytes(data.get(..4)?.try_into().ok()?));
            // Null must stay null even for a plugin at a nonzero load index.
            (raw.0 != 0).then(|| global(raw))
        };
        for sub in &record.subrecords {
            match sub.kind.as_bytes() {
                b"POBA" | b"POEA" | b"POCA" => {
                    let action = match sub.kind.as_bytes() {
                        b"POBA" => &mut actions.begin,
                        b"POEA" => &mut actions.end,
                        _ => &mut actions.change,
                    };
                    // The original loader clears the action on entry.
                    *action = PackageAction::default();
                    active = Some(action);
                }
                b"TNAM" => {
                    if let Some(action) = active.take() {
                        action.topic = form(&sub.data);
                    }
                }
                b"INAM" => {
                    if let Some(action) = active.as_deref_mut() {
                        action.idle = form(&sub.data);
                    }
                }
                b"SCHR" | b"SCDA" | b"SCTX" | b"SLSD" | b"SCVR" | b"SCRO" | b"SCRV" => {
                    if let Some(action) = active.as_deref_mut() {
                        let mut field = sub.clone();
                        if sub.kind.as_bytes() == b"SCRO" && sub.data.len() >= 4 {
                            let id = form(&sub.data).unwrap_or(FormId(0));
                            field.data[..4].copy_from_slice(&id.0.to_le_bytes());
                        }
                        action.script.push(field);
                    }
                }
                _ => {}
            }
        }
        actions
    }
}
