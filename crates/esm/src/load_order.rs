//! Combining the game's master file, DLC and mods into one view.
//!
//! Every plugin numbers its form IDs relative to its own master list: the top
//! byte is an index into that list, and one past the end means "this file".
//! A load order renumbers them so the top byte is the plugin's position in
//! the load order (what the in-game console shows), and resolves overrides:
//! when several plugins contain the same record, the one loaded last wins.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::SystemTime;

use crate::error::{Error, Result};
use crate::plugin::{Plugin, RecordEntry};
use crate::record::Record;
use crate::text;
use crate::types::{sig, FormId, FourCC};

/// The game's main master file, always loaded first.
pub const MAIN_MASTER: &str = "FalloutNV.esm";

/// The official master files: the base game, the five DLCs and the four
/// Courier's Stash packs.
pub const OFFICIAL_FILES: [&str; 10] = [
    "FalloutNV.esm",
    "DeadMoney.esm",
    "HonestHearts.esm",
    "OldWorldBlues.esm",
    "LonesomeRoad.esm",
    "GunRunnersArsenal.esm",
    "ClassicPack.esm",
    "MercenaryPack.esm",
    "TribalPack.esm",
    "CaravanPack.esm",
];

/// Record types for objects placed in cells: references to base objects,
/// NPCs, creatures, and placed grenades and missiles.
pub const PLACED_TYPES: [FourCC; 5] = [sig::REFR, sig::ACHR, sig::ACRE, sig::PGRE, sig::PMIS];

/// Form IDs have one byte for the load-order index, and 0xFF is reserved
/// for objects created while playing.
const MAX_PLUGINS: usize = 255;

/// Which plugins are active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivePlugins {
    /// FalloutNV.esm plus whichever official DLC and pack files are present.
    OfficialOnly,
    /// The names listed in a `plugins.txt` file. FalloutNV.esm is always
    /// loaded, listed or not.
    List(Vec<String>),
}

/// One plugin in a load order.
pub struct LoadedPlugin {
    /// File name, as found on disk.
    pub name: String,
    pub path: Option<PathBuf>,
    pub plugin: Plugin,
    /// Position in the load order: the top byte of this plugin's new form IDs.
    pub load_index: u8,
    /// Maps a local mod index (position in this plugin's master list, or the
    /// list's length for the plugin itself) to a load-order index.
    index_map: Vec<u8>,
}

impl LoadedPlugin {
    /// Converts a form ID as stored in this plugin to its load-order form ID.
    /// Indexes past the end of the master list refer to the plugin itself,
    /// matching the game's behaviour.
    pub fn to_global(&self, local: FormId) -> FormId {
        let i = usize::from(local.mod_index()).min(self.index_map.len() - 1);
        FormId((u32::from(self.index_map[i]) << 24) | local.object_id())
    }
}

/// A reference to one version of a record inside a load order.
#[derive(Clone, Copy)]
pub struct RecordRef<'a> {
    pub plugin_index: usize,
    pub plugin: &'a LoadedPlugin,
    pub entry: &'a RecordEntry,
    /// The load-order form ID.
    pub form_id: FormId,
}

impl RecordRef<'_> {
    pub fn record(&self) -> Result<Record> {
        self.plugin.plugin.record(self.entry)
    }

    pub fn editor_id(&self) -> Result<Option<String>> {
        self.plugin.plugin.editor_id_of(self.entry)
    }
}

/// (plugin index, record index within that plugin)
type Slot = (u16, u32);

/// Several plugins combined, with overrides resolved.
pub struct LoadOrder {
    plugins: Vec<LoadedPlugin>,
    /// File name for each load-order index.
    slot_names: Vec<String>,
    /// The winning (last-loaded) version of every record.
    winners: HashMap<FormId, Slot>,
    /// Every version, in load order, of records defined more than once.
    history: HashMap<FormId, Vec<Slot>>,
    warnings: Vec<String>,
    single: bool,
    /// Every record version inside each cell's child groups, by cell; built
    /// the first time it's needed.
    by_cell: OnceLock<HashMap<FormId, Vec<Slot>>>,
    /// Every dialogue line (`INFO`) version by topic; built when needed.
    by_topic: OnceLock<HashMap<FormId, Vec<Slot>>>,
    /// Every winning record by lower-case editor ID; built when needed.
    by_editor_id: OnceLock<HashMap<String, FormId>>,
}

