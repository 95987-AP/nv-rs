//! `fallout shared/tessaveloadgame.cpp` (Xbox PDB source unit), subsystem `fallout shared`: its functions in
//! FalloutNV.exe 1.4.0.525, translated (docs/ENGINE_CRATE.md).
//!
//! The unit has 192 functions (`ledger queue "fallout shared/tessaveloadgame.cpp"`);
//! it is translated in address order, a session at a time. State of this
//! file: the first 80 functions, `00486a90` and `00666050` (two
//! `NiTPointerMap` instance methods the linker placed far from the unit) and
//! `00854e10` to `0085a520`. The next session continues at `0085ac30`.
//!
//! What is here:
//!
//! - `ChangeData` and `ChangesMap` (the per-form record of what changed, and
//!   the hash map of those records keyed by form id): the accessors
//!   `00854e10` to `00855220`, the map's destructors and
//!   `ChangesMap::RemoveAllChanges`;
//! - the three smaller maps `InteriorCellNewReferencesMap`,
//!   `ExteriorCellNewReferencesMap` and `NumericIDBufferMap` (constructor,
//!   destructor and scalar deleting destructor of each);
//! - `SaveStats` (the statistics of a save, which `PrintStats` writes out as
//!   a text file), `SaveStats::Stats` and the small file-writing helper;
//! - `TESSaveLoadGame::RemoveChanges` and the save routine `00856ca0`;
//! - the small value types of the save format: the constructors of
//!   `ReferenceData`, `MovedReferenceData` (`008572f0`, `00857320`) and the
//!   `LoadFormHeader` fill (`00858aa0`);
//! - opening and closing the save file (`00857370`, `008578b0`, `00857950`),
//!   the buffer cursor (`008579b0`, `008579e0`, `00857bd0`, `CreateBuffer`,
//!   `WriteFile`, `00858700`), writing and reading file bytes (`00857b50`,
//!   `00857ba0`), the numeric id helpers (`SaveNumericID`, `LoadNumericID`,
//!   the plugin index maps `00857bf0`, `00857c70`);
//! - the global data of a save (`SaveGlobalData`, `SaveGlobals`,
//!   `SaveFinalData`);
//! - loading: `00857d10` (put a loaded reference at its saved place),
//!   `00858730` (load one form), `00858af0` (apply the queued initial data),
//!   `00859690` and `008598d0` (a cell's new references), `00859a90`
//!   (make a created reference), `00859f20` (load a moved reference from the
//!   plugins), `DeleteForm` and its helpers;
//! - `CheckNewReference`, `CheckFlags`, `GetInitialDataSaveSize` and
//!   `SaveInitialData`.
//!
//! Layouts and constants are below. What the next session needs:
//!
//! - `TESSaveLoadGame` (0x1C8 bytes, the size `TES`'s constructor allocates),
//!   `ChangeData`, `SaveStats`, `Stats`, `ExtraStat`, `LoadFormHeader`,
//!   `SaveFormHeader`, `FormAndFlags`, `ReferenceData`, `MovedReferenceData`,
//!   `CreatedReferenceData` and `ExteriorCellReferenceData` are declared here
//!   with the fields used so far;
//! - the pointer `011de45c` is the game's `TESSaveLoadGame`; the save code
//!   reads it through memory and passes it as `this` to several helpers that
//!   are members of the same class (`00857b50`, `008579e0`, `0085b320`,
//!   `0047c850`);
//! - the shared map and list helpers (`NiTMapBase::GetFirstPos` `004b9ba0`,
//!   `GetNext` `006b7f20` for a map keyed by `unsigned int`, `00863bc0` for
//!   `SaveStats`'s map keyed by a byte, `RemoveAll` `00438af0`, `RemoveAt`
//!   `00405430`, `GetAt` `00853130`, `SetAt` `00844700`; the `BSSimpleList`
//!   node item `006815c0`, next `00726070`, `RemoveAll` `00470470`, scalar
//!   deleting destructor `004702f0`) are called by address.
//!
//! Not translated: the compiler's exception-unwinding frames (the `FS:[0]`
//! chains and state variables) and the stack-cookie check of `00855ba0`.
//! The locals the game keeps on its stack and passes by address (the 4-byte
//! allocation-scope object, out parameters, the buffers and the file object
//! of `PrintStats`, the form header of `00856ca0`) are heap blocks here,
//! freed where the game's scope ends.
//!
//! The decompiler dropped or mis-attached several arguments in this unit
//! (the `this` of `0047c850`, `0085b320`, `00857b50`; the five stack
//! arguments of the "can't save" message, which belong to `007052f0`; the
//! base-class destructor calls of the map destructors; the argument of
//! `00410220` in `00857d10`, the arguments of `00403df0` and `00464f30` in
//! `00857370`, the `1` that `00469800` leaves on the stack for
//! `DeleteForm`'s virtual call), so the translations follow the disassembly.
//!
//! `0047c850` returns false in this build (`XOR AL,AL`). The functions that
//! start with "do nothing unless it is true" (`00858730`, `00858af0`,
//! `00859690`) therefore return at once in this build; they are translated
//! all the same. Several flag tests in this unit compare against a constant
//! 0 (`flags & 0`, the compiler stored the mask in the code but the mask is
//! 0 in this build); the code they guard is unreachable and not translated
//! (`008598d0`, which would call `00859a90` and `00859f20`; `008591b0`,
//! `0085a450`, `0085a520`
//! say so).

#[allow(unused_imports)]
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Layouts

layout! {
    /// `ChangeData` (Xbox PDB), 8 bytes: what is saved for one form.
    pub struct ChangeData: 0x08 {
        /// `iFlags` (Xbox PDB): the changed-parts bits.
        0x00 iFlags: u32,
        /// `pBuffer` (Xbox PDB): the form's pre-built save buffer, or null.
        0x04 pBuffer: Ptr,
    }

    /// `ChangesMap` (Xbox PDB): `NiTPointerMap<unsigned int, ChangeData *>`,
    /// 0x10 bytes. Vtable at +0 (slot 0x14 `NewItem`, 0x18 `DeleteItem`).
    pub struct ChangesMap: 0x10 {
        /// `m_uiHashSize` (Xbox PDB).
        0x04 m_uiHashSize: u32,
        /// `m_ppkHashTable` (Xbox PDB).
        0x08 m_ppkHashTable: Ptr,
        /// The allocator subobject (`AntiBloatAllocator`, Xbox PDB
        /// `m_kAllocator`), which holds the entry count (`NewItem` and
        /// `DeleteItem` pass its address).
        0x0C m_kAllocator: u32,
    }

    /// `InteriorCellNewReferencesMap` (Xbox PDB): a pointer map from a cell
    /// form id to a `BSSimpleList<unsigned int> *`, 0x10 bytes.
    pub struct InteriorCellNewReferencesMap: 0x10 {
        /// `m_uiHashSize` (Xbox PDB).
        0x04 m_uiHashSize: u32,
    }

    /// `ExteriorCellNewReferencesMap` (Xbox PDB): a pointer map to a
    /// `BSSimpleList<ExteriorCellReferenceData *> *`, 0x10 bytes.
    pub struct ExteriorCellNewReferencesMap: 0x10 {
        /// `m_uiHashSize` (Xbox PDB).
        0x04 m_uiHashSize: u32,
    }

    /// `NumericIDBufferMap` (Xbox PDB): a pointer map from an id to a buffer
    /// (`void *`), 0x10 bytes.
    pub struct NumericIDBufferMap: 0x10 {
        /// `m_uiHashSize` (Xbox PDB).
        0x04 m_uiHashSize: u32,
    }

    /// `SaveStats` (Xbox PDB), 8 bytes: the statistics of one save.
    pub struct SaveStats: 0x08 {
        /// `pStatsMap` (Xbox PDB): `NiTPointerMap<unsigned char,
        /// BSSimpleList<LoadFormHeader *> *> *`, from form type to the list
        /// of that type's headers, sorted by descending size.
        0x00 pStatsMap: Ptr,
        /// `pExtraStats` (Xbox PDB): `BSSimpleList<SaveStats::ExtraStat *> *`.
        0x04 pExtraStats: Ptr,
    }

    /// `SaveStats::Stats` (Xbox PDB), 0xC bytes: running count, total,
    /// minimum and maximum size.
    pub struct Stats: 0x0C {
        /// `iNum` (Xbox PDB).
        0x00 iNum: i32,
        /// `iTotalSize` (Xbox PDB).
        0x04 iTotalSize: i32,
        /// `iMinSize` (Xbox PDB).
        0x08 iMinSize: u16,
        /// `iMaxSize` (Xbox PDB).
        0x0A iMaxSize: u16,
    }

    /// `SaveStats::ExtraStat` (Xbox PDB), 8 bytes: a size and its description.
    pub struct ExtraStat: 0x08 {
        /// `iSize` (Xbox PDB).
        0x00 iSize: u32,
        /// `pDescription` (Xbox PDB): a heap copy of the text.
        0x04 pDescription: Ptr,
    }

    /// `LoadFormHeader` (Xbox PDB), 0xC bytes, packed (the flags word is
    /// unaligned).
    pub struct LoadFormHeader: 0x0C {
        /// `iFormID` (Xbox PDB).
        0x00 iFormID: u32,
        /// `cFormType` (Xbox PDB).
        0x04 cFormType: u8,
        /// `iFlags` (Xbox PDB), at the odd offset +5.
        0x05 iFlags: u32,
        /// `cVersion` (Xbox PDB).
        0x09 cVersion: u8,
        /// `iSize` (Xbox PDB).
        0x0A iSize: u16,
    }

    /// `SaveFormHeader` (Xbox PDB), 0xA bytes: the `LoadFormHeader` without
    /// the size.
    pub struct SaveFormHeader: 0x0A {
        /// `iFormID` (Xbox PDB).
        0x00 iFormID: u32,
        /// `cFormType` (Xbox PDB).
        0x04 cFormType: u8,
        /// `iFlags` (Xbox PDB), at the odd offset +5.
        0x05 iFlags: u32,
        /// `cVersion` (Xbox PDB).
        0x09 cVersion: u8,
    }

    /// `FormAndFlags` (Xbox PDB), 0x10 bytes.
    pub struct FormAndFlags: 0x10 {
        /// `pForm` (Xbox PDB).
        0x00 pForm: Ptr,
        /// `iFlags` (Xbox PDB).
        0x04 iFlags: u32,
        /// `iOldFlags` (Xbox PDB).
        0x08 iOldFlags: u32,
        /// `cVersion` (Xbox PDB).
        0x0C cVersion: u8,
    }

    /// `TESSaveLoadGame` (Xbox PDB), 0x1C8 bytes on the PC too (the size
    /// `TES` allocates); the fields this unit's first functions use, at the
    /// offsets the PC code uses (equal to the Xbox PDB's).
    pub struct TESSaveLoadGame: 0x1C8 {
        /// `m_pChanges` (Xbox PDB): the `ChangesMap`.
        0x00 m_pChanges: Ptr<ChangesMap>,
        /// `m_pInteriorCellMap` (Xbox PDB): the
        /// `InteriorCellNewReferencesMap`.
        0x08 m_pInteriorCellMap: Ptr,
        /// `m_pExteriorCellMap` (Xbox PDB): the
        /// `ExteriorCellNewReferencesMap`.
        0x0C m_pExteriorCellMap: Ptr,
        /// `m_pBuffer` (Xbox PDB): the current form's pre-built buffer.
        0x14 m_pBuffer: Ptr,
        /// `m_pInitArray` (Xbox PDB):
        /// `NiTLargePrimitiveArray<FormAndFlags *> *`, the forms whose
        /// initial data still has to be applied after a load.
        0x20 m_pInitArray: Ptr,
        /// `m_pSaveLoadStats` (Xbox PDB): the `SaveStats`, or null.
        0x44 m_pSaveLoadStats: Ptr<SaveStats>,
        /// `m_iSavedPluginCount` (Xbox PDB).
        0x4C m_iSavedPluginCount: u8,
        /// `m_pFileIndexArray` (Xbox PDB): the table that maps the plugin
        /// index a save was written with to the current one (0xFF: none).
        0x50 m_pFileIndexArray: Ptr,
        /// `m_iQueuedRemoveChanges` (Xbox PDB): flags of a `RemoveChanges`
        /// that was asked for while a form was loading.
        0x54 m_iQueuedRemoveChanges: u32,
        /// `m_pSaveGameList` (Xbox PDB): `BSSimpleList<SaveGameFile *> *`.
        0x70 m_pSaveGameList: Ptr,
        /// `m_bUseNumericIDArray` (Xbox PDB).
        0x81 m_bUseNumericIDArray: bool,
        /// `m_pCurrentlySavingFormHeader` (Xbox PDB).
        0x88 m_pCurrentlySavingFormHeader: Ptr,
        /// `m_iSimulationFileSize` (Xbox PDB): the bytes counted instead of
        /// written when the save only measures.
        0x94 m_iSimulationFileSize: u32,
    }

    /// `ReferenceData` (Xbox PDB), 0x1C bytes: where a reference is, as the
    /// save format stores it (the location's numeric id, position, angle).
    pub struct ReferenceData: 0x1C {
        /// `iLocationID` (Xbox PDB).
        0x00 iLocationID: u32,
        /// `Loc` (Xbox PDB), a `NiPoint3`: x.
        0x04 LocX: f32,
        /// `Loc`: y.
        0x08 LocY: f32,
        /// `Loc`: z.
        0x0C LocZ: f32,
        /// `Angle` (Xbox PDB), a `NiPoint3`: x.
        0x10 AngleX: f32,
        /// `Angle`: y.
        0x14 AngleY: f32,
        /// `Angle`: z.
        0x18 AngleZ: f32,
    }

    /// `MovedReferenceData` (Xbox PDB), 0x2C bytes: the original location id
    /// (`iOriginalLocationID`, +0) and position (`OriginalLoc`, +4), then a
    /// `ReferenceData` (`RefData`) at +0x10.
    pub struct MovedReferenceData: 0x2C {
        /// `iOriginalLocationID` (Xbox PDB).
        0x00 iOriginalLocationID: u32,
    }

    /// `CreatedReferenceData` (Xbox PDB), 0x24 bytes: the type, the bound
    /// object's id, then a `ReferenceData` (`RefData`) at +8.
    pub struct CreatedReferenceData: 0x24 {
        /// `eType` (Xbox PDB), a `TESCreatedReferenceType`: 0 normal, 1 arrow
        /// projectile, 2 magic projectile, 3 persistent.
        0x00 eType: u32,
        /// `iBoundID` (Xbox PDB): the numeric id of the base object.
        0x04 iBoundID: u32,
    }

    /// `ExteriorCellReferenceData` (Xbox PDB), 0xC bytes.
    pub struct ExteriorCellReferenceData: 0x0C {
        /// `iFormID` (Xbox PDB).
        0x00 iFormID: u32,
        /// `iCellX` (Xbox PDB).
        0x04 iCellX: i32,
        /// `iCellY` (Xbox PDB).
        0x08 iCellY: i32,
    }
}

// ---------------------------------------------------------------------------
// Constants: callees and data outside this file

/// `operator new(size)` (`00401000`).
const OPERATOR_NEW: u32 = 0x0040_1000;
/// `operator delete(block)` (`00401030`).
const OPERATOR_DELETE: u32 = 0x0040_1030;
/// The dword at `this + 0x0C` (`MOV EAX,[ECX+0xC]`): for a form, its form id.
const FORM_ID: u32 = 0x0084_e3a0;
/// `MOV EAX,[ECX]`: the word at the address `this` (the flags of a
/// `ChangeData`; the first word of a file object).
const READ_WORD: u32 = 0x0055_9450;
/// The global that holds the pointer to the game's `TESSaveLoadGame`.
const SAVE_LOAD_GAME: u32 = 0x011d_e45c;
/// `0047c850`: takes a `TESSaveLoadGame` and returns false in this build
/// (`XOR AL,AL`); the save code asks it before every write.
const SAVE_LOAD_UNAVAILABLE: u32 = 0x0047_c850;
/// `TESSaveLoadGame::GetSavingAllowed` (Xbox PDB), `this` the game.
const GET_SAVING_ALLOWED: u32 = 0x0086_16f0;
/// `(form flags at +8) & 0x4000` (`SETNZ`): the form is deleted.
const FORM_IS_DELETED: u32 = 0x0040_77c0;

/// The allocation scope: `this` is a 4-byte object, then (0x11, 1, file,
/// line); `00404eb0` constructs it, `00404ee0` destroys it.
const SCOPE_ENTER: u32 = 0x0040_4eb0;
const SCOPE_LEAVE: u32 = 0x0040_4ee0;
/// `"D:\_Fallout3\Platforms\Common\Code\Fallout Shared\TESSaveLoadGame.cpp"`.
const SOURCE_FILE: u32 = 0x0108_04c8;

/// `NiTMapBase<unsigned int, X *>::GetAt(key, &value) -> bool`.
const MAP_GET_AT: u32 = 0x0085_3130;
/// `NiTMapBase::SetAt(key, value)` of `ChangesMap`.
const CHANGES_MAP_SET_AT: u32 = 0x0084_4700;
/// `NiTMapBase<unsigned int, X *>::RemoveAt(key) -> bool`.
const MAP_REMOVE_AT: u32 = 0x0040_5430;
/// `NiTMapBase::GetFirstPos`: the first used slot's entry, or 0.
const MAP_FIRST_POSITION: u32 = 0x004b_9ba0;
/// `NiTMapBase<unsigned int, X *>::GetNext(&pos, &key, &value)`.
const MAP_NEXT: u32 = 0x006b_7f20;
/// `SaveStats`'s map: `GetNext(&pos, &key (a byte), &value)`.
const BYTE_MAP_NEXT: u32 = 0x0086_3bc0;
/// `SaveStats`'s map: `GetAt(key (a byte), &value) -> bool`.
const BYTE_MAP_GET_AT: u32 = 0x0086_3b40;
/// `SaveStats`'s map: `SetAt(key (a byte), value)`.
const BYTE_MAP_SET_AT: u32 = 0x0086_3a60;
/// `NiTMapBase::RemoveAll`.
const MAP_REMOVE_ALL: u32 = 0x0043_8af0;
/// The map allocator subobject's `Allocate` (called with `map + 0x0C`) and
/// `Deallocate(item)`.
const MAP_ALLOCATOR_NEW_ITEM: u32 = 0x0043_a010;
const MAP_ALLOCATOR_DELETE_ITEM: u32 = 0x0045_cee0;

/// `BSSimpleList` constructor (`0096a2d0`: item and next set to null; the
/// linker folded `ChangeData`'s constructor into it).
const SIMPLE_LIST_CONSTRUCT: u32 = 0x0096_a2d0;
/// `BSSimpleList` node's item address (`006815c0`: returns `this`; the same
/// code is the empty constructor of the form headers).
const LIST_NODE_ITEM: u32 = 0x0068_15c0;
/// A node's next node (`00726070`: `[this + 4]`).
const LIST_NODE_NEXT: u32 = 0x0072_6070;
/// `BSSimpleList::RemoveAll`.
const LIST_REMOVE_ALL: u32 = 0x0047_0470;
/// `BSSimpleList` scalar deleting destructor: `this` and the delete flag.
const LIST_SCALAR_DELETE: u32 = 0x0047_02f0;
/// `BSSimpleList::AddHead(&item)`.
const LIST_ADD_HEAD: u32 = 0x005a_e3d0;
/// `BSSimpleList::Insert(item, comparator)`, keeping the list sorted by the
/// comparator (a `cdecl` function of two items).
const LIST_INSERT: u32 = 0x007a_7eb0;
/// The comparator `fn_00855a20` passes to it.
const STATS_COMPARATOR: u32 = 0x0085_5b60;

/// `strlen` through the game's wrapper.
const STRLEN: u32 = 0x0044_a670;
/// `strcpy_s(destination, size, source)` through the game's wrapper.
const STRING_COPY: u32 = 0x0040_6d30;
/// `strcat_s(destination, size, source)`.
const STRING_CAT: u32 = 0x0040_6d50;
/// `sprintf_s(buffer, size, format, ...)`.
const FORMAT: u32 = 0x0040_6d00;
/// `strcmp(a, b)`.
const STRING_COMPARE: u32 = 0x0040_8b20;
/// `__RTDynamicCast(object, vfDelta, sourceType, targetType, isReference)`.
const DYNAMIC_CAST: u32 = 0x00ec_43fb;

/// The `NiTPointerMap` base constructors (`this`, hash size) and
/// destructors of the maps this unit owns.
const CHANGES_MAP_BASE_DESTRUCT: u32 = 0x0086_3640;
const INTERIOR_MAP_BASE_CONSTRUCT: u32 = 0x0086_3390;
const INTERIOR_MAP_BASE_DESTRUCT: u32 = 0x0086_3740;
const EXTERIOR_MAP_BASE_CONSTRUCT: u32 = 0x0086_33c0;
const EXTERIOR_MAP_BASE_DESTRUCT: u32 = 0x0086_3860;
const NUMERIC_ID_MAP_BASE_CONSTRUCT: u32 = 0x0086_33f0;
const NUMERIC_ID_MAP_BASE_DESTRUCT: u32 = 0x0086_3960;
/// `SaveStats`'s `NiTPointerMap<unsigned char, ...>` constructor (`this`,
/// hash size).
const STATS_MAP_CONSTRUCT: u32 = 0x0086_3420;
/// Hash size of the three reference maps and of `SaveStats`'s map.
const HASH_SIZE: u32 = 0x25;

/// Vtables of the maps this unit owns.
const CHANGES_MAP_VTABLE: u32 = 0x0108_04a8;
const INTERIOR_MAP_VTABLE: u32 = 0x0108_0514;
const EXTERIOR_MAP_VTABLE: u32 = 0x0108_0534;
const NUMERIC_ID_MAP_VTABLE: u32 = 0x0108_0554;

/// `TESSaveLoadGame` members the save routine calls (`this` the game unless
/// noted).
const SAVE_HEADER: u32 = 0x0086_1130; // (file, name)
const SAVE_PLUGIN_LIST: u32 = 0x0085_b240; // (file)
const SAVE_GLOBAL_DATA: u32 = 0x0085_8030; // (file)
const SAVE_FINAL_DATA: u32 = 0x0085_8570; // (file)
const SAVE_NUMERIC_ID_ARRAYS: u32 = 0x0086_1d10; // (file)
const CHECK_FLAGS: u32 = 0x0085_91b0; // (form, flags) -> flags
const GET_INITIAL_DATA_SAVE_SIZE: u32 = 0x0085_a450; // (form, flags) -> u16
const SAVE_INITIAL_DATA: u32 = 0x0085_a520; // (form, flags)
const CREATE_BUFFER: u32 = 0x0085_8600; // (size) -> buffer
const WRITE_FILE: u32 = 0x0085_86a0; // (file, buffer, size)
const FREE_BUFFER: u32 = 0x0085_8700; // (buffer)
/// Writes `size` bytes at `data` to the file: (file, data, size).
const WRITE_BYTES: u32 = 0x0085_7b50;
/// Reads 4 bytes of the current buffer into `data`: (data, size).
const READ_BYTES: u32 = 0x0085_79e0;
/// The version number of the save format.
const CURRENT_VERSION: u32 = 0x008d_f040;
/// Opens the save file: (file or null, name, 0) -> the file.
const OPEN_SAVE_FILE: u32 = 0x0085_7370;
/// The save's preparation steps, each `this` only.
const SAVE_PREPARE_A: u32 = 0x0086_27b0;
const SAVE_PREPARE_B: u32 = 0x0086_20f0;
const SAVE_PREPARE_C: u32 = 0x0085_6850;
/// The save's closing steps: (file) and (file, 0).
const SAVE_CLOSE_A: u32 = 0x0086_2150;
const SAVE_CLOSE_B: u32 = 0x0085_78b0;
/// The current position of a file (`this` the file).
const FILE_POSITION: u32 = 0x0047_20a0;
/// The form type byte of a form (`MOVZX EAX,byte [ECX+4]`).
const FORM_TYPE: u32 = 0x0040_1170;
/// `LookupFormByID(id)`, `cdecl`; null for none.
const LOOKUP_FORM: u32 = 0x0048_39c0;
/// The global that holds the object whose `Enter` and `Leave` (`00c3e310`,
/// `00c3e340`) bracket a save.
const SAVE_LOCK: u32 = 0x0120_2d98;
const SAVE_LOCK_ENTER: u32 = 0x00c3_e310;
const SAVE_LOCK_LEAVE: u32 = 0x00c3_e340;
/// File object virtual slots: `0x14` seek(position, mode), `0x18` name.
const FILE_SEEK: u32 = 0x14;
const FILE_GET_NAME: u32 = 0x18;
/// The seek mode the save routine uses (a global dword).
const SEEK_MODE: u32 = 0x010a_2480;
/// Form virtual slots: `0x50` the size of the changed data, `0x58` its save,
/// `0x130` its description.
const FORM_GET_CHANGES_SIZE: u32 = 0x50;
const FORM_SAVE_CHANGES: u32 = 0x58;
const FORM_GET_DESCRIPTION: u32 = 0x130;

/// The "can't save" message: the message queue getter (`this` the object at
/// `011d2364`) and `007052f0(queue, 0, icon, 0, time, 0)`.
const MESSAGE_QUEUE_OBJECT: u32 = 0x011d_2364;
const GET_MESSAGE_QUEUE: u32 = 0x004c_69f0;
const SHOW_MESSAGE: u32 = 0x0070_52f0;
/// `"Interface\Icons\Message Icons\glow_message_vaultboy_sad.dds"`.
const SAD_ICON: u32 = 0x0102_08a0;
/// The message's display time (a `float` constant).
const MESSAGE_TIME: u32 = 0x0101_62c0;
/// `"autosave"`.
const AUTOSAVE_NAME: u32 = 0x0107_fae0;

/// `FileFinder::Exist(path, 0, 0, -1)` and `BSSystemFile::DeleteFileA(path)`.
const FILE_EXISTS: u32 = 0x0045_6a20;
const FILE_DELETE: u32 = 0x00af_f0b0;
/// The text file object's constructor (`this`, path, 1, 2, 0) and
/// destructor.
const FILE_OBJECT_CONSTRUCT: u32 = 0x00b0_0900;
const FILE_OBJECT_DESTRUCT: u32 = 0x00b0_0950;
/// `BSSystemFile::DoWrite(this = file, text, size, 0, &scratch) -> error`.
const SYSTEM_FILE_DO_WRITE: u32 = 0x0085_6350;
/// `00aa15a0(file)`: the call that ends a save's use of the file.
const FILE_FLUSH: u32 = 0x00aa_15a0;
/// `TESSaveLoadGame::BuildChangesString(buffer, form, flags, type, 0)`,
/// `this` the singleton.
const BUILD_CHANGES_STRING: u32 = 0x0085_b320;
/// The name of a reference (`this` the reference) and the location name of
/// a map marker (`""` when it has none).
const REFERENCE_GET_NAME: u32 = 0x0055_d520;
const MAP_MARKER_GET_LOCATION_NAME: u32 = 0x0040_8da0;
/// The table of the form type names: 12 bytes per type, the first word the
/// name's address.
const FORM_TYPE_NAME_TABLE: u32 = 0x0118_7004;
/// The initializer of the record embedded at +8 of `fn_008572c0`'s object
/// (`fn_008572f0`, which this file also translates).
const EMBEDDED_RECORD_INIT: u32 = 0x0085_72f0;

// Globals the second batch of functions reads. Pointers (read with
// `e.global`) unless the doc says the address itself is the object.
/// The `TESDataHandler` (`TESDataHandler::GetNextID`, `00469800`, takes it as
/// `this`).
const DATA_HANDLER: u32 = 0x011c_3f2c;
/// The `TES` object (`TES::GetWorldSpace`, `TES::SaveGame` take it).
const TES_OBJECT: u32 = 0x011d_ea10;
/// The player's reference.
const PLAYER: u32 = 0x011d_ea3c;
/// The `ProcessLists` object (the address itself is the object).
const PROCESS_LISTS: u32 = 0x011e_0e80;
/// `TESSaveLoadGame::SaveGameCriticalSection` (Xbox PDB), a
/// `BSCriticalSection` (the address itself is the object): `004538a0`
/// enters it with a name, `004538c0` leaves it.
const LOAD_SECTION: u32 = 0x011d_e494;
/// The model loader (`ModelLoader::QueueReference` takes it as `this`).
const MODEL_LOADER: u32 = 0x011c_3b3c;
/// A global `NiPoint3` (three floats at this address) used as the position
/// of an actor that has none and as the starting vectors of the placement.
const DEFAULT_POSITION: u32 = 0x011f_426c;
/// A global `NiPoint3` passed to `FN_0057D0A0`.
const PLACEMENT_VECTOR: u32 = 0x011a_9478;
/// An object whose word at `+4` is a path string (`00403df0` reads it).
const PATH_OBJECT: u32 = 0x011c_3f74;

// Run-time type descriptors passed to `__RTDynamicCast`. The name says what
// the code does with the result; the first is the type every form is cast
// from.
const RTTI_FORM: u32 = 0x0118_3028;
const RTTI_REFERENCE: u32 = 0x0118_41cc;
const RTTI_CELL: u32 = 0x0118_3fb4;
const RTTI_WORLDSPACE: u32 = 0x0118_3fd0;
/// The base object of a reference (compared with `007af430`'s result).
const RTTI_BOUND_OBJECT: u32 = 0x0118_3108;
/// A reference that has a process (`Actor::InitPackageLocations` is called
/// on it).
const RTTI_ACTOR: u32 = 0x0118_46d4;
/// A reference with a character controller (`MobileObject` in the engine
/// map).
const RTTI_MOBILE_OBJECT: u32 = 0x0118_4920;

// Library calls and strings.
/// `memcpy(destination, source, size)` (`00401460`).
const MEMCPY: u32 = 0x0040_1460;
/// Returns the address of a constant string (the start of the save paths).
const PATH_PREFIX: u32 = 0x004d_c110;
/// `this` = `PATH_OBJECT`: returns the word at `this + 4`, or 0 for a null
/// `this`.
const PATH_OBJECT_GET: u32 = 0x0040_3df0;
/// Returns its argument (`00464f30`).
const IDENTITY: u32 = 0x0046_4f30;
/// Builds the default save name into a buffer: (game, buffer).
const DEFAULT_SAVE_NAME: u32 = 0x0086_0ae0;
/// `"%s%s%s.ess"`, `".ess"`, `"Save "`, `".bak"` and `""`.
const FORMAT_SAVE_PATH: u32 = 0x0108_06b8;
const ESS_EXTENSION: u32 = 0x0108_06b0;
const SAVE_NAME_PREFIX: u32 = 0x0108_06a8;
const BAK_EXTENSION: u32 = 0x0107_facc;
const EMPTY_STRING: u32 = 0x0101_1584;
/// Import slots of `CreateDirectoryA` (path, 0) and `DeleteFileA` (path), and
/// the CRT's `rename(old, new)`, `strrchr(text, char)` (`0040ab30`) and
/// `_strnicmp(a, b, count)`.
const CREATE_DIRECTORY_IMPORT: u32 = 0x00fd_f0b8;
const DELETE_FILE_IMPORT: u32 = 0x00fd_f0d4;
const RENAME: u32 = 0x00ec_862c;
const STRRCHR: u32 = 0x0040_ab30;
const STRNICMP: u32 = 0x00ec_7ec0;
/// A string comparison that returns 0 when equal (`004812f0`).
const STRING_COMPARE_NOCASE: u32 = 0x0048_12f0;
/// `_sprintf(buffer, format, ...)`.
const SPRINTF: u32 = 0x00ec_623a;
/// `Error(format, ...)` (`0040fbe0`) and the logging call `005b5e40(format,
/// ...)`: `cdecl`, and empty in this build.
const ERROR: u32 = 0x0040_fbe0;
const LOG_ERROR: u32 = 0x005b_5e40;
/// The file's write (`00473180`) and read (`00462d80`): `this` the file,
/// (data, size).
const FILE_WRITE: u32 = 0x0047_3180;
const FILE_READ: u32 = 0x0046_2d80;

// Files and lists.
/// `BSFile::BSFile(this, path, writeMode, bufferSize, 0)`, the size of a
/// `BSFile` on the PC and `BSFile::Close(this)`.
const BSFILE_CONSTRUCT: u32 = 0x00b0_0260;
const BSFILE_SIZE: u32 = 0x158;
const BSFILE_CLOSE: u32 = 0x00af_fd10;
/// File virtual slots: `0` the scalar deleting destructor (flag), `0x20`
/// `BSFile::Open(a, b)` (the slot of the PC's vtable at `010a4764` is
/// `00aff300`).
const FILE_DESTRUCT: u32 = 0x00;
const FILE_OPEN: u32 = 0x20;
/// `BSSimpleList<..>` calls: remove the item `*item_slot` (`00905330`), test
/// that the list holds `*item_slot` (`005f65d0`), remove the first node
/// (`0063f7b0`), the end test of a node (`008256d0`: item and next are both
/// null) and the destructor of a local list (`0046ffb0`).
const LIST_REMOVE: u32 = 0x0090_5330;
const LIST_CONTAINS: u32 = 0x005f_65d0;
const LIST_REMOVE_HEAD: u32 = 0x0063_f7b0;
const LIST_NODE_IS_END: u32 = 0x0082_56d0;
const LIST_DESTRUCT: u32 = 0x0046_ffb0;
/// The list of global variables embedded in the data handler (`00461190`
/// returns `this + 0xE8`), its count (`005ae380`) and a global variable's
/// value (`00526ac0`, a float).
const GLOBALS_LIST: u32 = 0x0046_1190;
const LIST_COUNT: u32 = 0x005a_e380;
const GLOBAL_VALUE: u32 = 0x0052_6ac0;
/// `NiTLargePrimitiveArray<FormAndFlags *>`: the destructor (`00863d60`), the
/// constructor (this, grow, size), `Add(&item)`, the cleanup before the
/// array is deleted and the address of the slot of an index (`00877a30`,
/// (array, index)). The size is the dword at `+0xC` (`0084e3a0`, the same
/// code as `FORM_ID`).
const INIT_ARRAY_DESTRUCT: u32 = 0x0086_3d60;
const INIT_ARRAY_CONSTRUCT: u32 = 0x0086_3e00;
const INIT_ARRAY_ADD: u32 = 0x0086_3d90;
const INIT_ARRAY_CLEANUP: u32 = 0x0086_3db0;
const ARRAY_ELEMENT_ADDRESS: u32 = 0x0087_7a30;
const ARRAY_SIZE: u32 = 0x0084_e3a0;

// `TESSaveLoadGame` members and neighbours.
/// `AddNumericIDToArray(game, id)` and `(game, numeric id) -> form id`
/// (`00861c00`); `00574900(game)` reads `m_bUseNumericIDArray`.
const ADD_NUMERIC_ID: u32 = 0x0086_1b70;
const RESOLVE_NUMERIC_ID: u32 = 0x0086_1c00;
const USE_NUMERIC_IDS: u32 = 0x0057_4900;
/// `00861ee0(game, version)` stores the version of the form being loaded
/// (and logs when it is below 0x13); `00856850(game)` copies
/// `m_cMinorVersion` to it.
const SET_LOAD_VERSION: u32 = 0x0086_1ee0;
const END_FORM_PROCESSING: u32 = 0x0085_6850;
/// `(game, form, flags)`: applies the initial data of a loaded form
/// (`0085ac30`, a later session's range).
const LOAD_INITIAL_DATA: u32 = 0x0085_ac30;
/// `4fb090(game, header)` stores `m_pCurrentlyLoadingFormHeader`.
const SET_LOADING_HEADER: u32 = 0x004f_b090;
/// A function that does nothing and takes one argument (`004534f0`); the
/// code calls it as `this`, 0 or 1 around loads.
const SET_LOADING_STATE: u32 = 0x0045_34f0;
/// `004538a0(section, name)` enters a critical section and
/// `004538c0(section)` leaves it.
const SECTION_ENTER: u32 = 0x0045_38a0;
const SECTION_LEAVE: u32 = 0x0045_38c0;
/// Stores its argument at `this + 4` (`006ecd40`): clears a `ChangeData`'s
/// `pBuffer`.
const CHANGE_DATA_SET_BUFFER: u32 = 0x006e_cd40;
/// The data handler's `GetNextID` (`00469800`) and "knows this form id"
/// (`00469860(handler, id)`).
const DATA_HANDLER_GET_NEXT_ID: u32 = 0x0046_9800;
const DATA_HANDLER_HAS_FORM: u32 = 0x0046_9860;
/// `00484af0` `TESForm::SetDisabled(form, 1)`.
const FORM_SET_DISABLED: u32 = 0x0048_4af0;
/// `00483c70(form)`, `00483710(player)` (empty), the empty-handed
/// `TESPackage::CreatePackage` peer that makes a form of a type
/// (`00670b90`, one byte argument) and `0046a010(form, 1)`.
const FN_00483C70: u32 = 0x0048_3c70;
const EMPTY_FN_00483710: u32 = 0x0048_3710;
const CREATE_FORM_OF_TYPE: u32 = 0x0067_0b90;
const FORM_FINISH: u32 = 0x0046_a010;

/// `BGSLoadFormBuffer::GetVersion(this)` (the byte at `+0x1C`) and the setter
/// (`this`, version).
const BUFFER_GET_VERSION: u32 = 0x008a_81c0;
const BUFFER_SET_VERSION: u32 = 0x008a_8150;
// Form and reference virtual slots.
/// `0x60` loads a form from its buffer (flags, 0); `0x68` and `0x6C` are the
/// hooks before and after a load (flags, old flags); `0x88` runs after a
/// reference was made; `0x128` (id, 1) assigns the form id; `0x134` (name)
/// the editor id.
const FORM_LOAD: u32 = 0x60;
const FORM_BEGIN_INIT: u32 = 0x68;
const FORM_END_INIT: u32 = 0x6C;
const FORM_POST_CREATE: u32 = 0x88;
const FORM_SET_FORM_ID: u32 = 0x128;
const FORM_SET_EDITOR_ID: u32 = 0x134;
/// References: `0x100` true for an actor-like reference, `0x1F4` the
/// position, `0x170` and `0x16C` (out buffer) the saved location and
/// rotation. For an actor-like one `0x290` says it has a saved location,
/// `0x298` and `0x294` give its cell and world space, `0x22C` (flag) is a
/// test. A process: `0x20C`.
const REFERENCE_IS_ACTOR: u32 = 0x100;
const REFERENCE_GET_POSITION_SLOT: u32 = 0x1F4;
const REFERENCE_GET_LOCATION_SLOT: u32 = 0x170;
const REFERENCE_GET_ROTATION_SLOT: u32 = 0x16C;
const ACTOR_HAS_LOCATION_SLOT: u32 = 0x290;
const ACTOR_WORLDSPACE_SLOT: u32 = 0x294;
const ACTOR_CELL_SLOT: u32 = 0x298;
const ACTOR_SLOT_22C: u32 = 0x22C;
const PROCESS_SLOT_20C: u32 = 0x20C;

// References.
/// `TESObjectREFR` calls (`this` the reference unless noted):
/// the world space (`00575d70`) and parent cell (`008d6f30`), the extra data
/// list (`005d43c0`, `this + 0x44`), the position (`00436aa0`, `this +
/// 0x30`) and rotation (`00430830`, `this + 0x24`), whether it persists
/// (`005653d0`), the base object (`007af430`, the dword at `+0x20`),
/// `SetLocationOnReference(this, &position)` (`00575830`), the two setters
/// `005757d0(this, float)` and `00575700(this, x, y, z)`,
/// `MoveRefToNewSpace(reference, cell, worldspace)` (`00573800`, `cdecl`),
/// `GetOrientation(this, out)` (`0056fa00`), the 3D node (`0043fcd0`) and
/// `SetObjectReference(this, base)` (`00575690`).
const REF_GET_WORLDSPACE: u32 = 0x0057_5d70;
const REF_GET_PARENT_CELL: u32 = 0x008d_6f30;
const REF_GET_EXTRA_LIST: u32 = 0x005d_43c0;
const REF_GET_POSITION: u32 = 0x0043_6aa0;
const REF_GET_ROTATION: u32 = 0x0043_0830;
const REF_PERSISTS: u32 = 0x0056_53d0;
const REFERENCE_GET_BASE: u32 = 0x007a_f430;
const REF_SET_POSITION: u32 = 0x0057_5830;
const FN_005757D0: u32 = 0x0057_57d0;
const FN_00575700: u32 = 0x0057_5700;
const REF_MOVE_TO_SPACE: u32 = 0x0057_3800;
const REF_GET_ORIENTATION: u32 = 0x0056_fa00;
const REF_GET_NODE: u32 = 0x0043_fcd0;
const REF_SET_BASE: u32 = 0x0057_5690;
/// The 3D node and collision calls of `fn_00857d10`: `00440460(node,
/// &position)`, `0043fa80(node, orientation)`, `bhkNiCollisionObject::ResetSim`
/// (`00c6bd00(node, 1)`, `cdecl`), the constructor `0043d410(this, float, 0,
/// 0)` of a 12-byte object and `00a59c60(node, object)`.
const FN_00440460: u32 = 0x0044_0460;
const FN_0043FA80: u32 = 0x0043_fa80;
const COLLISION_RESET_SIM: u32 = 0x00c6_bd00;
const FN_0043D410: u32 = 0x0043_d410;
const FN_00A59C60: u32 = 0x00a5_9c60;
/// `MobileObject::GetCharController` (`009306d0`), a test on the controller
/// (`005c0860`) and `bhkCharacterController::SetPosition(controller,
/// &position)` (`005620e0`).
const MOBILE_GET_CHAR_CONTROLLER: u32 = 0x0093_06d0;
const CHAR_CONTROLLER_TEST: u32 = 0x005c_0860;
const CHAR_CONTROLLER_SET_POSITION: u32 = 0x0056_20e0;
/// Extra data lists (`this` the list): `GetSeenData` (`00555bc0`),
/// `GetContainerChanges` (`00418520`), `BaseExtraList::GetExtraData(list,
/// type)` (`00410220`), `GetStartingWorldOrCell` (`0041b320`), the two
/// starting setters `(list, &scratch, reference, x, y, z)` (`0041b180`,
/// `0041b120`) and `0041d460`, which reads the dword at `+0xC` of the
/// extra data of type 0xC (0 when there is none).
const EXTRA_GET_SEEN: u32 = 0x0055_5bc0;
const EXTRA_GET_CONTAINER_CHANGES: u32 = 0x0041_8520;
const EXTRA_GET_DATA: u32 = 0x0041_0220;
const EXTRA_GET_STARTING_SPACE: u32 = 0x0041_b320;
const EXTRA_SET_STARTING_POSITION: u32 = 0x0041_b180;
const EXTRA_SET_STARTING_ROTATION: u32 = 0x0041_b120;
const EXTRA_GET_CELL_DATA: u32 = 0x0041_d460;
/// Cells (`this` the cell): `IsInterior` (`00425fd0`), the world space
/// (`TESObjectCELL::GetWorldSpace`, `0054ddd0`), the grid coordinates
/// (`00544c30`, `00544c60`), `GetCOCPlacementInfo(cell, &position,
/// &rotation)` (`0054cfd0`) and the number of plugin files (`005504e0`).
const CELL_IS_INTERIOR: u32 = 0x0042_5fd0;
const CELL_GET_WORLDSPACE: u32 = 0x0054_ddd0;
const CELL_GET_X: u32 = 0x0054_4c30;
const CELL_GET_Y: u32 = 0x0054_4c60;
const CELL_GET_PLACEMENT: u32 = 0x0054_cfd0;
const CELL_FILE_COUNT: u32 = 0x0055_04e0;
/// `float -> int` (`00406d90`) and `TESWorldSpace::GetCellFromCellCoord`
/// (`worldspace, x, y`).
const FLOAT_TO_INT: u32 = 0x0040_6d90;
const WORLDSPACE_GET_CELL: u32 = 0x0058_75a0;
/// Actors and their processes: the process of an actor (`008d8520`, the
/// engine map's `MiddleHighProcess::GetSavedAcquireObject` names the folded
/// body), `00933790(actor, package)`, `008aad40(actor, flags)`,
/// `87f890(actor, worldspace, cell, position, float)`,
/// `Actor::InitPackageLocations(actor, 0)` (`00893340`).
const ACTOR_GET_PROCESS: u32 = 0x008d_8520;
const ACTOR_PACKAGE_FLAGS: u32 = 0x0093_3790;
const ACTOR_TEST_FLAGS: u32 = 0x008a_ad40;
const ACTOR_PLACE: u32 = 0x0087_f890;
const ACTOR_INIT_PACKAGE_LOCATIONS: u32 = 0x0089_3340;
/// Constructors of what `fn_00859a90` makes: `Character` (0x1C8 bytes),
/// `Creature` (0x1C0), `TESObjectREFR` (0x68), the arrow projectile (0xC8)
/// and the three magic projectiles (0xC4, 0xD0, 0xD8); each takes the
/// block and returns it.
const CHARACTER_CONSTRUCT: u32 = 0x008d_1d30;
const CREATURE_CONSTRUCT: u32 = 0x008d_43a0;
const REFERENCE_CONSTRUCT: u32 = 0x0055_a2f0;
const ARROW_PROJECTILE_CONSTRUCT: u32 = 0x008c_ab20;
const MAGIC_PROJECTILE_CONSTRUCT_A: u32 = 0x0080_f6e0;
const MAGIC_PROJECTILE_CONSTRUCT_B: u32 = 0x0081_92a0;
const MAGIC_PROJECTILE_CONSTRUCT_C: u32 = 0x0081_17d0;
/// Plugin files (`fn_00859f20`): `TESForm::GetFile(cell, index)`
/// (`00484e60`), `TESFile::GetThreadSafeFile` (`004739b0`),
/// `TESFile::FindForm`-style test (`004734d0(file, cell)`), "contains the
/// form" (`00550e10(file, id)`, `cdecl`), the file's current record type
/// (`00472660`), `TESObjectREFR::CreateReference(type, 1)` (`00564480`,
/// `cdecl`), `TESDataHandler::LoadForm(reference, file)` (`004601d0`,
/// `cdecl`) and `TESWorldSpace::FindCellInFile(worldspace, file, x, y)`
/// (`005854f0`).
const FILE_OF_CELL: u32 = 0x0048_4e60;
const THREAD_SAFE_FILE: u32 = 0x0047_39b0;
const FILE_HAS_CELL: u32 = 0x0047_34d0;
const FILE_HAS_FORM: u32 = 0x0055_0e10;
const FILE_RECORD_TYPE: u32 = 0x0047_2660;
const CREATE_REFERENCE: u32 = 0x0056_4480;
const LOAD_FORM_FROM_FILE: u32 = 0x0046_01d0;
const WORLDSPACE_FIND_CELL_IN_FILE: u32 = 0x0058_54f0;

// Other objects' save routines (`SaveGlobalData`).
/// The data handler's dword at `+0x208` (`0084c580`); `TES` calls:
/// `GetWorldSpace` (`004fd3e0`), the dwords at `+0x24` (`0059bb30`) and
/// `+0x28` (`0045cd60`, also read from a process), the size of its save
/// (`00459100`) and `TES::SaveGame` (`00459230`).
const GLOBAL_DATA_SIZE: u32 = 0x0084_c580;
const TES_GET_WORLDSPACE: u32 = 0x004f_d3e0;
const READ_FIELD_24: u32 = 0x0059_bb30;
const READ_FIELD_28: u32 = 0x0045_cd60;
const TES_SAVE_SIZE: u32 = 0x0045_9100;
const TES_SAVE: u32 = 0x0045_9230;
/// `ProcessLists`: save size (`00975450`), `SaveGame` (`009754f0`), the temp
/// effects list size (`009755b0`) and `SaveTempEffectsList` (`009756c0`).
const PROCESS_LISTS_SAVE_SIZE: u32 = 0x0097_5450;
const PROCESS_LISTS_SAVE: u32 = 0x0097_54f0;
const TEMP_EFFECTS_SIZE: u32 = 0x0097_55b0;
const TEMP_EFFECTS_SAVE: u32 = 0x0097_56c0;
/// `Sky::GetInstance` (`0046dd00`), the sky's save size (`0063e940`) and
/// `Sky::SaveGame` (`0063e9f0`); the interface's (`007065c0`, `00706610`)
/// and the regions' (`004f12b0`, `004f1300`); `SaveCreatedBaseObjects`
/// (`00861820`, (game, file)).
const SKY_INSTANCE: u32 = 0x0046_dd00;
const SKY_SAVE_SIZE: u32 = 0x0063_e940;
const SKY_SAVE: u32 = 0x0063_e9f0;
const INTERFACE_SAVE_SIZE: u32 = 0x0070_65c0;
const INTERFACE_SAVE: u32 = 0x0070_6610;
const REGIONS_SAVE_SIZE: u32 = 0x004f_12b0;
const REGIONS_SAVE: u32 = 0x004f_1300;
const SAVE_CREATED_BASE_OBJECTS: u32 = 0x0086_1820;

// The reload steps of `fn_00858af0`.
/// The dword at `+0x34` of `TES` (`005f36f0`); `00 4543c0(cell)`, which gives
/// the world object of a cell (an interior's, else the exterior one); the
/// exterior world object (`00451010`); a Havok world's `00c66300` (adds 1 to
/// its counter at `+0x18`) and `00c6b540(world, 0)` (takes 1 off).
const READ_FIELD_34: u32 = 0x005f_36f0;
const CELL_GET_PHYSICS_WORLD: u32 = 0x0045_43c0;
const GET_EXTERIOR_WORLD: u32 = 0x0045_1010;
const WORLD_ADD_LOCK: u32 = 0x00c6_6300;
const WORLD_REMOVE_LOCK: u32 = 0x00c6_b540;
/// `00459920(TES)`; `PlayerCharacter::Get3D`-like `00950bb0(player, 0)`
/// (the 3D node of the reference); `ModelLoader::QueueReference(loader,
/// reference, 0, 0)`; `IOManager::LoadQueuedPriority` and `008495b0` (sets
/// the dword at `+0x68` to 5), both on the object at `SAVE_LOCK`;
/// `00451590(byte)` (stores a byte in a global) and `0057d0a0` (seven
/// words: a position, a vector and 1.0).
const FN_00459920: u32 = 0x0045_9920;
const PLAYER_GET_3D: u32 = 0x0095_0bb0;
const MODEL_LOADER_QUEUE_REFERENCE: u32 = 0x0044_4850;
const IO_MANAGER_LOAD_QUEUED_PRIORITY: u32 = 0x0045_6520;
const IO_MANAGER_SET_STATE_5: u32 = 0x0084_95b0;
const SET_GLOBAL_FLAG: u32 = 0x0045_1590;
const FN_0057D0A0: u32 = 0x0057_d0a0;

// Strings in `.rdata` (addresses) the functions pass on.
const LABEL_TES_CLASS: u32 = 0x0108_06f8;
const LABEL_PROCESS_LISTS: u32 = 0x0108_06e4;
const LABEL_SKY: u32 = 0x0108_06d8;
const LABEL_HUD_RETICLE: u32 = 0x0108_06cc;
const LABEL_INTERFACE: u32 = 0x0107_ca80;
const LABEL_REGIONS: u32 = 0x0108_06c4;
const LABEL_GLOBAL_VARIABLES: u32 = 0x0107_f540;
const LABEL_TEMP_EFFECTS: u32 = 0x0108_073c;
const MSG_PLAYER_HAS_NO_SPACE: u32 = 0x0108_0704;
const MSG_NO_SAVE_BUFFER: u32 = 0x0108_0750;
const MSG_LOAD_FORM_SECTION: u32 = 0x0108_07fc;
const FORMAT_LOAD_ERROR: u32 = 0x0108_0780;
const MSG_ACTOR_NO_EDITOR_LOCATION: u32 = 0x0108_0818;
const MSG_CELL_REFERENCE_NO_FLAG: u32 = 0x0108_0864;
const MSG_BOUND_OBJECT_MISSING: u32 = 0x0107_fa20;
const MSG_INVALID_CREATED_TYPE: u32 = 0x0108_08a0;
const MSG_NO_CELL_OR_WORLDSPACE: u32 = 0x0108_0958;
const MSG_REFERENCE_NOT_LOADED: u32 = 0x0108_0900;
const MSG_DELETE_FORM_NOT_LOADING: u32 = 0x0108_09a0;
const MSG_NON_PERSISTENT_NO_CELL: u32 = 0x0108_0ba8;
const MSG_PERSISTENT_NO_CELL: u32 = 0x0108_0b38;
const MSG_HIGH_PROCESS_NO_CELL: u32 = 0x0108_0ae8;
const MSG_MIDDLE_HIGH_PROCESS_NO_CELL: u32 = 0x0108_0a90;
/// The name of the form's type, `table[type * 12]` of `FORM_TYPE_NAME_TABLE`
/// (`00440e30`, `this` the form).
const FORM_TYPE_NAME: u32 = 0x0044_0e30;

// ---------------------------------------------------------------------------
// Helpers

/// The allocation scope object the game keeps on its stack (`00404eb0`);
/// returns the object. `line` is the source line of the scope.
fn scope_enter(e: &mut Engine, line: u32) -> Ptr {
    let scope = Ptr::new(e.mem.alloc(4));
    e.call(SCOPE_ENTER, &args![scope, 0x11u32, 1u32, SOURCE_FILE, line]);
    scope
}

/// The allocation scope object with another first argument (`0x31` in
/// `fn_00859a90`); see `scope_enter`.
fn scope_enter_kind(e: &mut Engine, kind: u32, line: u32) -> Ptr {
    let scope = Ptr::new(e.mem.alloc(4));
    e.call(SCOPE_ENTER, &args![scope, kind, 1u32, SOURCE_FILE, line]);
    scope
}

/// `__RTDynamicCast(object, 0, source, target, 0)`: the object seen as the
/// target type, or null.
fn dynamic_cast(e: &mut Engine, object: u32, source: u32, target: u32) -> u32 {
    e.call(DYNAMIC_CAST, &args![object, 0u32, source, target, 0u32])
        .u32()
}

/// The game's `TESSaveLoadGame` through the global pointer.
fn game_singleton(e: &mut Engine) -> Ptr<TESSaveLoadGame> {
    Ptr::new(e.global(SAVE_LOAD_GAME))
}

/// Destroys the scope object (`00404ee0`) and frees its block.
fn scope_leave(e: &mut Engine, scope: Ptr) {
    e.call(SCOPE_LEAVE, &args![scope]);
    e.mem.free(scope.addr());
}

/// `operator delete(block)`.
fn delete(e: &mut Engine, block: u32) {
    e.call(OPERATOR_DELETE, &args![block]);
}

/// Whether the save/load singleton reports the operation unavailable
/// (`0047c850` on `*011de45c`); false in this build.
fn singleton_unavailable(e: &mut Engine) -> bool {
    let singleton: u32 = e.global(SAVE_LOAD_GAME);
    e.call(SAVE_LOAD_UNAVAILABLE, &args![singleton]).bool()
}

/// `0047c850` on the game the save routine was called on.
fn game_unavailable(e: &mut Engine, game: Ptr<TESSaveLoadGame>) -> bool {
    e.call(SAVE_LOAD_UNAVAILABLE, &args![game]).bool()
}

/// The key of a form: the dword at +0x0C.
fn form_key(e: &mut Engine, form: Ptr) -> u32 {
    e.call(FORM_ID, &args![form]).u32()
}

/// Deletes a `BSSimpleList` the maps own: `RemoveAll`, then the scalar
/// deleting destructor (flag 1) when the list exists.
fn destroy_list(e: &mut Engine, list: u32) {
    e.call(LIST_REMOVE_ALL, &args![list]);
    if list != 0 {
        e.call(LIST_SCALAR_DELETE, &args![list, 1u32]);
    }
}

/// Walks a map keyed by `unsigned int` with the game's iterator
/// (`GetFirstPos`, then `GetNext(&pos, &key, &value)` until the position is
/// 0), calling `visit(e, key, value)` for each entry. The position, key and
/// value cells are a 12-byte block, as in the game's stack frame.
fn for_each_entry(e: &mut Engine, map: Ptr, mut visit: impl FnMut(&mut Engine, u32, u32)) {
    let cells = e.mem.alloc(12);
    let first = e.call(MAP_FIRST_POSITION, &args![map]).u32();
    e.mem.set_u32(cells, first);
    while e.mem.u32(cells) != 0 {
        e.mem.set_u32(cells + 4, 0);
        e.mem.set_u32(cells + 8, 0);
        e.call(MAP_NEXT, &args![map, cells, cells + 4, cells + 8]);
        let (key, value) = (e.mem.u32(cells + 4), e.mem.u32(cells + 8));
        visit(e, key, value);
    }
    e.mem.free(cells);
}

// Translated from 00486a90 (decompiled, FalloutNV.exe 1.4.0.525)
/// `NiTPointerMap<unsigned int, ChangeData *>::NewItem` (Xbox PDB): gets a
/// map entry from the allocator subobject at `this + 0x0C`.
pub fn ni_t_pointer_map_unsigned_int_change_data_p_new_item(
    e: &mut Engine,
    this: Ptr<ChangesMap>,
) -> Ptr {
    e.call(MAP_ALLOCATOR_NEW_ITEM, &args![this.byte_add(0x0C)])
        .ptr()
}

// Translated from 00666050 (decompiled, FalloutNV.exe 1.4.0.525)
/// `NiTPointerMap<unsigned int, ChangeData *>::DeleteItem` (Xbox PDB):
/// clears the entry's value (`+8`) and gives the entry back to the allocator
/// subobject at `this + 0x0C`.
pub fn ni_t_pointer_map_unsigned_int_change_data_p_delete_item(
    e: &mut Engine,
    this: Ptr<ChangesMap>,
    item: Ptr,
) {
    e.mem.set_u32(item.addr() + 8, 0);
    e.call(MAP_ALLOCATOR_DELETE_ITEM, &args![this.byte_add(0x0C), item]);
}

// Translated from 00854e10 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangeData::~ChangeData`: frees the form's buffer when there is one.
pub fn fn_00854e10(e: &mut Engine, this: Ptr<ChangeData>) {
    let buffer = e.get(this, ChangeData::pBuffer);
    if !buffer.is_null() {
        delete(e, buffer.addr());
    }
}

// Translated from 00854e40 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangeData::AddFlags`: sets bits in `iFlags` unless the data already
/// carries a buffer.
pub fn fn_00854e40(e: &mut Engine, this: Ptr<ChangeData>, flags: u32) {
    if e.get(this, ChangeData::pBuffer).is_null() {
        let current = e.get(this, ChangeData::iFlags);
        e.set(this, ChangeData::iFlags, current | flags);
    }
}

// Translated from 00854e70 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangeData::RemoveFlags`: clears bits in `iFlags` unless the data
/// already carries a buffer.
pub fn fn_00854e70(e: &mut Engine, this: Ptr<ChangeData>, flags: u32) {
    if e.get(this, ChangeData::pBuffer).is_null() {
        let current = e.get(this, ChangeData::iFlags);
        e.set(this, ChangeData::iFlags, !flags & current);
    }
}

// Translated from 00854ed0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangesMap::scalar deleting destructor` (Xbox PDB): destroys the map and
/// frees it when bit 0 of `flags` is set.
pub fn changes_map_scalar_deleting_destructor(
    e: &mut Engine,
    this: Ptr<ChangesMap>,
    flags: u32,
) -> Ptr<ChangesMap> {
    fn_00854f00(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 00854f00 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangesMap::~ChangesMap`: sets the map's vtable, deletes every
/// `ChangeData`, then runs the `NiTPointerMap` base destructor (`00863640`).
pub fn fn_00854f00(e: &mut Engine, this: Ptr<ChangesMap>) {
    e.mem.set_u32(this.addr(), CHANGES_MAP_VTABLE);
    changes_map_remove_all_changes(e, this);
    e.call(CHANGES_MAP_BASE_DESTRUCT, &args![this]);
}

// Translated from 00854f60 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangesMap::RemoveAllChanges` (Xbox PDB): deletes every `ChangeData`
/// (the destructor with flag 1) and empties the map.
pub fn changes_map_remove_all_changes(e: &mut Engine, this: Ptr<ChangesMap>) {
    for_each_entry(e, this.cast(), |e, _key, value| {
        if value != 0 {
            fn_00854fe0(e, Ptr::new(value), 1);
        }
    });
    e.call(MAP_REMOVE_ALL, &args![this]);
}

// Translated from 00854fe0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ChangeData` scalar deleting destructor: frees the buffer, and the
/// `ChangeData` itself when bit 0 of `flags` is set.
pub fn fn_00854fe0(e: &mut Engine, this: Ptr<ChangeData>, flags: u32) -> Ptr<ChangeData> {
    fn_00854e10(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 00855010 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds `flags` to the `ChangeData` of `form`, creating it (flags 0, no
/// buffer) and putting it in the map when the form has none. Returns the
/// `ChangeData`.
pub fn fn_00855010(
    e: &mut Engine,
    this: Ptr<ChangesMap>,
    form: Ptr,
    flags: u32,
) -> Ptr<ChangeData> {
    let key = form_key(e, form);
    let cell = e.mem.alloc(4);
    let found = e.call(MAP_GET_AT, &args![this, key, cell]).bool();
    if !found {
        let scope = scope_enter(e, 0x104);
        let block = e.call(OPERATOR_NEW, &args![8u32]).u32();
        let change_data = if block != 0 {
            e.call(SIMPLE_LIST_CONSTRUCT, &args![block]).u32()
        } else {
            0
        };
        e.mem.set_u32(cell, change_data);
        e.call(CHANGES_MAP_SET_AT, &args![this, key, change_data]);
        scope_leave(e, scope);
    }
    let change_data: Ptr<ChangeData> = Ptr::new(e.mem.u32(cell));
    e.mem.free(cell);
    fn_00854e40(e, change_data, flags);
    change_data
}

// Translated from 00855100 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `ChangeData` stored under `key`, or null.
pub fn fn_00855100(e: &mut Engine, this: Ptr<ChangesMap>, key: u32) -> Ptr<ChangeData> {
    let cell = e.mem.alloc(4);
    e.call(MAP_GET_AT, &args![this, key, cell]);
    let found = e.mem.u32(cell);
    e.mem.free(cell);
    Ptr::new(found)
}

// Translated from 00855130 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `ChangeData` of `form` (looked up by the form's key), or null.
pub fn fn_00855130(e: &mut Engine, this: Ptr<ChangesMap>, form: Ptr) -> Ptr<ChangeData> {
    let key = form_key(e, form);
    fn_00855100(e, this, key)
}

// Translated from 00855150 (decompiled, FalloutNV.exe 1.4.0.525)
/// Clears `flags` from the `ChangeData` of `form`; when none are left it is
/// removed from the map and deleted. False when saving is unavailable or the
/// form has no `ChangeData`.
pub fn fn_00855150(e: &mut Engine, this: Ptr<ChangesMap>, form: Ptr, flags: u32) -> bool {
    if singleton_unavailable(e) {
        return false;
    }
    let change_data = fn_00855130(e, this, form);
    if change_data.is_null() {
        return false;
    }
    fn_00854e70(e, change_data, flags);
    if e.call(READ_WORD, &args![change_data]).u32() == 0 {
        let key = form_key(e, form);
        e.call(MAP_REMOVE_AT, &args![this, key]);
        if !change_data.is_null() {
            fn_00854fe0(e, change_data, 1);
        }
    }
    true
}

// Translated from 008551f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `fn_00855220` for the key of `form`.
pub fn fn_008551f0(e: &mut Engine, this: Ptr<ChangesMap>, form: Ptr, force: u8) -> bool {
    let key = form_key(e, form);
    fn_00855220(e, this, key, force)
}

// Translated from 00855220 (decompiled, FalloutNV.exe 1.4.0.525)
/// Drops the `ChangeData` stored under `key` when it has no buffer or when
/// `force` is set. False when saving is unavailable or there is none.
pub fn fn_00855220(e: &mut Engine, this: Ptr<ChangesMap>, key: u32, force: u8) -> bool {
    if singleton_unavailable(e) {
        return false;
    }
    let change_data = fn_00855100(e, this, key);
    if change_data.is_null() {
        return false;
    }
    if e.get(change_data, ChangeData::pBuffer).is_null() || force != 0 {
        e.call(MAP_REMOVE_AT, &args![this, key]);
        if !change_data.is_null() {
            fn_00854fe0(e, change_data, 1);
        }
    }
    true
}

// Translated from 008552b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `InteriorCellNewReferencesMap::InteriorCellNewReferencesMap`: the pointer
/// map base with hash size 0x25, then this class's vtable.
pub fn fn_008552b0(
    e: &mut Engine,
    this: Ptr<InteriorCellNewReferencesMap>,
) -> Ptr<InteriorCellNewReferencesMap> {
    e.call(INTERIOR_MAP_BASE_CONSTRUCT, &args![this, HASH_SIZE]);
    e.mem.set_u32(this.addr(), INTERIOR_MAP_VTABLE);
    this
}

// Translated from 008552e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `InteriorCellNewReferencesMap::scalar deleting destructor` (Xbox PDB).
pub fn interior_cell_new_references_map_scalar_deleting_destructor(
    e: &mut Engine,
    this: Ptr<InteriorCellNewReferencesMap>,
    flags: u32,
) -> Ptr<InteriorCellNewReferencesMap> {
    fn_00855310(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 00855310 (decompiled, FalloutNV.exe 1.4.0.525)
/// `InteriorCellNewReferencesMap::~InteriorCellNewReferencesMap`: deletes
/// each cell's list of new references, empties the map, runs the base
/// destructor (`00863740`).
pub fn fn_00855310(e: &mut Engine, this: Ptr<InteriorCellNewReferencesMap>) {
    e.mem.set_u32(this.addr(), INTERIOR_MAP_VTABLE);
    for_each_entry(e, this.cast(), |e, _key, list| {
        if list != 0 {
            destroy_list(e, list);
        }
    });
    e.call(MAP_REMOVE_ALL, &args![this]);
    e.call(INTERIOR_MAP_BASE_DESTRUCT, &args![this]);
}

// Translated from 008553e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExteriorCellNewReferencesMap::ExteriorCellNewReferencesMap`.
pub fn fn_008553e0(
    e: &mut Engine,
    this: Ptr<ExteriorCellNewReferencesMap>,
) -> Ptr<ExteriorCellNewReferencesMap> {
    e.call(EXTERIOR_MAP_BASE_CONSTRUCT, &args![this, HASH_SIZE]);
    e.mem.set_u32(this.addr(), EXTERIOR_MAP_VTABLE);
    this
}

// Translated from 00855410 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExteriorCellNewReferencesMap::scalar deleting destructor` (Xbox PDB).
pub fn exterior_cell_new_references_map_scalar_deleting_destructor(
    e: &mut Engine,
    this: Ptr<ExteriorCellNewReferencesMap>,
    flags: u32,
) -> Ptr<ExteriorCellNewReferencesMap> {
    fn_00855440(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 00855440 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExteriorCellNewReferencesMap::~ExteriorCellNewReferencesMap`: for each
/// cell, frees every `ExteriorCellReferenceData` in its list, then the list;
/// empties the map and runs the base destructor (`00863860`).
pub fn fn_00855440(e: &mut Engine, this: Ptr<ExteriorCellNewReferencesMap>) {
    e.mem.set_u32(this.addr(), EXTERIOR_MAP_VTABLE);
    for_each_entry(e, this.cast(), |e, _key, list| {
        if list != 0 {
            let mut node = list;
            while node != 0 {
                let item_slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
                let item = e.mem.u32(item_slot);
                if item != 0 {
                    delete(e, item);
                }
                node = e.call(LIST_NODE_NEXT, &args![node]).u32();
            }
            destroy_list(e, list);
        }
    });
    e.call(MAP_REMOVE_ALL, &args![this]);
    e.call(EXTERIOR_MAP_BASE_DESTRUCT, &args![this]);
}

// Translated from 00855550 (decompiled, FalloutNV.exe 1.4.0.525)
/// `NumericIDBufferMap::NumericIDBufferMap`.
pub fn fn_00855550(e: &mut Engine, this: Ptr<NumericIDBufferMap>) -> Ptr<NumericIDBufferMap> {
    e.call(NUMERIC_ID_MAP_BASE_CONSTRUCT, &args![this, HASH_SIZE]);
    e.mem.set_u32(this.addr(), NUMERIC_ID_MAP_VTABLE);
    this
}

// Translated from 00855580 (decompiled, FalloutNV.exe 1.4.0.525)
/// `NumericIDBufferMap::scalar deleting destructor` (Xbox PDB).
pub fn numeric_id_buffer_map_scalar_deleting_destructor(
    e: &mut Engine,
    this: Ptr<NumericIDBufferMap>,
    flags: u32,
) -> Ptr<NumericIDBufferMap> {
    fn_008555b0(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 008555b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `NumericIDBufferMap::~NumericIDBufferMap`: frees every buffer, empties
/// the map and runs the base destructor (`00863960`).
pub fn fn_008555b0(e: &mut Engine, this: Ptr<NumericIDBufferMap>) {
    e.mem.set_u32(this.addr(), NUMERIC_ID_MAP_VTABLE);
    for_each_entry(e, this.cast(), |e, _key, buffer| {
        if buffer != 0 {
            delete(e, buffer);
        }
    });
    e.call(MAP_REMOVE_ALL, &args![this]);
    e.call(NUMERIC_ID_MAP_BASE_DESTRUCT, &args![this]);
}

// Translated from 00855660 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats::SaveStats`: makes the per-type map (hash size 0x25) and the
/// empty list of extra stats.
pub fn fn_00855660(e: &mut Engine, this: Ptr<SaveStats>) -> Ptr<SaveStats> {
    let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
    let stats_map = if block != 0 {
        e.call(STATS_MAP_CONSTRUCT, &args![block, HASH_SIZE]).ptr()
    } else {
        Ptr::NULL
    };
    e.set(this, SaveStats::pStatsMap, stats_map);
    let block = e.call(OPERATOR_NEW, &args![8u32]).u32();
    let extra_stats = if block != 0 {
        e.call(SIMPLE_LIST_CONSTRUCT, &args![block]).ptr()
    } else {
        Ptr::NULL
    };
    e.set(this, SaveStats::pExtraStats, extra_stats);
    this
}

// Translated from 00855730 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats::~SaveStats`: frees the headers of every type's list and the
/// lists, deletes the map through its destructor (vtable slot 0, flag 1),
/// then frees every extra stat (description and record) and its list.
pub fn fn_00855730(e: &mut Engine, this: Ptr<SaveStats>) {
    let stats_map = e.get(this, SaveStats::pStatsMap);
    // `00863bc0(&pos, &type, &list)`: three cells, the type a byte.
    let cells = e.mem.alloc(12);
    let first = e.call(MAP_FIRST_POSITION, &args![stats_map]).u32();
    e.mem.set_u32(cells, first);
    while e.mem.u32(cells) != 0 {
        e.call(
            BYTE_MAP_NEXT,
            &args![stats_map, cells, cells + 4, cells + 8],
        );
        let list = e.mem.u32(cells + 8);
        if list != 0 {
            let mut node = list;
            while node != 0 {
                let item_slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
                let header = e.mem.u32(item_slot);
                node = e.call(LIST_NODE_NEXT, &args![node]).u32();
                if header != 0 {
                    delete(e, header);
                }
            }
            destroy_list(e, list);
        }
    }
    e.mem.free(cells);
    let stats_map = e.get(this, SaveStats::pStatsMap);
    if !stats_map.is_null() {
        e.vcall(stats_map.addr(), 0, &args![1u32]);
    }
    let extra_stats = e.get(this, SaveStats::pExtraStats);
    if !extra_stats.is_null() {
        let mut node = extra_stats.addr();
        while node != 0 {
            let item_slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
            let stat: Ptr<ExtraStat> = Ptr::new(e.mem.u32(item_slot));
            if !stat.is_null() {
                let description = e.get(stat, ExtraStat::pDescription);
                delete(e, description.addr());
                delete(e, stat.addr());
            }
            node = e.call(LIST_NODE_NEXT, &args![node]).u32();
        }
        let extra_stats = e.get(this, SaveStats::pExtraStats);
        destroy_list(e, extra_stats.addr());
    }
}

// Translated from 008558a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats::AddExtraStat` (Xbox PDB): adds a record of `size` bytes with a
/// copy of `description` to the head of the extra stats list.
pub fn save_stats_add_extra_stat(
    e: &mut Engine,
    this: Ptr<SaveStats>,
    size: u32,
    description: Ptr,
) {
    let scope = scope_enter(e, 0x267);
    let stat: Ptr<ExtraStat> = e.call(OPERATOR_NEW, &args![8u32]).ptr();
    e.set(stat, ExtraStat::iSize, size);
    let length = e.call(STRLEN, &args![description]).u32().wrapping_add(1);
    let copy = e.call(OPERATOR_NEW, &args![length]).ptr();
    e.set(stat, ExtraStat::pDescription, copy);
    e.call(STRING_COPY, &args![copy, length, description]);
    let cell = e.mem.alloc(4);
    e.mem.set_u32(cell, stat.addr());
    let extra_stats = e.get(this, SaveStats::pExtraStats);
    e.call(LIST_ADD_HEAD, &args![extra_stats, cell]);
    e.mem.free(cell);
    scope_leave(e, scope);
}

// Translated from 00855970 (decompiled, FalloutNV.exe 1.4.0.525)
/// Records a saved form's header with its saved size: builds a
/// `LoadFormHeader` from the form id, type, flags and version of `header`
/// and `size`, and adds it to the stats (`fn_00855a20`).
pub fn fn_00855970(e: &mut Engine, this: Ptr<SaveStats>, header: Ptr<SaveFormHeader>, size: u16) {
    let scope = scope_enter(e, 0x276);
    let copy: Ptr<LoadFormHeader> = Ptr::new(e.mem.alloc(LoadFormHeader::SIZE));
    e.call(LIST_NODE_ITEM, &args![copy]);
    let flags = e.get(header, SaveFormHeader::iFlags);
    let form_id = e.get(header, SaveFormHeader::iFormID);
    let form_type = e.get(header, SaveFormHeader::cFormType);
    let version = e.get(header, SaveFormHeader::cVersion);
    e.set(copy, LoadFormHeader::iFlags, flags);
    e.set(copy, LoadFormHeader::iFormID, form_id);
    e.set(copy, LoadFormHeader::cFormType, form_type);
    e.set(copy, LoadFormHeader::cVersion, version);
    e.set(copy, LoadFormHeader::iSize, size);
    fn_00855a20(e, this, copy);
    e.mem.free(copy.addr());
    scope_leave(e, scope);
}

// Translated from 00855a20 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds a heap copy of `header` to the list of its form type in the stats
/// map (creating the list when the type has none), kept sorted by
/// `fn_00855b60` (largest first).
pub fn fn_00855a20(e: &mut Engine, this: Ptr<SaveStats>, header: Ptr<LoadFormHeader>) {
    let scope = scope_enter(e, 0x286);
    let block = e.call(OPERATOR_NEW, &args![0xCu32]).u32();
    let item: Ptr<LoadFormHeader> = if block != 0 {
        e.call(LIST_NODE_ITEM, &args![block]).ptr()
    } else {
        Ptr::NULL
    };
    for word in 0..3 {
        let value = e.mem.u32(header.addr() + 4 * word);
        e.mem.set_u32(item.addr() + 4 * word, value);
    }
    let form_type = e.get(item, LoadFormHeader::cFormType);
    let stats_map = e.get(this, SaveStats::pStatsMap);
    let list_cell = e.mem.alloc(4);
    let found = e
        .call(BYTE_MAP_GET_AT, &args![stats_map, form_type, list_cell])
        .bool();
    if !found {
        let block = e.call(OPERATOR_NEW, &args![8u32]).u32();
        let list = if block != 0 {
            e.call(SIMPLE_LIST_CONSTRUCT, &args![block]).u32()
        } else {
            0
        };
        e.mem.set_u32(list_cell, list);
        e.call(BYTE_MAP_SET_AT, &args![stats_map, form_type, list]);
    }
    let list = e.mem.u32(list_cell);
    e.mem.free(list_cell);
    e.call(LIST_INSERT, &args![list, item, STATS_COMPARATOR]);
    scope_leave(e, scope);
}

// Translated from 00855b60 (decompiled, FalloutNV.exe 1.4.0.525)
/// The comparison of two `LoadFormHeader`s the stats lists are sorted by:
/// -1 when `a` is larger than `b`, 1 when smaller, 0 when equal (so the list
/// runs from the largest to the smallest).
pub fn fn_00855b60(e: &mut Engine, a: Ptr<LoadFormHeader>, b: Ptr<LoadFormHeader>) -> i32 {
    let size_a = e.get(a, LoadFormHeader::iSize) as i32;
    let size_b = e.get(b, LoadFormHeader::iSize) as i32;
    if size_a > size_b {
        -1
    } else {
        (size_a < size_b) as i32
    }
}

// Translated from 00855ba0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats::PrintStats` (Xbox PDB): writes the statistics to the text
/// file `<path><extension>` (an existing file is deleted first): the table
/// header, then for each form type a section listing every saved form of
/// that type (form id, size, flags, version, name and the string of
/// changes) followed by that type's totals, then the extra stats and the
/// grand totals. A form that is not loaded prints "NOT LOADED"; otherwise
/// its name is the location name of a map marker or the name of a reference
/// when it has one, else its description (form vtable slot `0x130`).
/// Nothing is written when the file cannot be opened.
///
/// The `strlen` the game takes of the type heading is never used and is not
/// translated; the compiler's stack-cookie check and the exception frame are
/// not translated.
pub fn save_stats_print_stats(e: &mut Engine, this: Ptr<SaveStats>, path: Ptr) {
    const PATH: u32 = 0x000; // char[0x104]
    const FILE: u32 = 0x108; // the file object, 0x20 bytes
    const TOTAL: u32 = 0x128; // Stats
    const LINE: u32 = 0x138; // char[0x208]
    const CHANGES: u32 = 0x340; // char[0x1f8]
    const TYPE_NAME: u32 = 0x538; // char[10]
    const TYPE_STATS: u32 = 0x548; // Stats
    const CURSOR: u32 = 0x558; // position, type byte, list head
    const FRAME: u32 = 0x570;

    let frame = e.mem.alloc(FRAME);
    let (path_buffer, file, total) = (frame + PATH, frame + FILE, frame + TOTAL);
    let (line, changes, type_name) = (frame + LINE, frame + CHANGES, frame + TYPE_NAME);
    let (type_stats, cursor) = (frame + TYPE_STATS, frame + CURSOR);
    let singleton: u32 = e.global(SAVE_LOAD_GAME);

    e.call(STRING_COPY, &args![path_buffer, 0x104u32, path]);
    e.call(STRING_CAT, &args![path_buffer, 0x104u32, 0x0103_9788u32]);
    if e.call(FILE_EXISTS, &args![path_buffer, 0u32, 0u32, -1i32])
        .u32()
        != 0
    {
        e.call(FILE_DELETE, &args![path_buffer]);
    }
    e.call(
        FILE_OBJECT_CONSTRUCT,
        &args![file, path_buffer, 1u32, 2u32, 0u32],
    );
    if e.call(READ_WORD, &args![file]).u32() != 0 {
        e.call(FILE_OBJECT_DESTRUCT, &args![file]);
        e.mem.free(frame);
        return;
    }

    fn_008562c0(e, Ptr::new(total));
    e.call(
        FORMAT,
        &args![
            line,
            0x208u32,
            0x0108_0670u32,
            0x0108_0414u32,
            0x0108_041cu32,
            0x0104_4aecu32,
            0x0106_3d04u32,
            0x0105_ac4cu32,
            0x0108_0428u32
        ],
    );
    fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));

    let stats_map = e.get(this, SaveStats::pStatsMap);
    let first = e.call(MAP_FIRST_POSITION, &args![stats_map]).u32();
    e.mem.set_u32(cursor, first);
    while e.mem.u32(cursor) != 0 {
        e.call(
            BYTE_MAP_NEXT,
            &args![stats_map, cursor, cursor + 4, cursor + 8],
        );
        let form_type = e.mem.u8(cursor + 4);
        let list = e.mem.u32(cursor + 8);

        // The heading: "Form" for type 0, "Buffer" for 0x79, else the type's name.
        if form_type == 0 {
            e.call(FORMAT, &args![type_name, 10u32, 0x0104_469cu32]);
        } else if form_type == 0x79 {
            e.call(FORMAT, &args![type_name, 10u32, 0x0108_0668u32]);
        } else {
            let name = e.mem.u32(FORM_TYPE_NAME_TABLE + form_type as u32 * 12);
            e.call(FORMAT, &args![type_name, 10u32, 0x0101_9f08u32, name]);
        }
        e.call(FORMAT, &args![line, 0x208u32, 0x0108_03e4u32, type_name]);
        fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));
        fn_008562c0(e, Ptr::new(type_stats));

        let mut node = list;
        while node != 0 {
            let item_slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
            let header: Ptr<LoadFormHeader> = Ptr::new(e.mem.u32(item_slot));
            if !header.is_null() {
                let form_id = e.get(header, LoadFormHeader::iFormID);
                let size = e.get(header, LoadFormHeader::iSize);
                let flags = e.get(header, LoadFormHeader::iFlags);
                let version = e.get(header, LoadFormHeader::cVersion);
                let header_type = e.get(header, LoadFormHeader::cFormType);
                let form = e.call(LOOKUP_FORM, &args![form_id]).u32();

                let stats: Ptr<Stats> = Ptr::new(type_stats);
                if size > e.get(stats, Stats::iMaxSize) {
                    e.set(stats, Stats::iMaxSize, size);
                }
                if size < e.get(stats, Stats::iMinSize) {
                    e.set(stats, Stats::iMinSize, size);
                }
                let total_size = e.get(stats, Stats::iTotalSize);
                e.set(
                    stats,
                    Stats::iTotalSize,
                    total_size.wrapping_add(size as i32),
                );
                let count = e.get(stats, Stats::iNum);
                e.set(stats, Stats::iNum, count.wrapping_add(1));

                e.call(
                    BUILD_CHANGES_STRING,
                    &[singleton, changes, form, flags, header_type as u32, 0],
                );
                let map_marker = e
                    .call(
                        DYNAMIC_CAST,
                        &args![form, 0i32, 0x0118_3028u32, 0x0118_3158u32, 0i32],
                    )
                    .u32();
                let reference = e
                    .call(
                        DYNAMIC_CAST,
                        &args![form, 0i32, 0x0118_3028u32, 0x0118_41ccu32, 0i32],
                    )
                    .u32();
                let text = if form != 0 {
                    let mut name = 0u32;
                    if reference != 0 {
                        name = e.call(REFERENCE_GET_NAME, &args![reference]).u32();
                    }
                    if map_marker != 0 && (name == 0 || is_empty_string(e, name)) {
                        name = e
                            .call(MAP_MARKER_GET_LOCATION_NAME, &args![map_marker])
                            .u32();
                    }
                    if name == 0 || is_empty_string(e, name) {
                        name = e.vcall(form, FORM_GET_DESCRIPTION, &args![]).u32();
                    }
                    name
                } else {
                    0x0108_063c // "NOT LOADED"
                };
                e.call(
                    FORMAT,
                    &args![
                        line,
                        0x208u32,
                        0x0108_0648u32,
                        form_id,
                        size as u32,
                        flags,
                        version as u32,
                        text,
                        changes
                    ],
                );
                fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));
            }
            node = e.call(LIST_NODE_NEXT, &args![node]).u32();
        }

        // This type's totals.
        let stats: Ptr<Stats> = Ptr::new(type_stats);
        let (count, total_size) = (e.get(stats, Stats::iNum), e.get(stats, Stats::iTotalSize));
        let (min_size, max_size) = (e.get(stats, Stats::iMinSize), e.get(stats, Stats::iMaxSize));
        let average = total_size as f64 / count as f64;
        e.call(
            FORMAT,
            &args![
                line,
                0x208u32,
                0x0108_05d8u32,
                type_name,
                count,
                type_name,
                total_size,
                type_name,
                min_size as u32,
                type_name,
                max_size as u32,
                type_name,
                average
            ],
        );
        fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));

        let grand: Ptr<Stats> = Ptr::new(total);
        if max_size > e.get(grand, Stats::iMaxSize) {
            e.set(grand, Stats::iMaxSize, max_size);
        }
        if min_size < e.get(grand, Stats::iMinSize) {
            e.set(grand, Stats::iMinSize, min_size);
        }
        let grand_size = e.get(grand, Stats::iTotalSize);
        e.set(
            grand,
            Stats::iTotalSize,
            grand_size.wrapping_add(total_size),
        );
        let grand_count = e.get(grand, Stats::iNum);
        e.set(grand, Stats::iNum, grand_count.wrapping_add(count));
    }

    fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(0x0108_05c4));
    let mut node = e.get(this, SaveStats::pExtraStats).addr();
    while node != 0 {
        let item_slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
        let stat: Ptr<ExtraStat> = Ptr::new(e.mem.u32(item_slot));
        if !stat.is_null() {
            let size = e.get(stat, ExtraStat::iSize);
            let description = e.get(stat, ExtraStat::pDescription);
            e.call(
                FORMAT,
                &args![line, 0x208u32, 0x0108_0254u32, size, description],
            );
            fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));
            let grand: Ptr<Stats> = Ptr::new(total);
            let grand_size = e.get(grand, Stats::iTotalSize);
            e.set(
                grand,
                Stats::iTotalSize,
                grand_size.wrapping_add(size as i32),
            );
        }
        node = e.call(LIST_NODE_NEXT, &args![node]).u32();
    }

    let grand: Ptr<Stats> = Ptr::new(total);
    let (count, total_size) = (e.get(grand, Stats::iNum), e.get(grand, Stats::iTotalSize));
    let (min_size, max_size) = (e.get(grand, Stats::iMinSize), e.get(grand, Stats::iMaxSize));
    let average = total_size as f64 / count as f64;
    e.call(
        FORMAT,
        &args![
            line,
            0x208u32,
            0x0108_0570u32,
            count,
            total_size,
            min_size as u32,
            max_size as u32,
            average
        ],
    );
    fn_00856300(e, this.cast(), Ptr::new(file), Ptr::new(line));
    e.call(FILE_OBJECT_DESTRUCT, &args![file]);
    e.mem.free(frame);
}

/// `strcmp(name, "") == 0` against the game's empty string (`01011584`).
fn is_empty_string(e: &mut Engine, name: u32) -> bool {
    e.call(STRING_COMPARE, &args![name, 0x0101_1584u32]).i32() == 0
}

// Translated from 008562c0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats::Stats::Stats`: no entries, total 0, minimum 0xFFFF, maximum 0.
pub fn fn_008562c0(e: &mut Engine, this: Ptr<Stats>) -> Ptr<Stats> {
    e.set(this, Stats::iMaxSize, 0);
    e.set(this, Stats::iTotalSize, 0);
    e.set(this, Stats::iNum, 0);
    e.set(this, Stats::iMinSize, 0xFFFF);
    this
}

// Translated from 00856300 (decompiled, FalloutNV.exe 1.4.0.525)
/// Writes the text (without its terminator) to `file` through
/// `BSSystemFile::DoWrite`; true when the write reported no error. `this` is
/// not used.
pub fn fn_00856300(e: &mut Engine, _this: Ptr, file: Ptr, text: Ptr) -> bool {
    let length = e.call(STRLEN, &args![text]).u32();
    let scratch = e.mem.alloc(16);
    let error = e
        .call(
            SYSTEM_FILE_DO_WRITE,
            &args![file, text, length, 0u32, scratch],
        )
        .u32();
    e.mem.free(scratch);
    error == 0
}

// Translated from 00856c70 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::RemoveChanges` (Xbox PDB): unless the form is flagged
/// deleted (bit 0x4000 of its flags), drops its `ChangeData`
/// (`fn_008551f0`).
pub fn tes_save_load_game_remove_changes(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    form: Ptr,
    force: u8,
) {
    if !e.call(FORM_IS_DELETED, &args![form]).bool() {
        let changes = e.get(this, TESSaveLoadGame::m_pChanges);
        fn_008551f0(e, changes, form, force);
    }
}

// Translated from 00856ca0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Saves the game (the routine behind "sCantSaveNow"): refuses, with the
/// sad-Vault-Boy message, unless saving is allowed or the save is an
/// autosave; otherwise takes the save lock, opens the file (`file` is an
/// existing stream or null, `name` the save's name), writes the header, the
/// plugin list and the global data, then for every `ChangeData` of the
/// changes map the 10-byte form header, the size and the changes (the
/// pre-built buffer when the data has one, else the form's own initial data
/// and changes), the final data and the numeric id arrays, patches the
/// positions written at the start, prints the statistics when
/// `collect_stats` asked for them, and closes the file. True when it saved.
pub fn fn_00856ca0(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    file: Ptr,
    name: Ptr,
    collect_stats: bool,
) -> bool {
    // The frame cells the game keeps on its stack: the form header, the size
    // words, the reserved word, the end position and the form count.
    const HEADER: u32 = 0x00;
    const BUFFER_SIZE: u32 = 0x10; // the 4 bytes `READ_BYTES` fills
    const FORM_SIZE: u32 = 0x14; // u16
    const RESERVED: u32 = 0x18;
    const END_POSITION: u32 = 0x1C;
    const COUNT: u32 = 0x20;
    const FRAME: u32 = 0x24;

    let scope = scope_enter(e, 0x466);
    let allowed = game_unavailable(e, this)
        || e.call(GET_SAVING_ALLOWED, &args![this]).bool()
        || (!name.is_null() && e.call(STRING_COMPARE, &args![name, AUTOSAVE_NAME]).i32() == 0);
    if !allowed {
        let queue = e
            .call(GET_MESSAGE_QUEUE, &args![MESSAGE_QUEUE_OBJECT])
            .u32();
        let time: f32 = e.global(MESSAGE_TIME);
        e.call(
            SHOW_MESSAGE,
            &args![queue, 0u32, SAD_ICON, 0u32, time, 0u32],
        );
        scope_leave(e, scope);
        return false;
    }

    let lock: u32 = e.global(SAVE_LOCK);
    e.call(SAVE_LOCK_ENTER, &args![lock]);
    e.call(SAVE_PREPARE_A, &args![this]);
    let mut stream = Ptr::NULL;
    if !game_unavailable(e, this) {
        stream = e.call(OPEN_SAVE_FILE, &args![this, file, name, 0u32]).ptr();
    }
    if collect_stats {
        let block = e.call(OPERATOR_NEW, &args![8u32]).u32();
        let stats = if block != 0 {
            fn_00855660(e, Ptr::new(block))
        } else {
            Ptr::NULL
        };
        e.set(this, TESSaveLoadGame::m_pSaveLoadStats, stats);
    }
    e.call(SAVE_PREPARE_B, &args![this]);
    e.call(SAVE_PREPARE_C, &args![this]);
    e.call(SAVE_HEADER, &args![this, stream, name]);
    e.call(SAVE_PLUGIN_LIST, &args![this, stream]);

    let frame = e.mem.alloc(FRAME);
    let header: Ptr<SaveFormHeader> = Ptr::new(frame + HEADER);
    let mut start_position = 0u32;
    if !game_unavailable(e, this) {
        start_position = e.call(FILE_POSITION, &args![stream]).u32();
    }
    e.mem.set_u32(frame + RESERVED, 0);
    e.call(WRITE_BYTES, &args![this, stream, frame + RESERVED, 4u32]);
    e.call(WRITE_BYTES, &args![this, stream, frame + RESERVED, 4u32]);
    e.call(SAVE_GLOBAL_DATA, &args![this, stream]);
    e.mem.set_u32(frame + COUNT, 0);

    let singleton: u32 = e.global(SAVE_LOAD_GAME);
    let changes = e.get(this, TESSaveLoadGame::m_pChanges);
    for_each_entry(e, changes.cast(), |e, form_id, change_data| {
        if form_id == 0 || change_data == 0 {
            return;
        }
        let buffer = e.call(LIST_NODE_NEXT, &args![change_data]).u32();
        e.set(this, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
        let flags = e.call(READ_WORD, &args![change_data]).u32();
        e.call(LIST_NODE_ITEM, &args![header]);
        e.set(header, SaveFormHeader::iFormID, form_id);
        e.set(header, SaveFormHeader::iFlags, flags);
        let version = e.call(CURRENT_VERSION, &args![this]).u8();
        e.set(header, SaveFormHeader::cVersion, version);

        if buffer != 0 {
            // The data has its own buffer: its first four bytes are the
            // size, the type and the version.
            e.call(READ_BYTES, &args![singleton, frame + BUFFER_SIZE, 4u32]);
            let form_type = e.mem.u8(frame + BUFFER_SIZE + 2);
            let version = e.mem.u8(frame + BUFFER_SIZE + 3);
            e.set(header, SaveFormHeader::cFormType, form_type);
            e.set(header, SaveFormHeader::cVersion, version);
            e.call(WRITE_BYTES, &args![singleton, stream, header, 10u32]);
            let count = e.mem.u32(frame + COUNT);
            e.mem.set_u32(frame + COUNT, count.wrapping_add(1));
            e.call(WRITE_BYTES, &args![this, stream, frame + BUFFER_SIZE, 2u32]);
            let size = e.mem.u16(frame + BUFFER_SIZE);
            if size != 0 {
                let saved = e.get(this, TESSaveLoadGame::m_pBuffer);
                e.call(WRITE_BYTES, &args![this, stream, saved, size as u32]);
            }
            let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
            if !stats.is_null() {
                fn_00855970(e, stats, header, size);
            }
            e.set(this, TESSaveLoadGame::m_pBuffer, Ptr::NULL);
        } else {
            let form = e.call(LOOKUP_FORM, &args![form_id]).u32();
            if form == 0 {
                return;
            }
            let form_type = e.call(FORM_TYPE, &args![form]).u8();
            e.set(header, SaveFormHeader::cFormType, form_type);
            let flags = e.get(header, SaveFormHeader::iFlags);
            let flags = e.call(CHECK_FLAGS, &args![this, form, flags]).u32();
            e.set(header, SaveFormHeader::iFlags, flags);
            e.call(WRITE_BYTES, &args![singleton, stream, header, 10u32]);
            let count = e.mem.u32(frame + COUNT);
            e.mem.set_u32(frame + COUNT, count.wrapping_add(1));
            fn_00857230(e, this, header.cast());
            let mut size = e.vcall(form, FORM_GET_CHANGES_SIZE, &args![flags]).u16();
            let initial = e
                .call(GET_INITIAL_DATA_SAVE_SIZE, &args![this, form, flags])
                .u16();
            size = size.wrapping_add(initial);
            e.mem.set_u16(frame + FORM_SIZE, size);
            e.call(WRITE_BYTES, &args![this, stream, frame + FORM_SIZE, 2u32]);
            if size != 0 {
                let save_buffer = e.call(CREATE_BUFFER, &args![this, size as u32]).u32();
                e.call(SAVE_INITIAL_DATA, &args![this, form, flags]);
                e.vcall(form, FORM_SAVE_CHANGES, &args![flags]);
                e.call(WRITE_FILE, &args![this, stream, save_buffer, size as u32]);
                e.call(FREE_BUFFER, &args![this, save_buffer]);
            }
            fn_00857230(e, this, Ptr::NULL);
            let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
            if !stats.is_null() {
                fn_00855970(e, stats, header, size);
            }
        }
    });

    e.call(SAVE_FINAL_DATA, &args![this, stream]);
    e.mem.set_u32(frame + END_POSITION, 0);
    if !game_unavailable(e, this) {
        let position = e.call(FILE_POSITION, &args![stream]).u32();
        e.mem.set_u32(frame + END_POSITION, position);
    }
    e.call(SAVE_NUMERIC_ID_ARRAYS, &args![this, stream]);
    if !game_unavailable(e, this) {
        let mode: u32 = e.global(SEEK_MODE);
        e.vcall(stream.addr(), FILE_SEEK, &args![start_position, mode]);
        e.call(
            WRITE_BYTES,
            &args![this, stream, frame + END_POSITION, 4u32],
        );
        e.call(WRITE_BYTES, &args![this, stream, frame + COUNT, 4u32]);
    }
    e.mem.free(frame);

    let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
    if !stats.is_null() {
        let stream_name = e.vcall(stream.addr(), FILE_GET_NAME, &args![]).u32();
        save_stats_print_stats(e, stats, Ptr::new(stream_name));
        let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
        if !stats.is_null() {
            fn_00857250(e, stats, 1);
        }
        e.set(this, TESSaveLoadGame::m_pSaveLoadStats, Ptr::NULL);
    }
    if !game_unavailable(e, this) {
        fn_00857210(e, stream);
        e.call(SAVE_CLOSE_A, &args![this, stream]);
        e.call(SAVE_CLOSE_B, &args![this, stream, 0u32]);
    }
    e.call(SAVE_LOCK_LEAVE, &args![lock]);
    scope_leave(e, scope);
    true
}

// Translated from 00857210 (decompiled, FalloutNV.exe 1.4.0.525)
/// Ends the save's use of the file through `00aa15a0`.
pub fn fn_00857210(e: &mut Engine, this: Ptr) {
    e.call(FILE_FLUSH, &args![this]);
}

// Translated from 00857230 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets `m_pCurrentlySavingFormHeader`.
pub fn fn_00857230(e: &mut Engine, this: Ptr<TESSaveLoadGame>, header: Ptr) {
    e.set(this, TESSaveLoadGame::m_pCurrentlySavingFormHeader, header);
}

// Translated from 00857250 (decompiled, FalloutNV.exe 1.4.0.525)
/// `SaveStats` scalar deleting destructor: runs `fn_00855730`, frees the
/// object when bit 0 of `flags` is set.
pub fn fn_00857250(e: &mut Engine, this: Ptr<SaveStats>, flags: u32) -> Ptr<SaveStats> {
    fn_00855730(e, this);
    if flags & 1 != 0 {
        delete(e, this.addr());
    }
    this
}

// Translated from 00857280 (decompiled, FalloutNV.exe 1.4.0.525)
/// `FormAndFlags::FormAndFlags(form, flags, oldFlags, version)`.
pub fn fn_00857280(
    e: &mut Engine,
    this: Ptr<FormAndFlags>,
    form: Ptr,
    flags: u32,
    old_flags: u32,
    version: u8,
) -> Ptr<FormAndFlags> {
    e.set(this, FormAndFlags::pForm, form);
    e.set(this, FormAndFlags::iFlags, flags);
    e.set(this, FormAndFlags::iOldFlags, old_flags);
    e.set(this, FormAndFlags::cVersion, version);
    this
}

// Translated from 008572c0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `CreatedReferenceData` default constructor (Xbox PDB layout): the type
/// and the bound id are 0, then the `ReferenceData` embedded at +8 is
/// constructed (`fn_008572f0`).
pub fn fn_008572c0(e: &mut Engine, this: Ptr) -> Ptr {
    e.mem.set_u32(this.addr(), 0);
    e.mem.set_u32(this.addr() + 4, 0);
    e.call(EMBEDDED_RECORD_INIT, &args![this.byte_add(8)]);
    this
}

// Translated from 008572f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ReferenceData` default constructor (Xbox PDB layout): the location id
/// is 0 and the two embedded `NiPoint3` (their constructor, `006815c0`, does
/// nothing) are left alone.
pub fn fn_008572f0(e: &mut Engine, this: Ptr<ReferenceData>) -> Ptr<ReferenceData> {
    e.set(this, ReferenceData::iLocationID, 0);
    e.call(LIST_NODE_ITEM, &args![this.byte_add(4)]);
    e.call(LIST_NODE_ITEM, &args![this.byte_add(0x10)]);
    this
}

// Translated from 00857320 (decompiled, FalloutNV.exe 1.4.0.525)
/// `MovedReferenceData` default constructor (Xbox PDB layout): the original
/// location id is 0, the original position (+4) is left alone and the
/// `ReferenceData` at +0x10 is constructed.
pub fn fn_00857320(e: &mut Engine, this: Ptr<MovedReferenceData>) -> Ptr<MovedReferenceData> {
    e.set(this, MovedReferenceData::iOriginalLocationID, 0);
    e.call(LIST_NODE_ITEM, &args![this.byte_add(4)]);
    fn_008572f0(e, Ptr::new(this.addr() + 0x10));
    this
}

// Translated from 00857350 (decompiled, FalloutNV.exe 1.4.0.525)
/// The destructor of the `NiTLargePrimitiveArray<FormAndFlags *>` at
/// `TESSaveLoadGame + 0x20` (the engine map names the folded body
/// `~basic_streambuf<>`): runs `00863d60` on the object.
pub fn fn_00857350(e: &mut Engine, this: Ptr) {
    e.call(INIT_ARRAY_DESTRUCT, &args![this]);
}

// Translated from 00857370 (decompiled, FalloutNV.exe 1.4.0.525)
/// Opens, or hands back, the file a save or load works on. `mode` 0 opens
/// the save for writing (rotating the previous save to `.bak` first; when
/// `file` is given it is closed and deleted and a fresh save is opened, in
/// a name that keeps only "autosave" names), 1 opens `name` (or the default
/// name) for reading, 2 re-opens `file`, 3 and anything else return `file`.
/// `name` null means the default save name (`00860ae0`). Not translated: the
/// exception frame and the stack cookie.
pub fn fn_00857370(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    file: Ptr,
    name: Ptr,
    mode: u32,
) -> Ptr {
    // The character buffers the game keeps on its stack.
    const PATH: u32 = 0x000;
    const NAME: u32 = 0x104;
    const DIRECTORY: u32 = 0x208;
    const CURRENT_PATH: u32 = 0x30C;
    const BACKUP_PATH: u32 = 0x410;
    const TRIMMED: u32 = 0x514;
    const FRAME: u32 = 0x618;
    // The number of previous saves the rotation keeps: a constant stored in
    // a local, which the code clamps to 10.
    const KEPT_BACKUPS: i32 = 1;

    let frame = e.mem.alloc(FRAME);
    let (path, name_buf, directory, current_path, backup_path, trimmed) = (
        frame + PATH,
        frame + NAME,
        frame + DIRECTORY,
        frame + CURRENT_PATH,
        frame + BACKUP_PATH,
        frame + TRIMMED,
    );
    if !file.is_null() {
        let file_name = e.vcall(file.addr(), FILE_GET_NAME, &args![]).u32();
        e.call(STRING_COPY, &args![path, 0x104u32, file_name]);
    } else {
        if name.is_null() {
            e.call(DEFAULT_SAVE_NAME, &args![this, name_buf]);
        } else {
            e.call(STRING_COPY, &args![name_buf, 0x104u32, name]);
        }
        // "<base><folder><name>.ess"
        let middle = save_folder(e);
        let prefix = e.call(PATH_PREFIX, &args![]).u32();
        e.call(
            FORMAT,
            &args![path, 0x104u32, FORMAT_SAVE_PATH, prefix, middle, name_buf],
        );
        if mode == 0 {
            // Make the folder and rotate the previous save to ".bak".
            let prefix = e.call(PATH_PREFIX, &args![]).u32();
            e.call(STRING_COPY, &args![directory, 0x104u32, prefix]);
            let middle = save_folder(e);
            e.call(STRING_CAT, &args![directory, 0x104u32, middle]);
            e.call(CREATE_DIRECTORY_IMPORT, &args![directory, 0u32]);
            let mut count = KEPT_BACKUPS;
            if count > 10 {
                count = 10;
            }
            let mut index = count - 1;
            while index >= 0 {
                let prefix = e.call(PATH_PREFIX, &args![]).u32();
                e.call(STRING_COPY, &args![current_path, 0x104u32, prefix]);
                let middle = save_folder(e);
                e.call(STRING_CAT, &args![current_path, 0x104u32, middle]);
                e.call(STRING_CAT, &args![current_path, 0x104u32, name_buf]);
                for _ in 0..index {
                    e.call(STRING_CAT, &args![current_path, 0x104u32, BAK_EXTENSION]);
                }
                e.call(STRING_COPY, &args![backup_path, 0x104u32, current_path]);
                e.call(STRING_CAT, &args![backup_path, 0x104u32, BAK_EXTENSION]);
                if index == 0 {
                    e.call(STRING_COPY, &args![current_path, 0x104u32, path]);
                }
                let exists = e
                    .call(
                        FILE_EXISTS,
                        &args![current_path, 0u32, 0u32, 0xFFFF_FFFFu32],
                    )
                    .u32();
                if exists != 0 {
                    let old_exists = e
                        .call(FILE_EXISTS, &args![backup_path, 0u32, 0u32, 0xFFFF_FFFFu32])
                        .u32();
                    if old_exists != 0 {
                        e.call(DELETE_FILE_IMPORT, &args![backup_path]);
                    }
                    e.call(RENAME, &args![current_path, backup_path]);
                }
                index -= 1;
            }
        }
    }

    let result = match mode {
        0 => {
            if !file.is_null() {
                // Re-save: close and delete the stream, then open the save
                // again under its own name (kept only for "autosave").
                let file_name = e.vcall(file.addr(), FILE_GET_NAME, &args![]).u32();
                let last = e.call(STRRCHR, &args![file_name, 0x5Cu32]).u32();
                let leaf = last.wrapping_add(1);
                e.call(STRING_COPY, &args![trimmed, 0x104u32, leaf]);
                let length = e.call(STRLEN, &args![trimmed]).u32();
                if length > 4 {
                    let tail = trimmed + length - 4;
                    if e.call(STRNICMP, &args![tail, ESS_EXTENSION, 4u32]).i32() == 0 {
                        e.mem.set_u8(tail, 0);
                    }
                }
                fn_00857950(e, this, file, Ptr::NULL);
                let not_autosave_name = e
                    .call(STRNICMP, &args![trimmed, SAVE_NAME_PREFIX, 5u32])
                    .i32()
                    == 0
                    || e.call(STRING_COMPARE_NOCASE, &args![trimmed, AUTOSAVE_NAME])
                        .i32()
                        != 0;
                if not_autosave_name {
                    fn_00857370(e, this, Ptr::NULL, Ptr::NULL, 0)
                } else {
                    fn_00857370(e, this, Ptr::NULL, Ptr::new(trimmed), 0)
                }
            } else {
                open_bs_file(e, path, 1, false)
            }
        }
        1 => open_bs_file(e, path, 0, true),
        2 => {
            e.vcall(file.addr(), FILE_OPEN, &args![0u32, 0u32]);
            file
        }
        _ => file,
    };
    e.mem.free(frame);
    result
}

/// The folder part of a save's path: `00464f30(00403df0(PATH_OBJECT))`.
fn save_folder(e: &mut Engine) -> u32 {
    let text = e.call(PATH_OBJECT_GET, &args![PATH_OBJECT]).u32();
    e.call(IDENTITY, &args![text]).u32()
}

/// `new BSFile(path, write_mode, 0x20000, 0)`, then (for `open`) its
/// virtual slot `0x20` with two zeros. Null when the allocation fails.
fn open_bs_file(e: &mut Engine, path: u32, write_mode: u32, open: bool) -> Ptr {
    let block = e.call(OPERATOR_NEW, &args![BSFILE_SIZE]).u32();
    let file = if block != 0 {
        e.call(
            BSFILE_CONSTRUCT,
            &args![block, path, write_mode, 0x20000u32, 0u32],
        )
        .ptr()
    } else {
        Ptr::NULL
    };
    if open {
        e.vcall(file.addr(), FILE_OPEN, &args![0u32, 0u32]);
    }
    file
}

// Translated from 008578b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Lets go of a save file: for `mode` 0, 1 and 3 it is removed from the
/// game's list of save files (when there is one) and destroyed through its
/// virtual destructor; for `mode` 2 it is closed (`BSFile::Close`). Null
/// files and other modes do nothing.
pub fn fn_008578b0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, file: Ptr, mode: u32) {
    if file.is_null() {
        return;
    }
    match mode {
        0 | 1 | 3 => {
            let list = e.get(this, TESSaveLoadGame::m_pSaveGameList);
            if !list.is_null() {
                let cell = e.mem.alloc(4);
                e.mem.set_u32(cell, file.addr());
                e.call(LIST_REMOVE, &args![list, cell]);
                e.mem.free(cell);
            }
            e.vcall(file.addr(), FILE_DESTRUCT, &args![1u32]);
        }
        2 => {
            e.call(BSFILE_CLOSE, &args![file]);
        }
        _ => {}
    }
}

// Translated from 00857950 (decompiled, FalloutNV.exe 1.4.0.525)
/// Deletes a save file from disk: gets the stream for `file` (through
/// `fn_00857370` with mode 3, which returns it), deletes the file it names
/// and lets go of the stream.
pub fn fn_00857950(e: &mut Engine, this: Ptr<TESSaveLoadGame>, file: Ptr, name: Ptr) {
    if file.is_null() {
        return;
    }
    let mut stream = fn_00857370(e, this, file, name, 3);
    if stream.is_null() {
        stream = file;
    }
    let path = e.vcall(stream.addr(), FILE_GET_NAME, &args![]).u32();
    e.call(FILE_DELETE, &args![path]);
    fn_008578b0(e, this, stream, 3);
}

// Translated from 008579b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Copies `size` bytes from `data` into the current buffer and moves the
/// buffer pointer on (`memcpy` then `fn_00857bd0`).
pub fn fn_008579b0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, data: Ptr, size: u32) {
    let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
    e.call(MEMCPY, &args![buffer, data, size]);
    fn_00857bd0(e, this, size);
}

// Translated from 008579e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Copies `size` bytes from the current buffer into `data` and moves the
/// buffer pointer on.
pub fn fn_008579e0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, data: Ptr, size: u32) {
    let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
    e.call(MEMCPY, &args![data, buffer, size]);
    fn_00857bd0(e, this, size);
}

// Translated from 00857a10 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::SaveNumericID` (Xbox PDB): writes `size / 4` form ids
/// into the buffer; when the game uses the numeric id array each id goes
/// through `AddNumericIDToArray` first.
pub fn tes_save_load_game_save_numeric_id(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    ids: Ptr,
    size: u32,
) {
    let count = size >> 2;
    let cell = e.mem.alloc(4);
    for index in 0..count {
        let id = e.mem.u32(ids.addr() + index * 4);
        let value = if e.call(USE_NUMERIC_IDS, &args![this]).bool() {
            e.call(ADD_NUMERIC_ID, &args![this, id]).u32()
        } else {
            id
        };
        e.mem.set_u32(cell, value);
        let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
        e.call(MEMCPY, &args![buffer, cell, 4u32]);
        fn_00857bd0(e, this, 4);
    }
    e.mem.free(cell);
}

// Translated from 00857aa0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::LoadNumericID` (Xbox PDB): reads `size` bytes of ids
/// from the buffer into `ids`; when the game uses the numeric id array each
/// id is turned back into a form id. True when a non-zero id could not be
/// resolved.
pub fn tes_save_load_game_load_numeric_id(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    ids: Ptr,
    size: u32,
) -> bool {
    let mut unresolved = false;
    let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
    e.call(MEMCPY, &args![ids, buffer, size]);
    if e.call(USE_NUMERIC_IDS, &args![this]).bool() {
        for index in 0..(size >> 2) {
            let slot = ids.addr() + index * 4;
            let resolved = e
                .call(RESOLVE_NUMERIC_ID, &args![this, e.mem.u32(slot)])
                .u32();
            if e.mem.u32(slot) != 0 && resolved == 0 {
                unresolved = true;
            }
            e.mem.set_u32(slot, resolved);
        }
    }
    fn_00857bd0(e, this, size);
    unresolved
}

// Translated from 00857b50 (decompiled, FalloutNV.exe 1.4.0.525)
/// Writes `size` bytes at `data` to the save file (`TESFile`'s write,
/// `00473180`, on `file`), or, when the game only measures a save
/// (`0047c850` is true), adds `size` to `m_iSimulationFileSize`. Returns the
/// size or what the file's write returns.
pub fn fn_00857b50(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    file: Ptr,
    data: Ptr,
    size: u32,
) -> u32 {
    if game_unavailable(e, this) {
        let counted = e.get(this, TESSaveLoadGame::m_iSimulationFileSize);
        e.set(
            this,
            TESSaveLoadGame::m_iSimulationFileSize,
            counted.wrapping_add(size),
        );
        size
    } else {
        e.call(FILE_WRITE, &args![file, data, size]).u32()
    }
}

// Translated from 00857ba0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Reads `size` bytes from the file into `buffer` (`00462d80` on `file`);
/// returns what the read returns.
pub fn fn_00857ba0(e: &mut Engine, _this: Ptr, file: Ptr, buffer: Ptr, size: u32) -> u32 {
    e.call(FILE_READ, &args![file, buffer, size]).u32()
}

// Translated from 00857bd0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Moves the buffer pointer on by `count` bytes.
pub fn fn_00857bd0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, count: u32) {
    let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
    e.set(
        this,
        TESSaveLoadGame::m_pBuffer,
        Ptr::new(buffer.addr().wrapping_add(count)),
    );
}

// Translated from 00857bf0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Maps a saved form id to the current one: the top byte (the plugin
/// index) is looked up in `m_pFileIndexArray` (`m_iSavedPluginCount`
/// entries). Without the table, or for plugin index 0xFF, the id is
/// unchanged; an index outside the table, or mapped to 0xFF, gives 0.
pub fn fn_00857bf0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, id: u32) -> u32 {
    let plugin = (id >> 24) as u8;
    let table = e.get(this, TESSaveLoadGame::m_pFileIndexArray);
    if table.is_null() || plugin == 0xFF {
        return id;
    }
    let count = e.get(this, TESSaveLoadGame::m_iSavedPluginCount);
    if plugin < count {
        let mapped = e.mem.u8(table.addr() + plugin as u32);
        if mapped != 0xFF {
            return (id & 0x00FF_FFFF).wrapping_add((mapped as u32) << 24);
        }
    }
    0
}

// Translated from 00857c70 (decompiled, FalloutNV.exe 1.4.0.525)
/// The inverse of `fn_00857bf0`: finds the plugin index (the last one) whose
/// table entry is the id's top byte and puts that index in the top byte.
/// Without the table, or for plugin index 0xFF, the id is unchanged; a byte
/// that is not in the table gives 0.
pub fn fn_00857c70(e: &mut Engine, this: Ptr<TESSaveLoadGame>, id: u32) -> u32 {
    let plugin = (id >> 24) as u8;
    let table = e.get(this, TESSaveLoadGame::m_pFileIndexArray);
    if table.is_null() || plugin == 0xFF {
        return id;
    }
    let count = e.get(this, TESSaveLoadGame::m_iSavedPluginCount);
    let mut found = 0xFFu8;
    for index in 0..count as u32 {
        if e.mem.u8(table.addr() + index) == plugin {
            found = index as u8;
        }
    }
    if found != 0xFF {
        (id & 0x00FF_FFFF).wrapping_add((found as u32) << 24)
    } else {
        0
    }
}

// Translated from 00857d10 (decompiled, FalloutNV.exe 1.4.0.525)
/// Puts a loaded reference where its saved data says it is. `this` is the
/// load form buffer (`BGSLoadFormBuffer`, whose version byte at +0x1C is
/// cleared during the call and restored). For an actor-like reference with a
/// saved cell or world space (virtual slots `0x290`, `0x298`, `0x294`) it
/// sets the position and the angle from the actor's own (virtual slots
/// `0x170`, `0x16C`) and moves the reference into that space; otherwise it
/// uses the reference's extra data (type 0xF: the same position and rotation;
/// and the starting world space or cell). When it moved the reference and
/// the reference has a 3D node it also puts the node, its character
/// controller and its collision at the new position. `teleport` brackets
/// the move with `004534f0(this, 1)` and `004534f0(this, 0)`. Returns
/// whether the reference was moved.
pub fn fn_00857d10(e: &mut Engine, this: Ptr, reference: Ptr, teleport: bool) -> bool {
    let version = e.call(BUFFER_GET_VERSION, &args![this]).u8();
    e.call(BUFFER_SET_VERSION, &args![this, 0u32]);
    let mut moved = false;
    let mut actor = 0u32;
    if e.vcall(reference.addr(), REFERENCE_IS_ACTOR, &args![])
        .bool()
    {
        actor = reference.addr();
    }
    let located = actor != 0 && e.vcall(actor, ACTOR_HAS_LOCATION_SLOT, &args![]).bool();
    if located {
        let cell = e.vcall(actor, ACTOR_CELL_SLOT, &args![]).u32();
        let worldspace = e.vcall(actor, ACTOR_WORLDSPACE_SLOT, &args![]).u32();
        if cell != 0 || worldspace != 0 {
            let out = e.mem.alloc(0x18);
            let position = e
                .vcall(actor, REFERENCE_GET_LOCATION_SLOT, &args![out])
                .u32();
            e.call(REF_SET_POSITION, &args![reference, position]);
            let rotation = e
                .vcall(actor, REFERENCE_GET_ROTATION_SLOT, &args![out + 0xC])
                .u32();
            let angle_z = e.mem.f32(rotation + 8);
            e.call(FN_005757D0, &args![reference, angle_z]);
            e.mem.free(out);
            if teleport {
                e.call(SET_LOADING_STATE, &args![this, 1u32]);
            }
            e.call(REF_MOVE_TO_SPACE, &args![reference, cell, worldspace]);
            if teleport {
                e.call(SET_LOADING_STATE, &args![this, 0u32]);
            }
            moved = true;
        }
    } else if e.call(REF_GET_EXTRA_LIST, &args![reference]).u32() != 0 {
        let list = e.call(REF_GET_EXTRA_LIST, &args![reference]).u32();
        let data = e.call(EXTRA_GET_DATA, &args![list, 0xFu32]).u32();
        if data != 0 {
            let out = e.mem.alloc(0x18);
            e.vcall(reference.addr(), REFERENCE_GET_LOCATION_SLOT, &args![out]);
            e.vcall(
                reference.addr(),
                REFERENCE_GET_ROTATION_SLOT,
                &args![out + 0xC],
            );
            e.call(REF_SET_POSITION, &args![reference, out]);
            let (x, y, z) = (
                e.mem.u32(out + 0xC),
                e.mem.u32(out + 0x10),
                e.mem.u32(out + 0x14),
            );
            e.call(FN_00575700, &args![reference, x, y, z]);
            e.mem.free(out);
            moved = true;
        }
        let list = e.call(REF_GET_EXTRA_LIST, &args![reference]).u32();
        let space = e.call(EXTRA_GET_STARTING_SPACE, &args![list]).u32();
        if space != 0 {
            let cell = dynamic_cast(e, space, RTTI_FORM, RTTI_CELL);
            let worldspace = dynamic_cast(e, space, RTTI_FORM, RTTI_WORLDSPACE);
            if cell != 0 || worldspace != 0 {
                e.call(REF_MOVE_TO_SPACE, &args![reference, cell, worldspace]);
                moved = true;
            }
        }
    }

    if moved {
        let node = e.call(REF_GET_NODE, &args![reference]).u32();
        if node != 0 {
            let position_ptr = e
                .vcall(reference.addr(), REFERENCE_GET_POSITION_SLOT, &args![])
                .u32();
            let position = e.mem.alloc(0x0C);
            let words = e.mem.bytes(position_ptr, 0x0C);
            e.mem.write(position, &words);
            let mobile = dynamic_cast(e, reference.addr(), RTTI_REFERENCE, RTTI_MOBILE_OBJECT);
            if mobile != 0 {
                let controller = e.call(MOBILE_GET_CHAR_CONTROLLER, &args![mobile]).u32();
                if controller != 0 && !e.call(CHAR_CONTROLLER_TEST, &args![controller]).bool() {
                    e.call(CHAR_CONTROLLER_SET_POSITION, &args![controller, position]);
                }
            }
            e.call(FN_00440460, &args![node, position]);
            let matrix = e.mem.alloc(0x24);
            let orientation = e.call(REF_GET_ORIENTATION, &args![reference, matrix]).u32();
            e.call(FN_0043FA80, &args![node, orientation]);
            e.call(COLLISION_RESET_SIM, &args![node, 1u32]);
            let transform = e.mem.alloc(0x0C);
            e.call(FN_0043D410, &args![transform, 0.0f32, 0u32, 0u32]);
            e.call(FN_00A59C60, &args![node, transform]);
            e.mem.free(transform);
            e.mem.free(matrix);
            e.mem.free(position);
        }
    }
    e.call(BUFFER_SET_VERSION, &args![this, version as u32]);
    moved
}

/// Writes `bytes` (a local the game keeps on its stack) to the save file
/// through `fn_00857b50`.
fn put_bytes(e: &mut Engine, game: Ptr<TESSaveLoadGame>, file: Ptr, bytes: &[u8]) {
    let cell = e.mem.alloc(bytes.len() as u32);
    e.mem.write(cell, bytes);
    fn_00857b50(e, game, file, Ptr::new(cell), bytes.len() as u32);
    e.mem.free(cell);
}

/// One length-prefixed block of the global data: writes `size` (2 bytes),
/// and for a non-zero size notes it in the statistics under `label`, makes
/// a buffer of that size, lets `fill` write into it, writes the buffer to
/// the file and frees it.
fn save_sized_block(
    e: &mut Engine,
    game: Ptr<TESSaveLoadGame>,
    file: Ptr,
    size: u16,
    label: u32,
    fill: impl FnOnce(&mut Engine),
) {
    put_bytes(e, game, file, &size.to_le_bytes());
    if size != 0 {
        let stats = e.get(game, TESSaveLoadGame::m_pSaveLoadStats);
        if !stats.is_null() {
            save_stats_add_extra_stat(e, stats, size as u32, Ptr::new(label));
        }
        let buffer = tes_save_load_game_create_buffer(e, game, size as u32);
        fill(e);
        tes_save_load_game_write_file(e, game, file, buffer, size as u32);
        fn_00858700(e, game, buffer);
    }
}

// Translated from 00858030 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::SaveGlobalData` (Xbox PDB): writes the global data of
/// a save: the data handler's size word, the id of the `TES` world space,
/// two more `TES` words, the player's location (world space or parent cell
/// id, and position), the global variables (`SaveGlobals`), and then one
/// length-prefixed block each for the `TES` object, the process lists, the
/// sky and weather, the created base objects (a zero word, then
/// `SaveCreatedBaseObjects`), the (always empty) HUD reticle, the interface
/// and the regions. A player with neither a world space nor a parent cell
/// raises the "cannot save" error.
pub fn tes_save_load_game_save_global_data(e: &mut Engine, this: Ptr<TESSaveLoadGame>, file: Ptr) {
    let handler: u32 = e.global(DATA_HANDLER);
    let handler_size = e.call(GLOBAL_DATA_SIZE, &args![handler]).u32();
    put_bytes(e, this, file, &handler_size.to_le_bytes());

    let tes: u32 = e.global(TES_OBJECT);
    let tes_worldspace = e.call(TES_GET_WORLDSPACE, &args![tes]).u32();
    let tes_worldspace_id = e.call(FORM_ID, &args![tes_worldspace]).u32();
    put_bytes(e, this, file, &tes_worldspace_id.to_le_bytes());

    let first = e.call(READ_FIELD_24, &args![tes]).u32();
    let second = e.call(READ_FIELD_28, &args![tes]).u32();
    put_bytes(e, this, file, &first.to_le_bytes());
    put_bytes(e, this, file, &second.to_le_bytes());

    let player: u32 = e.global(PLAYER);
    let worldspace = e.call(REF_GET_WORLDSPACE, &args![player]).u32();
    let parent_cell = e.call(REF_GET_PARENT_CELL, &args![player]).u32();
    if worldspace == 0 && parent_cell == 0 {
        e.call(ERROR, &args![MSG_PLAYER_HAS_NO_SPACE]);
    }
    let mut location_id = 0u32;
    if worldspace != 0 {
        location_id = e.call(FORM_ID, &args![worldspace]).u32();
    } else if parent_cell != 0 {
        location_id = e.call(FORM_ID, &args![parent_cell]).u32();
    }
    let position_ptr = e.call(REF_GET_POSITION, &args![player]).u32();
    let position = e.mem.bytes(position_ptr, 0x0C);
    put_bytes(e, this, file, &location_id.to_le_bytes());
    put_bytes(e, this, file, &position);

    tes_save_load_game_save_globals(e, this, file);

    let size = e.call(TES_SAVE_SIZE, &args![tes]).u16();
    save_sized_block(e, this, file, size, LABEL_TES_CLASS, |e| {
        e.call(TES_SAVE, &args![tes]);
    });

    let size = e.call(PROCESS_LISTS_SAVE_SIZE, &args![PROCESS_LISTS]).u16();
    save_sized_block(e, this, file, size, LABEL_PROCESS_LISTS, |e| {
        e.call(PROCESS_LISTS_SAVE, &args![PROCESS_LISTS]);
    });

    let sky = e.call(SKY_INSTANCE, &args![]).u32();
    let size = e.call(SKY_SAVE_SIZE, &args![sky]).u16();
    save_sized_block(e, this, file, size, LABEL_SKY, |e| {
        let sky = e.call(SKY_INSTANCE, &args![]).u32();
        e.call(SKY_SAVE, &args![sky]);
    });

    put_bytes(e, this, file, &0u32.to_le_bytes());
    e.call(SAVE_CREATED_BASE_OBJECTS, &args![this, file]);

    // The reticle block is always empty: the size is a local set to zero.
    let size = 0u16;
    save_sized_block(e, this, file, size, LABEL_HUD_RETICLE, |_| {});

    let size = e.call(INTERFACE_SAVE_SIZE, &args![]).u16();
    save_sized_block(e, this, file, size, LABEL_INTERFACE, |e| {
        e.call(INTERFACE_SAVE, &args![]);
    });

    let size = e.call(REGIONS_SAVE_SIZE, &args![]).u16();
    save_sized_block(e, this, file, size, LABEL_REGIONS, |e| {
        e.call(REGIONS_SAVE, &args![]);
    });
}

// Translated from 00858480 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::SaveGlobals` (Xbox PDB): writes the global variables
/// of the data handler as one block: a 2-byte count, then for each variable
/// its numeric id (through `SaveNumericID`) and its value (a float). The
/// block is `count * 8 + 2` bytes.
pub fn tes_save_load_game_save_globals(e: &mut Engine, this: Ptr<TESSaveLoadGame>, file: Ptr) {
    let handler: u32 = e.global(DATA_HANDLER);
    let mut node = e.call(GLOBALS_LIST, &args![handler]).u32();
    let count = e.call(LIST_COUNT, &args![node]).u16();
    let size = (count as u32 * 8 + 2) as u16;
    let buffer = tes_save_load_game_create_buffer(e, this, size as u32);
    let cell = e.mem.alloc(8);
    e.mem.set_u16(cell, count);
    fn_008579b0(e, this, Ptr::new(cell), 2);
    while node != 0 {
        let slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
        let variable = e.mem.u32(slot);
        if variable != 0 {
            let id = e.call(FORM_ID, &args![variable]).u32();
            let value = e.call(GLOBAL_VALUE, &args![variable]).f32();
            e.mem.set_u32(cell, id);
            tes_save_load_game_save_numeric_id(e, this, Ptr::new(cell), 4);
            e.mem.set_f32(cell + 4, value);
            fn_008579b0(e, this, Ptr::new(cell + 4), 4);
        }
        node = e.call(LIST_NODE_NEXT, &args![node]).u32();
    }
    e.mem.free(cell);
    tes_save_load_game_write_file(e, this, file, buffer, size as u32);
    fn_00858700(e, this, buffer);
    let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
    if !stats.is_null() {
        save_stats_add_extra_stat(e, stats, size as u32, Ptr::new(LABEL_GLOBAL_VARIABLES));
    }
}

// Translated from 00858570 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::SaveFinalData` (Xbox PDB): writes the size (4 bytes)
/// of the process lists' temp effects list and, when it is not zero, the
/// list itself through a buffer of that size.
pub fn tes_save_load_game_save_final_data(e: &mut Engine, this: Ptr<TESSaveLoadGame>, file: Ptr) {
    let size = e.call(TEMP_EFFECTS_SIZE, &args![PROCESS_LISTS]).u32();
    put_bytes(e, this, file, &size.to_le_bytes());
    if size != 0 {
        let stats = e.get(this, TESSaveLoadGame::m_pSaveLoadStats);
        if !stats.is_null() {
            save_stats_add_extra_stat(e, stats, size, Ptr::new(LABEL_TEMP_EFFECTS));
        }
        let buffer = tes_save_load_game_create_buffer(e, this, size);
        e.call(TEMP_EFFECTS_SAVE, &args![PROCESS_LISTS]);
        tes_save_load_game_write_file(e, this, file, buffer, size);
        fn_00858700(e, this, buffer);
    }
}

// Translated from 00858600 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::CreateBuffer` (Xbox PDB): allocates `size` bytes (in an
/// allocation scope), makes them the current buffer and returns them; raises
/// an error when the allocation fails. Not translated: the exception frame.
pub fn tes_save_load_game_create_buffer(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    size: u32,
) -> Ptr {
    let scope = scope_enter(e, 0x103A);
    let block = e.call(OPERATOR_NEW, &args![size]).ptr::<()>();
    e.set(this, TESSaveLoadGame::m_pBuffer, block);
    if e.get(this, TESSaveLoadGame::m_pBuffer).is_null() {
        e.call(ERROR, &args![MSG_NO_SAVE_BUFFER]);
    }
    let buffer = e.get(this, TESSaveLoadGame::m_pBuffer);
    scope_leave(e, scope);
    buffer
}

// Translated from 008586a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::WriteFile` (Xbox PDB): `fn_00857b50`.
pub fn tes_save_load_game_write_file(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    file: Ptr,
    buffer: Ptr,
    size: u32,
) {
    fn_00857b50(e, this, file, buffer, size);
}

// Translated from 008586d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// The read counterpart of `WriteFile`: `fn_00857ba0`; returns its result.
pub fn fn_008586d0(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    file: Ptr,
    buffer: Ptr,
    size: u32,
) -> u32 {
    fn_00857ba0(e, this.cast(), file, buffer, size)
}

// Translated from 00858700 (decompiled, FalloutNV.exe 1.4.0.525)
/// Frees a buffer made by `CreateBuffer` and clears the current buffer.
pub fn fn_00858700(e: &mut Engine, this: Ptr<TESSaveLoadGame>, buffer: Ptr) {
    delete(e, buffer.addr());
    e.set(this, TESSaveLoadGame::m_pBuffer, Ptr::NULL);
}

// Translated from 00858730 (decompiled, FalloutNV.exe 1.4.0.525)
/// Loads one form from its pre-built buffer (the older load path; it does
/// nothing and returns false unless `0047c850` is true). Takes the form's
/// `ChangeData` and its buffer, enters the load section, reads the 4-byte
/// header (size, form type, version) and: when the saved form type is not
/// the form's own, formats the "Load Error" text (it is not shown), drops
/// the form's changes and returns false; otherwise builds the
/// `LoadFormHeader`, applies the initial data (`0085ac30`), lets the form
/// load its changes (virtual slot `0x60`), remembers the form and its
/// version in the init array (making the array on first use), frees the
/// buffer, applies a queued `RemoveChanges` and returns true. Not
/// translated: the exception frame and the stack cookie.
pub fn fn_00858730(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form: Ptr) -> bool {
    // The frame cells: the 4-byte header read from the buffer (size, type,
    // version), the 12-byte `LoadFormHeader` made from it, the text of the
    // load error and the slot passed to the init array's `Add`.
    const SOURCE: u32 = 0x00;
    const HEADER: u32 = 0x10;
    const TEXT: u32 = 0x20;
    const SLOT: u32 = 0x130;
    const FRAME: u32 = 0x140;

    if !game_unavailable(e, this) {
        return false;
    }
    let changes = e.get(this, TESSaveLoadGame::m_pChanges);
    let change_data = fn_00855130(e, changes, form);
    if change_data.is_null() {
        return false;
    }
    let buffer = e.call(LIST_NODE_NEXT, &args![change_data]).u32();
    if buffer == 0 {
        return false;
    }
    e.call(SECTION_ENTER, &args![LOAD_SECTION, MSG_LOAD_FORM_SECTION]);
    e.set(this, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
    let frame = e.mem.alloc(FRAME);
    let game = game_singleton(e);
    fn_008579e0(e, game, Ptr::new(frame + SOURCE), 4);
    let saved_type = e.mem.u8(frame + SOURCE + 2);
    let form_type = e.call(FORM_TYPE, &args![form]).u32();
    if form_type != saved_type as u32 {
        let current_name = e.call(FORM_TYPE_NAME, &args![form]).u32();
        let saved_name = e.mem.u32(FORM_TYPE_NAME_TABLE + saved_type as u32 * 12);
        let id = e.call(FORM_ID, &args![form]).u32();
        e.call(
            SPRINTF,
            &args![
                frame + TEXT,
                FORMAT_LOAD_ERROR,
                id,
                saved_name,
                current_name
            ],
        );
        let changes = e.get(this, TESSaveLoadGame::m_pChanges);
        fn_008551f0(e, changes, form, 1);
        e.set(this, TESSaveLoadGame::m_pBuffer, Ptr::NULL);
        e.call(SECTION_LEAVE, &args![LOAD_SECTION]);
        e.mem.free(frame);
        return false;
    }

    let size = e.mem.u16(frame + SOURCE) as u32;
    let flags = e.call(READ_WORD, &args![change_data]).u32();
    let flags = tes_save_load_game_check_new_reference(e, this.cast(), form, flags);
    let version = e.mem.u8(frame + SOURCE + 3);
    e.call(SET_LOAD_VERSION, &args![this, version as u32]);
    let id = e.call(FORM_ID, &args![form]).u32();
    let header: Ptr<LoadFormHeader> = Ptr::new(frame + HEADER);
    fn_00858aa0(e, header, Ptr::new(frame + SOURCE), id, flags);
    e.call(SET_LOADING_HEADER, &args![this, header]);
    let was_set = game_unavailable(e, this);
    e.call(SET_LOADING_STATE, &args![this, 1u32]);
    e.call(FORM_FINISH, &args![form, 1u32]);
    e.call(LOAD_INITIAL_DATA, &args![this, form, flags]);
    e.vcall(form.addr(), FORM_LOAD, &args![flags, 0u32]);
    e.call(SET_LOADING_STATE, &args![this, was_set as u32]);
    e.call(SET_LOADING_HEADER, &args![this, 0u32]);
    e.call(END_FORM_PROCESSING, &args![this]);

    if e.get(this, TESSaveLoadGame::m_pInitArray).is_null() {
        let block = e.call(OPERATOR_NEW, &args![0x18u32]).u32();
        let array = if block != 0 {
            e.call(INIT_ARRAY_CONSTRUCT, &args![block, 0x32u32, 0x32u32])
                .ptr::<()>()
        } else {
            Ptr::NULL
        };
        e.set(this, TESSaveLoadGame::m_pInitArray, array);
    }
    let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
    let record = if block != 0 {
        fn_00857280(e, Ptr::new(block), form, flags, 0, version).addr()
    } else {
        0
    };
    e.mem.set_u32(frame + SLOT, record);
    let array = e.get(this, TESSaveLoadGame::m_pInitArray);
    e.call(INIT_ARRAY_ADD, &args![array, frame + SLOT]);
    // The game also computes how far the buffer pointer is past the form's
    // data (`m_pBuffer - (buffer + 4 + size)`) and does not use it.
    let _ = size;
    fn_00858700(e, this, Ptr::new(buffer));
    e.call(CHANGE_DATA_SET_BUFFER, &args![change_data, 0u32]);
    let queued = e.get(this, TESSaveLoadGame::m_iQueuedRemoveChanges);
    if queued != 0 {
        let changes = e.get(this, TESSaveLoadGame::m_pChanges);
        fn_00855150(e, changes, form, queued);
        e.set(this, TESSaveLoadGame::m_iQueuedRemoveChanges, 0);
    }
    e.call(SECTION_LEAVE, &args![LOAD_SECTION]);
    e.mem.free(frame);
    true
}

// Translated from 00858aa0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Fills a `LoadFormHeader` from the 4 bytes read from a buffer (`source`:
/// size word, form type, version) and the form id and flags.
pub fn fn_00858aa0(
    e: &mut Engine,
    this: Ptr<LoadFormHeader>,
    source: Ptr,
    form_id: u32,
    flags: u32,
) -> Ptr<LoadFormHeader> {
    e.set(this, LoadFormHeader::iFormID, form_id);
    e.set(this, LoadFormHeader::iFlags, flags);
    let form_type = e.mem.u8(source.addr() + 2);
    e.set(this, LoadFormHeader::cFormType, form_type);
    let version = e.mem.u8(source.addr() + 3);
    e.set(this, LoadFormHeader::cVersion, version);
    let size = e.mem.u16(source.addr());
    e.set(this, LoadFormHeader::iSize, size);
    this
}

// Translated from 00858af0 (decompiled, FalloutNV.exe 1.4.0.525)
/// After forms were loaded, applies what is queued: the forms of
/// `init_array` (when null, the game's own `m_pInitArray`), each with the
/// version it was loaded with, get their pre-initialization hook (virtual
/// slot `0x68`), the actors among them that are in no cell or world space
/// are collected (only when the array was given), and then those actors are
/// given their package locations, flagged in the changes map and moved to
/// the placement cell (or disabled when there is none). When `reload` is
/// set the world and the loader are locked and flushed around the work and
/// the player's camera is placed again. The forms then get their
/// post-initialization hook (slot `0x6C`) and are freed, and the game's
/// array is destroyed. `location` is an optional pair of words (+4, +8) given
/// to the player's two hooks. Does nothing unless `0047c850` is true. Not
/// translated: the exception frame.
pub fn fn_00858af0(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    init_array: Ptr,
    location: Ptr,
    reload: bool,
) {
    if !game_unavailable(e, this) {
        return;
    }
    e.call(SET_LOADING_STATE, &args![this, 1u32]);
    let io_manager: u32 = e.global(SAVE_LOCK);
    if reload {
        e.call(IO_MANAGER_SET_STATE_5, &args![io_manager]);
    }
    let mut array = init_array;
    let mut supplied = true;
    if array.is_null() {
        array = e.get(this, TESSaveLoadGame::m_pInitArray);
        supplied = false;
    }
    let player: u32 = e.global(PLAYER);
    if !location.is_null() {
        let (first, second) = (
            e.mem.u32(location.addr() + 4),
            e.mem.u32(location.addr() + 8),
        );
        e.vcall(player, FORM_BEGIN_INIT, &args![first, second]);
    }
    let list = e.mem.alloc(8);
    e.call(SIMPLE_LIST_CONSTRUCT, &args![list]);
    let cell = e.mem.alloc(4);

    if !array.is_null() {
        let count = e.call(ARRAY_SIZE, &args![array]).u32();
        for index in 0..count {
            let slot = e.call(ARRAY_ELEMENT_ADDRESS, &args![array, index]).u32();
            let item = e.mem.u32(slot);
            if item != 0 && e.mem.u32(item) != player {
                let version = e.mem.u8(item + 0xC);
                e.call(SET_LOAD_VERSION, &args![this, version as u32]);
                let form = e.mem.u32(item);
                let (flags, old_flags) = (e.mem.u32(item + 4), e.mem.u32(item + 8));
                e.vcall(form, FORM_BEGIN_INIT, &args![flags, old_flags]);
                if supplied {
                    let actor = dynamic_cast(e, form, RTTI_FORM, RTTI_ACTOR);
                    e.mem.set_u32(cell, actor);
                    if actor != 0
                        && e.call(REF_GET_PARENT_CELL, &args![actor]).u32() == 0
                        && e.call(REF_GET_WORLDSPACE, &args![actor]).u32() == 0
                    {
                        e.call(LIST_ADD_HEAD, &args![list, cell]);
                    }
                }
                e.call(END_FORM_PROCESSING, &args![this]);
            }
        }
    }

    if supplied {
        let mut node = list;
        while node != 0 && !e.call(LIST_NODE_IS_END, &args![node]).bool() {
            let slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
            let actor = e.mem.u32(slot);
            e.call(ACTOR_INIT_PACKAGE_LOCATIONS, &args![actor, 0u32]);
            let changes = e.get(this, TESSaveLoadGame::m_pChanges);
            fn_00855010(e, changes, Ptr::new(actor), 2);
            if e.call(REF_GET_PARENT_CELL, &args![actor]).u32() == 0
                && e.call(REF_GET_WORLDSPACE, &args![actor]).u32() == 0
            {
                // The placement cell comes from a lookup of form id 0.
                let default_form = e.call(LOOKUP_FORM, &args![0u32]).u32();
                let placement_cell = dynamic_cast(e, default_form, RTTI_FORM, RTTI_CELL);
                if placement_cell == 0 {
                    e.call(FORM_SET_DISABLED, &args![actor, 1u32]);
                } else {
                    let vectors = e.mem.alloc(0x18);
                    let words = e.mem.bytes(DEFAULT_POSITION, 0x0C);
                    e.mem.write(vectors, &words);
                    e.mem.write(vectors + 0xC, &words);
                    e.call(
                        CELL_GET_PLACEMENT,
                        &args![placement_cell, vectors, vectors + 0xC],
                    );
                    e.call(REF_SET_POSITION, &args![actor, vectors]);
                    let (x, y, z) = (
                        e.mem.u32(vectors + 0xC),
                        e.mem.u32(vectors + 0x10),
                        e.mem.u32(vectors + 0x14),
                    );
                    e.call(FN_00575700, &args![actor, x, y, z]);
                    e.call(REF_MOVE_TO_SPACE, &args![actor, placement_cell, 0u32]);
                    e.mem.free(vectors);
                }
            }
            node = e.call(LIST_NODE_NEXT, &args![node]).u32();
        }
        e.call(LIST_REMOVE_ALL, &args![list]);
    }

    let tes: u32 = e.global(TES_OBJECT);
    if reload {
        let world = locked_world(e, tes);
        if world != 0 {
            e.call(WORLD_ADD_LOCK, &args![world]);
        }
        lock_other_world(e);
        e.call(FN_00459920, &args![tes]);
        if e.call(PLAYER_GET_3D, &args![player, 0u32]).u32() == 0 {
            let loader: u32 = e.global(MODEL_LOADER);
            e.call(
                MODEL_LOADER_QUEUE_REFERENCE,
                &args![loader, player, 0u32, 0u32],
            );
        }
        e.call(IO_MANAGER_LOAD_QUEUED_PRIORITY, &args![io_manager]);
        e.call(IO_MANAGER_SET_STATE_5, &args![io_manager]);
        unlock_worlds(e, world);
        let position_ptr = e.vcall(player, REFERENCE_GET_POSITION_SLOT, &args![]).u32();
        let position = e.mem.bytes(position_ptr, 0x0C);
        e.call(SET_GLOBAL_FLAG, &args![0u32]);
        let (x, y, z) = (
            u32::from_le_bytes(position[0..4].try_into().unwrap()),
            u32::from_le_bytes(position[4..8].try_into().unwrap()),
            u32::from_le_bytes(position[8..12].try_into().unwrap()),
        );
        let (a, b, c) = (
            e.mem.u32(PLACEMENT_VECTOR),
            e.mem.u32(PLACEMENT_VECTOR + 4),
            e.mem.u32(PLACEMENT_VECTOR + 8),
        );
        e.call(FN_0057D0A0, &args![x, y, z, a, b, c, 1.0f32]);
        e.call(SET_GLOBAL_FLAG, &args![1u32]);
        let _ = world;
    }
    if !location.is_null() && e.call(REF_GET_PARENT_CELL, &args![player]).u32() != 0 {
        e.call(EMPTY_FN_00483710, &args![player]);
    }
    if !location.is_null() {
        let (first, second) = (
            e.mem.u32(location.addr() + 4),
            e.mem.u32(location.addr() + 8),
        );
        e.vcall(player, FORM_END_INIT, &args![first, second]);
    }

    if !array.is_null() {
        let count = e.call(ARRAY_SIZE, &args![array]).u32();
        for index in 0..count {
            let slot = e.call(ARRAY_ELEMENT_ADDRESS, &args![array, index]).u32();
            let item = e.mem.u32(slot);
            if item != 0 {
                let version = e.mem.u8(item + 0xC);
                e.call(SET_LOAD_VERSION, &args![this, version as u32]);
                let form = e.mem.u32(item);
                let (flags, old_flags) = (e.mem.u32(item + 4), e.mem.u32(item + 8));
                e.vcall(form, FORM_END_INIT, &args![flags, old_flags]);
                e.call(END_FORM_PROCESSING, &args![this]);
                delete(e, item);
            }
        }
    }
    let game_array = e.get(this, TESSaveLoadGame::m_pInitArray);
    if !game_array.is_null() {
        e.call(INIT_ARRAY_CLEANUP, &args![game_array]);
        let game_array = e.get(this, TESSaveLoadGame::m_pInitArray);
        if !game_array.is_null() {
            e.vcall(game_array.addr(), FILE_DESTRUCT, &args![1u32]);
        }
        e.set(this, TESSaveLoadGame::m_pInitArray, Ptr::NULL);
    }

    if reload {
        let world = locked_world(e, tes);
        if world != 0 {
            e.call(WORLD_ADD_LOCK, &args![world]);
        }
        lock_other_world(e);
        e.call(IO_MANAGER_LOAD_QUEUED_PRIORITY, &args![io_manager]);
        if world != 0 {
            e.call(WORLD_REMOVE_LOCK, &args![world, 0u32]);
        }
        unlock_other_world(e);
    }
    e.call(SET_LOADING_STATE, &args![this, 0u32]);
    e.call(LIST_DESTRUCT, &args![list]);
    e.mem.free(cell);
    e.mem.free(list);
}

/// The object `00 5f36f0` (a field of `TES`) leads to through `004543c0`,
/// or 0: the code asks `TES` twice.
fn locked_world(e: &mut Engine, tes: u32) -> u32 {
    if e.call(READ_FIELD_34, &args![tes]).u32() != 0 {
        let mover = e.call(READ_FIELD_34, &args![tes]).u32();
        e.call(CELL_GET_PHYSICS_WORLD, &args![mover]).u32()
    } else {
        0
    }
}

/// Locks the second world object (`00451010`) when there is one.
fn lock_other_world(e: &mut Engine) {
    if e.call(GET_EXTERIOR_WORLD, &args![]).u32() != 0 {
        let other = e.call(GET_EXTERIOR_WORLD, &args![]).u32();
        e.call(WORLD_ADD_LOCK, &args![other]);
    }
}

/// Unlocks the second world object (`00451010`) when there is one.
fn unlock_other_world(e: &mut Engine) {
    if e.call(GET_EXTERIOR_WORLD, &args![]).u32() != 0 {
        let other = e.call(GET_EXTERIOR_WORLD, &args![]).u32();
        e.call(WORLD_REMOVE_LOCK, &args![other, 0u32]);
    }
}

/// Unlocks the world `world` (when not 0) and then the second one.
fn unlock_worlds(e: &mut Engine, world: u32) {
    if world != 0 {
        e.call(WORLD_REMOVE_LOCK, &args![world, 0u32]);
    }
    unlock_other_world(e);
}

// Translated from 00859120 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::CheckNewReference` (Xbox PDB): for a form the data
/// handler knows (`00469860` on its id), clears bit 1 of the flags when the
/// form is a reference; returns the flags. (The code also casts the form to a
/// cell and then leaves the flags as they are.)
pub fn tes_save_load_game_check_new_reference(
    e: &mut Engine,
    _this: Ptr,
    form: Ptr,
    flags: u32,
) -> u32 {
    let mut flags = flags;
    let id = e.call(FORM_ID, &args![form]).u32();
    let handler: u32 = e.global(DATA_HANDLER);
    if e.call(DATA_HANDLER_HAS_FORM, &args![handler, id]).bool() {
        if dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_REFERENCE) != 0 {
            flags &= !2;
        }
        dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_CELL);
    }
    flags
}

// Translated from 008591b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::CheckFlags` (Xbox PDB): adjusts the changed-parts flags
/// a form is saved with. Starts from `CheckNewReference`. A cell loses the
/// top bit when it has no seen data. A reference loses bit 5 unless its
/// extra data list has container changes. An actor (a reference with a
/// process) gets the flags of the package its process runs, and bit 2 is set
/// when `008aad40` says so, otherwise cleared when the actor's virtual slot
/// `0x22C` says no. A non-persistent reference other than the player that
/// has bits 1 or 2 is looked up in its cell (its own, or the one at its
/// position), which only matters for the calls made. Several tests the
/// compiler folded away (`flags & 0`) have no code here.
pub fn tes_save_load_game_check_flags(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    form: Ptr,
    flags: u32,
) -> u32 {
    let mut flags = tes_save_load_game_check_new_reference(e, this.cast(), form, flags);
    let reference = dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_REFERENCE);
    let cell = dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_CELL);
    if cell != 0 {
        if flags & 0x8000_0000 != 0 && e.call(EXTRA_GET_SEEN, &args![cell]).u32() == 0 {
            flags &= 0x7FFF_FFFF;
        }
        if !e.call(CELL_IS_INTERIOR, &args![cell]).bool() {
            e.call(CELL_GET_X, &args![cell]);
            e.call(CELL_GET_Y, &args![cell]);
        }
        return flags;
    }
    if reference == 0 {
        return flags;
    }
    if flags & 0x20 != 0 {
        let has_changes = e.call(REF_GET_EXTRA_LIST, &args![reference]).u32() != 0 && {
            let list = e.call(REF_GET_EXTRA_LIST, &args![reference]).u32();
            e.call(EXTRA_GET_CONTAINER_CHANGES, &args![list]).u32() != 0
        };
        if !has_changes {
            flags &= !0x20;
        }
    }
    let actor = dynamic_cast(e, reference, RTTI_REFERENCE, RTTI_ACTOR);
    if actor != 0 {
        if e.call(ACTOR_GET_PROCESS, &args![actor]).u32() != 0 {
            let process = e.call(ACTOR_GET_PROCESS, &args![actor]).u32();
            e.vcall(process, PROCESS_SLOT_20C, &args![]);
        }
        if e.call(ACTOR_GET_PROCESS, &args![actor]).u32() != 0 {
            let process = e.call(ACTOR_GET_PROCESS, &args![actor]).u32();
            if base_process_get_package_that_is_running(e, Ptr::new(process)) != 0 {
                let process = e.call(ACTOR_GET_PROCESS, &args![actor]).u32();
                let package = base_process_get_package_that_is_running(e, Ptr::new(process));
                flags |= e.call(ACTOR_PACKAGE_FLAGS, &args![actor, package]).u32();
            }
        }
        if e.call(ACTOR_TEST_FLAGS, &args![actor, flags]).bool() {
            flags |= 4;
        } else if !e.vcall(actor, ACTOR_SLOT_22C, &args![0u32]).bool() {
            flags &= !4;
        }
    }

    let player: u32 = e.global(PLAYER);
    if flags & 6 != 0 && !e.call(REF_PERSISTS, &args![reference]).bool() && reference != player {
        let mut located = 0u32;
        if e.vcall(reference, REFERENCE_IS_ACTOR, &args![]).bool() {
            located = reference;
        }
        let cell = if located != 0 {
            e.vcall(located, ACTOR_CELL_SLOT, &args![]).u32()
        } else {
            0
        };
        let worldspace = if located != 0 {
            e.vcall(located, ACTOR_WORLDSPACE_SLOT, &args![]).u32()
        } else {
            0
        };
        if worldspace != 0 || cell != 0 {
            let out = e.mem.alloc(0x14);
            let position_ptr = if located != 0 {
                e.vcall(located, REFERENCE_GET_LOCATION_SLOT, &args![out])
                    .u32()
            } else {
                DEFAULT_POSITION
            };
            let (x, y) = (e.mem.f32(position_ptr), e.mem.f32(position_ptr + 4));
            if cell != 0 {
                e.call(REF_GET_PARENT_CELL, &args![reference]);
            } else if worldspace != 0
                && worldspace == e.call(REF_GET_WORLDSPACE, &args![reference]).u32()
            {
                let cell_x = e.call(FLOAT_TO_INT, &args![x]).i32() >> 12;
                let cell_y = e.call(FLOAT_TO_INT, &args![y]).i32() >> 12;
                e.call(
                    WORLDSPACE_GET_CELL,
                    &args![worldspace, cell_x as u32, cell_y as u32],
                );
                e.call(REF_GET_PARENT_CELL, &args![reference]);
            }
            e.mem.free(out);
        } else {
            let parent = e.call(REF_GET_PARENT_CELL, &args![reference]).u32();
            if parent != 0 {
                let parent = e.call(REF_GET_PARENT_CELL, &args![reference]).u32();
                if !e.call(CELL_IS_INTERIOR, &args![parent]).bool() {
                    if located != 0 {
                        e.call(LOG_ERROR, &args![MSG_ACTOR_NO_EDITOR_LOCATION]);
                    }
                    let out = e.mem.alloc(0x0C);
                    e.vcall(reference, REFERENCE_GET_LOCATION_SLOT, &args![out]);
                    let (x, y) = (e.mem.f32(out), e.mem.f32(out + 4));
                    let cell_x = e.call(FLOAT_TO_INT, &args![x]).i32() >> 12;
                    let cell_y = e.call(FLOAT_TO_INT, &args![y]).i32() >> 12;
                    let worldspace = e.call(REF_GET_WORLDSPACE, &args![reference]).u32();
                    e.call(
                        WORLDSPACE_GET_CELL,
                        &args![worldspace, cell_x as u32, cell_y as u32],
                    );
                    e.call(REF_GET_PARENT_CELL, &args![reference]);
                    e.mem.free(out);
                }
            }
        }
    }
    flags
}

// Translated from 00859670 (decompiled, FalloutNV.exe 1.4.0.525)
/// `BaseProcess::GetPackageThatIsRunning` (Xbox PDB): the process's virtual
/// slot `0x22C`.
pub fn base_process_get_package_that_is_running(e: &mut Engine, this: Ptr) -> u32 {
    e.vcall(this.addr(), ACTOR_SLOT_22C, &args![]).u32()
}

// Translated from 00859690 (decompiled, FalloutNV.exe 1.4.0.525)
/// Brings back what a save recorded for a cell: loads the cell itself
/// (`fn_00858730`) and then every reference the save lists for it (the
/// interior map is keyed by the cell's id and holds a list of form ids; the
/// exterior map is keyed by the cell's world space and holds the
/// references of each grid cell, of which the entries for this cell's
/// coordinates are used and removed), forgetting what it handled. Returns
/// true when anything was loaded. Does nothing unless `0047c850` is true.
pub fn fn_00859690(e: &mut Engine, this: Ptr<TESSaveLoadGame>, cell: Ptr) -> bool {
    if !game_unavailable(e, this) {
        return false;
    }
    let mut result = fn_00858730(e, this, cell);
    let out = e.mem.alloc(4);
    if e.call(CELL_IS_INTERIOR, &args![cell]).bool() {
        let id = e.call(FORM_ID, &args![cell]).u32();
        let map = e.get(this, TESSaveLoadGame::m_pInteriorCellMap);
        if !e.call(MAP_GET_AT, &args![map, id, out]).bool() {
            e.mem.free(out);
            return result;
        }
        let list = e.mem.u32(out);
        let mut node = list;
        while node != 0 {
            let slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
            let id = e.mem.u32(slot);
            if id != 0 {
                fn_008598d0(e, this, id);
                result = true;
            }
            node = e.call(LIST_NODE_NEXT, &args![node]).u32();
        }
        let id = e.call(FORM_ID, &args![cell]).u32();
        let map = e.get(this, TESSaveLoadGame::m_pInteriorCellMap);
        e.call(MAP_REMOVE_AT, &args![map, id]);
        destroy_list(e, list);
    } else {
        let worldspace = e.call(CELL_GET_WORLDSPACE, &args![cell]).u32();
        let id = e.call(FORM_ID, &args![worldspace]).u32();
        let map = e.get(this, TESSaveLoadGame::m_pExteriorCellMap);
        if !e.call(MAP_GET_AT, &args![map, id, out]).bool() {
            e.mem.free(out);
            return result;
        }
        let list = e.mem.u32(out);
        let mut node = list;
        let mut previous = 0u32;
        let cell_x = e.call(CELL_GET_X, &args![cell]).i32();
        let cell_y = e.call(CELL_GET_Y, &args![cell]).i32();
        let item_slot = e.mem.alloc(4);
        while node != 0 {
            let slot = e.call(LIST_NODE_ITEM, &args![node]).u32();
            let item = e.mem.u32(slot);
            e.mem.set_u32(item_slot, item);
            if item != 0 && cell_x == e.mem.i32(item + 4) && cell_y == e.mem.i32(item + 8) {
                let form_id = e.mem.u32(item);
                fn_008598d0(e, this, form_id);
                result = true;
                if previous != 0 {
                    e.call(LIST_REMOVE, &args![previous, item_slot]);
                    node = e.call(LIST_NODE_NEXT, &args![previous]).u32();
                } else {
                    e.call(LIST_REMOVE_HEAD, &args![node]);
                }
                delete(e, item);
            } else {
                previous = node;
                node = e.call(LIST_NODE_NEXT, &args![node]).u32();
            }
        }
        e.mem.free(item_slot);
        if e.call(LIST_NODE_IS_END, &args![list]).bool() {
            let worldspace = e.call(CELL_GET_WORLDSPACE, &args![cell]).u32();
            let id = e.call(FORM_ID, &args![worldspace]).u32();
            let map = e.get(this, TESSaveLoadGame::m_pExteriorCellMap);
            e.call(MAP_REMOVE_AT, &args![map, id]);
            if list != 0 {
                e.call(LIST_SCALAR_DELETE, &args![list, 1u32]);
            }
        }
    }
    e.mem.free(out);
    result
}

// Translated from 008598d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Handles one reference id of a cell's new-references list: when the
/// changes map has a `ChangeData` for it, reads its flags and logs "CELLS:
/// Reference in cell map has neither required flag." (the two flag tests
/// before it compare against 0 in this build, so the loading code that
/// follows them is not reachable and is not translated).
pub fn fn_008598d0(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form_id: u32) {
    let changes = e.get(this, TESSaveLoadGame::m_pChanges);
    let change_data = fn_00855100(e, changes, form_id);
    if !change_data.is_null() {
        e.call(READ_WORD, &args![change_data]);
        e.call(LOG_ERROR, &args![MSG_CELL_REFERENCE_NO_FLAG]);
    }
}

// Translated from 00859a90 (decompiled, FalloutNV.exe 1.4.0.525)
/// Makes (or finds) the reference a `CreatedReferenceData` describes under
/// `form_id`. An existing form is kept only when it is a reference whose
/// base object is the record's bound object and the record is of type 0 or
/// 3; otherwise it is deleted (`DeleteForm`) and made again: type 0 and 3
/// build a `Character` (base form type 0x2A), a `Creature` (0x2B) or a plain
/// reference around the bound object, type 1 an arrow projectile, type 2 one
/// of three magic projectiles chosen by the record's `iBoundID`; each gets
/// its form id through virtual slot `0x128`. A record whose bound object no
/// longer exists logs an error and gives null, an unknown type logs one and
/// gives null too (the form is then not made). Returns the form, after
/// `0046a010(form, 1)`. Not translated: the exception frame.
pub fn fn_00859a90(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    form_id: u32,
    record: Ptr<CreatedReferenceData>,
) -> Ptr {
    let scope = scope_enter_kind(e, 0x31, 0x133B);
    let mut form = e.call(LOOKUP_FORM, &args![form_id]).u32();
    let mut bound = 0u32;
    let kind = e.get(record, CreatedReferenceData::eType);
    let bound_id = e.get(record, CreatedReferenceData::iBoundID);
    if kind != 2 {
        let object = e.call(LOOKUP_FORM, &args![bound_id]).u32();
        bound = dynamic_cast(e, object, RTTI_FORM, RTTI_BOUND_OBJECT);
        if bound == 0 {
            e.call(
                LOG_ERROR,
                &args![MSG_BOUND_OBJECT_MISSING, bound_id, form_id],
            );
            scope_leave(e, scope);
            return Ptr::NULL;
        }
    }
    if form != 0 {
        let reference = dynamic_cast(e, form, RTTI_FORM, RTTI_REFERENCE);
        let keep = reference != 0
            && (kind == 0 || kind == 3)
            && e.call(REFERENCE_GET_BASE, &args![reference]).u32() == bound;
        if !keep {
            tes_save_load_game_delete_form(e, this, Ptr::new(form));
            form = 0;
        }
    }
    if form == 0 {
        if kind == 0 || kind == 3 {
            let base_type = e.call(FORM_TYPE, &args![bound]).u32();
            form = match base_type {
                0x2A => construct_form(e, 0x1C8, CHARACTER_CONSTRUCT),
                0x2B => construct_form(e, 0x1C0, CREATURE_CONSTRUCT),
                _ => construct_form(e, 0x68, REFERENCE_CONSTRUCT),
            };
            let reference = dynamic_cast(e, form, RTTI_FORM, RTTI_REFERENCE);
            e.call(REF_SET_BASE, &args![reference, bound]);
            e.vcall(form, FORM_SET_FORM_ID, &args![form_id, 1u32]);
        } else if kind == 1 {
            form = construct_form(e, 0xC8, ARROW_PROJECTILE_CONSTRUCT);
            let reference = dynamic_cast(e, form, RTTI_FORM, RTTI_REFERENCE);
            e.call(REF_SET_BASE, &args![reference, bound]);
            e.vcall(form, FORM_SET_FORM_ID, &args![form_id, 1u32]);
        } else if kind == 2 {
            form = match bound_id {
                0 => construct_form(e, 0xC4, MAGIC_PROJECTILE_CONSTRUCT_A),
                3 => construct_form(e, 0xD0, MAGIC_PROJECTILE_CONSTRUCT_B),
                1 => construct_form(e, 0xD8, MAGIC_PROJECTILE_CONSTRUCT_C),
                _ => 0,
            };
            e.vcall(form, FORM_SET_FORM_ID, &args![form_id, 1u32]);
        } else {
            let location = e.mem.u32(record.addr() + 8);
            e.call(
                LOG_ERROR,
                &args![MSG_INVALID_CREATED_TYPE, kind, form_id, bound_id, location],
            );
        }
    }
    if form != 0 {
        e.call(FORM_FINISH, &args![form, 1u32]);
    }
    scope_leave(e, scope);
    Ptr::new(form)
}

/// `new` of `size` bytes and, when the allocation worked, the constructor
/// at `construct` on it; the constructed object (the constructor returns
/// `this`) or 0.
fn construct_form(e: &mut Engine, size: u32, construct: u32) -> u32 {
    let block = e.call(OPERATOR_NEW, &args![size]).u32();
    if block != 0 {
        e.call(construct, &args![block]).u32()
    } else {
        0
    }
}

// Translated from 00859f20 (decompiled, FalloutNV.exe 1.4.0.525)
/// Loads a moved reference from the plugins that have it. The location is
/// the record's original location id (or its `RefData` location id when
/// that is 0); looked up it is a cell, or a world space (then the record's
/// original position says which grid cell). For each plugin file of the
/// cell or world space that contains the cell and the reference, the
/// reference is made (`CreateReference`, with the record type of the plugin
/// file) and loaded from the file; the last one is the result. With no
/// result an error is logged and null returned. The reference's post-create
/// hook (virtual slot `0x88`) runs; an actor is placed in the cell with its
/// position and angle, other references get their extra data starting
/// position and rotation set.
pub fn fn_00859f20(
    e: &mut Engine,
    _this: Ptr,
    form_id: u32,
    moved: Ptr<MovedReferenceData>,
) -> Ptr {
    let mut result = 0u32;
    let mut location = e.get(moved, MovedReferenceData::iOriginalLocationID);
    if location == 0 {
        // MovedReferenceData::RefData.iLocationID
        location = e.mem.u32(moved.addr() + 0x10);
    }
    let location_form = e.call(LOOKUP_FORM, &args![location]).u32();
    let cell = dynamic_cast(e, location_form, RTTI_FORM, RTTI_CELL);
    let worldspace = dynamic_cast(e, location_form, RTTI_FORM, RTTI_WORLDSPACE);
    let (mut grid_x, mut grid_y) = (0i32, 0i32);
    if cell != 0 {
        let count = e.call(CELL_FILE_COUNT, &args![cell]).i32();
        for index in 0..count {
            let entry = e.call(FILE_OF_CELL, &args![cell, index as u32]).u32();
            let file = e.call(THREAD_SAFE_FILE, &args![entry]).u32();
            if e.call(FILE_HAS_CELL, &args![file, cell]).bool()
                && e.call(FILE_HAS_FORM, &args![file, form_id]).bool()
            {
                result = load_from_file(e, file);
            }
        }
    } else if worldspace != 0 {
        let count = e.call(CELL_FILE_COUNT, &args![worldspace]).i32();
        // MovedReferenceData::OriginalLoc.x and .y
        let x = e.mem.f32(moved.addr() + 4);
        grid_x = e.call(FLOAT_TO_INT, &args![x]).i32() >> 12;
        let y = e.mem.f32(moved.addr() + 8);
        grid_y = e.call(FLOAT_TO_INT, &args![y]).i32() >> 12;
        for index in 0..count {
            let entry = e.call(FILE_OF_CELL, &args![worldspace, index as u32]).u32();
            let file = e.call(THREAD_SAFE_FILE, &args![entry]).u32();
            if e.call(
                WORLDSPACE_FIND_CELL_IN_FILE,
                &args![worldspace, file, grid_x as u32, grid_y as u32],
            )
            .bool()
                && e.call(FILE_HAS_FORM, &args![file, form_id]).bool()
            {
                result = load_from_file(e, file);
            }
        }
    } else {
        e.call(LOG_ERROR, &args![MSG_NO_CELL_OR_WORLDSPACE]);
    }
    if result == 0 {
        e.call(
            LOG_ERROR,
            &args![
                MSG_REFERENCE_NOT_LOADED,
                form_id,
                location,
                grid_x as u32,
                grid_y as u32
            ],
        );
        return Ptr::NULL;
    }

    e.vcall(result, FORM_POST_CREATE, &args![]);
    let actor = dynamic_cast(e, result, RTTI_REFERENCE, RTTI_ACTOR);
    if actor != 0 {
        let rotation = e.call(REF_GET_ROTATION, &args![result]).u32();
        let angle_z = e.mem.f32(rotation + 8);
        let position = e.vcall(result, REFERENCE_GET_POSITION_SLOT, &args![]).u32();
        e.call(
            ACTOR_PLACE,
            &args![actor, worldspace, cell, position, angle_z],
        );
    } else {
        let position = e.vcall(result, REFERENCE_GET_POSITION_SLOT, &args![]).u32();
        let words = [
            e.mem.u32(position),
            e.mem.u32(position + 4),
            e.mem.u32(position + 8),
        ];
        let scratch = e.mem.alloc(0x10);
        let list = e.call(REF_GET_EXTRA_LIST, &args![result]).u32();
        e.call(
            EXTRA_SET_STARTING_POSITION,
            &args![list, scratch, result, words[0], words[1], words[2]],
        );
        let rotation = e.call(REF_GET_ROTATION, &args![result]).u32();
        let words = [
            e.mem.u32(rotation),
            e.mem.u32(rotation + 4),
            e.mem.u32(rotation + 8),
        ];
        let list = e.call(REF_GET_EXTRA_LIST, &args![result]).u32();
        e.call(
            EXTRA_SET_STARTING_ROTATION,
            &args![list, scratch, result, words[0], words[1], words[2]],
        );
        e.mem.free(scratch);
    }
    Ptr::new(result)
}

/// Makes the reference of the plugin file's current record (its record type
/// byte, `00472660`) and loads it from the file; returns the reference.
fn load_from_file(e: &mut Engine, file: u32) -> u32 {
    let record_type = e.call(FILE_RECORD_TYPE, &args![file]).u8();
    let reference = e
        .call(CREATE_REFERENCE, &args![record_type as u32, 1u32])
        .u32();
    e.call(LOAD_FORM_FROM_FILE, &args![reference, file]);
    reference
}

// Translated from 0085a240 (decompiled, FalloutNV.exe 1.4.0.525)
/// Makes a form of `form_type` (`00670b90`) and gives it `form_id` (virtual
/// slot `0x128`) after `fn_0085a290` has dealt with an existing form of that
/// id. Returns the new form.
pub fn fn_0085a240(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form_id: u32, form_type: u8) -> Ptr {
    fn_0085a290(e, this, form_id);
    let form = e
        .call(CREATE_FORM_OF_TYPE, &args![form_type as u32])
        .ptr::<()>();
    e.vcall(form.addr(), FORM_SET_FORM_ID, &args![form_id, 1u32]);
    form
}

// Translated from 0085a290 (decompiled, FalloutNV.exe 1.4.0.525)
/// Gets rid of what is saved under `form_id`: an existing form is deleted
/// (`DeleteForm`), otherwise the id's changes are dropped from the changes
/// map (`fn_00855220`, force 1).
pub fn fn_0085a290(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form_id: u32) {
    let form = e.call(LOOKUP_FORM, &args![form_id]).u32();
    if form != 0 {
        tes_save_load_game_delete_form(e, this, Ptr::new(form));
    } else {
        let changes = e.get(this, TESSaveLoadGame::m_pChanges);
        fn_00855220(e, changes, form_id, 1);
    }
}

// Translated from 0085a2e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::DeleteForm` (Xbox PDB): logs an error when it is not
/// called while loading (`0047c850` false), drops the form's changes
/// (`fn_008551f0`, force 1). A form already flagged deleted, or a cell, is
/// given a new form id from the data handler (virtual slot `0x128`).
/// Any other form is given id 0, cleared (`00483c70`), given an empty editor
/// id (slot `0x134`) and put on the deferred deletion list (once).
pub fn tes_save_load_game_delete_form(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form: Ptr) {
    if !game_unavailable(e, this) {
        e.call(LOG_ERROR, &args![MSG_DELETE_FORM_NOT_LOADING]);
    }
    let changes = e.get(this, TESSaveLoadGame::m_pChanges);
    fn_008551f0(e, changes, form, 1);
    let cell = dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_CELL);
    if e.call(FORM_IS_DELETED, &args![form]).bool() || cell != 0 {
        let handler: u32 = e.global(DATA_HANDLER);
        let id = e.call(DATA_HANDLER_GET_NEXT_ID, &args![handler]).u32();
        e.vcall(form.addr(), FORM_SET_FORM_ID, &args![id, 1u32]);
    } else {
        e.vcall(form.addr(), FORM_SET_FORM_ID, &args![0u32, 1u32]);
        e.call(FN_00483C70, &args![form]);
        e.vcall(form.addr(), FORM_SET_EDITOR_ID, &args![EMPTY_STRING]);
        tes_save_load_game_add_form_to_deferred_deletions_list(e, this, form);
    }
}

// Translated from 0085a3d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::AddFormToDeferredDeletionsList` (Xbox PDB): adds the
/// form to `m_DeferredDeleteList` unless it is already in it.
pub fn tes_save_load_game_add_form_to_deferred_deletions_list(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    form: Ptr,
) {
    let slot = e.mem.alloc(4);
    e.mem.set_u32(slot, form.addr());
    let list = this.addr() + 0x34;
    if !e.call(LIST_CONTAINS, &args![list, slot]).bool() {
        e.call(LIST_ADD_HEAD, &args![list, slot]);
    }
    e.mem.free(slot);
}

// Translated from 0085a410 (decompiled, FalloutNV.exe 1.4.0.525)
/// Removes the form from `m_DeferredDeleteList` when it is in it.
pub fn fn_0085a410(e: &mut Engine, this: Ptr<TESSaveLoadGame>, form: Ptr) {
    let slot = e.mem.alloc(4);
    e.mem.set_u32(slot, form.addr());
    let list = this.addr() + 0x34;
    if e.call(LIST_CONTAINS, &args![list, slot]).bool() {
        e.call(LIST_REMOVE, &args![list, slot]);
    }
    e.mem.free(slot);
}

// Translated from 0085a450 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::GetInitialDataSaveSize` (Xbox PDB): the bytes
/// `SaveInitialData` writes for the form: 0x1C (a `ReferenceData`) for a
/// reference whose flags have bit 1 or 2, otherwise 0. (The cell cases and
/// the other flag tests compare against 0 in this build and give 0.)
pub fn tes_save_load_game_get_initial_data_save_size(
    e: &mut Engine,
    _this: Ptr<TESSaveLoadGame>,
    form: Ptr,
    flags: u32,
) -> u16 {
    let reference = dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_REFERENCE);
    if reference == 0 {
        dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_CELL);
        return 0;
    }
    if flags & 6 != 0 {
        0x1C
    } else {
        0
    }
}

// Translated from 0085a520 (decompiled, FalloutNV.exe 1.4.0.525)
/// `TESSaveLoadGame::SaveInitialData` (Xbox PDB): for a reference writes
/// its `ReferenceData` (0x1C bytes: the numeric id of its world space or
/// parent cell, position, angle) into the current buffer when the flags have
/// bit 1 or 2. The location id comes from the parent cell's world space,
/// else the parent cell; a reference with neither logs why (non-persistent,
/// persistent without a cell, or an actor whose process level says it
/// should have one). A cell is cast and nothing is written. The code for the
/// flags the compiler folded to 0 is not translated.
pub fn tes_save_load_game_save_initial_data(
    e: &mut Engine,
    this: Ptr<TESSaveLoadGame>,
    form: Ptr,
    flags: u32,
) {
    let reference = dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_REFERENCE);
    if reference == 0 {
        dynamic_cast(e, form.addr(), RTTI_FORM, RTTI_CELL);
        return;
    }
    let record: Ptr<ReferenceData> = Ptr::new(e.mem.alloc(0x1C));
    fn_008572f0(e, record);
    let parent_cell = e.call(REF_GET_PARENT_CELL, &args![reference]).u32();
    let mut worldspace = 0u32;
    if parent_cell != 0 {
        worldspace = e.call(CELL_GET_WORLDSPACE, &args![parent_cell]).u32();
    }
    if worldspace != 0 {
        let id = e.call(FORM_ID, &args![worldspace]).u32();
        let numeric = e.call(ADD_NUMERIC_ID, &args![this, id]).u32();
        e.set(record, ReferenceData::iLocationID, numeric);
    } else if parent_cell != 0 {
        let id = e.call(FORM_ID, &args![parent_cell]).u32();
        let numeric = e.call(ADD_NUMERIC_ID, &args![this, id]).u32();
        e.set(record, ReferenceData::iLocationID, numeric);
    } else {
        if !e.call(REF_PERSISTS, &args![reference]).bool() {
            let id = e.call(FORM_ID, &args![reference]).u32();
            e.call(LOG_ERROR, &args![MSG_NON_PERSISTENT_NO_CELL, id]);
        }
        if e.call(REF_PERSISTS, &args![reference]).bool() {
            let list = e.call(REF_GET_EXTRA_LIST, &args![reference]).u32();
            if e.call(EXTRA_GET_CELL_DATA, &args![list]).u32() == 0 {
                let id = e.call(FORM_ID, &args![reference]).u32();
                e.call(LOG_ERROR, &args![MSG_PERSISTENT_NO_CELL, id]);
            }
        }
        let mobile = dynamic_cast(e, reference, RTTI_REFERENCE, RTTI_MOBILE_OBJECT);
        if mobile != 0 && e.call(ACTOR_GET_PROCESS, &args![mobile]).u32() != 0 {
            let process = e.call(ACTOR_GET_PROCESS, &args![mobile]).u32();
            if e.call(READ_FIELD_28, &args![process]).u32() == 0 {
                let id = e.call(FORM_ID, &args![reference]).u32();
                e.call(LOG_ERROR, &args![MSG_HIGH_PROCESS_NO_CELL, id]);
            } else {
                let process = e.call(ACTOR_GET_PROCESS, &args![mobile]).u32();
                if e.call(READ_FIELD_28, &args![process]).u32() == 1 {
                    let id = e.call(FORM_ID, &args![reference]).u32();
                    e.call(LOG_ERROR, &args![MSG_MIDDLE_HIGH_PROCESS_NO_CELL, id]);
                }
            }
        }
    }
    let position = e.call(REF_GET_POSITION, &args![reference]).u32();
    let words = e.mem.bytes(position, 0x0C);
    e.mem.write(record.addr() + 4, &words);
    let rotation = e.call(REF_GET_ROTATION, &args![reference]).u32();
    let words = e.mem.bytes(rotation, 0x0C);
    e.mem.write(record.addr() + 0x10, &words);
    if flags & 6 != 0 {
        let game = game_singleton(e);
        fn_008579b0(e, game, record.cast(), 0x1C);
    }
    e.mem.free(record.addr());
}

/// This unit's translated functions, by exe address.
pub fn funcs() -> Vec<(u32, AbiFn)> {
    vec![
        entry!(
            0x00486a90,
            ni_t_pointer_map_unsigned_int_change_data_p_new_item(Ptr<ChangesMap>) -> Ptr
        ),
        entry!(
            0x00666050,
            ni_t_pointer_map_unsigned_int_change_data_p_delete_item(Ptr<ChangesMap>, Ptr)
        ),
        entry!(0x00854e10, fn_00854e10(Ptr<ChangeData>)),
        entry!(0x00854e40, fn_00854e40(Ptr<ChangeData>, u32)),
        entry!(0x00854e70, fn_00854e70(Ptr<ChangeData>, u32)),
        entry!(
            0x00854ed0,
            changes_map_scalar_deleting_destructor(Ptr<ChangesMap>, u32) -> Ptr<ChangesMap>
        ),
        entry!(0x00854f00, fn_00854f00(Ptr<ChangesMap>)),
        entry!(0x00854f60, changes_map_remove_all_changes(Ptr<ChangesMap>)),
        entry!(
            0x00854fe0,
            fn_00854fe0(Ptr<ChangeData>, u32) -> Ptr<ChangeData>
        ),
        entry!(
            0x00855010,
            fn_00855010(Ptr<ChangesMap>, Ptr, u32) -> Ptr<ChangeData>
        ),
        entry!(
            0x00855100,
            fn_00855100(Ptr<ChangesMap>, u32) -> Ptr<ChangeData>
        ),
        entry!(
            0x00855130,
            fn_00855130(Ptr<ChangesMap>, Ptr) -> Ptr<ChangeData>
        ),
        entry!(0x00855150, fn_00855150(Ptr<ChangesMap>, Ptr, u32) -> bool),
        entry!(0x008551f0, fn_008551f0(Ptr<ChangesMap>, Ptr, u8) -> bool),
        entry!(0x00855220, fn_00855220(Ptr<ChangesMap>, u32, u8) -> bool),
        entry!(
            0x008552b0,
            fn_008552b0(Ptr<InteriorCellNewReferencesMap>) -> Ptr<InteriorCellNewReferencesMap>
        ),
        entry!(
            0x008552e0,
            interior_cell_new_references_map_scalar_deleting_destructor(
                Ptr<InteriorCellNewReferencesMap>,
                u32,
            ) -> Ptr<
                InteriorCellNewReferencesMap,
            >
        ),
        entry!(0x00855310, fn_00855310(Ptr<InteriorCellNewReferencesMap>)),
        entry!(
            0x008553e0,
            fn_008553e0(Ptr<ExteriorCellNewReferencesMap>) -> Ptr<ExteriorCellNewReferencesMap>
        ),
        entry!(
            0x00855410,
            exterior_cell_new_references_map_scalar_deleting_destructor(
                Ptr<ExteriorCellNewReferencesMap>,
                u32,
            ) -> Ptr<
                ExteriorCellNewReferencesMap,
            >
        ),
        entry!(0x00855440, fn_00855440(Ptr<ExteriorCellNewReferencesMap>)),
        entry!(
            0x00855550,
            fn_00855550(Ptr<NumericIDBufferMap>) -> Ptr<NumericIDBufferMap>
        ),
        entry!(
            0x00855580,
            numeric_id_buffer_map_scalar_deleting_destructor(
                Ptr<NumericIDBufferMap>,
                u32,
            )
                -> Ptr<NumericIDBufferMap>
        ),
        entry!(0x008555b0, fn_008555b0(Ptr<NumericIDBufferMap>)),
        entry!(0x00855660, fn_00855660(Ptr<SaveStats>) -> Ptr<SaveStats>),
        entry!(0x00855730, fn_00855730(Ptr<SaveStats>)),
        entry!(
            0x008558a0,
            save_stats_add_extra_stat(Ptr<SaveStats>, u32, Ptr)
        ),
        entry!(
            0x00855970,
            fn_00855970(Ptr<SaveStats>, Ptr<SaveFormHeader>, u16)
        ),
        entry!(0x00855a20, fn_00855a20(Ptr<SaveStats>, Ptr<LoadFormHeader>)),
        entry!(
            0x00855b60,
            fn_00855b60(Ptr<LoadFormHeader>, Ptr<LoadFormHeader>) -> i32
        ),
        entry!(0x00855ba0, save_stats_print_stats(Ptr<SaveStats>, Ptr)),
        entry!(0x008562c0, fn_008562c0(Ptr<Stats>) -> Ptr<Stats>),
        entry!(0x00856300, fn_00856300(Ptr, Ptr, Ptr) -> bool),
        entry!(
            0x00856c70,
            tes_save_load_game_remove_changes(Ptr<TESSaveLoadGame>, Ptr, u8)
        ),
        entry!(
            0x00856ca0,
            fn_00856ca0(Ptr<TESSaveLoadGame>, Ptr, Ptr, bool) -> bool
        ),
        entry!(0x00857210, fn_00857210(Ptr)),
        entry!(0x00857230, fn_00857230(Ptr<TESSaveLoadGame>, Ptr)),
        entry!(
            0x00857250,
            fn_00857250(Ptr<SaveStats>, u32) -> Ptr<SaveStats>
        ),
        entry!(
            0x00857280,
            fn_00857280(Ptr<FormAndFlags>, Ptr, u32, u32, u8) -> Ptr<FormAndFlags>
        ),
        entry!(0x008572c0, fn_008572c0(Ptr) -> Ptr),
        entry!(
            0x008572f0,
            fn_008572f0(Ptr<ReferenceData>) -> Ptr<ReferenceData>
        ),
        entry!(
            0x00857320,
            fn_00857320(Ptr<MovedReferenceData>) -> Ptr<MovedReferenceData>
        ),
        entry!(0x00857350, fn_00857350(Ptr)),
        entry!(
            0x00857370,
            fn_00857370(Ptr<TESSaveLoadGame>, Ptr, Ptr, u32) -> Ptr
        ),
        entry!(0x008578b0, fn_008578b0(Ptr<TESSaveLoadGame>, Ptr, u32)),
        entry!(0x00857950, fn_00857950(Ptr<TESSaveLoadGame>, Ptr, Ptr)),
        entry!(0x008579b0, fn_008579b0(Ptr<TESSaveLoadGame>, Ptr, u32)),
        entry!(0x008579e0, fn_008579e0(Ptr<TESSaveLoadGame>, Ptr, u32)),
        entry!(
            0x00857a10,
            tes_save_load_game_save_numeric_id(Ptr<TESSaveLoadGame>, Ptr, u32)
        ),
        entry!(
            0x00857aa0,
            tes_save_load_game_load_numeric_id(Ptr<TESSaveLoadGame>, Ptr, u32) -> bool
        ),
        entry!(
            0x00857b50,
            fn_00857b50(Ptr<TESSaveLoadGame>, Ptr, Ptr, u32) -> u32
        ),
        entry!(0x00857ba0, fn_00857ba0(Ptr, Ptr, Ptr, u32) -> u32),
        entry!(0x00857bd0, fn_00857bd0(Ptr<TESSaveLoadGame>, u32)),
        entry!(0x00857bf0, fn_00857bf0(Ptr<TESSaveLoadGame>, u32) -> u32),
        entry!(0x00857c70, fn_00857c70(Ptr<TESSaveLoadGame>, u32) -> u32),
        entry!(0x00857d10, fn_00857d10(Ptr, Ptr, bool) -> bool),
        entry!(
            0x00858030,
            tes_save_load_game_save_global_data(Ptr<TESSaveLoadGame>, Ptr)
        ),
        entry!(
            0x00858480,
            tes_save_load_game_save_globals(Ptr<TESSaveLoadGame>, Ptr)
        ),
        entry!(
            0x00858570,
            tes_save_load_game_save_final_data(Ptr<TESSaveLoadGame>, Ptr)
        ),
        entry!(
            0x00858600,
            tes_save_load_game_create_buffer(Ptr<TESSaveLoadGame>, u32) -> Ptr
        ),
        entry!(
            0x008586a0,
            tes_save_load_game_write_file(Ptr<TESSaveLoadGame>, Ptr, Ptr, u32)
        ),
        entry!(
            0x008586d0,
            fn_008586d0(Ptr<TESSaveLoadGame>, Ptr, Ptr, u32) -> u32
        ),
        entry!(0x00858700, fn_00858700(Ptr<TESSaveLoadGame>, Ptr)),
        entry!(0x00858730, fn_00858730(Ptr<TESSaveLoadGame>, Ptr) -> bool),
        entry!(
            0x00858aa0,
            fn_00858aa0(Ptr<LoadFormHeader>, Ptr, u32, u32) -> Ptr<LoadFormHeader>
        ),
        entry!(
            0x00858af0,
            fn_00858af0(Ptr<TESSaveLoadGame>, Ptr, Ptr, bool)
        ),
        entry!(
            0x00859120,
            tes_save_load_game_check_new_reference(Ptr, Ptr, u32) -> u32
        ),
        entry!(
            0x008591b0,
            tes_save_load_game_check_flags(Ptr<TESSaveLoadGame>, Ptr, u32) -> u32
        ),
        entry!(
            0x00859670,
            base_process_get_package_that_is_running(Ptr) -> u32
        ),
        entry!(0x00859690, fn_00859690(Ptr<TESSaveLoadGame>, Ptr) -> bool),
        entry!(0x008598d0, fn_008598d0(Ptr<TESSaveLoadGame>, u32)),
        entry!(
            0x00859a90,
            fn_00859a90(Ptr<TESSaveLoadGame>, u32, Ptr<CreatedReferenceData>) -> Ptr
        ),
        entry!(
            0x00859f20,
            fn_00859f20(Ptr, u32, Ptr<MovedReferenceData>) -> Ptr
        ),
        entry!(
            0x0085a240,
            fn_0085a240(Ptr<TESSaveLoadGame>, u32, u8) -> Ptr
        ),
        entry!(0x0085a290, fn_0085a290(Ptr<TESSaveLoadGame>, u32)),
        entry!(
            0x0085a2e0,
            tes_save_load_game_delete_form(Ptr<TESSaveLoadGame>, Ptr)
        ),
        entry!(
            0x0085a3d0,
            tes_save_load_game_add_form_to_deferred_deletions_list(Ptr<TESSaveLoadGame>, Ptr)
        ),
        entry!(0x0085a410, fn_0085a410(Ptr<TESSaveLoadGame>, Ptr)),
        entry!(
            0x0085a450,
            tes_save_load_game_get_initial_data_save_size(Ptr<TESSaveLoadGame>, Ptr, u32) -> u16
        ),
        entry!(
            0x0085a520,
            tes_save_load_game_save_initial_data(Ptr<TESSaveLoadGame>, Ptr, u32)
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    type Log = Vec<(u32, Vec<u32>)>;
    type Lines = Rc<RefCell<Vec<String>>>;
    type Calls = Rc<RefCell<Vec<Vec<u32>>>>;
    type FileWrites = Rc<RefCell<Vec<(u32, Vec<u8>)>>>;
    type ByteWrites = Rc<RefCell<Vec<Vec<u8>>>>;
    type Added = Rc<RefCell<Vec<(u32, u32, u32, u32, u8)>>>;
    type Hooks = Rc<RefCell<Vec<(u32, u32, u32, u32)>>>;

    fn returns(value: u32) -> Ret {
        Ret {
            eax: value,
            ..Ret::default()
        }
    }

    fn stub(e: &mut Engine, address: u32) {
        e.register(address, |_, _| Ret::default());
    }

    /// A double that returns `value` whatever it is called with.
    fn constant(e: &mut Engine, address: u32, value: u32) {
        e.register_double(address, move |_, _| returns(value));
    }

    /// An engine with the globals the unit reads mapped and working doubles
    /// for the small callees nearly every function uses: the dword getters,
    /// the list node accessors and constructor, `strlen`/`strcpy_s`, and the
    /// allocation scope (logged only). `operator new` and `delete` are the
    /// crate's own. Calls are logged.
    fn game() -> Engine {
        let mut e = Engine::new();
        e.map(0x011d_e000, 0x1000); // the TESSaveLoadGame pointer
        e.map(0x0120_2000, 0x1000); // the save lock
        e.map(0x010a_2000, 0x1000); // the seek mode
        e.map(0x0101_6000, 0x1000); // the message time
        let singleton = e.new_object::<TESSaveLoadGame>();
        e.set_global(SAVE_LOAD_GAME, singleton.addr());
        e.set_global(MESSAGE_TIME, 2.0f32);
        e.register(FORM_ID, |e, a| returns(e.mem.u32(a[0] + 0xc)));
        e.register(READ_WORD, |e, a| returns(e.mem.u32(a[0])));
        e.register(LIST_NODE_ITEM, |_, a| returns(a[0]));
        e.register(LIST_NODE_NEXT, |e, a| returns(e.mem.u32(a[0] + 4)));
        e.register(SAVE_LOAD_UNAVAILABLE, |_, _| returns(0));
        e.register(SIMPLE_LIST_CONSTRUCT, |e, a| {
            e.mem.set_u32(a[0], 0);
            e.mem.set_u32(a[0] + 4, 0);
            returns(a[0])
        });
        e.register(STRLEN, |e, a| returns(e.mem.cstr(a[0]).len() as u32));
        e.register(STRING_COPY, |e, a| {
            let text = e.mem.cstr(a[2]);
            e.mem.set_cstr(a[0], &text);
            Ret::default()
        });
        e.register(STRING_COMPARE, |e, a| {
            returns(e.mem.cstr(a[0]).cmp(&e.mem.cstr(a[1])) as i32 as u32)
        });
        stub(&mut e, SCOPE_ENTER);
        stub(&mut e, SCOPE_LEAVE);
        e.call_log = Some(vec![]);
        e
    }

    /// The logged argument lists of the calls to `address`.
    fn calls_to(e: &Engine, address: u32) -> Vec<Vec<u32>> {
        let log: &Log = e.call_log.as_ref().unwrap();
        log.iter()
            .filter(|(a, _)| *a == address)
            .map(|(_, args)| args.clone())
            .collect()
    }

    fn freed(e: &Engine, block: u32) -> bool {
        e.mem.block_size(block).is_none()
    }

    /// A singly linked `BSSimpleList` of the items; returns the first node
    /// (0 for none).
    fn list_of(e: &mut Engine, items: &[u32]) -> u32 {
        let mut next = 0;
        for &item in items.iter().rev() {
            let node = e.mem.alloc(8);
            e.mem.set_u32(node, item);
            e.mem.set_u32(node + 4, next);
            next = node;
        }
        next
    }

    /// The map iteration as the game's `GetFirstPos` / `GetNext` give it,
    /// over `entries` (key, value).
    fn install_entries(e: &mut Engine, entries: &[(u32, u32)]) {
        let queue = Rc::new(RefCell::new(VecDeque::from(entries.to_vec())));
        let first = queue.clone();
        e.register_double(MAP_FIRST_POSITION, move |_, _| {
            returns(!first.borrow().is_empty() as u32)
        });
        e.register_double(MAP_NEXT, move |e, a| {
            let (key, value) = queue.borrow_mut().pop_front().unwrap();
            e.mem.set_u32(a[2], key);
            e.mem.set_u32(a[3], value);
            e.mem.set_u32(a[1], !queue.borrow().is_empty() as u32);
            Ret::default()
        });
    }

    /// `install_entries` for `SaveStats`'s map, keyed by a byte.
    fn install_byte_entries(e: &mut Engine, entries: &[(u8, u32)]) {
        let queue = Rc::new(RefCell::new(VecDeque::from(entries.to_vec())));
        let first = queue.clone();
        e.register_double(MAP_FIRST_POSITION, move |_, _| {
            returns(!first.borrow().is_empty() as u32)
        });
        e.register_double(BYTE_MAP_NEXT, move |e, a| {
            let (key, value) = queue.borrow_mut().pop_front().unwrap();
            e.mem.set_u8(a[2], key);
            e.mem.set_u32(a[3], value);
            e.mem.set_u32(a[1], !queue.borrow().is_empty() as u32);
            Ret::default()
        });
    }

    type Table = Rc<RefCell<Vec<(u32, u32)>>>;

    /// A key-value store behind `GetAt`, `SetAt` and `RemoveAt` of the
    /// given addresses.
    fn install_table(e: &mut Engine, get_at: u32, set_at: Option<u32>, remove_at: u32) -> Table {
        let table: Table = Rc::new(RefCell::new(vec![]));
        let t = table.clone();
        e.register_double(get_at, move |e, a| {
            let found = t.borrow().iter().find(|(k, _)| *k == a[1]).map(|kv| kv.1);
            if let Some(value) = found {
                e.mem.set_u32(a[2], value);
            }
            returns(found.is_some() as u32)
        });
        if let Some(set_at) = set_at {
            let t = table.clone();
            e.register_double(set_at, move |_, a| {
                t.borrow_mut().push((a[1], a[2]));
                Ret::default()
            });
        }
        let t = table.clone();
        e.register_double(remove_at, move |_, a| {
            let before = t.borrow().len();
            t.borrow_mut().retain(|(k, _)| *k != a[1]);
            returns((t.borrow().len() != before) as u32)
        });
        table
    }

    /// A form whose key is `key` (`+0x0C`).
    fn form_with_key(e: &mut Engine, key: u32) -> Ptr {
        let form = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(form.addr() + 0xc, key);
        form
    }

    fn change_data(e: &mut Engine, flags: u32, buffer: u32) -> Ptr<ChangeData> {
        let data: Ptr<ChangeData> = e.new_object();
        e.set(data, ChangeData::iFlags, flags);
        e.set(data, ChangeData::pBuffer, Ptr::new(buffer));
        data
    }

    #[test]
    fn new_item_asks_the_allocator_subobject() {
        let mut e = game();
        constant(&mut e, MAP_ALLOCATOR_NEW_ITEM, 0x1234);
        let map: Ptr<ChangesMap> = e.new_object();
        let item = ni_t_pointer_map_unsigned_int_change_data_p_new_item(&mut e, map);
        assert_eq!(item.addr(), 0x1234);
        assert_eq!(
            calls_to(&e, MAP_ALLOCATOR_NEW_ITEM),
            vec![vec![map.addr() + 0xc]]
        );
    }

    #[test]
    fn delete_item_clears_the_value_and_gives_the_entry_back() {
        let mut e = game();
        stub(&mut e, MAP_ALLOCATOR_DELETE_ITEM);
        let map: Ptr<ChangesMap> = e.new_object();
        let item = e.mem.alloc(12);
        e.mem.set_u32(item + 8, 0x77);
        ni_t_pointer_map_unsigned_int_change_data_p_delete_item(&mut e, map, Ptr::new(item));
        assert_eq!(e.mem.u32(item + 8), 0);
        assert_eq!(
            calls_to(&e, MAP_ALLOCATOR_DELETE_ITEM),
            vec![vec![map.addr() + 0xc, item]]
        );
    }

    #[test]
    fn change_data_destructor_frees_only_a_buffer() {
        let mut e = game();
        let buffer = e.mem.alloc(16);
        let with_buffer = change_data(&mut e, 1, buffer);
        fn_00854e10(&mut e, with_buffer);
        assert!(freed(&e, buffer));
        let without = change_data(&mut e, 1, 0);
        fn_00854e10(&mut e, without);
        assert_eq!(calls_to(&e, OPERATOR_DELETE), vec![vec![buffer]]);
    }

    #[test]
    fn add_flags_only_without_a_buffer() {
        let mut e = game();
        let plain = change_data(&mut e, 0b0101, 0);
        fn_00854e40(&mut e, plain, 0b0010);
        assert_eq!(e.get(plain, ChangeData::iFlags), 0b0111);
        let buffered = change_data(&mut e, 0b0101, 0x1000);
        fn_00854e40(&mut e, buffered, 0b0010);
        assert_eq!(e.get(buffered, ChangeData::iFlags), 0b0101);
    }

    #[test]
    fn remove_flags_only_without_a_buffer() {
        let mut e = game();
        let plain = change_data(&mut e, 0b0111, 0);
        fn_00854e70(&mut e, plain, 0b0010);
        assert_eq!(e.get(plain, ChangeData::iFlags), 0b0101);
        let buffered = change_data(&mut e, 0b0111, 0x1000);
        fn_00854e70(&mut e, buffered, 0b0010);
        assert_eq!(e.get(buffered, ChangeData::iFlags), 0b0111);
    }

    #[test]
    fn changes_map_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        install_entries(&mut e, &[]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, CHANGES_MAP_BASE_DESTRUCT);
        let kept: Ptr<ChangesMap> = e.new_object();
        let result = changes_map_scalar_deleting_destructor(&mut e, kept, 0);
        assert_eq!(result, kept);
        assert!(!freed(&e, kept.addr()));
        assert_eq!(e.mem.u32(kept.addr()), CHANGES_MAP_VTABLE);
        let deleted: Ptr<ChangesMap> = e.new_object();
        changes_map_scalar_deleting_destructor(&mut e, deleted, 1);
        assert!(freed(&e, deleted.addr()));
        assert_eq!(calls_to(&e, CHANGES_MAP_BASE_DESTRUCT).len(), 2);
    }

    #[test]
    fn changes_map_destructor_removes_the_changes_then_the_base() {
        let mut e = game();
        let data = change_data(&mut e, 1, 0);
        install_entries(&mut e, &[(0x100, data.addr())]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, CHANGES_MAP_BASE_DESTRUCT);
        let map: Ptr<ChangesMap> = e.new_object();
        fn_00854f00(&mut e, map);
        assert_eq!(e.mem.u32(map.addr()), CHANGES_MAP_VTABLE);
        assert!(freed(&e, data.addr()));
        let order: Vec<u32> = e
            .call_log
            .as_ref()
            .unwrap()
            .iter()
            .map(|(a, _)| *a)
            .filter(|a| *a == MAP_REMOVE_ALL || *a == CHANGES_MAP_BASE_DESTRUCT)
            .collect();
        assert_eq!(order, vec![MAP_REMOVE_ALL, CHANGES_MAP_BASE_DESTRUCT]);
    }

    #[test]
    fn remove_all_changes_deletes_each_change_data_and_its_buffer() {
        let mut e = game();
        let buffer = e.mem.alloc(8);
        let with_buffer = change_data(&mut e, 1, buffer);
        let plain = change_data(&mut e, 2, 0);
        install_entries(
            &mut e,
            &[
                (0x100, with_buffer.addr()),
                (0x200, 0),
                (0x300, plain.addr()),
            ],
        );
        stub(&mut e, MAP_REMOVE_ALL);
        let map: Ptr<ChangesMap> = e.new_object();
        changes_map_remove_all_changes(&mut e, map);
        assert!(freed(&e, buffer));
        assert!(freed(&e, with_buffer.addr()));
        assert!(freed(&e, plain.addr()));
        assert_eq!(calls_to(&e, MAP_REMOVE_ALL), vec![vec![map.addr()]]);
    }

    #[test]
    fn change_data_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        let buffer = e.mem.alloc(8);
        let data = change_data(&mut e, 1, buffer);
        assert_eq!(fn_00854fe0(&mut e, data, 0), data);
        assert!(freed(&e, buffer));
        assert!(!freed(&e, data.addr()));
        let other = change_data(&mut e, 1, 0);
        fn_00854fe0(&mut e, other, 1);
        assert!(freed(&e, other.addr()));
    }

    #[test]
    fn get_or_create_adds_flags_to_the_existing_change_data() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, Some(CHANGES_MAP_SET_AT), MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let form = form_with_key(&mut e, 0x77);
        let existing = change_data(&mut e, 0b01, 0);
        table.borrow_mut().push((0x77, existing.addr()));
        let result = fn_00855010(&mut e, map, form, 0b10);
        assert_eq!(result, existing);
        assert_eq!(e.get(existing, ChangeData::iFlags), 0b11);
        assert!(calls_to(&e, SCOPE_ENTER).is_empty());
    }

    #[test]
    fn get_or_create_makes_and_stores_a_new_change_data() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, Some(CHANGES_MAP_SET_AT), MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let form = form_with_key(&mut e, 0x78);
        let created = fn_00855010(&mut e, map, form, 0b100);
        assert!(!created.is_null());
        assert_eq!(e.get(created, ChangeData::iFlags), 0b100);
        assert_eq!(table.borrow().as_slice(), &[(0x78, created.addr())]);
        // The allocation scope: (0x11, 1, file, line 0x104).
        let scope = calls_to(&e, SCOPE_ENTER);
        assert_eq!(scope.len(), 1);
        assert_eq!(&scope[0][1..], &[0x11, 1, SOURCE_FILE, 0x104]);
        assert_eq!(calls_to(&e, SCOPE_LEAVE).len(), 1);
    }

    #[test]
    fn lookup_by_key_returns_the_stored_change_data_or_null() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((5, data.addr()));
        assert_eq!(fn_00855100(&mut e, map, 5), data);
        assert!(fn_00855100(&mut e, map, 6).is_null());
    }

    #[test]
    fn lookup_by_form_uses_the_forms_key() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((0x1234, data.addr()));
        let form = form_with_key(&mut e, 0x1234);
        assert_eq!(fn_00855130(&mut e, map, form), data);
        let other = form_with_key(&mut e, 0x4321);
        assert!(fn_00855130(&mut e, map, other).is_null());
    }

    #[test]
    fn clear_flags_keeps_the_change_data_while_flags_remain() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let form = form_with_key(&mut e, 9);
        let data = change_data(&mut e, 0b11, 0);
        table.borrow_mut().push((9, data.addr()));
        assert!(fn_00855150(&mut e, map, form, 0b01));
        assert_eq!(e.get(data, ChangeData::iFlags), 0b10);
        assert_eq!(table.borrow().len(), 1);
        assert!(!freed(&e, data.addr()));
    }

    #[test]
    fn clear_flags_removes_the_change_data_once_empty() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let form = form_with_key(&mut e, 9);
        let data = change_data(&mut e, 0b11, 0);
        table.borrow_mut().push((9, data.addr()));
        assert!(fn_00855150(&mut e, map, form, 0b11));
        assert!(table.borrow().is_empty());
        assert!(freed(&e, data.addr()));
        // No change data, or saving unavailable: false.
        assert!(!fn_00855150(&mut e, map, form, 1));
        constant(&mut e, SAVE_LOAD_UNAVAILABLE, 1);
        assert!(!fn_00855150(&mut e, map, form, 1));
    }

    #[test]
    fn drop_by_form_passes_the_forms_key_and_the_flag() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let form = form_with_key(&mut e, 0x55);
        let buffer = e.mem.alloc(8);
        let data = change_data(&mut e, 1, buffer);
        table.borrow_mut().push((0x55, data.addr()));
        // A buffered change data stays unless forced.
        assert!(fn_008551f0(&mut e, map, form, 0));
        assert_eq!(table.borrow().len(), 1);
        assert!(!freed(&e, data.addr()));
        assert!(fn_008551f0(&mut e, map, form, 1));
        assert!(table.borrow().is_empty());
        assert!(freed(&e, data.addr()));
        assert!(freed(&e, buffer));
        assert_eq!(calls_to(&e, MAP_REMOVE_AT), vec![vec![map.addr(), 0x55]]);
    }

    #[test]
    fn drop_by_key_removes_unbuffered_data_and_false_when_missing() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((3, data.addr()));
        assert!(fn_00855220(&mut e, map, 3, 0));
        assert!(table.borrow().is_empty());
        assert!(freed(&e, data.addr()));
        assert!(!fn_00855220(&mut e, map, 3, 0));
        constant(&mut e, SAVE_LOAD_UNAVAILABLE, 1);
        assert!(!fn_00855220(&mut e, map, 3, 1));
    }

    #[test]
    fn interior_map_constructor_sets_base_and_vtable() {
        let mut e = game();
        e.register(INTERIOR_MAP_BASE_CONSTRUCT, |_, a| returns(a[0]));
        let map: Ptr<InteriorCellNewReferencesMap> = e.new_object();
        assert_eq!(fn_008552b0(&mut e, map), map);
        assert_eq!(e.mem.u32(map.addr()), INTERIOR_MAP_VTABLE);
        assert_eq!(
            calls_to(&e, INTERIOR_MAP_BASE_CONSTRUCT),
            vec![vec![map.addr(), 0x25]]
        );
    }

    #[test]
    fn interior_map_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        install_entries(&mut e, &[]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, INTERIOR_MAP_BASE_DESTRUCT);
        let map: Ptr<InteriorCellNewReferencesMap> = e.new_object();
        interior_cell_new_references_map_scalar_deleting_destructor(&mut e, map, 0);
        assert!(!freed(&e, map.addr()));
        interior_cell_new_references_map_scalar_deleting_destructor(&mut e, map, 1);
        assert!(freed(&e, map.addr()));
    }

    #[test]
    fn interior_map_destructor_deletes_each_list() {
        let mut e = game();
        let list = e.mem.alloc(8);
        install_entries(&mut e, &[(1, list), (2, 0)]);
        stub(&mut e, LIST_REMOVE_ALL);
        e.register(LIST_SCALAR_DELETE, |e, a| {
            e.mem.free(a[0]);
            returns(a[0])
        });
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, INTERIOR_MAP_BASE_DESTRUCT);
        let map: Ptr<InteriorCellNewReferencesMap> = e.new_object();
        fn_00855310(&mut e, map);
        assert_eq!(e.mem.u32(map.addr()), INTERIOR_MAP_VTABLE);
        assert!(freed(&e, list));
        assert_eq!(calls_to(&e, LIST_REMOVE_ALL), vec![vec![list]]);
        assert_eq!(calls_to(&e, INTERIOR_MAP_BASE_DESTRUCT).len(), 1);
    }

    #[test]
    fn exterior_map_constructor_sets_base_and_vtable() {
        let mut e = game();
        e.register(EXTERIOR_MAP_BASE_CONSTRUCT, |_, a| returns(a[0]));
        let map: Ptr<ExteriorCellNewReferencesMap> = e.new_object();
        assert_eq!(fn_008553e0(&mut e, map), map);
        assert_eq!(e.mem.u32(map.addr()), EXTERIOR_MAP_VTABLE);
        assert_eq!(
            calls_to(&e, EXTERIOR_MAP_BASE_CONSTRUCT),
            vec![vec![map.addr(), 0x25]]
        );
    }

    #[test]
    fn exterior_map_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        install_entries(&mut e, &[]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, EXTERIOR_MAP_BASE_DESTRUCT);
        let map: Ptr<ExteriorCellNewReferencesMap> = e.new_object();
        exterior_cell_new_references_map_scalar_deleting_destructor(&mut e, map, 0);
        assert!(!freed(&e, map.addr()));
        exterior_cell_new_references_map_scalar_deleting_destructor(&mut e, map, 1);
        assert!(freed(&e, map.addr()));
    }

    #[test]
    fn exterior_map_destructor_frees_the_items_of_each_list() {
        let mut e = game();
        let (first, second) = (e.mem.alloc(12), e.mem.alloc(12));
        let list = list_of(&mut e, &[first, 0, second]);
        install_entries(&mut e, &[(1, list), (2, 0)]);
        stub(&mut e, LIST_REMOVE_ALL);
        e.register(LIST_SCALAR_DELETE, |_, a| returns(a[0]));
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, EXTERIOR_MAP_BASE_DESTRUCT);
        let map: Ptr<ExteriorCellNewReferencesMap> = e.new_object();
        fn_00855440(&mut e, map);
        assert!(freed(&e, first));
        assert!(freed(&e, second));
        assert_eq!(calls_to(&e, LIST_REMOVE_ALL), vec![vec![list]]);
        assert_eq!(calls_to(&e, LIST_SCALAR_DELETE), vec![vec![list, 1]]);
        assert_eq!(e.mem.u32(map.addr()), EXTERIOR_MAP_VTABLE);
    }

    #[test]
    fn numeric_id_map_constructor_sets_base_and_vtable() {
        let mut e = game();
        e.register(NUMERIC_ID_MAP_BASE_CONSTRUCT, |_, a| returns(a[0]));
        let map: Ptr<NumericIDBufferMap> = e.new_object();
        assert_eq!(fn_00855550(&mut e, map), map);
        assert_eq!(e.mem.u32(map.addr()), NUMERIC_ID_MAP_VTABLE);
        assert_eq!(
            calls_to(&e, NUMERIC_ID_MAP_BASE_CONSTRUCT),
            vec![vec![map.addr(), 0x25]]
        );
    }

    #[test]
    fn numeric_id_map_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        install_entries(&mut e, &[]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, NUMERIC_ID_MAP_BASE_DESTRUCT);
        let map: Ptr<NumericIDBufferMap> = e.new_object();
        numeric_id_buffer_map_scalar_deleting_destructor(&mut e, map, 0);
        assert!(!freed(&e, map.addr()));
        numeric_id_buffer_map_scalar_deleting_destructor(&mut e, map, 1);
        assert!(freed(&e, map.addr()));
    }

    #[test]
    fn numeric_id_map_destructor_frees_the_buffers() {
        let mut e = game();
        let buffer = e.mem.alloc(8);
        install_entries(&mut e, &[(1, buffer), (2, 0)]);
        stub(&mut e, MAP_REMOVE_ALL);
        stub(&mut e, NUMERIC_ID_MAP_BASE_DESTRUCT);
        let map: Ptr<NumericIDBufferMap> = e.new_object();
        fn_008555b0(&mut e, map);
        assert!(freed(&e, buffer));
        assert_eq!(e.mem.u32(map.addr()), NUMERIC_ID_MAP_VTABLE);
        assert_eq!(calls_to(&e, NUMERIC_ID_MAP_BASE_DESTRUCT).len(), 1);
    }

    #[test]
    fn save_stats_constructor_makes_the_map_and_the_list() {
        let mut e = game();
        e.register(STATS_MAP_CONSTRUCT, |_, a| returns(a[0]));
        let stats: Ptr<SaveStats> = e.new_object();
        assert_eq!(fn_00855660(&mut e, stats), stats);
        let map = e.get(stats, SaveStats::pStatsMap);
        let list = e.get(stats, SaveStats::pExtraStats);
        assert!(!map.is_null() && !list.is_null());
        assert_eq!(e.mem.block_size(map.addr()), Some(0x10));
        assert_eq!(e.mem.block_size(list.addr()), Some(8));
        assert_eq!(
            calls_to(&e, STATS_MAP_CONSTRUCT),
            vec![vec![map.addr(), 0x25]]
        );
    }

    #[test]
    fn save_stats_destructor_frees_headers_extra_stats_and_lists() {
        let mut e = game();
        let (header_a, header_b) = (e.mem.alloc(12), e.mem.alloc(12));
        let type_list = list_of(&mut e, &[header_a, header_b]);
        install_byte_entries(&mut e, &[(5, type_list)]);
        stub(&mut e, LIST_REMOVE_ALL);
        e.register(LIST_SCALAR_DELETE, |_, a| returns(a[0]));
        let map_destructor = 0x0200_1000;
        let vtable = 0x0200_0000;
        e.put_vtable(vtable, &[map_destructor]);
        stub(&mut e, map_destructor);
        let stats_map = e.mem.alloc(0x10);
        e.mem.set_u32(stats_map, vtable);
        let description = e.mem.alloc(8);
        let stat = e.mem.alloc(8);
        e.mem.set_u32(stat + 4, description);
        let extra_list = list_of(&mut e, &[stat, 0]);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(stats_map));
        e.set(stats, SaveStats::pExtraStats, Ptr::new(extra_list));
        fn_00855730(&mut e, stats);
        assert!(freed(&e, header_a));
        assert!(freed(&e, header_b));
        assert!(freed(&e, description));
        assert!(freed(&e, stat));
        assert_eq!(
            calls_to(&e, LIST_SCALAR_DELETE),
            vec![vec![type_list, 1], vec![extra_list, 1]]
        );
        // The map's own destructor, vtable slot 0, with flag 1.
        assert_eq!(calls_to(&e, map_destructor), vec![vec![stats_map, 1]]);
    }

    #[test]
    fn add_extra_stat_copies_the_description_to_the_list_head() {
        let mut e = game();
        // The record is read while the list head call runs: the cell that
        // holds its address is freed afterwards.
        let seen = Rc::new(RefCell::new(Vec::new()));
        let sink = seen.clone();
        e.register_double(LIST_ADD_HEAD, move |e, a| {
            let record = e.mem.u32(a[1]);
            sink.borrow_mut().push((a[0], record));
            Ret::default()
        });
        let list = e.mem.alloc(8);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pExtraStats, Ptr::new(list));
        let text = e.mem.alloc(16);
        e.mem.set_cstr(text, b"Animations");
        save_stats_add_extra_stat(&mut e, stats, 1234, Ptr::new(text));
        let seen = seen.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].0, list);
        let record: Ptr<ExtraStat> = Ptr::new(seen[0].1);
        assert_eq!(e.get(record, ExtraStat::iSize), 1234);
        let copy = e.get(record, ExtraStat::pDescription);
        assert_ne!(copy.addr(), text);
        assert_eq!(e.mem.cstr(copy.addr()), b"Animations");
        // The copy has room for the terminator.
        assert_eq!(e.mem.block_size(copy.addr()), Some(16));
        assert_eq!(
            &calls_to(&e, SCOPE_ENTER)[0][1..],
            &[0x11, 1, SOURCE_FILE, 0x267]
        );
        assert_eq!(calls_to(&e, SCOPE_LEAVE).len(), 1);
    }

    #[test]
    fn stats_record_copies_the_header_and_adds_the_size() {
        let mut e = game();
        install_table(
            &mut e,
            BYTE_MAP_GET_AT,
            Some(BYTE_MAP_SET_AT),
            MAP_REMOVE_AT,
        );
        stub(&mut e, LIST_INSERT);
        let map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(map));
        let header: Ptr<SaveFormHeader> = e.new_object();
        e.set(header, SaveFormHeader::iFormID, 0xAABBCCDD);
        e.set(header, SaveFormHeader::cFormType, 0x2A);
        e.set(header, SaveFormHeader::iFlags, 0x11223344);
        e.set(header, SaveFormHeader::cVersion, 0x0F);
        fn_00855970(&mut e, stats, header, 0x0123);
        // The list insert receives the heap copy of the 12-byte header.
        let insert = calls_to(&e, LIST_INSERT);
        assert_eq!(insert.len(), 1);
        assert_eq!(
            e.mem.bytes(insert[0][1], 12),
            vec![0xDD, 0xCC, 0xBB, 0xAA, 0x2A, 0x44, 0x33, 0x22, 0x11, 0x0F, 0x23, 0x01]
        );
        // Two scopes: the one of this function (line 0x276), then the one of
        // the insert (line 0x286).
        let scopes = calls_to(&e, SCOPE_ENTER);
        assert_eq!(scopes[0][4], 0x276);
        assert_eq!(scopes[1][4], 0x286);
    }

    #[test]
    fn stats_insert_creates_the_type_list_when_missing() {
        let mut e = game();
        let get = install_table(
            &mut e,
            BYTE_MAP_GET_AT,
            Some(BYTE_MAP_SET_AT),
            MAP_REMOVE_AT,
        );
        stub(&mut e, LIST_INSERT);
        let map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(map));
        let header: Ptr<LoadFormHeader> = e.new_object();
        e.set(header, LoadFormHeader::iFormID, 0x42);
        e.set(header, LoadFormHeader::cFormType, 9);
        e.set(header, LoadFormHeader::iSize, 77);
        fn_00855a20(&mut e, stats, header);
        // A list was created and stored under the type.
        let stored = get.borrow().clone();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].0, 9);
        let insert = calls_to(&e, LIST_INSERT);
        assert_eq!(insert.len(), 1);
        assert_eq!(insert[0][0], stored[0].1);
        assert_eq!(insert[0][2], STATS_COMPARATOR);
        // The inserted item is a copy of the header.
        let copy = insert[0][1];
        assert_ne!(copy, header.addr());
        assert_eq!(e.mem.u32(copy), 0x42);
        assert_eq!(e.mem.u16(copy + 0xa), 77);
        assert_eq!(
            &calls_to(&e, SCOPE_ENTER)[0][1..],
            &[0x11, 1, SOURCE_FILE, 0x286]
        );
    }

    #[test]
    fn stats_insert_reuses_the_existing_type_list() {
        let mut e = game();
        let table = install_table(
            &mut e,
            BYTE_MAP_GET_AT,
            Some(BYTE_MAP_SET_AT),
            MAP_REMOVE_AT,
        );
        stub(&mut e, LIST_INSERT);
        let list = e.mem.alloc(8);
        table.borrow_mut().push((9, list));
        let map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(map));
        let header: Ptr<LoadFormHeader> = e.new_object();
        e.set(header, LoadFormHeader::cFormType, 9);
        fn_00855a20(&mut e, stats, header);
        assert_eq!(table.borrow().len(), 1);
        assert_eq!(calls_to(&e, LIST_INSERT)[0][0], list);
    }

    #[test]
    fn stats_comparator_orders_largest_first() {
        let mut e = game();
        let small: Ptr<LoadFormHeader> = e.new_object();
        let large: Ptr<LoadFormHeader> = e.new_object();
        let twin: Ptr<LoadFormHeader> = e.new_object();
        e.set(small, LoadFormHeader::iSize, 10);
        e.set(large, LoadFormHeader::iSize, 0xFFF0);
        e.set(twin, LoadFormHeader::iSize, 10);
        assert_eq!(fn_00855b60(&mut e, large, small), -1);
        assert_eq!(fn_00855b60(&mut e, small, large), 1);
        assert_eq!(fn_00855b60(&mut e, small, twin), 0);
    }

    /// The doubles `PrintStats` needs; returns the lines written to the file
    /// and the `FORMAT` calls (all argument words).
    fn print_engine() -> (Engine, Lines, Calls) {
        let mut e = game();
        e.map(0x0118_7000, 0x1000); // the form type names
        e.map(0x0108_0000, 0x1000); // the format strings
        e.map(0x0101_1000, 0x1000); // the empty string
        e.mem.set_cstr(0x0108_05c4, b"Extra Stats:\r\n\r\n");
        stub(&mut e, STRING_CAT);
        e.register(FILE_EXISTS, |_, _| returns(0));
        stub(&mut e, FILE_DELETE);
        stub(&mut e, FILE_OBJECT_CONSTRUCT);
        stub(&mut e, FILE_OBJECT_DESTRUCT);
        let lines = Rc::new(RefCell::new(Vec::new()));
        let formats = Rc::new(RefCell::new(Vec::new()));
        let sink = formats.clone();
        e.register_double(FORMAT, move |e, a| {
            sink.borrow_mut().push(a.to_vec());
            // The "text" is the format's address, so the written lines can
            // be told apart.
            e.mem.set_cstr(a[0], format!("{:08x}", a[2]).as_bytes());
            Ret::default()
        });
        let written = lines.clone();
        e.register_double(SYSTEM_FILE_DO_WRITE, move |e, a| {
            let text = e.mem.cstr(a[1]);
            assert_eq!(text.len() as u32, a[2]);
            written.borrow_mut().push(String::from_utf8(text).unwrap());
            returns(0)
        });
        stub(&mut e, BUILD_CHANGES_STRING);
        e.register(DYNAMIC_CAST, |_, _| returns(0));
        (e, lines, formats)
    }

    fn load_header(e: &mut Engine, id: u32, form_type: u8, size: u16, flags: u32) -> u32 {
        let header: Ptr<LoadFormHeader> = e.new_object();
        e.set(header, LoadFormHeader::iFormID, id);
        e.set(header, LoadFormHeader::cFormType, form_type);
        e.set(header, LoadFormHeader::iFlags, flags);
        e.set(header, LoadFormHeader::cVersion, 15);
        e.set(header, LoadFormHeader::iSize, size);
        header.addr()
    }

    /// A form object whose description (vtable slot `0x130`) is `text`.
    fn form_describing(e: &mut Engine, vtable: u32, text: &[u8]) -> u32 {
        let describe = vtable + 0x200;
        e.mem.map(vtable, 0x300);
        e.mem.set_u32(vtable + 0x130, describe);
        let name = e.mem.alloc(16);
        e.mem.set_cstr(name, text);
        constant(e, describe, name);
        let form = e.mem.alloc(0x40);
        e.mem.set_u32(form, vtable);
        form
    }

    #[test]
    fn print_stats_writes_the_sections_and_the_totals() {
        let (mut e, lines, formats) = print_engine();
        // Type 0x79 with two forms: 0x100 not loaded (size 10), 0x200 loaded
        // (size 30); one extra stat of 5.
        let first = load_header(&mut e, 0x100, 0x79, 10, 0xA);
        let second = load_header(&mut e, 0x200, 0x79, 30, 0xB);
        let type_list = list_of(&mut e, &[second, first]);
        install_byte_entries(&mut e, &[(0x79, type_list)]);
        let form = form_describing(&mut e, 0x0200_0000, b"Boone");
        e.register_double(LOOKUP_FORM, move |_, a| {
            returns(if a[0] == 0x200 { form } else { 0 })
        });
        let description = e.mem.alloc(8);
        e.mem.set_cstr(description, b"Textures");
        let stat = e.mem.alloc(8);
        e.mem.set_u32(stat, 5);
        e.mem.set_u32(stat + 4, description);
        let extra_list = list_of(&mut e, &[stat]);
        let stats_map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(stats_map));
        e.set(stats, SaveStats::pExtraStats, Ptr::new(extra_list));
        let path = e.mem.alloc(16);
        e.mem.set_cstr(path, b"Saves\\x");

        save_stats_print_stats(&mut e, stats, Ptr::new(path));

        // The file object is built for the path with the extension.
        let construct = calls_to(&e, FILE_OBJECT_CONSTRUCT);
        assert_eq!(construct.len(), 1);
        assert_eq!(&construct[0][2..], &[1, 2, 0]);
        assert_eq!(calls_to(&e, FILE_OBJECT_DESTRUCT).len(), 1);
        assert!(calls_to(&e, FILE_DELETE).is_empty());

        let formats = formats.borrow();
        let used: Vec<u32> = formats.iter().map(|f| f[2]).collect();
        assert_eq!(
            used,
            vec![
                0x0108_0670, // table header
                0x0108_0668, // "Buffer"
                0x0108_03e4, // section heading
                0x0108_0648, // form 0x200 (largest first)
                0x0108_0648, // form 0x100
                0x0108_05d8, // section totals
                0x0108_0254, // extra stat
                0x0108_0570, // grand totals
            ]
        );
        // Rows: the loaded form prints its description, the other NOT LOADED.
        let row_loaded = &formats[3];
        assert_eq!(&row_loaded[3..7], &[0x200, 30, 0xB, 15]);
        assert_eq!(e.mem.cstr(row_loaded[7]), b"Boone");
        let row_missing = &formats[4];
        assert_eq!(&row_missing[3..8], &[0x100, 10, 0xA, 15, 0x0108_063c]);
        // Section totals: count 2, total 40, minimum 10, maximum 30, mean 20.
        let totals = &formats[5];
        let mean = f64::from_bits(totals[12] as u64 | (totals[13] as u64) << 32);
        assert_eq!(
            (totals[4], totals[6], totals[8], totals[10]),
            (2, 40, 10, 30)
        );
        assert_eq!(mean, 20.0);
        // Grand totals add the extra stat: 2 forms, 45 bytes, mean 22.5.
        let grand = &formats[7];
        let mean = f64::from_bits(grand[7] as u64 | (grand[8] as u64) << 32);
        assert_eq!(&grand[3..7], &[2, 45, 10, 30]);
        assert_eq!(mean, 22.5);
        // One line per format call except the type heading, plus the
        // "Extra Stats:" line.
        let lines = lines.borrow();
        assert_eq!(lines.len(), 8);
        assert_eq!(lines[0], format!("{:08x}", 0x0108_0670u32));
        assert_eq!(lines[5], "Extra Stats:\r\n\r\n");
    }

    #[test]
    fn print_stats_names_a_form_from_its_reference_marker_or_description() {
        let (mut e, _lines, formats) = print_engine();
        // Three loaded forms of type 0: A has a reference with a name, B a
        // map marker with a location, C a reference and a marker whose
        // names are empty, so its description is used.
        let ids = [0xA_u32, 0xB, 0xC];
        let headers: Vec<u32> = ids
            .iter()
            .map(|&id| load_header(&mut e, id, 0, 4, 1))
            .collect();
        let list = list_of(&mut e, &headers);
        install_byte_entries(&mut e, &[(0, list)]);
        let mut forms = Vec::new();
        for (i, text) in [b"descA", b"descB", b"descC"].iter().enumerate() {
            forms.push(form_describing(
                &mut e,
                0x0200_0000 + 0x1000 * i as u32,
                *text,
            ));
        }
        let table = forms.clone();
        e.register_double(LOOKUP_FORM, move |_, a| {
            returns(table[(a[0] - 0xA) as usize])
        });
        let strings: Vec<u32> = [b"RefName".as_slice(), b"Place", b""]
            .iter()
            .map(|text| {
                let block = e.mem.alloc(16);
                e.mem.set_cstr(block, text);
                block
            })
            .collect();
        e.mem.set_cstr(0x0101_1584, b"");
        // The casts: target 0x11841cc is the reference, 0x1183158 the marker.
        let (reference_cast, marker_cast) = (0x0200_5000, 0x0200_5004);
        let by_form = forms.clone();
        e.register_double(DYNAMIC_CAST, move |_, a| {
            let index = by_form.iter().position(|&f| f == a[0]).unwrap();
            let target = a[3];
            returns(match (index, target) {
                (0, 0x0118_41cc) | (2, 0x0118_41cc) => reference_cast + index as u32,
                (1, 0x0118_3158) | (2, 0x0118_3158) => marker_cast + index as u32,
                _ => 0,
            })
        });
        let (name_a, name_empty) = (strings[0], strings[2]);
        e.register_double(REFERENCE_GET_NAME, move |_, a| {
            returns(if a[0] == reference_cast {
                name_a
            } else {
                name_empty
            })
        });
        let (place, marker_empty) = (strings[1], strings[2]);
        e.register_double(MAP_MARKER_GET_LOCATION_NAME, move |_, a| {
            returns(if a[0] == marker_cast + 1 {
                place
            } else {
                marker_empty
            })
        });
        let stats_map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(stats_map));
        let path = e.mem.alloc(8);
        e.mem.set_cstr(path, b"x");

        save_stats_print_stats(&mut e, stats, Ptr::new(path));

        let formats = formats.borrow();
        let rows: Vec<&Vec<u32>> = formats.iter().filter(|f| f[2] == 0x0108_0648).collect();
        assert_eq!(rows.len(), 3);
        let names: Vec<Vec<u8>> = rows.iter().map(|row| e.mem.cstr(row[7])).collect();
        assert_eq!(
            names,
            vec![b"RefName".to_vec(), b"Place".to_vec(), b"descC".to_vec()]
        );
        // The heading of type 0 is the text at `0104469c` ("Form").
        assert!(formats.iter().any(|f| f[2] == 0x0104_469c));
    }

    #[test]
    fn print_stats_titles_a_section_with_the_type_name() {
        let (mut e, _lines, formats) = print_engine();
        e.mem.set_u32(0x0118_7004 + 12 * 3, 0x0200_0100);
        let header = load_header(&mut e, 0x10, 3, 8, 0);
        let list = list_of(&mut e, &[header]);
        install_byte_entries(&mut e, &[(3, list)]);
        e.register(LOOKUP_FORM, |_, _| returns(0));
        let stats_map = e.mem.alloc(0x10);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(stats, SaveStats::pStatsMap, Ptr::new(stats_map));
        let path = e.mem.alloc(8);
        e.mem.set_cstr(path, b"x");
        save_stats_print_stats(&mut e, stats, Ptr::new(path));
        let heading = formats
            .borrow()
            .iter()
            .find(|f| f[2] == 0x0101_9f08)
            .cloned()
            .unwrap();
        assert_eq!(heading[3], 0x0200_0100);
    }

    #[test]
    fn print_stats_writes_nothing_when_the_file_will_not_open() {
        let (mut e, lines, formats) = print_engine();
        e.register(FILE_EXISTS, |_, _| returns(1));
        // The file object's first word is non-zero after construction.
        e.register(FILE_OBJECT_CONSTRUCT, |e, a| {
            e.mem.set_u32(a[0], 1);
            Ret::default()
        });
        let stats: Ptr<SaveStats> = e.new_object();
        let path = e.mem.alloc(8);
        e.mem.set_cstr(path, b"x");
        save_stats_print_stats(&mut e, stats, Ptr::new(path));
        assert!(formats.borrow().is_empty());
        assert!(lines.borrow().is_empty());
        // An existing file was deleted first, and the object was destroyed.
        assert_eq!(calls_to(&e, FILE_DELETE).len(), 1);
        assert_eq!(calls_to(&e, FILE_OBJECT_DESTRUCT).len(), 1);
    }

    #[test]
    fn stats_constructor_resets_the_counters() {
        let mut e = game();
        let stats: Ptr<Stats> = e.new_object();
        e.mem.write(stats.addr(), &[0x55; 12]);
        assert_eq!(fn_008562c0(&mut e, stats), stats);
        assert_eq!(e.get(stats, Stats::iNum), 0);
        assert_eq!(e.get(stats, Stats::iTotalSize), 0);
        assert_eq!(e.get(stats, Stats::iMinSize), 0xFFFF);
        assert_eq!(e.get(stats, Stats::iMaxSize), 0);
    }

    #[test]
    fn write_text_reports_success_of_the_file_write() {
        let mut e = game();
        e.register(SYSTEM_FILE_DO_WRITE, |_, a| returns(a[2] & 1));
        let file = e.mem.alloc(0x20);
        let text = e.mem.alloc(8);
        e.mem.set_cstr(text, b"ab");
        assert!(fn_00856300(
            &mut e,
            Ptr::NULL,
            Ptr::new(file),
            Ptr::new(text)
        ));
        e.mem.set_cstr(text, b"abc");
        assert!(!fn_00856300(
            &mut e,
            Ptr::NULL,
            Ptr::new(file),
            Ptr::new(text)
        ));
        // (file, text, length, 0, scratch)
        let writes = calls_to(&e, SYSTEM_FILE_DO_WRITE);
        assert_eq!(&writes[0][..4], &[file, text, 2, 0]);
    }

    #[test]
    fn remove_changes_skips_deleted_forms() {
        let mut e = game();
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let map: Ptr<ChangesMap> = e.new_object();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, map);
        let form = form_with_key(&mut e, 0x31);
        e.register(FORM_IS_DELETED, |e, a| {
            returns((e.mem.u32(a[0] + 8) & 0x4000 != 0) as u32)
        });
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((0x31, data.addr()));
        e.mem.set_u32(form.addr() + 8, 0x4000);
        tes_save_load_game_remove_changes(&mut e, game_object, form, 1);
        assert_eq!(table.borrow().len(), 1);
        e.mem.set_u32(form.addr() + 8, 0);
        tes_save_load_game_remove_changes(&mut e, game_object, form, 1);
        assert!(table.borrow().is_empty());
    }

    // ---- the save routine ------------------------------------------------

    #[test]
    fn save_refuses_with_the_sad_message_unless_allowed() {
        let mut e = game();
        e.map(0x0107_f000, 0x1000);
        e.mem.set_cstr(AUTOSAVE_NAME, b"autosave");
        e.register(GET_SAVING_ALLOWED, |_, _| returns(0));
        constant(&mut e, GET_MESSAGE_QUEUE, 0x5000);
        stub(&mut e, SHOW_MESSAGE);
        let name = e.mem.alloc(16);
        e.mem.set_cstr(name, b"Quicksave");
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        // A named save that is not an autosave, and an unnamed one, refuse.
        for name in [Ptr::new(name), Ptr::NULL] {
            assert!(!fn_00856ca0(&mut e, game_object, Ptr::NULL, name, false));
        }
        let message = calls_to(&e, SHOW_MESSAGE);
        assert_eq!(message.len(), 2);
        assert_eq!(&message[0][..4], &[0x5000, 0, SAD_ICON, 0]);
        assert_eq!(f32::from_bits(message[0][4]), 2.0);
        assert_eq!(message[0][5], 0);
        assert!(calls_to(&e, SAVE_LOCK_ENTER).is_empty());
        assert_eq!(
            calls_to(&e, GET_MESSAGE_QUEUE)[0],
            vec![MESSAGE_QUEUE_OBJECT]
        );
        assert_eq!(
            &calls_to(&e, SCOPE_ENTER)[0][1..],
            &[0x11, 1, SOURCE_FILE, 0x466]
        );
        assert_eq!(calls_to(&e, SCOPE_LEAVE).len(), 2);
    }

    /// What the full save needs: everything it calls, with doubles that log
    /// what the file receives.
    struct SaveRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        stream: u32,
        writes: FileWrites,
        seeks: Rc<RefCell<Vec<(u32, u32)>>>,
    }

    fn save_rig() -> SaveRig {
        let mut e = game();
        constant(&mut e, GET_SAVING_ALLOWED, 1);
        stub(&mut e, SAVE_LOCK_ENTER);
        stub(&mut e, SAVE_LOCK_LEAVE);
        for step in [
            SAVE_PREPARE_A,
            SAVE_PREPARE_B,
            SAVE_PREPARE_C,
            SAVE_HEADER,
            SAVE_PLUGIN_LIST,
            SAVE_GLOBAL_DATA,
            SAVE_FINAL_DATA,
            SAVE_NUMERIC_ID_ARRAYS,
            SAVE_CLOSE_A,
            SAVE_CLOSE_B,
            FILE_FLUSH,
        ] {
            stub(&mut e, step);
        }
        constant(&mut e, CURRENT_VERSION, 7);
        let positions = Rc::new(RefCell::new(VecDeque::from(vec![100u32, 500])));
        e.register_double(FILE_POSITION, move |_, _| {
            returns(positions.borrow_mut().pop_front().unwrap())
        });
        let writes = Rc::new(RefCell::new(Vec::new()));
        let sink = writes.clone();
        e.register_double(WRITE_BYTES, move |e, a| {
            sink.borrow_mut().push((a[1], e.mem.bytes(a[2], a[3])));
            Ret::default()
        });
        // The stream: seek is slot 0x14, name slot 0x18.
        let vtable = 0x0200_0000;
        e.mem.map(vtable, 0x100);
        e.mem.set_u32(vtable + 0x14, 0x0200_1000);
        e.mem.set_u32(vtable + 0x18, 0x0200_1004);
        let stream = e.mem.alloc(0x20);
        e.mem.set_u32(stream, vtable);
        let seeks = Rc::new(RefCell::new(Vec::new()));
        let sink = seeks.clone();
        e.register_double(0x0200_1000, move |_, a| {
            sink.borrow_mut().push((a[1], a[2]));
            Ret::default()
        });
        let name = e.mem.alloc(16);
        e.mem.set_cstr(name, b"Saves\\save");
        constant(&mut e, 0x0200_1004, name);
        constant(&mut e, OPEN_SAVE_FILE, stream);
        e.set_global(SEEK_MODE, 0u32);
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        SaveRig {
            e,
            game: game_object,
            stream,
            writes,
            seeks,
        }
    }

    #[test]
    fn save_writes_headers_sizes_and_patches_the_positions() {
        let mut rig = save_rig();
        let e = &mut rig.e;
        // Two changed forms: 0x100 (no buffer, a real form) and 0x200 (with
        // a pre-built buffer of 2 bytes "AB").
        let plain = change_data(e, 0x3, 0);
        let buffer = e.mem.alloc(8);
        e.mem.write(buffer, b"AB");
        let buffered = change_data(e, 0x9, buffer);
        let map: Ptr<ChangesMap> = e.new_object();
        e.set(rig.game, TESSaveLoadGame::m_pChanges, map);
        install_entries(e, &[(0x100, plain.addr()), (0x200, buffered.addr())]);
        // The form: vtable slot 0x50 changes size, 0x58 saves the changes.
        let form_vtable = 0x0200_2000;
        e.mem.map(form_vtable, 0x100);
        e.mem.set_u32(form_vtable + 0x50, 0x0200_3000);
        e.mem.set_u32(form_vtable + 0x58, 0x0200_3004);
        constant(e, 0x0200_3000, 4);
        stub(e, 0x0200_3004);
        let form = e.mem.alloc(0x40);
        e.mem.set_u32(form, form_vtable);
        e.register_double(LOOKUP_FORM, move |_, a| {
            returns(if a[0] == 0x100 { form } else { 0 })
        });
        constant(e, FORM_TYPE, 0x2B);
        e.register(CHECK_FLAGS, |_, a| returns(a[2] | 0x100));
        constant(e, GET_INITIAL_DATA_SAVE_SIZE, 3);
        let save_buffer = e.mem.alloc(16);
        constant(e, CREATE_BUFFER, save_buffer);
        stub(e, SAVE_INITIAL_DATA);
        stub(e, WRITE_FILE);
        stub(e, FREE_BUFFER);
        // The buffer's first four bytes: size 2, type 0x33, version 5.
        e.register(READ_BYTES, |e, a| {
            e.mem.write(a[1], &[2, 0, 0x33, 5]);
            Ret::default()
        });

        let saved = fn_00856ca0(e, rig.game, Ptr::NULL, Ptr::NULL, false);
        assert!(saved);

        let writes = rig.writes.borrow();
        let stream = rig.stream;
        let sizes: Vec<usize> = writes.iter().map(|(_, bytes)| bytes.len()).collect();
        // Two zero words, then for form 0x100: header, size, and for 0x200:
        // header, size, data; then the patched end position and count.
        assert_eq!(sizes, vec![4, 4, 10, 2, 10, 2, 2, 4, 4]);
        assert!(writes.iter().all(|(file, _)| *file == stream));
        assert_eq!(writes[0].1, vec![0; 4]);
        // Header of 0x100: id, type 0x2B, flags 0x3 | 0x100 (CheckFlags),
        // version 7.
        assert_eq!(
            writes[2].1,
            vec![0x00, 0x01, 0x00, 0x00, 0x2B, 0x03, 0x01, 0x00, 0x00, 0x07]
        );
        assert_eq!(writes[3].1, vec![7, 0]); // 4 + 3
                                             // Header of the buffered form: type and version from the buffer.
        assert_eq!(
            writes[4].1,
            vec![0x00, 0x02, 0x00, 0x00, 0x33, 0x09, 0x00, 0x00, 0x00, 0x05]
        );
        assert_eq!(writes[5].1, vec![2, 0]);
        assert_eq!(writes[6].1, b"AB".to_vec());
        // The end position (500) and the form count (2), after seeking back
        // to the start position (100).
        assert_eq!(writes[7].1, 500u32.to_le_bytes().to_vec());
        assert_eq!(writes[8].1, 2u32.to_le_bytes().to_vec());
        assert_eq!(rig.seeks.borrow().as_slice(), &[(100, 0)]);
        // The first form's data went through the buffer helpers.
        assert_eq!(
            calls_to(e, WRITE_FILE),
            vec![vec![rig.game.addr(), stream, save_buffer, 7]]
        );
        assert_eq!(
            calls_to(e, FREE_BUFFER),
            vec![vec![rig.game.addr(), save_buffer]]
        );
        // The routine bracketed itself with the lock and closed the file.
        assert_eq!(calls_to(e, SAVE_LOCK_ENTER).len(), 1);
        assert_eq!(calls_to(e, SAVE_LOCK_LEAVE).len(), 1);
        assert_eq!(calls_to(e, FILE_FLUSH), vec![vec![stream]]);
        assert_eq!(
            calls_to(e, SAVE_CLOSE_B),
            vec![vec![rig.game.addr(), stream, 0]]
        );
        // The form header pointer is cleared again, and the buffer too.
        assert!(e
            .get(rig.game, TESSaveLoadGame::m_pCurrentlySavingFormHeader)
            .is_null());
        assert!(e.get(rig.game, TESSaveLoadGame::m_pBuffer).is_null());
    }

    #[test]
    fn save_allows_an_autosave_when_saving_is_not_allowed() {
        let mut rig = save_rig();
        let e = &mut rig.e;
        constant(e, GET_SAVING_ALLOWED, 0);
        e.map(0x0107_f000, 0x1000);
        e.mem.set_cstr(AUTOSAVE_NAME, b"autosave");
        install_entries(e, &[]);
        let map: Ptr<ChangesMap> = e.new_object();
        e.set(rig.game, TESSaveLoadGame::m_pChanges, map);
        let name = e.mem.alloc(16);
        e.mem.set_cstr(name, b"autosave");
        stub(e, SHOW_MESSAGE);
        assert!(fn_00856ca0(e, rig.game, Ptr::NULL, Ptr::new(name), false));
        assert!(calls_to(e, SHOW_MESSAGE).is_empty());
        // No forms: the two zero words and the patched end and count.
        let sizes: Vec<usize> = rig.writes.borrow().iter().map(|w| w.1.len()).collect();
        assert_eq!(sizes, vec![4, 4, 4, 4]);
    }

    #[test]
    fn save_prints_and_deletes_the_statistics_when_asked() {
        let mut rig = save_rig();
        let e = &mut rig.e;
        install_entries(e, &[]);
        let map: Ptr<ChangesMap> = e.new_object();
        e.set(rig.game, TESSaveLoadGame::m_pChanges, map);
        // `SaveStats`'s own map: constructed with a destructor in slot 0.
        let map_vtable = 0x0200_4000;
        e.put_vtable(map_vtable, &[0x0200_4100]);
        stub(e, 0x0200_4100);
        e.register_double(STATS_MAP_CONSTRUCT, move |e, a| {
            e.mem.set_u32(a[0], map_vtable);
            returns(a[0])
        });
        // `PrintStats` runs (it is this unit's own): it builds the text file
        // object for the stream's name, which fails to open here.
        let printed = Rc::new(RefCell::new(Vec::new()));
        let sink = printed.clone();
        e.register_double(FILE_OBJECT_CONSTRUCT, move |e, a| {
            sink.borrow_mut().push(e.mem.cstr(a[1]));
            e.mem.set_u32(a[0], 1);
            Ret::default()
        });
        e.register(FILE_EXISTS, |_, _| returns(0));
        stub(e, STRING_CAT);
        stub(e, FILE_OBJECT_DESTRUCT);
        stub(e, LIST_REMOVE_ALL);
        stub(e, LIST_SCALAR_DELETE);
        let saved = fn_00856ca0(e, rig.game, Ptr::NULL, Ptr::NULL, true);
        assert!(saved);
        assert_eq!(printed.borrow().len(), 1);
        assert_eq!(printed.borrow()[0], b"Saves\\save");
        // The statistics were deleted (the map's destructor, flag 1) and
        // the pointer cleared.
        assert!(e.get(rig.game, TESSaveLoadGame::m_pSaveLoadStats).is_null());
        assert_eq!(calls_to(e, 0x0200_4100).len(), 1);
    }

    #[test]
    fn flush_calls_the_file_helper() {
        let mut e = game();
        stub(&mut e, FILE_FLUSH);
        fn_00857210(&mut e, Ptr::new(0x1234));
        assert_eq!(calls_to(&e, FILE_FLUSH), vec![vec![0x1234]]);
    }

    #[test]
    fn set_saving_form_header_stores_the_pointer() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        fn_00857230(&mut e, game_object, Ptr::new(0x4444));
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pCurrentlySavingFormHeader)
                .addr(),
            0x4444
        );
    }

    #[test]
    fn save_stats_scalar_deleting_destructor_frees_on_bit_zero() {
        let mut e = game();
        install_byte_entries(&mut e, &[]);
        let stats: Ptr<SaveStats> = e.new_object();
        assert_eq!(fn_00857250(&mut e, stats, 0), stats);
        assert!(!freed(&e, stats.addr()));
        fn_00857250(&mut e, stats, 1);
        assert!(freed(&e, stats.addr()));
    }

    #[test]
    fn form_and_flags_constructor_stores_the_four_fields() {
        let mut e = game();
        let record: Ptr<FormAndFlags> = e.new_object();
        let result = fn_00857280(&mut e, record, Ptr::new(0x1111), 5, 6, 9);
        assert_eq!(result, record);
        assert_eq!(e.get(record, FormAndFlags::pForm).addr(), 0x1111);
        assert_eq!(e.get(record, FormAndFlags::iFlags), 5);
        assert_eq!(e.get(record, FormAndFlags::iOldFlags), 6);
        assert_eq!(e.get(record, FormAndFlags::cVersion), 9);
    }

    #[test]
    fn local_record_initializer_clears_the_words_and_runs_the_embedded_one() {
        let mut e = game();
        stub(&mut e, EMBEDDED_RECORD_INIT);
        let local = e.mem.alloc(0x40);
        e.mem.write(local, &[0xFF; 16]);
        assert_eq!(fn_008572c0(&mut e, Ptr::new(local)).addr(), local);
        assert_eq!(e.mem.u32(local), 0);
        assert_eq!(e.mem.u32(local + 4), 0);
        assert_eq!(calls_to(&e, EMBEDDED_RECORD_INIT), vec![vec![local + 8]]);
    }

    // ---- the second batch of functions: helpers ----

    /// Doubles that return 0 (calls are logged).
    fn quiet(e: &mut Engine, addresses: &[u32]) {
        for &address in addresses {
            stub(e, address);
        }
    }

    /// `__RTDynamicCast` over a table of (object, target type, result); any
    /// other cast gives null.
    fn casts(e: &mut Engine, table: &[(u32, u32, u32)]) {
        let table = table.to_vec();
        e.register_double(DYNAMIC_CAST, move |_, a| {
            returns(
                table
                    .iter()
                    .find(|(object, target, _)| *object == a[0] && *target == a[3])
                    .map_or(0, |entry| entry.2),
            )
        });
    }

    /// Maps the pages of the globals the second batch reads.
    fn map_globals(e: &mut Engine) {
        for page in [
            0x011c_3000u32,
            0x011d_e000,
            0x011a_9000,
            0x011f_4000,
            0x0107_f000,
            0x0118_7000,
            0x0120_2000,
            0x0108_0000,
            0x0107_c000,
        ] {
            e.map(page, 0x1000);
        }
    }

    /// A game object whose current buffer is a fresh block of `size` bytes.
    fn game_with_buffer(e: &mut Engine, size: u32) -> (Ptr<TESSaveLoadGame>, u32) {
        let game: Ptr<TESSaveLoadGame> = e.new_object();
        let buffer = e.mem.alloc(size);
        e.set(game, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
        (game, buffer)
    }

    /// An object with a vtable whose slots are `(offset, address)`.
    fn object_with_slots(e: &mut Engine, slots: &[(u32, u32)]) -> u32 {
        let vtable = e.mem.alloc(0x400);
        for &(offset, target) in slots {
            e.mem.set_u32(vtable + offset, target);
        }
        let object = e.mem.alloc(0x80);
        e.mem.set_u32(object, vtable);
        object
    }

    /// The text at an address, as a `String`.
    fn text(e: &Engine, address: u32) -> String {
        String::from_utf8(e.mem.cstr(address)).unwrap()
    }

    // ---- the second batch of functions ----

    #[test]
    fn reference_data_constructor_clears_the_location_id() {
        let mut e = game();
        let record = e.mem.alloc(0x1C);
        e.mem.set_u32(record, 0xDEAD);
        let result = fn_008572f0(&mut e, Ptr::new(record));
        assert_eq!(result.addr(), record);
        assert_eq!(e.mem.u32(record), 0);
        // The two embedded points are built (their constructor does nothing).
        assert_eq!(
            calls_to(&e, LIST_NODE_ITEM),
            vec![vec![record + 4], vec![record + 0x10]]
        );
    }

    #[test]
    fn moved_reference_constructor_builds_the_inner_reference_data() {
        let mut e = game();
        let record = e.mem.alloc(0x2C);
        e.mem.set_u32(record, 0xDEAD);
        e.mem.set_u32(record + 0x10, 0xBEEF);
        let result = fn_00857320(&mut e, Ptr::new(record));
        assert_eq!(result.addr(), record);
        assert_eq!(e.mem.u32(record), 0);
        assert_eq!(e.mem.u32(record + 0x10), 0);
        assert_eq!(
            calls_to(&e, LIST_NODE_ITEM),
            vec![vec![record + 4], vec![record + 0x14], vec![record + 0x20]]
        );
    }

    #[test]
    fn init_array_destructor_runs_the_array_destructor() {
        let mut e = game();
        stub(&mut e, INIT_ARRAY_DESTRUCT);
        fn_00857350(&mut e, Ptr::new(0x4321));
        assert_eq!(calls_to(&e, INIT_ARRAY_DESTRUCT), vec![vec![0x4321]]);
    }

    /// What the file-opening doubles saw.
    struct OpenRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        events: Rc<RefCell<Vec<String>>>,
        /// The `BSFile` constructor's (path, write mode, buffer size).
        constructed: Rc<RefCell<Vec<(String, u32, u32)>>>,
    }

    /// The file-opening world: "base\" and "Saves\" are the path pieces,
    /// string functions work on memory, files named in `existing` exist.
    fn open_rig(existing: &[&str]) -> OpenRig {
        let mut e = game();
        map_globals(&mut e);
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));
        let prefix = e.mem.alloc(16);
        e.mem.set_cstr(prefix, b"base\\");
        let folder = e.mem.alloc(16);
        e.mem.set_cstr(folder, b"Saves\\");
        constant(&mut e, PATH_PREFIX, prefix);
        constant(&mut e, PATH_OBJECT_GET, folder);
        e.register(IDENTITY, |_, a| returns(a[0]));
        e.register(STRING_CAT, |e, a| {
            let mut joined = e.mem.cstr(a[0]);
            joined.extend(e.mem.cstr(a[2]));
            e.mem.set_cstr(a[0], &joined);
            Ret::default()
        });
        e.register(FORMAT, |e, a| {
            // "%s%s%s.ess"
            let mut joined = e.mem.cstr(a[3]);
            joined.extend(e.mem.cstr(a[4]));
            joined.extend(e.mem.cstr(a[5]));
            joined.extend(b".ess");
            e.mem.set_cstr(a[0], &joined);
            Ret::default()
        });
        e.mem.set_cstr(BAK_EXTENSION, b".bak");
        e.mem.set_cstr(ESS_EXTENSION, b".ess");
        e.mem.set_cstr(SAVE_NAME_PREFIX, b"Save ");
        e.mem.set_cstr(AUTOSAVE_NAME, b"autosave");
        let existing: Vec<String> = existing.iter().map(|s| s.to_string()).collect();
        let sink = events.clone();
        e.register_double(FILE_EXISTS, move |e, a| {
            let path = text(e, a[0]);
            sink.borrow_mut().push(format!("exists {path}"));
            returns(existing.contains(&path) as u32)
        });
        let sink = events.clone();
        e.register_double(DELETE_FILE_IMPORT, move |e, a| {
            sink.borrow_mut().push(format!("delete {}", text(e, a[0])));
            returns(1)
        });
        let sink = events.clone();
        e.register_double(CREATE_DIRECTORY_IMPORT, move |e, a| {
            sink.borrow_mut().push(format!("mkdir {}", text(e, a[0])));
            returns(1)
        });
        let sink = events.clone();
        e.register_double(RENAME, move |e, a| {
            sink.borrow_mut()
                .push(format!("rename {} {}", text(e, a[0]), text(e, a[1])));
            returns(0)
        });
        e.register(STRRCHR, |e, a| {
            let bytes = e.mem.cstr(a[0]);
            returns(
                bytes
                    .iter()
                    .rposition(|&b| b == a[1] as u8)
                    .map_or(0, |i| a[0] + i as u32),
            )
        });
        e.register(STRNICMP, |e, a| {
            let (left, right) = (e.mem.cstr(a[0]), e.mem.cstr(a[1]));
            let n = a[2] as usize;
            let cut = |v: &[u8]| {
                v.iter()
                    .take(n)
                    .map(u8::to_ascii_lowercase)
                    .collect::<Vec<u8>>()
            };
            returns(cut(&left).cmp(&cut(&right)) as i32 as u32)
        });
        e.register(STRING_COMPARE_NOCASE, |e, a| {
            let (left, right) = (e.mem.cstr(a[0]), e.mem.cstr(a[1]));
            returns(left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase()) as i32 as u32)
        });
        e.register(DEFAULT_SAVE_NAME, |e, a| {
            e.mem.set_cstr(a[1], b"Save 9");
            Ret::default()
        });
        // A file object: name at +4, the vtable has the destructor (0), the
        // name (0x18) and `Open` (0x20).
        e.register_double(0x0300_0000, |_, _| Ret::default());
        e.register(0x0300_0018, |e, a| returns(e.mem.u32(a[0] + 4)));
        let sink = events.clone();
        e.register_double(0x0300_0020, move |_, a| {
            sink.borrow_mut()
                .push(format!("open {} {} {}", a[0], a[1], a[2]));
            returns(1)
        });
        let sink = events.clone();
        e.register_double(0x0300_0000, move |_, a| {
            sink.borrow_mut().push(format!("destroy {} {}", a[0], a[1]));
            Ret::default()
        });
        e.register(FILE_DELETE, |_, _| returns(1));
        let constructed: Rc<RefCell<Vec<(String, u32, u32)>>> = Rc::new(RefCell::new(vec![]));
        let sink = constructed.clone();
        e.register_double(BSFILE_CONSTRUCT, move |e, a| {
            sink.borrow_mut().push((text(e, a[1]), a[2], a[3]));
            let vtable = e.mem.alloc(0x40);
            e.mem.set_u32(vtable, 0x0300_0000);
            e.mem.set_u32(vtable + 0x18, 0x0300_0018);
            e.mem.set_u32(vtable + 0x20, 0x0300_0020);
            e.mem.set_u32(a[0], vtable);
            returns(a[0])
        });
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        OpenRig {
            e,
            game: game_object,
            events,
            constructed,
        }
    }

    /// A file object like the ones `BSFile::BSFile` makes, named `name`.
    fn file_named(e: &mut Engine, name: &[u8]) -> Ptr {
        let file = e.mem.alloc(0x20);
        let vtable = e.mem.alloc(0x40);
        e.mem.set_u32(vtable, 0x0300_0000);
        e.mem.set_u32(vtable + 0x18, 0x0300_0018);
        e.mem.set_u32(vtable + 0x20, 0x0300_0020);
        e.mem.set_u32(file, vtable);
        let name_block = e.mem.alloc(0x80);
        e.mem.set_cstr(name_block, name);
        e.mem.set_u32(file + 4, name_block);
        Ptr::new(file)
    }

    #[test]
    fn opening_a_save_for_writing_rotates_the_old_save_to_bak() {
        let mut rig = open_rig(&["base\\Saves\\quick.ess", "base\\Saves\\quick.bak"]);
        let name = rig.e.mem.alloc(16);
        rig.e.mem.set_cstr(name, b"quick");
        let file = fn_00857370(&mut rig.e, rig.game, Ptr::NULL, Ptr::new(name), 0);
        assert!(!file.is_null());
        assert_eq!(
            *rig.events.borrow(),
            vec![
                "mkdir base\\Saves\\",
                "exists base\\Saves\\quick.ess",
                "exists base\\Saves\\quick.bak",
                "delete base\\Saves\\quick.bak",
                "rename base\\Saves\\quick.ess base\\Saves\\quick.bak",
            ]
        );
        // The file is opened for writing with a 0x20000 byte buffer and is
        // not opened again.
        assert_eq!(
            *rig.constructed.borrow(),
            vec![("base\\Saves\\quick.ess".to_string(), 1, 0x20000)]
        );
    }

    #[test]
    fn opening_a_save_without_an_old_one_does_not_rename() {
        let mut rig = open_rig(&[]);
        let default_name = fn_00857370(&mut rig.e, rig.game, Ptr::NULL, Ptr::NULL, 0);
        assert!(!default_name.is_null());
        // The default name (`00860ae0`) is "Save 9".
        assert_eq!(
            *rig.constructed.borrow(),
            vec![("base\\Saves\\Save 9.ess".to_string(), 1, 0x20000)]
        );
        assert_eq!(
            *rig.events.borrow(),
            vec!["mkdir base\\Saves\\", "exists base\\Saves\\Save 9.ess"]
        );
    }

    #[test]
    fn opening_a_save_for_reading_opens_the_file() {
        let mut rig = open_rig(&[]);
        let name = rig.e.mem.alloc(16);
        rig.e.mem.set_cstr(name, b"quick");
        let file = fn_00857370(&mut rig.e, rig.game, Ptr::NULL, Ptr::new(name), 1);
        assert_eq!(
            *rig.constructed.borrow(),
            vec![("base\\Saves\\quick.ess".to_string(), 0, 0x20000)]
        );
        // Nothing is rotated, and the slot at 0x20 is called with two zeros.
        assert_eq!(
            *rig.events.borrow(),
            vec![format!("open {} 0 0", file.addr())]
        );
    }

    #[test]
    fn opening_with_a_file_reopens_it_or_hands_it_back() {
        let mut rig = open_rig(&[]);
        let file = file_named(&mut rig.e, b"base\\Saves\\quick.ess");
        // Mode 2 opens the file again.
        let again = fn_00857370(&mut rig.e, rig.game, file, Ptr::NULL, 2);
        assert_eq!(again, file);
        assert_eq!(
            *rig.events.borrow(),
            vec![format!("open {} 0 0", file.addr())]
        );
        // Mode 3 and an unknown mode return the file untouched.
        let before = rig.events.borrow().len();
        assert_eq!(fn_00857370(&mut rig.e, rig.game, file, Ptr::NULL, 3), file);
        assert_eq!(fn_00857370(&mut rig.e, rig.game, file, Ptr::NULL, 9), file);
        assert_eq!(rig.events.borrow().len(), before);
        assert!(rig.constructed.borrow().is_empty());
    }

    #[test]
    fn saving_over_a_numbered_save_deletes_it_and_opens_the_default_name() {
        let mut rig = open_rig(&[]);
        let file = file_named(&mut rig.e, b"base\\Saves\\Save 3 Vault.ess");
        let result = fn_00857370(&mut rig.e, rig.game, file, Ptr::NULL, 0);
        assert!(!result.is_null());
        // The old save is deleted and its stream destroyed (flag 1) ...
        let events = rig.events.borrow().clone();
        assert!(events.contains(&format!("destroy {} 1", file.addr())));
        assert_eq!(calls_to(&rig.e, FILE_DELETE).len(), 1);
        // ... and the new one is opened under the default name ("Save 9").
        assert_eq!(
            rig.constructed.borrow()[0].0,
            "base\\Saves\\Save 9.ess".to_string()
        );
    }

    #[test]
    fn saving_over_an_autosave_keeps_the_name() {
        let mut rig = open_rig(&[]);
        let file = file_named(&mut rig.e, b"base\\Saves\\autosave.ess");
        fn_00857370(&mut rig.e, rig.game, file, Ptr::NULL, 0);
        assert_eq!(
            rig.constructed.borrow()[0].0,
            "base\\Saves\\autosave.ess".to_string()
        );
    }

    #[test]
    fn letting_go_of_a_file_closes_or_destroys_it() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let file = object_with_slots(&mut e, &[(0, 0x0300_0000)]);
        e.register(0x0300_0000, |_, _| Ret::default());
        quiet(&mut e, &[BSFILE_CLOSE, LIST_REMOVE]);
        // Mode 2 closes.
        fn_008578b0(&mut e, game_object, Ptr::new(file), 2);
        assert_eq!(calls_to(&e, BSFILE_CLOSE), vec![vec![file]]);
        assert!(calls_to(&e, 0x0300_0000).is_empty());
        // Mode 3 without a list destroys it, with flag 1.
        fn_008578b0(&mut e, game_object, Ptr::new(file), 3);
        assert_eq!(calls_to(&e, 0x0300_0000), vec![vec![file, 1]]);
        assert!(calls_to(&e, LIST_REMOVE).is_empty());
        // With a list the file is removed from it first.
        e.set(
            game_object,
            TESSaveLoadGame::m_pSaveGameList,
            Ptr::new(0x5000),
        );
        fn_008578b0(&mut e, game_object, Ptr::new(file), 0);
        let removed = calls_to(&e, LIST_REMOVE);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0][0], 0x5000);
        // Null files and other modes do nothing.
        fn_008578b0(&mut e, game_object, Ptr::NULL, 0);
        fn_008578b0(&mut e, game_object, Ptr::new(file), 7);
        assert_eq!(calls_to(&e, 0x0300_0000).len(), 2);
        assert_eq!(calls_to(&e, BSFILE_CLOSE).len(), 1);
    }

    #[test]
    fn deleting_a_save_file_removes_the_named_file_and_the_stream() {
        let mut rig = open_rig(&[]);
        let file = file_named(&mut rig.e, b"base\\Saves\\old.ess");
        fn_00857950(&mut rig.e, rig.game, file, Ptr::NULL);
        let deleted = calls_to(&rig.e, FILE_DELETE);
        assert_eq!(deleted.len(), 1);
        assert_eq!(text(&rig.e, deleted[0][0]), "base\\Saves\\old.ess");
        assert!(rig
            .events
            .borrow()
            .contains(&format!("destroy {} 1", file.addr())));
        // A null file does nothing.
        fn_00857950(&mut rig.e, rig.game, Ptr::NULL, Ptr::NULL);
        assert_eq!(calls_to(&rig.e, FILE_DELETE).len(), 1);
    }

    #[test]
    fn buffer_writes_and_reads_move_the_buffer_pointer() {
        let mut e = game();
        let (game_object, buffer) = game_with_buffer(&mut e, 32);
        let source = e.mem.alloc(8);
        e.mem.write(source, &[1, 2, 3, 4, 5, 6]);
        fn_008579b0(&mut e, game_object, Ptr::new(source), 6);
        assert_eq!(e.mem.bytes(buffer, 6), vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pBuffer).addr(),
            buffer + 6
        );
        // Reading takes the bytes from the buffer's current position.
        e.mem.write(buffer + 6, &[9, 8, 7, 6]);
        let target = e.mem.alloc(8);
        fn_008579e0(&mut e, game_object, Ptr::new(target), 4);
        assert_eq!(e.mem.bytes(target, 4), vec![9, 8, 7, 6]);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pBuffer).addr(),
            buffer + 10
        );
    }

    #[test]
    fn numeric_ids_are_saved_through_the_id_array_when_it_is_used() {
        let mut e = game();
        let (game_object, buffer) = game_with_buffer(&mut e, 32);
        let ids = e.mem.alloc(16);
        e.mem
            .write(ids, &[0x11, 0, 0, 0, 0x22, 0, 0, 0, 0x33, 0, 0, 0]);
        // Without the array the ids are written as they are (size / 4 of
        // them: 9 bytes is two ids).
        constant(&mut e, USE_NUMERIC_IDS, 0);
        tes_save_load_game_save_numeric_id(&mut e, game_object, Ptr::new(ids), 9);
        assert_eq!(e.mem.u32(buffer), 0x11);
        assert_eq!(e.mem.u32(buffer + 4), 0x22);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pBuffer).addr(),
            buffer + 8
        );
        // With the array each id goes through `AddNumericIDToArray`.
        constant(&mut e, USE_NUMERIC_IDS, 1);
        e.register(ADD_NUMERIC_ID, |_, a| returns(a[1] + 0x1000));
        tes_save_load_game_save_numeric_id(&mut e, game_object, Ptr::new(ids), 12);
        assert_eq!(e.mem.u32(buffer + 8), 0x1011);
        assert_eq!(e.mem.u32(buffer + 12), 0x1022);
        assert_eq!(e.mem.u32(buffer + 16), 0x1033);
    }

    #[test]
    fn numeric_ids_are_loaded_and_resolved_when_the_array_is_used() {
        let mut e = game();
        let (game_object, buffer) = game_with_buffer(&mut e, 32);
        e.mem.write(buffer, &[5, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0]);
        let ids = e.mem.alloc(16);
        // Without the array: copied as they are.
        constant(&mut e, USE_NUMERIC_IDS, 0);
        assert!(!tes_save_load_game_load_numeric_id(
            &mut e,
            game_object,
            Ptr::new(ids),
            12
        ));
        assert_eq!(e.mem.u32(ids), 5);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pBuffer).addr(),
            buffer + 12
        );
        // With the array: 6 does not resolve (gives 0), 0 stays 0 without
        // counting as a failure.
        constant(&mut e, USE_NUMERIC_IDS, 1);
        e.register(RESOLVE_NUMERIC_ID, |_, a| {
            returns(if a[1] == 6 { 0 } else { a[1] + 100 })
        });
        e.set(game_object, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
        let failed = tes_save_load_game_load_numeric_id(&mut e, game_object, Ptr::new(ids), 12);
        assert!(failed);
        assert_eq!(e.mem.u32(ids), 105);
        assert_eq!(e.mem.u32(ids + 4), 0);
        // Only 0 and resolvable ids: no failure.
        e.mem.write(buffer, &[5, 0, 0, 0, 0, 0, 0, 0]);
        e.set(game_object, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
        assert!(!tes_save_load_game_load_numeric_id(
            &mut e,
            game_object,
            Ptr::new(ids),
            8
        ));
    }

    #[test]
    fn writing_bytes_goes_to_the_file_or_is_only_counted() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        e.register(FILE_WRITE, |_, a| returns(a[2] + 100));
        assert_eq!(
            fn_00857b50(&mut e, game_object, Ptr::new(0x10), Ptr::new(0x20), 6),
            106
        );
        assert_eq!(calls_to(&e, FILE_WRITE), vec![vec![0x10, 0x20, 6]]);
        // When the game only measures (`0047c850` true) the size is added.
        e.register(SAVE_LOAD_UNAVAILABLE, |_, _| returns(1));
        assert_eq!(
            fn_00857b50(&mut e, game_object, Ptr::new(0x10), Ptr::new(0x20), 4),
            4
        );
        fn_00857b50(&mut e, game_object, Ptr::new(0x10), Ptr::new(0x20), 5);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_iSimulationFileSize),
            9
        );
        assert_eq!(calls_to(&e, FILE_WRITE).len(), 1);
    }

    #[test]
    fn reading_bytes_forwards_to_the_file() {
        let mut e = game();
        e.register(FILE_READ, |_, a| returns(a[2] * 2));
        let result = fn_00857ba0(&mut e, Ptr::new(0x1), Ptr::new(0x30), Ptr::new(0x40), 5);
        assert_eq!(result, 10);
        assert_eq!(calls_to(&e, FILE_READ), vec![vec![0x30, 0x40, 5]]);
    }

    #[test]
    fn advancing_the_buffer_adds_the_count() {
        let mut e = game();
        let (game_object, buffer) = game_with_buffer(&mut e, 16);
        fn_00857bd0(&mut e, game_object, 3);
        fn_00857bd0(&mut e, game_object, 4);
        assert_eq!(
            e.get(game_object, TESSaveLoadGame::m_pBuffer).addr(),
            buffer + 7
        );
    }

    fn plugin_table(e: &mut Engine, game_object: Ptr<TESSaveLoadGame>, table: &[u8]) {
        let block = e.mem.alloc(16);
        e.mem.write(block, table);
        e.set(
            game_object,
            TESSaveLoadGame::m_pFileIndexArray,
            Ptr::new(block),
        );
        e.set(
            game_object,
            TESSaveLoadGame::m_iSavedPluginCount,
            table.len() as u8,
        );
    }

    #[test]
    fn saved_ids_are_mapped_to_the_current_plugin_index() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        // No table: unchanged.
        assert_eq!(fn_00857bf0(&mut e, game_object, 0x0200_1234), 0x0200_1234);
        plugin_table(&mut e, game_object, &[0x07, 0xFF, 0x02]);
        assert_eq!(fn_00857bf0(&mut e, game_object, 0x0012_3456), 0x0712_3456);
        assert_eq!(fn_00857bf0(&mut e, game_object, 0x0212_3456), 0x0212_3456);
        // Mapped to 0xFF, or beyond the table: 0. Plugin 0xFF: unchanged.
        assert_eq!(fn_00857bf0(&mut e, game_object, 0x0112_3456), 0);
        assert_eq!(fn_00857bf0(&mut e, game_object, 0x0312_3456), 0);
        assert_eq!(fn_00857bf0(&mut e, game_object, 0xFF12_3456), 0xFF12_3456);
    }

    #[test]
    fn current_ids_are_mapped_back_to_the_saved_plugin_index() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        assert_eq!(fn_00857c70(&mut e, game_object, 0x0200_1234), 0x0200_1234);
        plugin_table(&mut e, game_object, &[0x05, 0x07, 0x05]);
        // 5 is at indexes 0 and 2: the last one wins.
        assert_eq!(fn_00857c70(&mut e, game_object, 0x0512_3456), 0x0212_3456);
        assert_eq!(fn_00857c70(&mut e, game_object, 0x0712_3456), 0x0112_3456);
        assert_eq!(fn_00857c70(&mut e, game_object, 0x0912_3456), 0);
        assert_eq!(fn_00857c70(&mut e, game_object, 0xFF12_3456), 0xFF12_3456);
    }

    /// The reference world of `fn_00857d10`: one object that is both the
    /// reference and the actor, with the virtual slots it calls.
    struct PlaceRig {
        e: Engine,
        buffer: Ptr,
        reference: Ptr,
    }

    fn place_rig(actor: bool, saved_location: bool) -> PlaceRig {
        let mut e = game();
        // Slots: 0x100 actor test, 0x290 has location, 0x294 world space,
        // 0x298 cell, 0x170 location, 0x16C rotation, 0x1F4 position.
        for (slot, value) in [
            (0x100u32, actor as u32),
            (0x290, saved_location as u32),
            (0x294, 0x5555),
            (0x298, 0xCE11),
        ] {
            constant(&mut e, 0x0310_0000 + slot, value);
        }
        e.register(0x0310_0170, |e, a| {
            for (i, value) in [1.0f32, 2.0, 3.0].iter().enumerate() {
                e.mem.set_f32(a[1] + 4 * i as u32, *value);
            }
            returns(a[1])
        });
        e.register(0x0310_016C, |e, a| {
            for (i, value) in [4.0f32, 5.0, 6.0].iter().enumerate() {
                e.mem.set_f32(a[1] + 4 * i as u32, *value);
            }
            returns(a[1])
        });
        let position = e.mem.alloc(16);
        for (i, value) in [7.0f32, 8.0, 9.0].iter().enumerate() {
            e.mem.set_f32(position + 4 * i as u32, *value);
        }
        constant(&mut e, 0x0310_01F4, position);
        let slots: Vec<(u32, u32)> = [0x100u32, 0x290, 0x294, 0x298, 0x170, 0x16C, 0x1F4]
            .iter()
            .map(|slot| (*slot, 0x0310_0000 + slot))
            .collect();
        let reference = Ptr::new(object_with_slots(&mut e, &slots));
        let buffer = Ptr::new(e.mem.alloc(0x40));
        constant(&mut e, BUFFER_GET_VERSION, 9);
        quiet(
            &mut e,
            &[
                BUFFER_SET_VERSION,
                SET_LOADING_STATE,
                REF_SET_POSITION,
                FN_005757D0,
                FN_00575700,
                REF_MOVE_TO_SPACE,
                FN_00440460,
                FN_0043FA80,
                COLLISION_RESET_SIM,
                FN_0043D410,
                FN_00A59C60,
                CHAR_CONTROLLER_SET_POSITION,
                EXTRA_GET_STARTING_SPACE,
            ],
        );
        casts(&mut e, &[]);
        PlaceRig {
            e,
            buffer,
            reference,
        }
    }

    #[test]
    fn a_saved_actor_location_moves_the_reference_and_its_node() {
        let mut rig = place_rig(true, true);
        let e = &mut rig.e;
        constant(e, REF_GET_NODE, 0x9900);
        constant(e, REF_GET_ORIENTATION, 0x9A00);
        let mobile = 0x9B00;
        casts(e, &[(rig.reference.addr(), RTTI_MOBILE_OBJECT, mobile)]);
        constant(e, MOBILE_GET_CHAR_CONTROLLER, 0x9C00);
        constant(e, CHAR_CONTROLLER_TEST, 0);
        // The positions passed by address live in blocks the game frees, so
        // read their first float when the call is made.
        let seen: Rc<RefCell<Vec<(u32, f32)>>> = Rc::new(RefCell::new(vec![]));
        for address in [REF_SET_POSITION, CHAR_CONTROLLER_SET_POSITION] {
            let sink = seen.clone();
            e.register_double(address, move |e, a| {
                sink.borrow_mut().push((address, e.mem.f32(a[1])));
                Ret::default()
            });
        }
        let moved = fn_00857d10(e, rig.buffer, rig.reference, true);
        assert!(moved);
        // The position is the location the actor gave (1, 2, 3) and the
        // character controller gets the reference's position (7, 8, 9); the
        // angle is the z of the rotation.
        assert_eq!(
            *seen.borrow(),
            vec![(REF_SET_POSITION, 1.0), (CHAR_CONTROLLER_SET_POSITION, 7.0)]
        );
        assert_eq!(calls_to(e, REF_SET_POSITION)[0][0], rig.reference.addr());
        assert_eq!(
            calls_to(e, FN_005757D0),
            vec![vec![rig.reference.addr(), 6.0f32.to_bits()]]
        );
        // Teleporting brackets the move; the cell and the world space follow.
        let order: Vec<u32> = e
            .call_log
            .as_ref()
            .unwrap()
            .iter()
            .map(|(address, _)| *address)
            .filter(|a| [SET_LOADING_STATE, REF_MOVE_TO_SPACE].contains(a))
            .collect();
        assert_eq!(
            order,
            vec![SET_LOADING_STATE, REF_MOVE_TO_SPACE, SET_LOADING_STATE]
        );
        assert_eq!(
            calls_to(e, REF_MOVE_TO_SPACE),
            vec![vec![rig.reference.addr(), 0xCE11, 0x5555]]
        );
        // The node: the character controller gets the position, the node the
        // position and the orientation, the collision is reset.
        assert_eq!(calls_to(e, CHAR_CONTROLLER_SET_POSITION).len(), 1);
        assert_eq!(calls_to(e, CHAR_CONTROLLER_SET_POSITION)[0][0], 0x9C00);
        assert_eq!(calls_to(e, FN_0043FA80), vec![vec![0x9900, 0x9A00]]);
        assert_eq!(calls_to(e, COLLISION_RESET_SIM), vec![vec![0x9900, 1]]);
        assert_eq!(calls_to(e, FN_00A59C60).len(), 1);
        // The version byte is cleared and put back.
        let versions = calls_to(e, BUFFER_SET_VERSION);
        assert_eq!(
            versions,
            vec![vec![rig.buffer.addr(), 0], vec![rig.buffer.addr(), 9]]
        );
    }

    #[test]
    fn a_reference_without_saved_data_is_not_moved() {
        let mut rig = place_rig(false, false);
        let e = &mut rig.e;
        constant(e, REF_GET_EXTRA_LIST, 0x7000);
        constant(e, EXTRA_GET_DATA, 0);
        let moved = fn_00857d10(e, rig.buffer, rig.reference, false);
        assert!(!moved);
        assert!(calls_to(e, REF_MOVE_TO_SPACE).is_empty());
        assert!(calls_to(e, REF_GET_NODE).is_empty());
        assert_eq!(calls_to(e, BUFFER_SET_VERSION).len(), 2);
    }

    #[test]
    fn extra_data_gives_the_position_and_the_starting_space() {
        let mut rig = place_rig(false, false);
        let e = &mut rig.e;
        constant(e, REF_GET_EXTRA_LIST, 0x7000);
        constant(e, EXTRA_GET_DATA, 1);
        constant(e, EXTRA_GET_STARTING_SPACE, 0x7100);
        constant(e, REF_GET_NODE, 0);
        casts(e, &[(0x7100, RTTI_CELL, 0x7100)]);
        let moved = fn_00857d10(e, rig.buffer, rig.reference, true);
        assert!(moved);
        // Extra data of type 0xF was asked of the list.
        assert_eq!(calls_to(e, EXTRA_GET_DATA), vec![vec![0x7000, 0xF]]);
        // The rotation words come from the out buffer (4, 5, 6).
        assert_eq!(
            calls_to(e, FN_00575700),
            vec![vec![
                rig.reference.addr(),
                4.0f32.to_bits(),
                5.0f32.to_bits(),
                6.0f32.to_bits()
            ]]
        );
        // The starting cell moves the reference (no world space), and the
        // teleport bracket is not used on this path.
        assert_eq!(
            calls_to(e, REF_MOVE_TO_SPACE),
            vec![vec![rig.reference.addr(), 0x7100, 0]]
        );
        assert!(calls_to(e, SET_LOADING_STATE).is_empty());
    }

    /// The doubles and the log of `SaveGlobalData`.
    fn global_data_rig() -> (Engine, Ptr<TESSaveLoadGame>, ByteWrites) {
        let mut e = game();
        map_globals(&mut e);
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let handler = e.mem.alloc(0x20);
        let tes = e.mem.alloc(0x20);
        let player = e.mem.alloc(0x20);
        e.set_global(DATA_HANDLER, handler);
        e.set_global(TES_OBJECT, tes);
        e.set_global(PLAYER, player);
        constant(&mut e, GLOBAL_DATA_SIZE, 0x1122_3344);
        let tes_worldspace = e.mem.alloc(0x20);
        e.mem.set_u32(tes_worldspace + 0xC, 0x3C);
        constant(&mut e, TES_GET_WORLDSPACE, tes_worldspace);
        constant(&mut e, READ_FIELD_24, 5);
        constant(&mut e, READ_FIELD_28, 6);
        let worldspace = e.mem.alloc(0x20);
        e.mem.set_u32(worldspace + 0xC, 0x77);
        constant(&mut e, REF_GET_WORLDSPACE, worldspace);
        constant(&mut e, REF_GET_PARENT_CELL, 0);
        let position = e.mem.alloc(16);
        e.mem.set_f32(position, 1.5);
        constant(&mut e, REF_GET_POSITION, position);
        quiet(
            &mut e,
            &[
                GLOBALS_LIST,
                LIST_COUNT,
                SAVE_CREATED_BASE_OBJECTS,
                TES_SAVE,
                PROCESS_LISTS_SAVE,
                SKY_SAVE,
                INTERFACE_SAVE,
                REGIONS_SAVE,
                SKY_INSTANCE,
                ERROR,
                TES_SAVE_SIZE,
                PROCESS_LISTS_SAVE_SIZE,
                SKY_SAVE_SIZE,
                INTERFACE_SAVE_SIZE,
                REGIONS_SAVE_SIZE,
            ],
        );
        let writes: Rc<RefCell<Vec<Vec<u8>>>> = Rc::new(RefCell::new(vec![]));
        let sink = writes.clone();
        e.register_double(FILE_WRITE, move |e, a| {
            sink.borrow_mut().push(e.mem.bytes(a[1], a[2]));
            returns(a[2])
        });
        (e, game_object, writes)
    }

    #[test]
    fn global_data_writes_the_header_words_and_the_blocks() {
        let (mut e, game_object, writes) = global_data_rig();
        constant(&mut e, TES_SAVE_SIZE, 3);
        tes_save_load_game_save_global_data(&mut e, game_object, Ptr::new(0x6000));
        let writes = writes.borrow();
        let lengths: Vec<usize> = writes.iter().map(Vec::len).collect();
        // handler size, TES world space id, two TES words, player location
        // id and position, the globals block, then the TES size word and
        // its block (3 bytes), the other blocks' size words (all 0), the
        // zero word before the created base objects, and the reticle,
        // interface and regions size words.
        assert_eq!(lengths, vec![4, 4, 4, 4, 4, 12, 2, 2, 3, 2, 2, 4, 2, 2, 2]);
        assert_eq!(writes[0], 0x1122_3344u32.to_le_bytes());
        assert_eq!(writes[1], 0x3Cu32.to_le_bytes());
        assert_eq!(writes[2], 5u32.to_le_bytes());
        assert_eq!(writes[3], 6u32.to_le_bytes());
        // The player is in a world space: its id is the location.
        assert_eq!(writes[4], 0x77u32.to_le_bytes());
        assert_eq!(writes[5][0..4], 1.5f32.to_le_bytes());
        assert_eq!(writes[7], 3u16.to_le_bytes());
        assert_eq!(calls_to(&e, TES_SAVE).len(), 1);
        assert!(calls_to(&e, PROCESS_LISTS_SAVE).is_empty());
        assert_eq!(
            calls_to(&e, SAVE_CREATED_BASE_OBJECTS),
            vec![vec![game_object.addr(), 0x6000]]
        );
        assert!(calls_to(&e, ERROR).is_empty());
        // The buffer made for the block was freed.
        assert!(e.get(game_object, TESSaveLoadGame::m_pBuffer).is_null());
    }

    #[test]
    fn global_data_complains_when_the_player_has_no_location() {
        let (mut e, game_object, writes) = global_data_rig();
        constant(&mut e, REF_GET_WORLDSPACE, 0);
        tes_save_load_game_save_global_data(&mut e, game_object, Ptr::new(0x6000));
        assert_eq!(calls_to(&e, ERROR), vec![vec![MSG_PLAYER_HAS_NO_SPACE]]);
        assert_eq!(writes.borrow()[4], 0u32.to_le_bytes());
    }

    #[test]
    fn global_data_uses_the_parent_cell_and_notes_blocks_in_the_statistics() {
        let (mut e, game_object, writes) = global_data_rig();
        let cell = e.mem.alloc(0x20);
        e.mem.set_u32(cell + 0xC, 0x99);
        constant(&mut e, REF_GET_WORLDSPACE, 0);
        constant(&mut e, REF_GET_PARENT_CELL, cell);
        constant(&mut e, PROCESS_LISTS_SAVE_SIZE, 5);
        let stats: Ptr<SaveStats> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pSaveLoadStats, stats);
        // `AddExtraStat` (this file's own) puts an `ExtraStat` on the list
        // at `pExtraStats`: note the sizes it was given.
        let sizes: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(vec![]));
        let sink = sizes.clone();
        e.register_double(LIST_ADD_HEAD, move |e, a| {
            let stat = e.mem.u32(a[1]);
            sink.borrow_mut().push(e.mem.u32(stat));
            Ret::default()
        });
        tes_save_load_game_save_global_data(&mut e, game_object, Ptr::new(0x6000));
        assert_eq!(writes.borrow()[4], 0x99u32.to_le_bytes());
        assert_eq!(calls_to(&e, PROCESS_LISTS_SAVE), vec![vec![PROCESS_LISTS]]);
        // The globals block (2 bytes) and the process lists block (5) are
        // noted.
        assert_eq!(*sizes.borrow(), vec![2, 5]);
    }

    #[test]
    fn globals_are_written_as_a_count_and_id_value_pairs() {
        let (mut e, game_object, writes) = global_data_rig();
        let variable = e.mem.alloc(0x20);
        e.mem.set_u32(variable + 0xC, 0x42);
        let list = list_of(&mut e, &[variable, 0]);
        constant(&mut e, GLOBALS_LIST, list);
        constant(&mut e, LIST_COUNT, 2);
        constant(&mut e, USE_NUMERIC_IDS, 0);
        e.register(GLOBAL_VALUE, |_, _| Ret {
            st0: 2.5,
            ..Ret::default()
        });
        tes_save_load_game_save_globals(&mut e, game_object, Ptr::new(0x6000));
        let writes = writes.borrow();
        assert_eq!(writes.len(), 1);
        // 2 * 8 + 2 bytes: the count, then one pair (the second item is null).
        assert_eq!(writes[0].len(), 18);
        assert_eq!(writes[0][0..2], 2u16.to_le_bytes());
        assert_eq!(writes[0][2..6], 0x42u32.to_le_bytes());
        assert_eq!(writes[0][6..10], 2.5f32.to_le_bytes());
    }

    #[test]
    fn final_data_writes_the_size_and_the_temp_effects() {
        let (mut e, game_object, writes) = global_data_rig();
        stub(&mut e, TEMP_EFFECTS_SAVE);
        constant(&mut e, TEMP_EFFECTS_SIZE, 0);
        tes_save_load_game_save_final_data(&mut e, game_object, Ptr::new(0x6000));
        assert_eq!(writes.borrow().len(), 1);
        assert_eq!(writes.borrow()[0], 0u32.to_le_bytes());
        assert!(calls_to(&e, TEMP_EFFECTS_SAVE).is_empty());
        constant(&mut e, TEMP_EFFECTS_SIZE, 6);
        tes_save_load_game_save_final_data(&mut e, game_object, Ptr::new(0x6000));
        let writes = writes.borrow();
        assert_eq!(writes.len(), 3);
        assert_eq!(writes[1], 6u32.to_le_bytes());
        assert_eq!(writes[2].len(), 6);
        assert_eq!(calls_to(&e, TEMP_EFFECTS_SAVE), vec![vec![PROCESS_LISTS]]);
    }

    #[test]
    fn create_buffer_makes_the_current_buffer_and_complains_on_failure() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        quiet(&mut e, &[ERROR]);
        let buffer = tes_save_load_game_create_buffer(&mut e, game_object, 24);
        assert!(!buffer.is_null());
        assert_eq!(e.mem.block_size(buffer.addr()), Some(24));
        assert_eq!(e.get(game_object, TESSaveLoadGame::m_pBuffer), buffer);
        assert!(calls_to(&e, ERROR).is_empty());
        // A failed allocation raises the error and gives null.
        constant(&mut e, OPERATOR_NEW, 0);
        let none = tes_save_load_game_create_buffer(&mut e, game_object, 24);
        assert!(none.is_null());
        assert_eq!(calls_to(&e, ERROR), vec![vec![MSG_NO_SAVE_BUFFER]]);
        // The scope is entered with the source line of the call.
        let scopes = calls_to(&e, SCOPE_ENTER);
        assert_eq!(scopes[0][1..], [0x11, 1, SOURCE_FILE, 0x103A]);
    }

    #[test]
    fn write_file_and_read_file_forward_to_the_file_functions() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        e.register(FILE_WRITE, |_, a| returns(a[2]));
        e.register(FILE_READ, |_, a| returns(a[2] + 1));
        tes_save_load_game_write_file(&mut e, game_object, Ptr::new(0x10), Ptr::new(0x20), 7);
        assert_eq!(calls_to(&e, FILE_WRITE), vec![vec![0x10, 0x20, 7]]);
        let read = fn_008586d0(&mut e, game_object, Ptr::new(0x10), Ptr::new(0x20), 7);
        assert_eq!(read, 8);
        assert_eq!(calls_to(&e, FILE_READ), vec![vec![0x10, 0x20, 7]]);
    }

    #[test]
    fn free_buffer_frees_it_and_clears_the_pointer() {
        let mut e = game();
        let (game_object, buffer) = game_with_buffer(&mut e, 16);
        fn_00858700(&mut e, game_object, Ptr::new(buffer));
        assert!(freed(&e, buffer));
        assert!(e.get(game_object, TESSaveLoadGame::m_pBuffer).is_null());
    }

    /// `0047c850` is true for the first `n` calls and false after that.
    fn measuring_for(e: &mut Engine, n: u32) {
        let left = Rc::new(RefCell::new(n));
        e.register_double(SAVE_LOAD_UNAVAILABLE, move |_, _| {
            let mut left = left.borrow_mut();
            if *left > 0 {
                *left -= 1;
                returns(1)
            } else {
                returns(0)
            }
        });
    }

    #[test]
    fn load_header_is_built_from_the_buffer_bytes() {
        let mut e = game();
        let source = e.mem.alloc(8);
        e.mem.write(source, &[0x10, 0x00, 0x28, 0x05]);
        let header: Ptr<LoadFormHeader> = e.new_object();
        let result = fn_00858aa0(&mut e, header, Ptr::new(source), 0x1234, 0x40);
        assert_eq!(result, header);
        assert_eq!(e.get(header, LoadFormHeader::iFormID), 0x1234);
        assert_eq!(e.get(header, LoadFormHeader::cFormType), 0x28);
        assert_eq!(e.get(header, LoadFormHeader::iFlags), 0x40);
        assert_eq!(e.get(header, LoadFormHeader::cVersion), 5);
        assert_eq!(e.get(header, LoadFormHeader::iSize), 0x10);
    }

    /// The load world of `fn_00858730`: the game is the singleton, a form
    /// `0x1234` of type `form_type` has a `ChangeData` whose buffer starts
    /// with a header of type `saved_type`.
    struct LoadRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        form: Ptr,
        change_data: Ptr<ChangeData>,
        buffer: u32,
    }

    fn load_rig(saved_type: u8, form_type: u8) -> LoadRig {
        let mut e = game();
        map_globals(&mut e);
        let game_object = game_singleton(&mut e);
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let form = Ptr::new(object_with_slots(&mut e, &[(0x60, 0x0320_0060)]));
        e.mem.set_u32(form.addr() + 0xC, 0x1234);
        let buffer = e.mem.alloc(16);
        // The header: size 0x10, form type, version 5.
        e.mem.write(buffer, &[0x10, 0x00, saved_type, 0x05]);
        let data = change_data(&mut e, 0x40, buffer);
        table.borrow_mut().push((0x1234, data.addr()));
        e.set_global(DATA_HANDLER, 0x0055_0000u32);
        constant(&mut e, DATA_HANDLER_HAS_FORM, 0);
        constant(&mut e, FORM_TYPE, form_type as u32);
        quiet(
            &mut e,
            &[
                SECTION_ENTER,
                SECTION_LEAVE,
                SET_LOAD_VERSION,
                SET_LOADING_HEADER,
                SET_LOADING_STATE,
                FORM_FINISH,
                LOAD_INITIAL_DATA,
                END_FORM_PROCESSING,
                CHANGE_DATA_SET_BUFFER,
                0x0320_0060,
            ],
        );
        LoadRig {
            e,
            game: game_object,
            form,
            change_data: data,
            buffer,
        }
    }

    #[test]
    fn load_form_does_nothing_unless_the_game_is_measuring_flag_is_set() {
        let mut rig = load_rig(0x28, 0x28);
        let e = &mut rig.e;
        // `0047c850` is false here.
        assert!(!fn_00858730(e, rig.game, rig.form));
        assert!(calls_to(e, SECTION_ENTER).is_empty());
    }

    #[test]
    fn load_form_needs_a_change_data_with_a_buffer() {
        let mut rig = load_rig(0x28, 0x28);
        let e = &mut rig.e;
        measuring_for(e, 100);
        let unknown = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(unknown.addr() + 0xC, 0x9999);
        assert!(!fn_00858730(e, rig.game, unknown));
        e.set(rig.change_data, ChangeData::pBuffer, Ptr::NULL);
        assert!(!fn_00858730(e, rig.game, rig.form));
        assert!(calls_to(e, SECTION_ENTER).is_empty());
    }

    #[test]
    fn load_form_loads_the_form_from_its_buffer() {
        let mut rig = load_rig(0x28, 0x28);
        let e = &mut rig.e;
        // True for the start and for the saved state; the queued remove is
        // not set, so nothing else asks.
        measuring_for(e, 2);
        let array = e.mem.alloc(0x18);
        constant(e, INIT_ARRAY_CONSTRUCT, array);
        let added: Added = Rc::new(RefCell::new(vec![]));
        let sink = added.clone();
        e.register_double(INIT_ARRAY_ADD, move |e, a| {
            let record = e.mem.u32(a[1]);
            sink.borrow_mut().push((
                a[0],
                e.mem.u32(record),
                e.mem.u32(record + 4),
                e.mem.u32(record + 8),
                e.mem.u8(record + 0xC),
            ));
            Ret::default()
        });
        assert!(fn_00858730(e, rig.game, rig.form));
        // The section is entered with the load name and left again.
        assert_eq!(
            calls_to(e, SECTION_ENTER),
            vec![vec![LOAD_SECTION, MSG_LOAD_FORM_SECTION]]
        );
        assert_eq!(calls_to(e, SECTION_LEAVE), vec![vec![LOAD_SECTION]]);
        // The initial data and the changes are applied with the flags.
        assert_eq!(
            calls_to(e, LOAD_INITIAL_DATA),
            vec![vec![rig.game.addr(), rig.form.addr(), 0x40]]
        );
        assert_eq!(
            calls_to(e, 0x0320_0060),
            vec![vec![rig.form.addr(), 0x40, 0]]
        );
        // The version read from the buffer is used, the form is noted in the
        // init array (made on first use) with the flags and the version.
        assert_eq!(
            calls_to(e, SET_LOAD_VERSION),
            vec![vec![rig.game.addr(), 5]]
        );
        assert_eq!(e.get(rig.game, TESSaveLoadGame::m_pInitArray).addr(), array);
        assert_eq!(*added.borrow(), vec![(array, rig.form.addr(), 0x40, 0, 5)]);
        // The buffer is freed and the change data lets go of it.
        assert!(freed(e, rig.buffer));
        assert_eq!(
            calls_to(e, CHANGE_DATA_SET_BUFFER),
            vec![vec![rig.change_data.addr(), 0]]
        );
        assert!(e.get(rig.game, TESSaveLoadGame::m_pBuffer).is_null());
    }

    #[test]
    fn load_form_applies_a_queued_remove_changes() {
        let mut rig = load_rig(0x28, 0x28);
        let e = &mut rig.e;
        // The start, the saved state and the flag check of `fn_00855150`.
        measuring_for(e, 2);
        constant(e, INIT_ARRAY_CONSTRUCT, 0);
        stub(e, INIT_ARRAY_ADD);
        let array = e.mem.alloc(0x18);
        e.set(rig.game, TESSaveLoadGame::m_pInitArray, Ptr::new(array));
        e.set(rig.game, TESSaveLoadGame::m_iQueuedRemoveChanges, 0x40);
        assert!(fn_00858730(e, rig.game, rig.form));
        // The queued flags were cleared from the change data (`fn_00854e70`)
        // and the request is gone.
        assert_eq!(e.get(rig.game, TESSaveLoadGame::m_iQueuedRemoveChanges), 0);
        assert_eq!(calls_to(e, CHANGE_DATA_SET_BUFFER).len(), 1);
        // The queued flags were looked up in the changes map a second time.
        assert_eq!(calls_to(e, MAP_GET_AT).len(), 2);
    }

    #[test]
    fn load_form_skips_a_form_whose_type_changed() {
        let mut rig = load_rig(0x28, 0x2A);
        let e = &mut rig.e;
        measuring_for(e, 1);
        // The type name table and the form's own type name.
        e.mem.set_u32(FORM_TYPE_NAME_TABLE + 0x28 * 12, 0x0108_0000);
        constant(e, FORM_TYPE_NAME, 0x0108_0010);
        stub(e, SPRINTF);
        assert!(!fn_00858730(e, rig.game, rig.form));
        // The text of the error is formatted with the id and both type names.
        assert_eq!(
            calls_to(e, SPRINTF)[0][1..],
            [FORMAT_LOAD_ERROR, 0x1234, 0x0108_0000, 0x0108_0010]
        );
        // The section is left, nothing was loaded and the buffer is not kept.
        assert_eq!(calls_to(e, SECTION_LEAVE), vec![vec![LOAD_SECTION]]);
        assert!(calls_to(e, LOAD_INITIAL_DATA).is_empty());
        assert!(e.get(rig.game, TESSaveLoadGame::m_pBuffer).is_null());
    }

    /// A form record of the init array: pForm, flags, old flags, version.
    fn form_and_flags(e: &mut Engine, form: u32, flags: u32, old: u32, version: u8) -> u32 {
        let record = e.mem.alloc(0x10);
        e.mem.set_u32(record, form);
        e.mem.set_u32(record + 4, flags);
        e.mem.set_u32(record + 8, old);
        e.mem.set_u8(record + 0xC, version);
        record
    }

    /// The after-load world: an init array of the given records.
    struct InitRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        array: u32,
        player: u32,
        hooks: Hooks,
    }

    fn init_rig(records: &[u32]) -> InitRig {
        let mut e = game();
        map_globals(&mut e);
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        measuring_for(&mut e, 1000);
        let array = e.mem.alloc(0x40);
        e.mem.set_u32(array + 0xC, records.len() as u32);
        for (i, record) in records.iter().enumerate() {
            e.mem.set_u32(array + 0x20 + 4 * i as u32, *record);
        }
        e.register(ARRAY_ELEMENT_ADDRESS, |_, a| {
            returns(a[0] + 0x20 + 4 * a[1])
        });
        let player = object_with_slots(&mut e, &[(0x68, 0x0330_0068), (0x6C, 0x0330_006C)]);
        e.set_global(PLAYER, player);
        let hooks: Hooks = Rc::new(RefCell::new(vec![]));
        for slot in [0x68u32, 0x6C] {
            let sink = hooks.clone();
            e.register_double(0x0330_0000 + slot, move |_, a| {
                sink.borrow_mut().push((slot, a[0], a[1], a[2]));
                Ret::default()
            });
        }
        quiet(
            &mut e,
            &[
                SET_LOADING_STATE,
                SET_LOAD_VERSION,
                END_FORM_PROCESSING,
                IO_MANAGER_SET_STATE_5,
                REF_GET_PARENT_CELL,
                REF_GET_WORLDSPACE,
                ACTOR_INIT_PACKAGE_LOCATIONS,
                LIST_REMOVE_ALL,
                LIST_DESTRUCT,
                FORM_SET_DISABLED,
                LOOKUP_FORM,
            ],
        );
        // The local list: its head node holds the first item.
        e.register(LIST_ADD_HEAD, |e, a| {
            let item = e.mem.u32(a[1]);
            e.mem.set_u32(a[0], item);
            Ret::default()
        });
        e.register(LIST_NODE_IS_END, |e, a| {
            returns((e.mem.u32(a[0]) == 0 && e.mem.u32(a[0] + 4) == 0) as u32)
        });
        casts(&mut e, &[]);
        InitRig {
            e,
            game: game_object,
            array,
            player,
            hooks,
        }
    }

    #[test]
    fn after_load_runs_the_hooks_and_frees_the_records() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        let first: Ptr = Ptr::new(object_with_slots(
            e,
            &[(0x68, 0x0330_0068), (0x6C, 0x0330_006C)],
        ));
        let record_a = form_and_flags(e, first.addr(), 1, 2, 7);
        // The player's own record has no first hook and is still finished.
        let record_b = form_and_flags(e, rig.player, 3, 4, 8);
        e.mem.set_u32(rig.array + 0xC, 2);
        e.mem.set_u32(rig.array + 0x20, record_a);
        e.mem.set_u32(rig.array + 0x24, record_b);
        let array = Ptr::new(rig.array);
        fn_00858af0(e, rig.game, array, Ptr::NULL, false);
        let hooks = rig.hooks.borrow();
        assert_eq!(
            *hooks,
            vec![
                (0x68, first.addr(), 1, 2),
                (0x6C, first.addr(), 1, 2),
                (0x6C, rig.player, 3, 4),
            ]
        );
        assert_eq!(
            calls_to(e, SET_LOAD_VERSION),
            vec![
                vec![rig.game.addr(), 7],
                vec![rig.game.addr(), 7],
                vec![rig.game.addr(), 8]
            ]
        );
        assert!(freed(e, record_a));
        assert!(freed(e, record_b));
        // The loading state brackets the work.
        let states = calls_to(e, SET_LOADING_STATE);
        assert_eq!(states.first().unwrap()[1], 1);
        assert_eq!(states.last().unwrap()[1], 0);
    }

    #[test]
    fn after_load_places_or_disables_actors_that_are_in_no_space() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        let table = install_table(e, MAP_GET_AT, Some(CHANGES_MAP_SET_AT), MAP_REMOVE_AT);
        let actor: Ptr = Ptr::new(object_with_slots(
            e,
            &[(0x68, 0x0330_0068), (0x6C, 0x0330_006C)],
        ));
        e.mem.set_u32(actor.addr() + 0xC, 0xA11);
        let record = form_and_flags(e, actor.addr(), 1, 2, 7);
        e.mem.set_u32(rig.array + 0xC, 1);
        e.mem.set_u32(rig.array + 0x20, record);
        casts(e, &[(actor.addr(), RTTI_ACTOR, actor.addr())]);
        // No placement cell (the lookup of form id 0 is null): disabled.
        fn_00858af0(e, rig.game, Ptr::new(rig.array), Ptr::NULL, false);
        assert_eq!(
            calls_to(e, ACTOR_INIT_PACKAGE_LOCATIONS),
            vec![vec![actor.addr(), 0]]
        );
        assert_eq!(calls_to(e, FORM_SET_DISABLED), vec![vec![actor.addr(), 1]]);
        // The actor was flagged in the changes map (flags 2).
        let stored = table.borrow().clone();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].0, 0xA11);
        assert_eq!(e.mem.u32(stored[0].1), 2);
    }

    #[test]
    fn after_load_moves_an_actor_to_the_placement_cell() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        install_table(e, MAP_GET_AT, Some(CHANGES_MAP_SET_AT), MAP_REMOVE_AT);
        let actor: Ptr = Ptr::new(object_with_slots(
            e,
            &[(0x68, 0x0330_0068), (0x6C, 0x0330_006C)],
        ));
        let record = form_and_flags(e, actor.addr(), 1, 2, 7);
        e.mem.set_u32(rig.array + 0xC, 1);
        e.mem.set_u32(rig.array + 0x20, record);
        // The placement cell exists and fills the position and rotation.
        casts(
            e,
            &[
                (actor.addr(), RTTI_ACTOR, actor.addr()),
                (0x0777, RTTI_CELL, 0x0777),
            ],
        );
        constant(e, LOOKUP_FORM, 0x0777);
        e.mem.set_f32(DEFAULT_POSITION, 1.0);
        e.register(CELL_GET_PLACEMENT, |e, a| {
            e.mem.set_f32(a[1], 10.0);
            e.mem.set_f32(a[2], 20.0);
            Ret::default()
        });
        let seen: Rc<RefCell<Vec<f32>>> = Rc::new(RefCell::new(vec![]));
        let sink = seen.clone();
        e.register_double(REF_SET_POSITION, move |e, a| {
            sink.borrow_mut().push(e.mem.f32(a[1]));
            Ret::default()
        });
        quiet(e, &[FN_00575700, REF_MOVE_TO_SPACE]);
        fn_00858af0(e, rig.game, Ptr::new(rig.array), Ptr::NULL, false);
        assert!(calls_to(e, FORM_SET_DISABLED).is_empty());
        assert_eq!(*seen.borrow(), vec![10.0]);
        // The rotation vector's x is the second block's first float.
        assert_eq!(
            calls_to(e, FN_00575700),
            vec![vec![actor.addr(), 20.0f32.to_bits(), 0, 0]]
        );
        assert_eq!(
            calls_to(e, REF_MOVE_TO_SPACE),
            vec![vec![actor.addr(), 0x0777, 0]]
        );
    }

    #[test]
    fn after_load_with_reload_refreshes_the_world_and_the_camera() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        let tes = e.mem.alloc(0x40);
        e.set_global(TES_OBJECT, tes);
        e.set_global(SAVE_LOCK, 0x0120_2d00u32);
        e.set_global(MODEL_LOADER, 0x0066_0000u32);
        // The player has no 3D: it is queued for loading.
        constant(e, PLAYER_GET_3D, 0);
        constant(e, READ_FIELD_34, 0x1111);
        constant(e, CELL_GET_PHYSICS_WORLD, 0x2222);
        constant(e, GET_EXTERIOR_WORLD, 0x3333);
        quiet(
            e,
            &[
                FN_00459920,
                MODEL_LOADER_QUEUE_REFERENCE,
                IO_MANAGER_LOAD_QUEUED_PRIORITY,
                WORLD_ADD_LOCK,
                WORLD_REMOVE_LOCK,
                FN_0057D0A0,
                SET_GLOBAL_FLAG,
            ],
        );
        // The player's position slot (0x1F4).
        let position = e.mem.alloc(16);
        for (i, v) in [1.0f32, 2.0, 3.0].iter().enumerate() {
            e.mem.set_f32(position + 4 * i as u32, *v);
        }
        constant(e, 0x0330_01F4, position);
        let player = rig.player;
        let vtable = e.mem.u32(player);
        e.mem.set_u32(vtable + 0x1F4, 0x0330_01F4);
        for (i, v) in [7.0f32, 8.0, 9.0].iter().enumerate() {
            e.mem.set_f32(PLACEMENT_VECTOR + 4 * i as u32, *v);
        }
        fn_00858af0(e, rig.game, Ptr::NULL, Ptr::NULL, true);
        // Both worlds are locked and unlocked, twice (before and after the
        // queue is flushed).
        assert_eq!(
            calls_to(e, WORLD_ADD_LOCK),
            vec![vec![0x2222], vec![0x3333], vec![0x2222], vec![0x3333]]
        );
        assert_eq!(
            calls_to(e, WORLD_REMOVE_LOCK),
            vec![
                vec![0x2222, 0],
                vec![0x3333, 0],
                vec![0x2222, 0],
                vec![0x3333, 0]
            ]
        );
        assert_eq!(
            calls_to(e, MODEL_LOADER_QUEUE_REFERENCE),
            vec![vec![0x0066_0000, player, 0, 0]]
        );
        assert_eq!(calls_to(e, PLAYER_GET_3D), vec![vec![player, 0]]);
        // The camera: the player's position, the placement vector and 1.0,
        // between the flag being cleared and set.
        assert_eq!(
            calls_to(e, FN_0057D0A0),
            vec![vec![
                1.0f32.to_bits(),
                2.0f32.to_bits(),
                3.0f32.to_bits(),
                7.0f32.to_bits(),
                8.0f32.to_bits(),
                9.0f32.to_bits(),
                1.0f32.to_bits()
            ]]
        );
        assert_eq!(calls_to(e, SET_GLOBAL_FLAG), vec![vec![0], vec![1]]);
        assert_eq!(calls_to(e, FN_00459920), vec![vec![tes]]);
    }

    #[test]
    fn after_load_destroys_the_games_init_array() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        let array = object_with_slots(e, &[(0, 0x0340_0000)]);
        e.mem.set_u32(array + 0xC, 0);
        stub(e, 0x0340_0000);
        stub(e, INIT_ARRAY_CLEANUP);
        e.set(rig.game, TESSaveLoadGame::m_pInitArray, Ptr::new(array));
        fn_00858af0(e, rig.game, Ptr::NULL, Ptr::NULL, false);
        assert_eq!(calls_to(e, INIT_ARRAY_CLEANUP), vec![vec![array]]);
        assert_eq!(calls_to(e, 0x0340_0000), vec![vec![array, 1]]);
        assert!(e.get(rig.game, TESSaveLoadGame::m_pInitArray).is_null());
    }

    #[test]
    fn after_load_does_nothing_unless_the_measuring_flag_is_set() {
        let mut rig = init_rig(&[]);
        let e = &mut rig.e;
        e.register(SAVE_LOAD_UNAVAILABLE, |_, _| returns(0));
        fn_00858af0(e, rig.game, Ptr::new(rig.array), Ptr::NULL, true);
        assert!(calls_to(e, SET_LOADING_STATE).is_empty());
    }

    #[test]
    fn a_known_reference_loses_bit_one() {
        let mut e = game();
        map_globals(&mut e);
        e.set_global(DATA_HANDLER, 0x0055_0000u32);
        let form = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(form.addr() + 0xC, 0x1234);
        casts(&mut e, &[(form.addr(), RTTI_REFERENCE, form.addr())]);
        let known: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
        let state = known.clone();
        e.register_double(DATA_HANDLER_HAS_FORM, move |_, a| {
            assert_eq!(a[1], 0x1234);
            returns(*state.borrow() as u32)
        });
        assert_eq!(
            tes_save_load_game_check_new_reference(&mut e, Ptr::NULL, form, 0xFF),
            0xFD
        );
        // A form the handler does not know keeps its flags, and so does one
        // that is not a reference.
        *known.borrow_mut() = false;
        assert_eq!(
            tes_save_load_game_check_new_reference(&mut e, Ptr::NULL, form, 0xFF),
            0xFF
        );
        *known.borrow_mut() = true;
        casts(&mut e, &[]);
        assert_eq!(
            tes_save_load_game_check_new_reference(&mut e, Ptr::NULL, form, 0xFF),
            0xFF
        );
    }

    /// The flag-checking world: a form that casts to a reference (and an
    /// actor), a game and the doubles every path needs.
    struct FlagsRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        form: Ptr,
    }

    fn flags_rig() -> FlagsRig {
        let mut e = game();
        map_globals(&mut e);
        e.set_global(DATA_HANDLER, 0x0055_0000u32);
        constant(&mut e, DATA_HANDLER_HAS_FORM, 0);
        let player = e.mem.alloc(0x40);
        e.set_global(PLAYER, player);
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let form = Ptr::new(e.mem.alloc(0x40));
        quiet(
            &mut e,
            &[
                EXTRA_GET_SEEN,
                CELL_IS_INTERIOR,
                CELL_GET_X,
                CELL_GET_Y,
                REF_GET_EXTRA_LIST,
                EXTRA_GET_CONTAINER_CHANGES,
                ACTOR_GET_PROCESS,
                ACTOR_PACKAGE_FLAGS,
                ACTOR_TEST_FLAGS,
                REF_PERSISTS,
                REF_GET_PARENT_CELL,
                REF_GET_WORLDSPACE,
                WORLDSPACE_GET_CELL,
                FLOAT_TO_INT,
                LOG_ERROR,
            ],
        );
        casts(&mut e, &[]);
        FlagsRig {
            e,
            game: game_object,
            form,
        }
    }

    #[test]
    fn check_flags_clears_the_top_bit_of_a_cell_without_seen_data() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        casts(e, &[(rig.form.addr(), RTTI_CELL, rig.form.addr())]);
        let flags = tes_save_load_game_check_flags(e, rig.game, rig.form, 0x8000_0003);
        assert_eq!(flags, 3);
        // An exterior cell also has its coordinates read.
        assert_eq!(calls_to(e, CELL_GET_X).len(), 1);
        assert_eq!(calls_to(e, CELL_GET_Y).len(), 1);
        // With seen data the bit stays.
        constant(e, EXTRA_GET_SEEN, 1);
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0x8000_0003),
            0x8000_0003
        );
    }

    #[test]
    fn check_flags_leaves_other_forms_alone() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0x1234),
            0x1234
        );
    }

    #[test]
    fn check_flags_drops_bit_five_without_container_changes() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        let reference = rig.form.addr();
        casts(e, &[(reference, RTTI_REFERENCE, reference)]);
        // No extra data list at all.
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0x21),
            0x01
        );
        // A list without container changes.
        constant(e, REF_GET_EXTRA_LIST, 0x7000);
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0x21),
            0x01
        );
        // With container changes bit 5 stays.
        constant(e, EXTRA_GET_CONTAINER_CHANGES, 1);
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0x21),
            0x21
        );
    }

    #[test]
    fn check_flags_takes_an_actors_package_flags() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        let reference = rig.form.addr();
        // The actor is a reference with a process; its slots 0x20C and 0x22C
        // are on the process object, 0x22C and 0x100 also on the actor.
        let process = object_with_slots(e, &[(0x20C, 0x0350_020C), (0x22C, 0x0350_022C)]);
        stub(e, 0x0350_020C);
        constant(e, 0x0350_022C, 0x77);
        constant(e, ACTOR_GET_PROCESS, process);
        constant(e, ACTOR_PACKAGE_FLAGS, 0x100);
        let actor = object_with_slots(e, &[(0x22C, 0x0350_0230)]);
        constant(e, 0x0350_0230, 0);
        casts(
            e,
            &[
                (reference, RTTI_REFERENCE, reference),
                (reference, RTTI_ACTOR, actor),
            ],
        );
        // The test says no and the actor's slot says no: bit 2 is cleared.
        let flags = tes_save_load_game_check_flags(e, rig.game, rig.form, 0x4);
        assert_eq!(flags, 0x100);
        // The package flags come from the package the process runs.
        assert_eq!(calls_to(e, ACTOR_PACKAGE_FLAGS), vec![vec![actor, 0x77]]);
        assert_eq!(calls_to(e, 0x0350_020C), vec![vec![process]]);
        // The test saying yes sets bit 2.
        constant(e, ACTOR_TEST_FLAGS, 1);
        // (A persistent reference is not looked up in a cell.)
        constant(e, REF_PERSISTS, 1);
        assert_eq!(
            tes_save_load_game_check_flags(e, rig.game, rig.form, 0),
            0x104
        );
    }

    #[test]
    fn check_flags_looks_up_the_cell_of_a_located_actor() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        let reference = rig.form.addr();
        // An actor-like reference (slot 0x100) with a saved world space
        // (0x294) and no cell (0x298); its location (slot 0x170) is x = 4096,
        // y = 8192.
        let object = object_with_slots(
            e,
            &[
                (0x100, 0x0360_0100),
                (0x294, 0x0360_0294),
                (0x298, 0x0360_0298),
                (0x170, 0x0360_0170),
            ],
        );
        constant(e, 0x0360_0100, 1);
        constant(e, 0x0360_0294, 0x5050);
        constant(e, 0x0360_0298, 0);
        e.register(0x0360_0170, |e, a| {
            e.mem.set_f32(a[1], 4096.0);
            e.mem.set_f32(a[1] + 4, 8192.0);
            returns(a[1])
        });
        casts(e, &[(reference, RTTI_REFERENCE, object)]);
        constant(e, REF_GET_WORLDSPACE, 0x5050);
        e.register(FLOAT_TO_INT, |_, a| {
            returns(f32::from_bits(a[0]) as i32 as u32)
        });
        // The flags have bit 1: the cell at (4096, 8192) >> 12 is looked up
        // in the reference's own world space.
        let flags = tes_save_load_game_check_flags(e, rig.game, rig.form, 0x2);
        assert_eq!(flags, 0x2);
        assert_eq!(calls_to(e, WORLDSPACE_GET_CELL), vec![vec![0x5050, 1, 2]]);
    }

    #[test]
    fn check_flags_complains_about_an_actor_without_an_editor_location() {
        let mut rig = flags_rig();
        let e = &mut rig.e;
        let reference = rig.form.addr();
        let object = object_with_slots(
            e,
            &[
                (0x100, 0x0360_0100),
                (0x294, 0x0360_0294),
                (0x298, 0x0360_0298),
                (0x170, 0x0360_0170),
            ],
        );
        constant(e, 0x0360_0100, 1);
        constant(e, 0x0360_0294, 0);
        constant(e, 0x0360_0298, 0);
        e.register(0x0360_0170, |e, a| {
            e.mem.set_f32(a[1], 4096.0);
            e.mem.set_f32(a[1] + 4, 4096.0);
            returns(a[1])
        });
        casts(e, &[(reference, RTTI_REFERENCE, object)]);
        // No cell, no world space; a parent cell that is not an interior.
        constant(e, REF_GET_PARENT_CELL, 0x6060);
        constant(e, CELL_IS_INTERIOR, 0);
        constant(e, REF_GET_WORLDSPACE, 0x5050);
        e.register(FLOAT_TO_INT, |_, a| {
            returns(f32::from_bits(a[0]) as i32 as u32)
        });
        tes_save_load_game_check_flags(e, rig.game, rig.form, 0x4);
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![vec![MSG_ACTOR_NO_EDITOR_LOCATION]]
        );
        assert_eq!(calls_to(e, WORLDSPACE_GET_CELL), vec![vec![0x5050, 1, 1]]);
    }

    #[test]
    fn the_running_package_is_the_process_slot_22c() {
        let mut e = game();
        let process = object_with_slots(&mut e, &[(0x22C, 0x0370_0000)]);
        constant(&mut e, 0x0370_0000, 0x4242);
        assert_eq!(
            base_process_get_package_that_is_running(&mut e, Ptr::new(process)),
            0x4242
        );
    }

    #[test]
    fn a_change_data_entry_without_a_required_flag_is_logged() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        quiet(&mut e, &[LOG_ERROR]);
        fn_008598d0(&mut e, game_object, 0x55);
        assert!(calls_to(&e, LOG_ERROR).is_empty());
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((0x55, data.addr()));
        fn_008598d0(&mut e, game_object, 0x55);
        assert_eq!(
            calls_to(&e, LOG_ERROR),
            vec![vec![MSG_CELL_REFERENCE_NO_FLAG]]
        );
    }

    /// The new-references world: a game with both cell maps, a cell and the
    /// list helpers that act on memory.
    struct CellRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        cell: Ptr,
        interior_map: u32,
        exterior_map: u32,
        table: Table,
    }

    fn cell_rig() -> CellRig {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        let interior_map = e.mem.alloc(0x10);
        let exterior_map = e.mem.alloc(0x10);
        e.set(
            game_object,
            TESSaveLoadGame::m_pInteriorCellMap,
            Ptr::new(interior_map),
        );
        e.set(
            game_object,
            TESSaveLoadGame::m_pExteriorCellMap,
            Ptr::new(exterior_map),
        );
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let cell = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(cell.addr() + 0xC, 0xCE11);
        quiet(&mut e, &[LOG_ERROR, LIST_REMOVE_ALL, LIST_SCALAR_DELETE]);
        measuring_for(&mut e, 1);
        // The cell is not loaded itself here.
        CellRig {
            e,
            game: game_object,
            cell,
            interior_map,
            exterior_map,
            table,
        }
    }

    #[test]
    fn an_interior_cell_loads_its_listed_references_and_forgets_the_list() {
        let mut rig = cell_rig();
        let e = &mut rig.e;
        constant(e, CELL_IS_INTERIOR, 1);
        let list = list_of(e, &[0x111, 0, 0x222]);
        rig.table.borrow_mut().push((0xCE11, list));
        assert!(fn_00859690(e, rig.game, rig.cell));
        // Each non-zero id was handled (its change data looked up).
        let lookups: Vec<u32> = calls_to(e, MAP_GET_AT)
            .iter()
            .filter(|c| c[0] != rig.interior_map)
            .map(|c| c[1])
            .collect();
        assert_eq!(lookups, vec![0x111, 0x222]);
        assert_eq!(
            calls_to(e, MAP_REMOVE_AT),
            vec![vec![rig.interior_map, 0xCE11]]
        );
        assert_eq!(calls_to(e, LIST_REMOVE_ALL), vec![vec![list]]);
        assert_eq!(calls_to(e, LIST_SCALAR_DELETE), vec![vec![list, 1]]);
    }

    #[test]
    fn an_interior_cell_without_a_list_loads_nothing() {
        let mut rig = cell_rig();
        let e = &mut rig.e;
        constant(e, CELL_IS_INTERIOR, 1);
        assert!(!fn_00859690(e, rig.game, rig.cell));
        assert!(calls_to(e, MAP_REMOVE_AT).is_empty());
    }

    #[test]
    fn new_references_do_nothing_unless_the_measuring_flag_is_set() {
        let mut rig = cell_rig();
        let e = &mut rig.e;
        e.register(SAVE_LOAD_UNAVAILABLE, |_, _| returns(0));
        assert!(!fn_00859690(e, rig.game, rig.cell));
        assert!(calls_to(e, MAP_GET_AT).is_empty());
    }

    /// An `ExteriorCellReferenceData`.
    fn exterior_item(e: &mut Engine, id: u32, x: i32, y: i32) -> u32 {
        let item = e.mem.alloc(12);
        e.mem.set_u32(item, id);
        e.mem.set_i32(item + 4, x);
        e.mem.set_i32(item + 8, y);
        item
    }

    #[test]
    fn an_exterior_cell_takes_only_the_references_at_its_coordinates() {
        let mut rig = cell_rig();
        let e = &mut rig.e;
        constant(e, CELL_IS_INTERIOR, 0);
        let worldspace = e.mem.alloc(0x40);
        e.mem.set_u32(worldspace + 0xC, 0x7777);
        constant(e, CELL_GET_WORLDSPACE, worldspace);
        constant(e, CELL_GET_X, 3);
        constant(e, CELL_GET_Y, 4);
        let (first, other, last) = (
            exterior_item(e, 0xA1, 3, 4),
            exterior_item(e, 0xB2, 9, 9),
            exterior_item(e, 0xC3, 3, 4),
        );
        let list = list_of(e, &[first, other, last]);
        rig.table.borrow_mut().push((0x7777, list));
        // `RemoveHead` copies the next node into the head; `Remove` unlinks
        // the node holding the item after the given one.
        e.register(LIST_REMOVE_HEAD, |e, a| {
            let next = e.mem.u32(a[0] + 4);
            if next != 0 {
                let (item, after) = (e.mem.u32(next), e.mem.u32(next + 4));
                e.mem.set_u32(a[0], item);
                e.mem.set_u32(a[0] + 4, after);
            }
            Ret::default()
        });
        e.register(LIST_REMOVE, |e, a| {
            let item = e.mem.u32(a[1]);
            let mut node = a[0];
            while node != 0 {
                let next = e.mem.u32(node + 4);
                if next != 0 && e.mem.u32(next) == item {
                    let after = e.mem.u32(next + 4);
                    e.mem.set_u32(node + 4, after);
                }
                node = next;
            }
            Ret::default()
        });
        e.register(LIST_NODE_IS_END, |e, a| {
            returns((e.mem.u32(a[0]) == 0 && e.mem.u32(a[0] + 4) == 0) as u32)
        });
        assert!(fn_00859690(e, rig.game, rig.cell));
        // The two matching references were handled; the other stays, so the
        // world space's entry stays too.
        let handled: Vec<u32> = calls_to(e, MAP_GET_AT)
            .iter()
            .filter(|c| c[0] != rig.exterior_map)
            .map(|c| c[1])
            .collect();
        assert_eq!(handled, vec![0xA1, 0xC3]);
        assert!(freed(e, first));
        assert!(freed(e, last));
        assert!(!freed(e, other));
        assert!(calls_to(e, MAP_REMOVE_AT).is_empty());
    }

    #[test]
    fn an_exterior_cell_with_nothing_left_removes_the_world_space_entry() {
        let mut rig = cell_rig();
        let e = &mut rig.e;
        constant(e, CELL_IS_INTERIOR, 0);
        let worldspace = e.mem.alloc(0x40);
        e.mem.set_u32(worldspace + 0xC, 0x7777);
        constant(e, CELL_GET_WORLDSPACE, worldspace);
        constant(e, CELL_GET_X, 3);
        constant(e, CELL_GET_Y, 4);
        let only = exterior_item(e, 0xA1, 3, 4);
        let list = list_of(e, &[only]);
        rig.table.borrow_mut().push((0x7777, list));
        e.register(LIST_REMOVE_HEAD, |e, a| {
            e.mem.set_u32(a[0], 0);
            Ret::default()
        });
        e.register(LIST_NODE_IS_END, |e, a| {
            returns((e.mem.u32(a[0]) == 0 && e.mem.u32(a[0] + 4) == 0) as u32)
        });
        assert!(fn_00859690(e, rig.game, rig.cell));
        assert_eq!(
            calls_to(e, MAP_REMOVE_AT),
            vec![vec![rig.exterior_map, 0x7777]]
        );
        assert_eq!(calls_to(e, LIST_SCALAR_DELETE), vec![vec![list, 1]]);
    }

    /// The world of `fn_00859a90`: forms are found in `forms`, the types
    /// of base objects in `types`, and each constructor makes an object
    /// whose slot `0x128` is recorded in `ids`.
    struct CreateRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        /// The objects the constructors made: (constructor address, block).
        made: Rc<RefCell<Vec<(u32, u32, u32)>>>,
        /// The slot `0x128` calls: (object, id, flag).
        ids: Rc<RefCell<Vec<(u32, u32, u32)>>>,
    }

    fn create_rig(forms: &[(u32, u32)]) -> CreateRig {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let forms = forms.to_vec();
        e.register_double(LOOKUP_FORM, move |_, a| {
            returns(forms.iter().find(|f| f.0 == a[0]).map_or(0, |f| f.1))
        });
        casts(&mut e, &[]);
        quiet(&mut e, &[LOG_ERROR, REF_SET_BASE, FORM_FINISH, FN_00483C70]);
        let ids: Rc<RefCell<Vec<(u32, u32, u32)>>> = Rc::new(RefCell::new(vec![]));
        let sink = ids.clone();
        e.register_double(0x0380_0128, move |_, a| {
            sink.borrow_mut().push((a[0], a[1], a[2]));
            Ret::default()
        });
        let made: Rc<RefCell<Vec<(u32, u32, u32)>>> = Rc::new(RefCell::new(vec![]));
        let vtable = e.mem.alloc(0x400);
        e.mem.set_u32(vtable + 0x128, 0x0380_0128);
        for constructor in [
            CHARACTER_CONSTRUCT,
            CREATURE_CONSTRUCT,
            REFERENCE_CONSTRUCT,
            ARROW_PROJECTILE_CONSTRUCT,
            MAGIC_PROJECTILE_CONSTRUCT_A,
            MAGIC_PROJECTILE_CONSTRUCT_B,
            MAGIC_PROJECTILE_CONSTRUCT_C,
        ] {
            let sink = made.clone();
            e.register_double(constructor, move |e, a| {
                let size = e.mem.block_size(a[0]).unwrap();
                sink.borrow_mut().push((constructor, a[0], size));
                e.mem.set_u32(a[0], vtable);
                returns(a[0])
            });
        }
        CreateRig {
            e,
            game: game_object,
            made,
            ids,
        }
    }

    /// A `CreatedReferenceData` of the given type and bound id with the
    /// location 0x77.
    fn created_reference(e: &mut Engine, kind: u32, bound_id: u32) -> Ptr<CreatedReferenceData> {
        let record: Ptr<CreatedReferenceData> = e.new_object();
        e.set(record, CreatedReferenceData::eType, kind);
        e.set(record, CreatedReferenceData::iBoundID, bound_id);
        e.mem.set_u32(record.addr() + 8, 0x77);
        record
    }

    #[test]
    fn a_created_reference_whose_bound_object_is_gone_is_not_made() {
        let mut rig = create_rig(&[]);
        let e = &mut rig.e;
        let record = created_reference(e, 0, 0x2222);
        let form = fn_00859a90(e, rig.game, 0x1111, record);
        assert!(form.is_null());
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![vec![MSG_BOUND_OBJECT_MISSING, 0x2222, 0x1111]]
        );
        assert_eq!(calls_to(e, SCOPE_LEAVE).len(), 1);
        assert!(rig.made.borrow().is_empty());
    }

    #[test]
    fn a_normal_created_reference_is_built_by_the_base_objects_type() {
        for (base_type, constructor, size) in [
            (0x2Au32, CHARACTER_CONSTRUCT, 0x1C8u32),
            (0x2B, CREATURE_CONSTRUCT, 0x1C0),
            (0x33, REFERENCE_CONSTRUCT, 0x68),
        ] {
            let mut rig = create_rig(&[(0x2222, 0xB0B0)]);
            let e = &mut rig.e;
            casts(e, &[(0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0)]);
            constant(e, FORM_TYPE, base_type);
            let record = created_reference(e, 0, 0x2222);
            let form = fn_00859a90(e, rig.game, 0x1111, record);
            let made = rig.made.borrow().clone();
            assert_eq!(made.len(), 1);
            assert_eq!(made[0].0, constructor);
            assert_eq!(calls_to(e, OPERATOR_NEW), vec![vec![size]]);
            assert_eq!(form.addr(), made[0].1);
            // It gets the form id (with 1) and is finished.
            assert_eq!(*rig.ids.borrow(), vec![(form.addr(), 0x1111, 1)]);
            assert_eq!(calls_to(e, FORM_FINISH), vec![vec![form.addr(), 1]]);
            // The reference (a cast of the form; null here) gets the base.
            assert_eq!(calls_to(e, REF_SET_BASE), vec![vec![0, 0xB0B0]]);
        }
    }

    #[test]
    fn an_existing_reference_with_the_same_base_object_is_kept() {
        let mut rig = create_rig(&[(0x1111, 0xF0F0), (0x2222, 0xB0B0)]);
        let e = &mut rig.e;
        casts(
            e,
            &[
                (0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0),
                (0xF0F0, RTTI_REFERENCE, 0xF0F0),
            ],
        );
        constant(e, REFERENCE_GET_BASE, 0xB0B0);
        let record = created_reference(e, 3, 0x2222);
        let form = fn_00859a90(e, rig.game, 0x1111, record);
        assert_eq!(form.addr(), 0xF0F0);
        assert!(rig.made.borrow().is_empty());
        assert_eq!(calls_to(e, FORM_FINISH), vec![vec![0xF0F0, 1]]);
        assert!(calls_to(e, LOG_ERROR).is_empty());
    }

    #[test]
    fn an_existing_reference_with_another_base_object_is_deleted_and_made_again() {
        let old = 0xF0F0;
        let mut rig = create_rig(&[(0x1111, old), (0x2222, 0xB0B0)]);
        let e = &mut rig.e;
        casts(
            e,
            &[
                (0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0),
                (old, RTTI_REFERENCE, old),
            ],
        );
        constant(e, REFERENCE_GET_BASE, 0xC1C1);
        constant(e, FORM_TYPE, 0x33);
        // `DeleteForm` sets the old form's id: it needs an object.
        let old_form = object_with_slots(e, &[(0x128, 0x0380_0128), (0x134, 0x0380_0134)]);
        let forms = [(0x1111u32, old_form), (0x2222, 0xB0B0)];
        e.register_double(LOOKUP_FORM, move |_, a| {
            returns(forms.iter().find(|f| f.0 == a[0]).map_or(0, |f| f.1))
        });
        casts(
            e,
            &[
                (0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0),
                (old_form, RTTI_REFERENCE, old_form),
            ],
        );
        stub(e, 0x0380_0134);
        stub(e, LIST_CONTAINS);
        stub(e, LIST_ADD_HEAD);
        constant(e, FORM_IS_DELETED, 0);
        let record = created_reference(e, 0, 0x2222);
        let form = fn_00859a90(e, rig.game, 0x1111, record);
        // The old form was deleted: it is given id 0 and a new form is made.
        assert_eq!(rig.ids.borrow()[0], (old_form, 0, 1));
        assert_eq!(rig.made.borrow().len(), 1);
        assert_eq!(form.addr(), rig.made.borrow()[0].1);
    }

    #[test]
    fn projectile_records_pick_their_constructors() {
        for (kind, bound_id, constructor, size) in [
            (1u32, 0x2222u32, ARROW_PROJECTILE_CONSTRUCT, 0xC8u32),
            (2, 0, MAGIC_PROJECTILE_CONSTRUCT_A, 0xC4),
            (2, 3, MAGIC_PROJECTILE_CONSTRUCT_B, 0xD0),
            (2, 1, MAGIC_PROJECTILE_CONSTRUCT_C, 0xD8),
        ] {
            let mut rig = create_rig(&[(0x2222, 0xB0B0)]);
            let e = &mut rig.e;
            casts(e, &[(0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0)]);
            let record = created_reference(e, kind, bound_id);
            let form = fn_00859a90(e, rig.game, 0x1111, record);
            let made = rig.made.borrow().clone();
            assert_eq!(made.len(), 1, "kind {kind} bound {bound_id}");
            assert_eq!(made[0].0, constructor);
            assert_eq!(calls_to(e, OPERATOR_NEW), vec![vec![size]]);
            assert_eq!(form.addr(), made[0].1);
        }
    }

    #[test]
    fn an_unknown_created_reference_type_is_logged() {
        let mut rig = create_rig(&[(0x2222, 0xB0B0)]);
        let e = &mut rig.e;
        casts(e, &[(0xB0B0, RTTI_BOUND_OBJECT, 0xB0B0)]);
        let record = created_reference(e, 7, 0x2222);
        let form = fn_00859a90(e, rig.game, 0x1111, record);
        assert!(form.is_null());
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![vec![MSG_INVALID_CREATED_TYPE, 7, 0x1111, 0x2222, 0x77]]
        );
        assert!(calls_to(e, FORM_FINISH).is_empty());
    }

    /// The world of `fn_00859f20`: a location (a cell or a world space)
    /// with two plugin files, of which the second has the reference.
    struct MovedRig {
        e: Engine,
        moved: Ptr<MovedReferenceData>,
        reference: u32,
        loaded: Rc<RefCell<Vec<(u32, u32)>>>,
    }

    fn moved_rig() -> MovedRig {
        let mut e = game();
        // The location form 0x5000 is looked up by the record's location.
        let moved: Ptr<MovedReferenceData> = e.new_object();
        e.set(moved, MovedReferenceData::iOriginalLocationID, 0x5000);
        e.mem.set_f32(moved.addr() + 4, 8192.0);
        e.mem.set_f32(moved.addr() + 8, 4096.0);
        // The reference the file makes: slots 0x88 (post create), 0x1F4
        // position.
        let position = e.mem.alloc(16);
        for (i, v) in [1.0f32, 2.0, 3.0].iter().enumerate() {
            e.mem.set_f32(position + 4 * i as u32, *v);
        }
        let reference = object_with_slots(&mut e, &[(0x88, 0x0390_0088), (0x1F4, 0x0390_01F4)]);
        stub(&mut e, 0x0390_0088);
        constant(&mut e, 0x0390_01F4, position);
        let rotation = e.mem.alloc(16);
        for (i, v) in [4.0f32, 5.0, 6.0].iter().enumerate() {
            e.mem.set_f32(rotation + 4 * i as u32, *v);
        }
        constant(&mut e, REF_GET_ROTATION, rotation);
        constant(&mut e, REF_GET_EXTRA_LIST, 0x7000);
        quiet(
            &mut e,
            &[
                LOG_ERROR,
                EXTRA_SET_STARTING_POSITION,
                EXTRA_SET_STARTING_ROTATION,
                ACTOR_PLACE,
            ],
        );
        // Entry i of the cell or world space is the file i + 1 times 10.
        e.register(FILE_OF_CELL, |_, a| returns(a[1] + 1));
        e.register(THREAD_SAFE_FILE, |_, a| returns(a[0] * 10));
        e.register(FILE_HAS_FORM, |_, a| returns((a[0] == 20) as u32));
        constant(&mut e, FILE_RECORD_TYPE, 0x33);
        constant(&mut e, CREATE_REFERENCE, reference);
        let loaded: Rc<RefCell<Vec<(u32, u32)>>> = Rc::new(RefCell::new(vec![]));
        let sink = loaded.clone();
        e.register_double(LOAD_FORM_FROM_FILE, move |_, a| {
            sink.borrow_mut().push((a[0], a[1]));
            Ret::default()
        });
        constant(&mut e, CELL_FILE_COUNT, 2);
        e.register(FLOAT_TO_INT, |_, a| {
            returns(f32::from_bits(a[0]) as i32 as u32)
        });
        MovedRig {
            e,
            moved,
            reference,
            loaded,
        }
    }

    #[test]
    fn a_moved_reference_is_loaded_from_the_cells_plugin_that_has_it() {
        let mut rig = moved_rig();
        let e = &mut rig.e;
        constant(e, LOOKUP_FORM, 0x5000);
        casts(e, &[(0x5000, RTTI_CELL, 0x5000)]);
        // Both files have the cell; only file 20 has the reference.
        constant(e, FILE_HAS_CELL, 1);
        let result = fn_00859f20(e, Ptr::NULL, 0x1234, rig.moved);
        assert_eq!(result.addr(), rig.reference);
        assert_eq!(*rig.loaded.borrow(), vec![(rig.reference, 20)]);
        assert_eq!(calls_to(e, CREATE_REFERENCE), vec![vec![0x33, 1]]);
        // Not an actor: the starting position and rotation are set on the
        // extra data from the reference's own.
        let position = calls_to(e, EXTRA_SET_STARTING_POSITION);
        assert_eq!(position.len(), 1);
        assert_eq!(position[0][0], 0x7000);
        assert_eq!(position[0][2], rig.reference);
        assert_eq!(
            position[0][3..],
            [1.0f32.to_bits(), 2.0f32.to_bits(), 3.0f32.to_bits()]
        );
        let rotation = calls_to(e, EXTRA_SET_STARTING_ROTATION);
        assert_eq!(
            rotation[0][3..],
            [4.0f32.to_bits(), 5.0f32.to_bits(), 6.0f32.to_bits()]
        );
        assert!(calls_to(e, ACTOR_PLACE).is_empty());
    }

    #[test]
    fn a_moved_actor_is_placed_in_the_world_space_cell() {
        let mut rig = moved_rig();
        let e = &mut rig.e;
        constant(e, LOOKUP_FORM, 0x5000);
        let reference = rig.reference;
        casts(
            e,
            &[
                (0x5000, RTTI_WORLDSPACE, 0x5000),
                (reference, RTTI_ACTOR, 0xAC70),
            ],
        );
        // The cell (8192 / 4096 grid units: 2, 1) is in file 20 only.
        e.register(WORLDSPACE_FIND_CELL_IN_FILE, |_, a| {
            returns((a[1] == 20 && a[2] == 2 && a[3] == 1) as u32)
        });
        let result = fn_00859f20(e, Ptr::NULL, 0x1234, rig.moved);
        assert_eq!(result.addr(), reference);
        // The actor is placed with the world space, no cell, the position and
        // the z of the rotation.
        let placed = calls_to(e, ACTOR_PLACE);
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0][..3], [0xAC70, 0x5000, 0]);
        assert_eq!(placed[0][4], 6.0f32.to_bits());
        assert!(calls_to(e, EXTRA_SET_STARTING_POSITION).is_empty());
    }

    #[test]
    fn a_moved_reference_with_no_location_or_no_plugin_is_logged() {
        let mut rig = moved_rig();
        let e = &mut rig.e;
        // The location form is neither a cell nor a world space.
        constant(e, LOOKUP_FORM, 0);
        let result = fn_00859f20(e, Ptr::NULL, 0x1234, rig.moved);
        assert!(result.is_null());
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![
                vec![MSG_NO_CELL_OR_WORLDSPACE],
                vec![MSG_REFERENCE_NOT_LOADED, 0x1234, 0x5000, 0, 0]
            ]
        );
        // A cell no plugin has the reference for.
        constant(e, LOOKUP_FORM, 0x5000);
        casts(e, &[(0x5000, RTTI_CELL, 0x5000)]);
        constant(e, FILE_HAS_CELL, 0);
        assert!(fn_00859f20(e, Ptr::NULL, 0x1234, rig.moved).is_null());
        assert!(rig.loaded.borrow().is_empty());
    }

    #[test]
    fn a_moved_record_without_an_original_location_uses_the_reference_data() {
        let mut rig = moved_rig();
        let e = &mut rig.e;
        e.set(rig.moved, MovedReferenceData::iOriginalLocationID, 0);
        e.mem.set_u32(rig.moved.addr() + 0x10, 0x6000);
        let looked_up = Rc::new(RefCell::new(vec![]));
        let sink = looked_up.clone();
        e.register_double(LOOKUP_FORM, move |_, a| {
            sink.borrow_mut().push(a[0]);
            Ret::default()
        });
        fn_00859f20(e, Ptr::NULL, 0x1234, rig.moved);
        assert_eq!(*looked_up.borrow(), vec![0x6000]);
    }

    #[test]
    fn making_a_form_removes_what_is_saved_under_its_id() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let data = change_data(&mut e, 1, 0);
        table.borrow_mut().push((0x1234, data.addr()));
        constant(&mut e, LOOKUP_FORM, 0);
        let form = object_with_slots(&mut e, &[(0x128, 0x03A0_0128)]);
        constant(&mut e, CREATE_FORM_OF_TYPE, form);
        stub(&mut e, 0x03A0_0128);
        let result = fn_0085a240(&mut e, game_object, 0x1234, 0x2A);
        assert_eq!(result.addr(), form);
        // The type byte is passed, the id (with 1) is assigned, and the
        // change data of the id was dropped (force 1).
        assert_eq!(calls_to(&e, CREATE_FORM_OF_TYPE), vec![vec![0x2A]]);
        assert_eq!(calls_to(&e, 0x03A0_0128), vec![vec![form, 0x1234, 1]]);
        assert!(table.borrow().is_empty());
    }

    #[test]
    fn removing_an_id_deletes_an_existing_form_instead() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        let table = install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let form = object_with_slots(&mut e, &[(0x128, 0x03A0_0128), (0x134, 0x03A0_0134)]);
        e.mem.set_u32(form + 0xC, 0x1234);
        constant(&mut e, LOOKUP_FORM, form);
        stub(&mut e, 0x03A0_0128);
        stub(&mut e, 0x03A0_0134);
        quiet(
            &mut e,
            &[LOG_ERROR, FN_00483C70, LIST_CONTAINS, LIST_ADD_HEAD],
        );
        casts(&mut e, &[]);
        constant(&mut e, FORM_IS_DELETED, 0);
        fn_0085a290(&mut e, game_object, 0x1234);
        // The form was reset: id 0 and an empty editor id.
        assert_eq!(calls_to(&e, 0x03A0_0128), vec![vec![form, 0, 1]]);
        assert_eq!(calls_to(&e, 0x03A0_0134), vec![vec![form, EMPTY_STRING]]);
        assert!(table.borrow().is_empty());
    }

    #[test]
    fn deleting_a_form_resets_it_and_defers_its_deletion() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let form = object_with_slots(&mut e, &[(0x128, 0x03A0_0128), (0x134, 0x03A0_0134)]);
        e.mem.set_u32(form + 0xC, 0x1234);
        stub(&mut e, 0x03A0_0128);
        stub(&mut e, 0x03A0_0134);
        quiet(
            &mut e,
            &[LOG_ERROR, FN_00483C70, LIST_CONTAINS, LIST_ADD_HEAD],
        );
        casts(&mut e, &[]);
        constant(&mut e, FORM_IS_DELETED, 0);
        tes_save_load_game_delete_form(&mut e, game_object, Ptr::new(form));
        // Not loading: the error is logged.
        assert_eq!(
            calls_to(&e, LOG_ERROR),
            vec![vec![MSG_DELETE_FORM_NOT_LOADING]]
        );
        assert_eq!(calls_to(&e, 0x03A0_0128), vec![vec![form, 0, 1]]);
        assert_eq!(calls_to(&e, FN_00483C70), vec![vec![form]]);
        assert_eq!(calls_to(&e, 0x03A0_0134), vec![vec![form, EMPTY_STRING]]);
        // It was put on the deferred list (at +0x34), not found there before.
        assert_eq!(calls_to(&e, LIST_CONTAINS)[0][0], game_object.addr() + 0x34);
        let added = calls_to(&e, LIST_ADD_HEAD);
        assert_eq!(added.len(), 1);
        assert_eq!(added[0][0], game_object.addr() + 0x34);
    }

    #[test]
    fn deleting_an_already_deleted_form_or_a_cell_gives_it_a_new_id() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let changes: Ptr<ChangesMap> = e.new_object();
        e.set(game_object, TESSaveLoadGame::m_pChanges, changes);
        install_table(&mut e, MAP_GET_AT, None, MAP_REMOVE_AT);
        let form = object_with_slots(&mut e, &[(0x128, 0x03A0_0128)]);
        e.mem.set_u32(form + 0xC, 0x1234);
        stub(&mut e, 0x03A0_0128);
        quiet(&mut e, &[LOG_ERROR]);
        map_globals(&mut e);
        e.set_global(DATA_HANDLER, 0x0055_0000u32);
        constant(&mut e, DATA_HANDLER_GET_NEXT_ID, 0xFF00_0001);
        casts(&mut e, &[]);
        // Measuring: no error. A deleted form:
        e.register(SAVE_LOAD_UNAVAILABLE, |_, _| returns(1));
        constant(&mut e, FORM_IS_DELETED, 1);
        tes_save_load_game_delete_form(&mut e, game_object, Ptr::new(form));
        assert!(calls_to(&e, LOG_ERROR).is_empty());
        assert_eq!(calls_to(&e, 0x03A0_0128), vec![vec![form, 0xFF00_0001, 1]]);
        // A cell too.
        constant(&mut e, FORM_IS_DELETED, 0);
        casts(&mut e, &[(form, RTTI_CELL, form)]);
        tes_save_load_game_delete_form(&mut e, game_object, Ptr::new(form));
        assert_eq!(calls_to(&e, 0x03A0_0128).len(), 2);
        assert!(calls_to(&e, LIST_ADD_HEAD).is_empty());
    }

    #[test]
    fn the_deferred_deletion_list_gets_a_form_once() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let contained: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let state = contained.clone();
        e.register_double(LIST_CONTAINS, move |e, a| {
            assert_eq!(e.mem.u32(a[1]), 0xF0F0);
            returns(*state.borrow() as u32)
        });
        stub(&mut e, LIST_ADD_HEAD);
        stub(&mut e, LIST_REMOVE);
        tes_save_load_game_add_form_to_deferred_deletions_list(
            &mut e,
            game_object,
            Ptr::new(0xF0F0),
        );
        assert_eq!(calls_to(&e, LIST_ADD_HEAD).len(), 1);
        *contained.borrow_mut() = true;
        tes_save_load_game_add_form_to_deferred_deletions_list(
            &mut e,
            game_object,
            Ptr::new(0xF0F0),
        );
        assert_eq!(calls_to(&e, LIST_ADD_HEAD).len(), 1);
        // Removing takes it off only when it is there.
        fn_0085a410(&mut e, game_object, Ptr::new(0xF0F0));
        assert_eq!(calls_to(&e, LIST_REMOVE).len(), 1);
        *contained.borrow_mut() = false;
        fn_0085a410(&mut e, game_object, Ptr::new(0xF0F0));
        assert_eq!(calls_to(&e, LIST_REMOVE).len(), 1);
        assert_eq!(calls_to(&e, LIST_REMOVE)[0][0], game_object.addr() + 0x34);
    }

    #[test]
    fn the_initial_data_size_is_a_reference_data_for_references_with_location_flags() {
        let mut e = game();
        let game_object: Ptr<TESSaveLoadGame> = e.new_object();
        let form = Ptr::new(e.mem.alloc(0x40));
        // Not a reference and not a cell: nothing.
        casts(&mut e, &[]);
        assert_eq!(
            tes_save_load_game_get_initial_data_save_size(&mut e, game_object, form, 6),
            0
        );
        // A cell: nothing either.
        casts(&mut e, &[(form.addr(), RTTI_CELL, form.addr())]);
        assert_eq!(
            tes_save_load_game_get_initial_data_save_size(&mut e, game_object, form, 6),
            0
        );
        // A reference: 0x1C with bit 1 or 2, else nothing.
        casts(&mut e, &[(form.addr(), RTTI_REFERENCE, form.addr())]);
        for (flags, size) in [(2u32, 0x1Cu16), (4, 0x1C), (6, 0x1C), (1, 0), (0x38, 0)] {
            assert_eq!(
                tes_save_load_game_get_initial_data_save_size(&mut e, game_object, form, flags),
                size,
                "flags {flags:#x}"
            );
        }
    }

    /// The world of `SaveInitialData`: a reference with a position and a
    /// rotation, the singleton with a buffer.
    struct InitialRig {
        e: Engine,
        game: Ptr<TESSaveLoadGame>,
        form: Ptr,
        buffer: u32,
    }

    fn initial_rig() -> InitialRig {
        let mut e = game();
        let game_object = game_singleton(&mut e);
        let buffer = e.mem.alloc(0x40);
        e.set(game_object, TESSaveLoadGame::m_pBuffer, Ptr::new(buffer));
        let form = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(form.addr() + 0xC, 0xF00D);
        casts(&mut e, &[(form.addr(), RTTI_REFERENCE, form.addr())]);
        let position = e.mem.alloc(16);
        let rotation = e.mem.alloc(16);
        for i in 0..3 {
            e.mem.set_f32(position + 4 * i, 1.0 + i as f32);
            e.mem.set_f32(rotation + 4 * i, 10.0 + i as f32);
        }
        constant(&mut e, REF_GET_POSITION, position);
        constant(&mut e, REF_GET_ROTATION, rotation);
        e.register(ADD_NUMERIC_ID, |_, a| returns(a[1] + 0x100));
        quiet(
            &mut e,
            &[
                LOG_ERROR,
                REF_GET_PARENT_CELL,
                CELL_GET_WORLDSPACE,
                REF_PERSISTS,
                REF_GET_EXTRA_LIST,
                EXTRA_GET_CELL_DATA,
                ACTOR_GET_PROCESS,
                READ_FIELD_28,
            ],
        );
        InitialRig {
            e,
            game: game_object,
            form,
            buffer,
        }
    }

    #[test]
    fn initial_data_writes_the_location_position_and_angle() {
        let mut rig = initial_rig();
        let e = &mut rig.e;
        // The parent cell is in a world space: its id gives the numeric id.
        let cell = e.mem.alloc(0x40);
        let worldspace = e.mem.alloc(0x40);
        e.mem.set_u32(worldspace + 0xC, 0x3A);
        constant(e, REF_GET_PARENT_CELL, cell);
        constant(e, CELL_GET_WORLDSPACE, worldspace);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        assert_eq!(e.mem.u32(rig.buffer), 0x13A);
        assert_eq!(e.mem.f32(rig.buffer + 4), 1.0);
        assert_eq!(e.mem.f32(rig.buffer + 0xC), 3.0);
        assert_eq!(e.mem.f32(rig.buffer + 0x10), 10.0);
        assert_eq!(e.mem.f32(rig.buffer + 0x18), 12.0);
        // 0x1C bytes were written.
        assert_eq!(
            e.get(rig.game, TESSaveLoadGame::m_pBuffer).addr(),
            rig.buffer + 0x1C
        );
        // Without bits 1 and 2 nothing is written (the calls are still made).
        e.set(rig.game, TESSaveLoadGame::m_pBuffer, Ptr::new(rig.buffer));
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 0x38);
        assert_eq!(
            e.get(rig.game, TESSaveLoadGame::m_pBuffer).addr(),
            rig.buffer
        );
        assert!(calls_to(e, LOG_ERROR).is_empty());
    }

    #[test]
    fn initial_data_uses_the_parent_cell_when_it_has_no_world_space() {
        let mut rig = initial_rig();
        let e = &mut rig.e;
        let cell = e.mem.alloc(0x40);
        e.mem.set_u32(cell + 0xC, 0x4B);
        constant(e, REF_GET_PARENT_CELL, cell);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 4);
        assert_eq!(e.mem.u32(rig.buffer), 0x14B);
    }

    #[test]
    fn initial_data_logs_a_reference_that_is_in_no_cell() {
        let mut rig = initial_rig();
        let e = &mut rig.e;
        // Not persistent: one message; persistent without the extra data:
        // the other.
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![vec![MSG_NON_PERSISTENT_NO_CELL, 0xF00D]]
        );
        constant(e, REF_PERSISTS, 1);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        assert_eq!(
            calls_to(e, LOG_ERROR)[1..],
            [vec![MSG_PERSISTENT_NO_CELL, 0xF00D]]
        );
        // The location id stays 0.
        assert_eq!(e.mem.u32(rig.buffer), 0);
    }

    #[test]
    fn initial_data_logs_a_cell_less_actor_by_its_process_level() {
        let mut rig = initial_rig();
        let e = &mut rig.e;
        constant(e, REF_PERSISTS, 1);
        constant(e, EXTRA_GET_CELL_DATA, 1);
        let reference = rig.form.addr();
        casts(
            e,
            &[
                (reference, RTTI_REFERENCE, reference),
                (reference, RTTI_MOBILE_OBJECT, 0xAC70),
            ],
        );
        constant(e, ACTOR_GET_PROCESS, 0x9000);
        // Level 0 is a high process, 1 a middle high process; others are fine.
        constant(e, READ_FIELD_28, 0);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        constant(e, READ_FIELD_28, 1);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        constant(e, READ_FIELD_28, 2);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 2);
        assert_eq!(
            calls_to(e, LOG_ERROR),
            vec![
                vec![MSG_HIGH_PROCESS_NO_CELL, 0xF00D],
                vec![MSG_MIDDLE_HIGH_PROCESS_NO_CELL, 0xF00D]
            ]
        );
    }

    #[test]
    fn initial_data_for_a_cell_or_another_form_writes_nothing() {
        let mut rig = initial_rig();
        let e = &mut rig.e;
        let form = rig.form.addr();
        casts(e, &[(form, RTTI_CELL, form)]);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 6);
        casts(e, &[]);
        tes_save_load_game_save_initial_data(e, rig.game, rig.form, 6);
        assert_eq!(
            e.get(rig.game, TESSaveLoadGame::m_pBuffer).addr(),
            rig.buffer
        );
    }

    #[test]
    fn every_function_is_registered_once() {
        let list = funcs();
        assert_eq!(list.len(), 80);
        let mut addresses: Vec<u32> = list.iter().map(|(a, _)| *a).collect();
        addresses.sort_unstable();
        addresses.dedup();
        assert_eq!(addresses.len(), 80);
    }
}