impl LoadOrder {
    /// Wraps one plugin on its own, without loading its masters. Form IDs
    /// keep their local numbering, and records it overrides show their
    /// master's name as their origin.
    pub fn single(name: impl Into<String>, path: Option<PathBuf>, plugin: Plugin) -> Result<Self> {
        let name = name.into();
        let masters = plugin.header().masters.clone();
        if masters.len() >= MAX_PLUGINS {
            return Err(Error::TooManyPlugins {
                count: masters.len() + 1,
            });
        }
        let load_index = masters.len() as u8;
        let mut slot_names = masters;
        slot_names.push(name.clone());
        let loaded = LoadedPlugin {
            name,
            path,
            plugin,
            load_index,
            index_map: (0..=load_index).collect(),
        };
        Ok(Self::build(vec![loaded], slot_names, Vec::new(), true))
    }

    /// Combines plugins already in load order. Every plugin's masters must
    /// appear earlier in the list.
    pub fn from_plugins(list: Vec<(String, Option<PathBuf>, Plugin)>) -> Result<Self> {
        if list.len() > MAX_PLUGINS {
            return Err(Error::TooManyPlugins { count: list.len() });
        }
        let mut loaded = Vec::with_capacity(list.len());
        let mut slot_names: Vec<String> = Vec::with_capacity(list.len());
        for (i, (name, path, plugin)) in list.into_iter().enumerate() {
            let mut index_map = Vec::with_capacity(plugin.header().masters.len() + 1);
            for master in &plugin.header().masters {
                let position = slot_names
                    .iter()
                    .position(|loaded_name| loaded_name.eq_ignore_ascii_case(master))
                    .ok_or_else(|| Error::MissingMaster {
                        plugin: name.clone(),
                        master: master.clone(),
                    })?;
                index_map.push(position as u8);
            }
            index_map.push(i as u8);
            slot_names.push(name.clone());
            loaded.push(LoadedPlugin {
                name,
                path,
                plugin,
                load_index: i as u8,
                index_map,
            });
        }
        Ok(Self::build(loaded, slot_names, Vec::new(), false))
    }

    /// Loads the active plugins from a game's `Data` folder, ordered the way
    /// New Vegas orders them: FalloutNV.esm first, then files with the
    /// master flag, then the rest, each group by file modification time.
    pub fn from_data_dir(data_dir: impl AsRef<Path>, active: &ActivePlugins) -> Result<Self> {
        let dir = data_dir.as_ref();
        let files = plugin_files_in(dir)?;
        let find = |wanted: &str| {
            files
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(wanted))
        };

        let main = find(MAIN_MASTER).ok_or_else(|| {
            Error::DataDir(format!("there is no {MAIN_MASTER} in {}", dir.display()))
        })?;
        let mut chosen = vec![main.clone()];
        let mut warnings = Vec::new();
        let wanted: Vec<String> = match active {
            ActivePlugins::OfficialOnly => {
                OFFICIAL_FILES[1..].iter().map(|s| s.to_string()).collect()
            }
            ActivePlugins::List(names) => names.clone(),
        };
        for name in wanted {
            if chosen.iter().any(|(n, _)| n.eq_ignore_ascii_case(&name)) {
                continue;
            }
            match find(&name) {
                Some(file) => chosen.push(file.clone()),
                None if matches!(active, ActivePlugins::List(_)) => warnings.push(format!(
                    "{name} is listed as active but isn't in the Data folder, so it was skipped"
                )),
                None => {}
            }
        }

        let mut loaded = Vec::with_capacity(chosen.len());
        for (name, path) in chosen {
            let plugin = Plugin::open(&path).map_err(|source| Error::InPlugin {
                name: name.clone(),
                source: Box::new(source),
            })?;
            let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
            loaded.push((name, path, plugin, modified));
        }
        loaded.sort_by_cached_key(|(name, _, plugin, modified)| sort_key(name, plugin, *modified));

        let mut order = Self::from_plugins(
            loaded
                .into_iter()
                .map(|(name, path, plugin, _)| (name, Some(path), plugin))
                .collect(),
        )?;
        order.warnings.extend(warnings);
        Ok(order)
    }

    fn build(
        plugins: Vec<LoadedPlugin>,
        slot_names: Vec<String>,
        warnings: Vec<String>,
        single: bool,
    ) -> Self {
        let total: usize = plugins.iter().map(|p| p.plugin.records().len()).sum();
        let mut winners: HashMap<FormId, Slot> = HashMap::with_capacity(total);
        let mut history: HashMap<FormId, Vec<Slot>> = HashMap::new();
        for (pi, p) in plugins.iter().enumerate() {
            for (ri, entry) in p.plugin.records().iter().enumerate() {
                let id = p.to_global(entry.header.form_id);
                let slot = (pi as u16, ri as u32);
                if let Some(previous) = winners.insert(id, slot) {
                    history
                        .entry(id)
                        .or_insert_with(|| vec![previous])
                        .push(slot);
                }
            }
        }
        Self {
            plugins,
            slot_names,
            winners,
            history,
            warnings,
            single,
            by_cell: OnceLock::new(),
            by_topic: OnceLock::new(),
            by_editor_id: OnceLock::new(),
        }
    }

    pub fn plugins(&self) -> &[LoadedPlugin] {
        &self.plugins
    }

    /// True when built with [`LoadOrder::single`].
    pub fn is_single(&self) -> bool {
        self.single
    }

    /// Non-fatal problems found while building the load order.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The file that a load-order index refers to.
    pub fn slot_name(&self, mod_index: u8) -> Option<&str> {
        self.slot_names
            .get(usize::from(mod_index))
            .map(String::as_str)
    }

    /// Number of distinct records after overrides.
    pub fn len(&self) -> usize {
        self.winners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.winners.is_empty()
    }

    fn at(&self, (pi, ri): Slot, form_id: FormId) -> RecordRef<'_> {
        let plugin = &self.plugins[usize::from(pi)];
        RecordRef {
            plugin_index: usize::from(pi),
            plugin,
            entry: &plugin.plugin.records()[ri as usize],
            form_id,
        }
    }

    fn is_winner(&self, form_id: FormId, slot: Slot) -> bool {
        self.winners.get(&form_id) == Some(&slot)
    }

    /// The winning version of a record, by load-order form ID.
    pub fn get(&self, form_id: FormId) -> Option<RecordRef<'_>> {
        self.winners
            .get(&form_id)
            .map(|&slot| self.at(slot, form_id))
    }

    /// Every version of a record in load order; the last one wins.
    pub fn versions(&self, form_id: FormId) -> Vec<RecordRef<'_>> {
        match self.history.get(&form_id) {
            Some(slots) => slots.iter().map(|&s| self.at(s, form_id)).collect(),
            None => self.get(form_id).into_iter().collect(),
        }
    }

    /// The winning version of every record of one type, in load order.
    pub fn records_of_type(&self, kind: FourCC) -> impl Iterator<Item = RecordRef<'_>> + '_ {
        self.plugins.iter().enumerate().flat_map(move |(pi, p)| {
            p.plugin
                .record_indices_of_type(kind)
                .iter()
                .filter_map(move |&ri| {
                    let entry = &p.plugin.records()[ri];
                    let id = p.to_global(entry.header.form_id);
                    let slot = (pi as u16, ri as u32);
                    self.is_winner(id, slot).then(|| self.at(slot, id))
                })
        })
    }

    pub fn count_of_type(&self, kind: FourCC) -> usize {
        self.records_of_type(kind).count()
    }

    /// Record counts per type after overrides, most common first.
    pub fn type_counts(&self) -> Vec<(FourCC, usize)> {
        let mut counts: HashMap<FourCC, usize> = HashMap::new();
        for &(pi, ri) in self.winners.values() {
            let kind = self.plugins[usize::from(pi)].plugin.records()[ri as usize]
                .header
                .kind;
            *counts.entry(kind).or_default() += 1;
        }
        let mut counts: Vec<_> = counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        counts
    }

    /// How many records a plugin adds, and how many it overrides.
    pub fn plugin_stats(&self, plugin_index: usize) -> (usize, usize) {
        let p = &self.plugins[plugin_index];
        let new = p
            .plugin
            .records()
            .iter()
            .filter(|e| p.to_global(e.header.form_id).mod_index() == p.load_index)
            .count();
        (new, p.plugin.records().len() - new)
    }

    /// The cell a record version sits in, as a load-order form ID.
    pub fn cell_of(&self, record: &RecordRef<'_>) -> Option<FormId> {
        record.entry.cell.map(|c| record.plugin.to_global(c))
    }

    /// The winning versions of a topic's lines (`INFO`), in load order and
    /// file order. A line a later plugin moves to another topic belongs
    /// there.
    pub fn in_topic(&self, topic: FormId) -> Vec<RecordRef<'_>> {
        let index = self.by_topic.get_or_init(|| {
            let mut index: HashMap<FormId, Vec<Slot>> = HashMap::new();
            for (pi, p) in self.plugins.iter().enumerate() {
                for (ri, entry) in p.plugin.records().iter().enumerate() {
                    if let Some(t) = entry.topic {
                        index
                            .entry(p.to_global(t))
                            .or_default()
                            .push((pi as u16, ri as u32));
                    }
                }
            }
            index
        });
        let Some(slots) = index.get(&topic) else {
            return Vec::new();
        };
        slots
            .iter()
            .filter_map(|&slot| {
                let (pi, ri) = slot;
                let p = &self.plugins[usize::from(pi)];
                let id = p.to_global(p.plugin.records()[ri as usize].header.form_id);
                self.is_winner(id, slot).then(|| self.at(slot, id))
            })
            .collect()
    }

    /// The dialogue topic a line (`INFO`) belongs to, as a load-order form
    /// ID.
    pub fn topic_of(&self, record: &RecordRef<'_>) -> Option<FormId> {
        record.entry.topic.map(|t| record.plugin.to_global(t))
    }

    /// The worldspace a record version sits in, as a load-order form ID.
    pub fn world_of(&self, record: &RecordRef<'_>) -> Option<FormId> {
        record.entry.world.map(|w| record.plugin.to_global(w))
    }

    /// The winning versions of every object placed in a cell (references,
    /// actors, grenades, missiles), in load order. A reference that a later
    /// plugin moves to another cell belongs to the cell it was moved to.
    pub fn references_in_cell(&self, cell: FormId) -> Vec<RecordRef<'_>> {
        let mut found = self.in_cell(cell);
        found.retain(|r| PLACED_TYPES.contains(&r.entry.header.kind));
        // By type, then load order, as the records come out of the files.
        found.sort_by_key(|r| PLACED_TYPES.iter().position(|&k| k == r.entry.header.kind));
        found
    }

    /// The winning versions of every record in a cell's child groups:
    /// placed objects, terrain (`LAND`), navmeshes and so on, in load
    /// order. A record a later plugin moves to another cell belongs there.
    pub fn in_cell(&self, cell: FormId) -> Vec<RecordRef<'_>> {
        let index = self.by_cell.get_or_init(|| {
            let mut index: HashMap<FormId, Vec<Slot>> = HashMap::new();
            for (pi, p) in self.plugins.iter().enumerate() {
                for (ri, entry) in p.plugin.records().iter().enumerate() {
                    if let Some(c) = entry.cell {
                        index
                            .entry(p.to_global(c))
                            .or_default()
                            .push((pi as u16, ri as u32));
                    }
                }
            }
            index
        });
        let Some(slots) = index.get(&cell) else {
            return Vec::new();
        };
        slots
            .iter()
            .filter_map(|&slot| {
                let (pi, ri) = slot;
                let p = &self.plugins[usize::from(pi)];
                let id = p.to_global(p.plugin.records()[ri as usize].header.form_id);
                self.is_winner(id, slot).then(|| self.at(slot, id))
            })
            .collect()
    }

    /// How many placed objects each cell holds, counting winning versions.
    pub fn reference_counts(&self) -> HashMap<FormId, usize> {
        let mut counts = HashMap::new();
        for kind in PLACED_TYPES {
            for r in self.records_of_type(kind) {
                if let Some(cell) = self.cell_of(&r) {
                    *counts.entry(cell).or_default() += 1;
                }
            }
        }
        counts
    }

    /// The winning record with an editor ID (case-insensitive), from an
    /// index of every record built the first time it's asked (a few
    /// seconds for the whole game). When two records share an editor ID,
    /// the later-loaded one wins.
    pub fn form_by_editor_id(&self, editor_id: &str) -> Option<FormId> {
        let index = self.by_editor_id.get_or_init(|| {
            let mut index = HashMap::new();
            for (pi, p) in self.plugins.iter().enumerate() {
                for (ri, entry) in p.plugin.records().iter().enumerate() {
                    let id = p.to_global(entry.header.form_id);
                    if !self.is_winner(id, (pi as u16, ri as u32)) {
                        continue;
                    }
                    if let Ok(Some(edid)) = p.plugin.editor_id_of(entry) {
                        index.insert(edid.to_ascii_lowercase(), id);
                    }
                }
            }
            index
        });
        index.get(&editor_id.to_ascii_lowercase()).copied()
    }

    /// Finds the winning record with an editor ID (case-insensitive),
    /// optionally of one type. Later plugins are searched first.
    pub fn find_by_editor_id(
        &self,
        editor_id: &str,
        kind: Option<FourCC>,
    ) -> Result<Option<RecordRef<'_>>> {
        for (pi, p) in self.plugins.iter().enumerate().rev() {
            let indices: Box<dyn Iterator<Item = usize>> = match kind {
                Some(kind) => Box::new(p.plugin.record_indices_of_type(kind).iter().copied()),
                None => Box::new(0..p.plugin.records().len()),
            };
            for ri in indices {
                let entry = &p.plugin.records()[ri];
                let id = p.to_global(entry.header.form_id);
                let slot = (pi as u16, ri as u32);
                if !self.is_winner(id, slot) {
                    continue;
                }
                if let Some(found) = p.plugin.editor_id_of(entry)? {
                    if found.eq_ignore_ascii_case(editor_id) {
                        return Ok(Some(self.at(slot, id)));
                    }
                }
            }
        }
        Ok(None)
    }
}

fn sort_key(
    name: &str,
    plugin: &Plugin,
    modified: Option<SystemTime>,
) -> (bool, bool, Option<SystemTime>, String) {
    (
        !name.eq_ignore_ascii_case(MAIN_MASTER),
        !plugin.header().is_master,
        modified,
        name.to_ascii_lowercase(),
    )
}

/// `.esm` and `.esp` files in a folder, sorted by name.
fn plugin_files_in(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| Error::DataDir(format!("could not read the folder {}: {e}", dir.display())))?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let lower = name.to_ascii_lowercase();
        if (lower.ends_with(".esm") || lower.ends_with(".esp")) && entry.path().is_file() {
            files.push((name, entry.path()));
        }
    }
    files.sort_by_key(|(name, _)| name.to_ascii_lowercase());
    Ok(files)
}

/// Parses a `plugins.txt` active-plugin list: one file name per line, with
/// blank lines and `#` comments ignored. A leading `*` (used by later games
/// to mark active entries) is tolerated.
pub fn parse_plugins_txt(bytes: &[u8]) -> Vec<String> {
    text::decode_cp1252(bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.trim_start_matches('*').trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Where the game keeps its active-plugin list on Windows:
/// `%LOCALAPPDATA%\FalloutNV\plugins.txt`. Returns `None` if it doesn't exist.
pub fn default_plugins_txt() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let path = Path::new(&base).join("FalloutNV").join("plugins.txt");
    path.is_file().then_some(path)
}
