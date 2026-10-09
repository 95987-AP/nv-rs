//! `fallout shared/extradatalist.cpp` (Xbox PDB source unit), part 4: its functions from `00421400` up to
//! (not including) `0042dd90` in FalloutNV.exe 1.4.0.525, translated
//! (docs/ENGINE_CRATE.md). The unit's shared layouts and helpers are in
//! [`super::extradatalist`]; anything public there may be used here.
//!
//! Translated so far, in address order: the 40 functions from `00421400` to
//! `00422640`. They are the getters and setters of single extra data (the
//! merchant container, the leveled creature modifier and original base, the
//! cell detach time, the seen data, the north rotation, the X target, the
//! encounter zone, the emittance source, the multibound reference, data and
//! volume, the occlusion plane, the radius and radiation floats, the follower
//! list and the friend hits). The next function to translate is `00422670`.
//!
//! Every setter follows the compiler's one pattern: a null (or neutral)
//! value removes the extra data of its type; otherwise the existing extra
//! data is updated, or a new one is built (`operator new`, the subclass
//! constructor, the value stored at +0x0C) and added. The compiler's
//! exception-unwinding frames around the constructors are not translated.
//! The `float` setters and getters go through the x87 stack (`FLD`/`FSTP`),
//! so a signalling NaN comes out quiet.

#[allow(unused_imports)]
use super::extradatalist::*;
#[allow(unused_imports)]
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Callees outside this file, by address

/// `BaseExtraList::GetExtraData(type)` (Xbox PDB).
const GET_EXTRA_DATA: u32 = 0x0041_0220;
/// `BaseExtraList::AddExtra(extra)` (Xbox PDB).
const ADD_EXTRA: u32 = 0x0040_ff60;
/// `BaseExtraList::RemoveExtra(type)` (Xbox PDB, `_ov2`).
const REMOVE_EXTRA: u32 = 0x0041_0140;
/// `BaseExtraList::HasExtra(type)` (Xbox PDB).
const HAS_EXTRA: u32 = 0x0040_fe80;
/// `BSExtraData::BSExtraData(type)`: sets the base vtable, the type and a
/// null next.
const BS_EXTRA_DATA_INIT: u32 = 0x0040_ec80;
/// `operator new(size)`.
const OPERATOR_NEW: u32 = 0x0040_1000;
/// `operator delete(block)`.
const OPERATOR_DELETE: u32 = 0x0040_1030;
/// `MOV EAX,[ECX]`: reads the word a handle points at.
const READ_WORD: u32 = 0x0055_9450;
/// Assigns the handle at `this` (the word at +0x0C of the extra data) the
/// value on the stack: when it differs, one reference is released and one
/// taken (`00401970`, `0040f6e0`).
const ASSIGN_HANDLE: u32 = 0x0066_b0d0;

/// Constructors of the extra data (`this` in ECX, the new block; they return
/// it). Each is in a unit of its own and builds a 0x10-byte object unless
/// the size is given with the call.
const MERCHANT_CONTAINER_INIT: u32 = 0x0043_4fc0;
const LEV_CREA_MOD_INIT: u32 = 0x0043_5040;
/// Takes the byte to store (a stack word).
const NO_RUMORS_INIT: u32 = 0x0043_6090;
/// 0x14 bytes.
const LEVELED_CREATURE_INIT: u32 = 0x0043_11f0;
const SEEN_DATA_INIT: u32 = 0x0040_f590;
const NORTH_ROTATION_INIT: u32 = 0x0040_f680;
const DETACH_TIME_INIT: u32 = 0x0040_f6b0;
const X_TARGET_INIT: u32 = 0x0043_5370;
const ENCOUNTER_ZONE_INIT: u32 = 0x0043_2e60;
const EMITTANCE_SOURCE_INIT: u32 = 0x0043_5440;
const MULTIBOUND_REF_INIT: u32 = 0x0043_5510;
const MULTIBOUND_DATA_INIT: u32 = 0x0043_5590;
const MULTIBOUND_INIT: u32 = 0x0043_5690;
const OCCLUSION_PLANE_INIT: u32 = 0x0043_57a0;
const FOLLOWER_INIT: u32 = 0x0043_0dd0;
/// 0x1C bytes.
const FRIEND_HITS_INIT: u32 = 0x0043_5c40;

/// `float` of the type `0x1E` extra data, computed by the extra data itself
/// (ST0), and the word it also gives (EAX); `this` = the extra data.
const LEV_CREA_MOD_GET_FLOAT: u32 = 0x0043_50c0;
const LEV_CREA_MOD_GET_WORD: u32 = 0x0043_5100;
/// `ExtraFriendHits::AddHit` and `ExtraFriendHits::GetHitCount` (Xbox PDB).
const FRIEND_HITS_ADD_HIT: u32 = 0x0043_5d40;
const FRIEND_HITS_GET_HIT_COUNT: u32 = 0x0043_5e20;
/// A linked-list membership test: `this` is a list node, the stack word the
/// address of an item; true when a node's item (the first word) equals the
/// word there.
const LIST_CONTAINS: u32 = 0x005f_65d0;
/// Adds the item at the address on the stack at the head of the list at
/// `this` (`BSSimpleList::AddHead`).
const LIST_ADD_HEAD: u32 = 0x005a_e3d0;

// ---------------------------------------------------------------------------
// Data

/// The `double` `0.0` the float setters compare with.
const ZERO_DOUBLE: u32 = 0x0101_2060;
/// The player character singleton pointer (`011dea3c`).
const PLAYER_SINGLETON: u32 = 0x011d_ea3c;

/// Vtables set by the constructors of this file.
const VTABLE_EXTRA_RADIUS: u32 = 0x0101_5208;
const VTABLE_EXTRA_RADIATION: u32 = 0x0101_5214;

/// Extra data types, `EXTRA_DATA_TYPE` of the Xbox PDB.
const EXTRA_SEEN_DATA: u8 = 0x05;
const EXTRA_CELL_DETACH_TIME: u8 = 0x0b;
const EXTRA_FOLLOWER: u8 = 0x1d;
const EXTRA_LEV_CREA_MOD: u8 = 0x1e;
const EXTRA_LEVELED_CREATURE: u8 = 0x2e;
const EXTRA_MERCHANT_CONTAINER: u8 = 0x3c;
const EXTRA_NORTH_ROTATION: u8 = 0x43;
const EXTRA_X_TARGET: u8 = 0x44;
const EXTRA_FRIEND_HITS: u8 = 0x45;
const EXTRA_NO_RUMORS: u8 = 0x4e;
const EXTRA_RADIUS: u8 = 0x5c;
const EXTRA_RADIATION: u8 = 0x5d;
const EXTRA_MULTIBOUND: u8 = 0x61;
const EXTRA_MULTIBOUND_DATA: u8 = 0x62;
const EXTRA_MULTIBOUND_REF: u8 = 0x63;
const EXTRA_EMITTANCE_SOURCE: u8 = 0x67;
const EXTRA_OCCLUSION_PLANE: u8 = 0x71;
const EXTRA_ENCOUNTER_ZONE: u8 = 0x74;

// ---------------------------------------------------------------------------
// Helpers

/// `GetExtraData`: the first extra data of `extra_type`, or null.
fn find_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> Ptr<BSExtraData> {
    e.call(GET_EXTRA_DATA, &args![list, extra_type as u32])
        .ptr()
}

/// `AddExtra`.
fn add_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra: Ptr<BSExtraData>) {
    e.call(ADD_EXTRA, &args![list, extra]);
}

/// `RemoveExtra(type)`.
fn remove_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) {
    e.call(REMOVE_EXTRA, &args![list, extra_type as u32]);
}

/// The word at +0x0C of the first extra data of `extra_type`, or `default`
/// when the list has none.
fn extra_word_or(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8, default: u32) -> u32 {
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        default
    } else {
        e.mem.u32(extra.addr() + 0x0c)
    }
}

/// A `float` word loaded and stored through the x87 stack (`FLD`/`FSTP`):
/// the same bits, except that a signalling NaN comes out quiet.
fn x87_float_bits(bits: u32) -> u32 {
    let is_nan = bits & 0x7f80_0000 == 0x7f80_0000 && bits & 0x007f_ffff != 0;
    if is_nan {
        bits | 0x0040_0000
    } else {
        bits
    }
}

/// A `float` as the x87 stack stores it.
fn x87_float(value: f32) -> u32 {
    x87_float_bits(value.to_bits())
}

/// The `float` at +0x0C of the first extra data of `extra_type`, or 0.0
/// (`FLDZ`); returned in ST0.
fn extra_float_or_zero(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> f32 {
    let bits = extra_word_or(e, list, extra_type, 0);
    f32::from_bits(x87_float_bits(bits))
}

/// `new` and construct: allocates `size` bytes and runs the constructor at
/// `construct` on the block (it returns the object), or gives null when the
/// allocation failed.
fn new_extra(e: &mut Engine, size: u32, construct: u32) -> Ptr<BSExtraData> {
    let block = e.call(OPERATOR_NEW, &args![size]).u32();
    if block == 0 {
        Ptr::NULL
    } else {
        e.call(construct, &args![block]).ptr()
    }
}

/// The body of the word setters: `value == remove_value` removes the extra
/// data of `extra_type`; otherwise `release_old` runs on the word the
/// existing extra data holds before the new value replaces it, or a new
/// 0x10-byte extra data is built with `construct`, given the value and added.
fn set_word_extra(
    e: &mut Engine,
    list: Ptr<ExtraDataList>,
    extra_type: u8,
    value: u32,
    remove_value: u32,
    construct: u32,
    release_old: impl FnOnce(&mut Engine, u32),
) {
    if value == remove_value {
        remove_extra(e, list, extra_type);
        return;
    }
    let existing = find_extra(e, list, extra_type);
    if existing.is_null() {
        let extra = new_extra(e, 0x10, construct);
        e.mem.set_u32(extra.addr().wrapping_add(0x0c), value);
        add_extra(e, list, extra);
    } else {
        let old = e.mem.u32(existing.addr() + 0x0c);
        release_old(e, old);
        e.mem.set_u32(existing.addr() + 0x0c, value);
    }
}

/// The body of the handle setters: a zero `value` removes the extra data of
/// `extra_type`; otherwise the handle at +0x0C of the existing extra data, or
/// of a new 0x10-byte one built with `construct` (added afterwards), is
/// assigned `value` (`0066b0d0`).
fn set_handle_extra(
    e: &mut Engine,
    list: Ptr<ExtraDataList>,
    extra_type: u8,
    value: u32,
    construct: u32,
) {
    if value == 0 {
        remove_extra(e, list, extra_type);
        return;
    }
    let existing = find_extra(e, list, extra_type);
    if existing.is_null() {
        let extra = new_extra(e, 0x10, construct);
        e.call(
            ASSIGN_HANDLE,
            &args![extra.addr().wrapping_add(0x0c), value],
        );
        add_extra(e, list, extra);
    } else {
        e.call(ASSIGN_HANDLE, &args![existing.addr() + 0x0c, value]);
    }
}

/// Whether the `float` setters take `value` as the neutral value: `FCOMP`
/// against the `double` at [`ZERO_DOUBLE`] (a NaN compares unordered, so it
/// is stored).
fn float_is_zero(e: &Engine, value: f32) -> bool {
    value as f64 == e.global::<f64>(ZERO_DOUBLE)
}

// ---------------------------------------------------------------------------
// Translations

// Translated from 00421400 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetMerchantContainer` (Xbox PDB): the word at +0x0C of the
/// type `0x3C` extra data (`EXTRA_MERCHANTCONTAINER`), or 0.
pub fn extra_data_list_get_merchant_container(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_MERCHANT_CONTAINER, 0)
}

// Translated from 00421430 (decompiled, FalloutNV.exe 1.4.0.525)
/// Merchant container setter (type `0x3C`, `EXTRA_MERCHANTCONTAINER`): a null
/// `value` removes the extra data; otherwise it is stored in the existing
/// one or in a new 0x10-byte one.
pub fn fn_00421430(e: &mut Engine, this: Ptr<ExtraDataList>, value: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_MERCHANT_CONTAINER,
        value,
        0,
        MERCHANT_CONTAINER_INIT,
        |_, _| {},
    );
}

// Translated from 004214f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Reads the leveled creature modifier (type `0x1E`, `EXTRA_LEVCREA_MOD`):
/// stores `1.0` in `out_float` and 0 in `out_word`, then, when the list has
/// the extra data, replaces them with its float (`004350c0`, ST0) and its
/// word (`00435100`, EAX).
pub fn fn_004214f0(e: &mut Engine, this: Ptr<ExtraDataList>, out_float: Ptr, out_word: Ptr) {
    e.mem.set_f32(out_float.addr(), 1.0);
    e.mem.set_u32(out_word.addr(), 0);
    let extra = find_extra(e, this, EXTRA_LEV_CREA_MOD);
    if !extra.is_null() {
        let scale = e.call(LEV_CREA_MOD_GET_FLOAT, &args![extra]).f32();
        e.mem.set_f32(out_float.addr(), scale);
        let word = e.call(LEV_CREA_MOD_GET_WORD, &args![extra]).u32();
        e.mem.set_u32(out_word.addr(), word);
    }
}

// Translated from 00421540 (decompiled, FalloutNV.exe 1.4.0.525)
/// Leveled creature modifier setter (type `0x1E`, `EXTRA_LEVCREA_MOD`): the
/// value 4 removes the extra data; any other value is stored in the existing
/// one or in a new 0x10-byte one.
pub fn fn_00421540(e: &mut Engine, this: Ptr<ExtraDataList>, value: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_LEV_CREA_MOD,
        value,
        4,
        LEV_CREA_MOD_INIT,
        |_, _| {},
    );
}

// Translated from 00421600 (decompiled, FalloutNV.exe 1.4.0.525)
/// "No rumors" flag setter (type `0x4E`, `EXTRA_NO_RUMORS`): stores the byte
/// at +0x0C of the existing extra data, or builds a new 0x10-byte one (its
/// constructor, `00436090`, takes the byte) and adds it. The flag never
/// removes the extra data.
pub fn fn_00421600(e: &mut Engine, this: Ptr<ExtraDataList>, flag: u8) {
    let existing = find_extra(e, this, EXTRA_NO_RUMORS);
    if existing.is_null() {
        let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
        let extra: Ptr<BSExtraData> = if block == 0 {
            Ptr::NULL
        } else {
            e.call(NO_RUMORS_INIT, &args![block, flag as u32]).ptr()
        };
        add_extra(e, this, extra);
    } else {
        e.mem.set_u8(existing.addr() + 0x0c, flag);
    }
}

// Translated from 004216b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Removes the type `0x4E` extra data (`EXTRA_NO_RUMORS`).
pub fn fn_004216b0(e: &mut Engine, this: Ptr<ExtraDataList>) {
    remove_extra(e, this, EXTRA_NO_RUMORS);
}

// Translated from 004216d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Whether the list has a type `0x2E` extra data (`EXTRA_LEVELEDCREATURE`):
/// `BaseExtraList::HasExtra`.
pub fn fn_004216d0(e: &mut Engine, this: Ptr<ExtraDataList>) -> bool {
    e.call(HAS_EXTRA, &args![this, EXTRA_LEVELED_CREATURE as u32])
        .bool()
}

// Translated from 004216f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetLevCreaOriginalBase` (Xbox PDB): the word at +0x0C of
/// the type `0x2E` extra data (`EXTRA_LEVELEDCREATURE`), or 0.
pub fn extra_data_list_get_lev_crea_original_base(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_LEVELED_CREATURE, 0)
}

// Translated from 00421720 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x10 of the type `0x2E` extra data (`EXTRA_LEVELEDCREATURE`),
/// or 0.
pub fn fn_00421720(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra = find_extra(e, this, EXTRA_LEVELED_CREATURE);
    if extra.is_null() {
        0
    } else {
        e.mem.u32(extra.addr() + 0x10)
    }
}

// Translated from 00421750 (decompiled, FalloutNV.exe 1.4.0.525)
/// Leveled creature setter (type `0x2E`, `EXTRA_LEVELEDCREATURE`): when
/// either word is 0 the extra data is removed; otherwise the words go to +0x0C
/// and +0x10 of the existing extra data, or of a new 0x14-byte one
/// (constructor `004311f0`) that is added first.
pub fn fn_00421750(e: &mut Engine, this: Ptr<ExtraDataList>, first: u32, second: u32) {
    if first == 0 || second == 0 {
        remove_extra(e, this, EXTRA_LEVELED_CREATURE);
        return;
    }
    let mut extra = find_extra(e, this, EXTRA_LEVELED_CREATURE);
    if extra.is_null() {
        extra = new_extra(e, 0x14, LEVELED_CREATURE_INIT);
        add_extra(e, this, extra);
    }
    e.mem.set_u32(extra.addr().wrapping_add(0x0c), first);
    e.mem.set_u32(extra.addr().wrapping_add(0x10), second);
}

// Translated from 00421820 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetDetachTime` (Xbox PDB): the word at +0x0C of the type
/// `0x0B` extra data (`EXTRA_CELLDETACHTIME`), or 0.
pub fn extra_data_list_get_detach_time(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_CELL_DETACH_TIME, 0)
}

// Translated from 00421850 (decompiled, FalloutNV.exe 1.4.0.525)
/// Cell detach time setter (type `0x0B`, `EXTRA_CELLDETACHTIME`): a zero
/// `time` removes the extra data; otherwise it is stored in the existing one
/// or in a new 0x10-byte one (constructor `0040f6b0`).
pub fn fn_00421850(e: &mut Engine, this: Ptr<ExtraDataList>, time: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_CELL_DETACH_TIME,
        time,
        0,
        DETACH_TIME_INIT,
        |_, _| {},
    );
}

// Translated from 00421910 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x0C of the type `5` extra data (`EXTRA_SEENDATA`), or 0.
pub fn fn_00421910(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_SEEN_DATA, 0)
}

// Translated from 00421940 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetSeenData` (Xbox PDB), type `5` (`EXTRA_SEENDATA`): a
/// null `data` removes the extra data; otherwise it is stored in the
/// existing extra data, after the object the old pointer holds is deleted
/// (virtual slot 0 with 1), or in a new 0x10-byte one (constructor
/// `0040f590`). The compiler's exception-unwinding frame is not translated.
pub fn extra_data_list_set_seen_data(e: &mut Engine, this: Ptr<ExtraDataList>, data: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_SEEN_DATA,
        data,
        0,
        SEEN_DATA_INIT,
        |e, old| {
            if old != 0 {
                e.vcall(old, 0, &args![1u32]);
            }
        },
    );
}

// Translated from 00421a40 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `float` at +0x0C of the type `0x43` extra data
/// (`EXTRA_NORTHROTATION`), or 0.0. Returned in ST0.
pub fn fn_00421a40(e: &mut Engine, this: Ptr<ExtraDataList>) -> f32 {
    extra_float_or_zero(e, this, EXTRA_NORTH_ROTATION)
}

// Translated from 00421a70 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetNorthRotation` (Xbox PDB), type `0x43`
/// (`EXTRA_NORTHROTATION`): 0.0 removes the extra data; otherwise the angle
/// is stored in the existing one or in a new 0x10-byte one (constructor
/// `0040f680`).
pub fn extra_data_list_set_north_rotation(e: &mut Engine, this: Ptr<ExtraDataList>, angle: f32) {
    if float_is_zero(e, angle) {
        remove_extra(e, this, EXTRA_NORTH_ROTATION);
        return;
    }
    let existing = find_extra(e, this, EXTRA_NORTH_ROTATION);
    if existing.is_null() {
        let extra = new_extra(e, 0x10, NORTH_ROTATION_INIT);
        e.mem
            .set_u32(extra.addr().wrapping_add(0x0c), x87_float(angle));
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(existing.addr() + 0x0c, x87_float(angle));
    }
}

// Translated from 00421b40 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetXTarget` (Xbox PDB): the word at +0x0C of the type
/// `0x44` extra data (`EXTRA_XTARGET`), or 0.
pub fn extra_data_list_get_x_target(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_X_TARGET, 0)
}

// Translated from 00421b70 (decompiled, FalloutNV.exe 1.4.0.525)
/// X target setter (type `0x44`, `EXTRA_XTARGET`): a zero `target` removes
/// the extra data; otherwise it is stored in the existing one or in a new
/// 0x10-byte one (constructor `00435370`).
pub fn fn_00421b70(e: &mut Engine, this: Ptr<ExtraDataList>, target: u32) {
    set_word_extra(e, this, EXTRA_X_TARGET, target, 0, X_TARGET_INIT, |_, _| {});
}

// Translated from 00421c30 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x0C of the type `0x74` extra data (`EXTRA_ENCOUNTERZONE`),
/// or 0.
pub fn fn_00421c30(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_ENCOUNTER_ZONE, 0)
}

// Translated from 00421c60 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetEncounterZone` (Xbox PDB), type `0x74`
/// (`EXTRA_ENCOUNTERZONE`): a zero `zone` removes the extra data; otherwise
/// it is stored in the existing one or in a new 0x10-byte one
/// (`ExtraEncounterZone::ExtraEncounterZone`, `00432e60`).
pub fn extra_data_list_set_encounter_zone(e: &mut Engine, this: Ptr<ExtraDataList>, zone: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_ENCOUNTER_ZONE,
        zone,
        0,
        ENCOUNTER_ZONE_INIT,
        |_, _| {},
    );
}

// Translated from 00421d20 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x0C of the type `0x67` extra data (`EXTRA_EMITTANCE_SOURCE`),
/// or 0.
pub fn fn_00421d20(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_EMITTANCE_SOURCE, 0)
}

// Translated from 00421d50 (decompiled, FalloutNV.exe 1.4.0.525)
/// Emittance source setter (type `0x67`, `EXTRA_EMITTANCE_SOURCE`): a zero
/// `source` removes the extra data; otherwise it is stored in the existing one
/// or in a new 0x10-byte one (constructor `00435440`).
pub fn fn_00421d50(e: &mut Engine, this: Ptr<ExtraDataList>, source: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_EMITTANCE_SOURCE,
        source,
        0,
        EMITTANCE_SOURCE_INIT,
        |_, _| {},
    );
}

// Translated from 00421e10 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x0C of the type `0x63` extra data (`EXTRA_MULTIBOUND_REF`),
/// or 0.
pub fn fn_00421e10(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_MULTIBOUND_REF, 0)
}

// Translated from 00421e40 (decompiled, FalloutNV.exe 1.4.0.525)
/// Multibound reference setter (type `0x63`, `EXTRA_MULTIBOUND_REF`): a zero
/// `reference` removes the extra data; otherwise it is stored in the existing
/// one or in a new 0x10-byte one (constructor `00435510`).
pub fn fn_00421e40(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_MULTIBOUND_REF,
        reference,
        0,
        MULTIBOUND_REF_INIT,
        |_, _| {},
    );
}

// Translated from 00421f00 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word at +0x0C of the type `0x62` extra data (`EXTRA_MULTIBOUND_DATA`),
/// or 0.
pub fn fn_00421f00(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or(e, this, EXTRA_MULTIBOUND_DATA, 0)
}

// Translated from 00421f30 (decompiled, FalloutNV.exe 1.4.0.525)
/// Multibound data setter (type `0x62`, `EXTRA_MULTIBOUND_DATA`): a zero
/// `data` removes the extra data; otherwise it is stored in the existing one,
/// after the block its old pointer holds is freed (`operator delete`, when
/// not null), or in a new 0x10-byte one (constructor `00435590`).
pub fn fn_00421f30(e: &mut Engine, this: Ptr<ExtraDataList>, data: u32) {
    set_word_extra(
        e,
        this,
        EXTRA_MULTIBOUND_DATA,
        data,
        0,
        MULTIBOUND_DATA_INIT,
        |e, old| {
            if old != 0 {
                e.call(OPERATOR_DELETE, &args![old]);
            }
        },
    );
}

// Translated from 00422020 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetMultiBound` (Xbox PDB): the word the handle at +0x0C of
/// the type `0x61` extra data (`EXTRA_MULTIBOUND`) points at (`00559450`), or 0.
pub fn extra_data_list_get_multi_bound(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra = find_extra(e, this, EXTRA_MULTIBOUND);
    if extra.is_null() {
        0
    } else {
        e.call(READ_WORD, &args![extra.addr() + 0x0c]).u32()
    }
}

// Translated from 00422050 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetMultiBound` (Xbox PDB), type `0x61`
/// (`EXTRA_MULTIBOUND`): a zero `value` removes the extra data; otherwise the
/// handle at its +0x0C is assigned (`0066b0d0`); a new extra data
/// (constructor `00435690`) gets the handle assigned before it is added.
pub fn extra_data_list_set_multi_bound(e: &mut Engine, this: Ptr<ExtraDataList>, value: u32) {
    set_handle_extra(e, this, EXTRA_MULTIBOUND, value, MULTIBOUND_INIT);
}

// Translated from 00422120 (decompiled, FalloutNV.exe 1.4.0.525)
/// The word the handle at +0x0C of the type `0x71` extra data
/// (`EXTRA_OCCLUSION_PLANE`) points at (`00559450`), or 0.
pub fn fn_00422120(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra = find_extra(e, this, EXTRA_OCCLUSION_PLANE);
    if extra.is_null() {
        0
    } else {
        e.call(READ_WORD, &args![extra.addr() + 0x0c]).u32()
    }
}

// Translated from 00422150 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetOcclusionPlane` (Xbox PDB), type `0x71`
/// (`EXTRA_OCCLUSION_PLANE`): as `SetMultiBound`, with the constructor
/// `004357a0`.
pub fn extra_data_list_set_occlusion_plane(e: &mut Engine, this: Ptr<ExtraDataList>, value: u32) {
    set_handle_extra(e, this, EXTRA_OCCLUSION_PLANE, value, OCCLUSION_PLANE_INIT);
}

// Translated from 00422220 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetRadius` (Xbox PDB), type `0x5C` (`EXTRA_RADIUS`): 0.0
/// removes the extra data; otherwise the radius is stored in the existing one
/// or a new 0x10-byte one (built with the radius, `004222f0`) is added.
pub fn extra_data_list_set_radius(e: &mut Engine, this: Ptr<ExtraDataList>, radius: f32) {
    if float_is_zero(e, radius) {
        remove_extra(e, this, EXTRA_RADIUS);
        return;
    }
    let existing = find_extra(e, this, EXTRA_RADIUS);
    if existing.is_null() {
        let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
        let extra: Ptr<BSExtraData> = if block == 0 {
            Ptr::NULL
        } else {
            fn_004222f0(e, Ptr::new(block), radius).cast()
        };
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(existing.addr() + 0x0c, x87_float(radius));
    }
}

// Translated from 004222f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of `ExtraRadius` (RTTI name; type `0x5C`): the `BSExtraData`
/// base with the type, the vtable at `01015208` and the radius at +0x0C.
/// Returns `this`.
pub fn fn_004222f0(e: &mut Engine, this: Ptr, radius: f32) -> Ptr {
    e.call(BS_EXTRA_DATA_INIT, &args![this, EXTRA_RADIUS as u32]);
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_RADIUS);
    e.mem.set_u32(this.addr() + 0x0c, x87_float(radius));
    this
}

// Translated from 00422320 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetRadius` (Xbox PDB): the `float` at +0x0C of the type
/// `0x5C` extra data (`EXTRA_RADIUS`), or 0.0. Returned in ST0.
pub fn extra_data_list_get_radius(e: &mut Engine, this: Ptr<ExtraDataList>) -> f32 {
    extra_float_or_zero(e, this, EXTRA_RADIUS)
}

// Translated from 00422350 (decompiled, FalloutNV.exe 1.4.0.525)
/// Radiation setter (type `0x5D`, `EXTRA_RADIATION`): 0.0 removes the extra
/// data; otherwise the value is stored in the existing one or a new 0x10-byte
/// one (built with the value, `00422420`) is added.
pub fn fn_00422350(e: &mut Engine, this: Ptr<ExtraDataList>, radiation: f32) {
    if float_is_zero(e, radiation) {
        remove_extra(e, this, EXTRA_RADIATION);
        return;
    }
    let existing = find_extra(e, this, EXTRA_RADIATION);
    if existing.is_null() {
        let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
        let extra: Ptr<BSExtraData> = if block == 0 {
            Ptr::NULL
        } else {
            fn_00422420(e, Ptr::new(block), radiation).cast()
        };
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(existing.addr() + 0x0c, x87_float(radiation));
    }
}

// Translated from 00422420 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of `ExtraRadiation` (RTTI name; type `0x5D`): the
/// `BSExtraData` base with the type, the vtable at `01015214` and the value at
/// +0x0C. Returns `this`.
pub fn fn_00422420(e: &mut Engine, this: Ptr, radiation: f32) -> Ptr {
    e.call(BS_EXTRA_DATA_INIT, &args![this, EXTRA_RADIATION as u32]);
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_RADIATION);
    e.mem.set_u32(this.addr() + 0x0c, x87_float(radiation));
    this
}

// Translated from 00422450 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `float` at +0x0C of the type `0x5D` extra data (`EXTRA_RADIATION`), or
/// 0.0. Returned in ST0.
pub fn fn_00422450(e: &mut Engine, this: Ptr<ExtraDataList>) -> f32 {
    extra_float_or_zero(e, this, EXTRA_RADIATION)
}

// Translated from 00422480 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::AddFollower` (Xbox PDB): unless `follower` is the player
/// character (the pointer at `011dea3c`), makes sure the list has a type
/// `0x1D` extra data (`EXTRA_FOLLOWER`; a new one is built by
/// `ExtraFollower::ExtraFollower`, `00430dd0`, 0x10 bytes) and adds
/// `follower` at the head of the list its +0x0C points at, unless that list
/// already holds it (`005f65d0`, `005ae3d0`, both given the address of a
/// stack slot holding `follower`).
pub fn extra_data_list_add_follower(e: &mut Engine, this: Ptr<ExtraDataList>, follower: Ptr) {
    if follower.addr() == e.global::<u32>(PLAYER_SINGLETON) {
        return;
    }
    let mut extra = find_extra(e, this, EXTRA_FOLLOWER);
    if extra.is_null() {
        extra = new_extra(e, 0x10, FOLLOWER_INIT);
        add_extra(e, this, extra);
    }
    let followers = e.mem.u32(extra.addr() + 0x0c);
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), follower.addr());
        let known = e.call(LIST_CONTAINS, &args![followers, slot]).bool();
        if !known {
            e.call(LIST_ADD_HEAD, &args![followers, slot]);
        }
    });
}

// Translated from 00422550 (decompiled, FalloutNV.exe 1.4.0.525)
/// Whether the follower list of the type `0x1D` extra data
/// (`EXTRA_FOLLOWER`) holds `follower` (`005f65d0`, given the address of a
/// stack slot holding it); false when the list has no such extra data.
pub fn fn_00422550(e: &mut Engine, this: Ptr<ExtraDataList>, follower: Ptr) -> bool {
    let extra = find_extra(e, this, EXTRA_FOLLOWER);
    if extra.is_null() {
        return false;
    }
    let followers = e.mem.u32(extra.addr() + 0x0c);
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), follower.addr());
        e.call(LIST_CONTAINS, &args![followers, slot]).bool()
    })
}

// Translated from 00422590 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::AddFriendHit` (Xbox PDB): makes sure the list has a type
/// `0x45` extra data (`EXTRA_FRIEND_HITS`; a new 0x1C-byte one is built by
/// `ExtraFriendHits::ExtraFriendHits`, `00435c40`), then calls
/// `ExtraFriendHits::AddHit` and `ExtraFriendHits::GetHitCount` on it (the
/// count is not used).
pub fn extra_data_list_add_friend_hit(e: &mut Engine, this: Ptr<ExtraDataList>) {
    let mut extra = find_extra(e, this, EXTRA_FRIEND_HITS);
    if extra.is_null() {
        extra = new_extra(e, 0x1c, FRIEND_HITS_INIT);
        add_extra(e, this, extra);
    }
    e.call(FRIEND_HITS_ADD_HIT, &args![extra]);
    e.call(FRIEND_HITS_GET_HIT_COUNT, &args![extra]);
}

// Translated from 00422640 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetFriendHitCount` (Xbox PDB): `ExtraFriendHits::GetHitCount`
/// of the type `0x45` extra data (`EXTRA_FRIEND_HITS`), or 0.
pub fn extra_data_list_get_friend_hit_count(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra = find_extra(e, this, EXTRA_FRIEND_HITS);
    if extra.is_null() {
        0
    } else {
        e.call(FRIEND_HITS_GET_HIT_COUNT, &args![extra]).u32()
    }
}

/// This part's translated functions, by exe address.
pub fn funcs() -> Vec<(u32, AbiFn)> {
    vec![
        entry!(
            0x00421400,
            extra_data_list_get_merchant_container(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x00421430, fn_00421430(Ptr<ExtraDataList>, u32)),
        entry!(0x004214f0, fn_004214f0(Ptr<ExtraDataList>, Ptr, Ptr)),
        entry!(0x00421540, fn_00421540(Ptr<ExtraDataList>, u32)),
        entry!(0x00421600, fn_00421600(Ptr<ExtraDataList>, u8)),
        entry!(0x004216b0, fn_004216b0(Ptr<ExtraDataList>)),
        entry!(0x004216d0, fn_004216d0(Ptr<ExtraDataList>) -> bool),
        entry!(
            0x004216f0,
            extra_data_list_get_lev_crea_original_base(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x00421720, fn_00421720(Ptr<ExtraDataList>) -> u32),
        entry!(0x00421750, fn_00421750(Ptr<ExtraDataList>, u32, u32)),
        entry!(
            0x00421820,
            extra_data_list_get_detach_time(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x00421850, fn_00421850(Ptr<ExtraDataList>, u32)),
        entry!(0x00421910, fn_00421910(Ptr<ExtraDataList>) -> u32),
        entry!(
            0x00421940,
            extra_data_list_set_seen_data(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x00421a40, fn_00421a40(Ptr<ExtraDataList>) -> f32),
        entry!(
            0x00421a70,
            extra_data_list_set_north_rotation(Ptr<ExtraDataList>, f32)
        ),
        entry!(
            0x00421b40,
            extra_data_list_get_x_target(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x00421b70, fn_00421b70(Ptr<ExtraDataList>, u32)),
        entry!(0x00421c30, fn_00421c30(Ptr<ExtraDataList>) -> u32),
        entry!(
            0x00421c60,
            extra_data_list_set_encounter_zone(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x00421d20, fn_00421d20(Ptr<ExtraDataList>) -> u32),
        entry!(0x00421d50, fn_00421d50(Ptr<ExtraDataList>, u32)),
        entry!(0x00421e10, fn_00421e10(Ptr<ExtraDataList>) -> u32),
        entry!(0x00421e40, fn_00421e40(Ptr<ExtraDataList>, u32)),
        entry!(0x00421f00, fn_00421f00(Ptr<ExtraDataList>) -> u32),
        entry!(0x00421f30, fn_00421f30(Ptr<ExtraDataList>, u32)),
        entry!(
            0x00422020,
            extra_data_list_get_multi_bound(Ptr<ExtraDataList>) -> u32
        ),
        entry!(
            0x00422050,
            extra_data_list_set_multi_bound(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x00422120, fn_00422120(Ptr<ExtraDataList>) -> u32),
        entry!(
            0x00422150,
            extra_data_list_set_occlusion_plane(Ptr<ExtraDataList>, u32)
        ),
        entry!(
            0x00422220,
            extra_data_list_set_radius(Ptr<ExtraDataList>, f32)
        ),
        entry!(0x004222f0, fn_004222f0(Ptr, f32) -> Ptr),
        entry!(
            0x00422320,
            extra_data_list_get_radius(Ptr<ExtraDataList>) -> f32
        ),
        entry!(0x00422350, fn_00422350(Ptr<ExtraDataList>, f32)),
        entry!(0x00422420, fn_00422420(Ptr, f32) -> Ptr),
        entry!(0x00422450, fn_00422450(Ptr<ExtraDataList>) -> f32),
        entry!(
            0x00422480,
            extra_data_list_add_follower(Ptr<ExtraDataList>, Ptr)
        ),
        entry!(0x00422550, fn_00422550(Ptr<ExtraDataList>, Ptr) -> bool),
        entry!(
            0x00422590,
            extra_data_list_add_friend_hit(Ptr<ExtraDataList>)
        ),
        entry!(
            0x00422640,
            extra_data_list_get_friend_hit_count(Ptr<ExtraDataList>) -> u32
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    type Log = Vec<(u32, Vec<u32>)>;
    /// The `(list, item)` pairs `LIST_ADD_HEAD` was given.
    type Added = Rc<RefCell<Vec<(u32, u32)>>>;

    /// Test vtable of the extra data (slot 0 the scalar deleting destructor).
    const VTABLE: u32 = 0x0200_0000;
    const DESTRUCTOR: u32 = 0x0200_1000;

    // Callees of the list operations in `extradatalist.rs`, which run for
    // real here on top of these doubles.
    const GET_TYPE: u32 = 0x004f_1540;
    const GET_NEXT: u32 = 0x0044_ddc0;
    const SET_NEXT: u32 = 0x0040_3550;
    const SIMPLE_LIST_NEXT: u32 = 0x0072_6070;
    const SIMPLE_LIST_ITEM: u32 = 0x0068_15c0;
    const MEMSET: u32 = 0x0040_3d30;
    const LOCK: u32 = 0x0040_fbf0;
    const UNLOCK: u32 = 0x0040_fba0;

    /// (constructor, extra data type): each double sets the vtable, the type
    /// and a null next, and stores the argument word, if any, at +0x0C.
    const CONSTRUCTORS: &[(u32, u8)] = &[
        (MERCHANT_CONTAINER_INIT, EXTRA_MERCHANT_CONTAINER),
        (LEV_CREA_MOD_INIT, EXTRA_LEV_CREA_MOD),
        (NO_RUMORS_INIT, EXTRA_NO_RUMORS),
        (LEVELED_CREATURE_INIT, EXTRA_LEVELED_CREATURE),
        (SEEN_DATA_INIT, EXTRA_SEEN_DATA),
        (X_TARGET_INIT, EXTRA_X_TARGET),
        (ENCOUNTER_ZONE_INIT, EXTRA_ENCOUNTER_ZONE),
        (EMITTANCE_SOURCE_INIT, EXTRA_EMITTANCE_SOURCE),
        (MULTIBOUND_REF_INIT, EXTRA_MULTIBOUND_REF),
        (MULTIBOUND_DATA_INIT, EXTRA_MULTIBOUND_DATA),
        (MULTIBOUND_INIT, EXTRA_MULTIBOUND),
        (OCCLUSION_PLANE_INIT, EXTRA_OCCLUSION_PLANE),
        (FOLLOWER_INIT, EXTRA_FOLLOWER),
        (FRIEND_HITS_INIT, EXTRA_FRIEND_HITS),
    ];

    fn returns(value: u32) -> Ret {
        Ret {
            eax: value,
            ..Ret::default()
        }
    }

    fn stub(e: &mut Engine, address: u32) {
        e.register(address, |_, _| Ret::default());
    }

    /// An engine with doubles for the small callees of the list operations
    /// (type and next accessors, the lock, `operator new`/`delete`, the
    /// `BSExtraData` constructor, the handle accessors) and for the
    /// constructors of the extra data other units own.
    fn engine() -> Engine {
        let mut e = Engine::new();
        e.map(0x011c_3000, 0x1000);
        e.put_vtable(VTABLE, &[DESTRUCTOR]);
        stub(&mut e, DESTRUCTOR);
        // The vtables the real constructors of this unit set.
        for vtable in [
            0x0101_42a0,
            0x0101_42ac,
            VTABLE_EXTRA_RADIUS,
            VTABLE_EXTRA_RADIATION,
        ] {
            e.put_vtable(vtable, &[DESTRUCTOR]);
        }
        e.register(GET_TYPE, |e, a| returns(e.mem.u8(a[0] + 4) as u32));
        e.register(GET_NEXT, |e, a| returns(e.mem.u32(a[0] + 8)));
        e.register(SET_NEXT, |e, a| {
            e.mem.set_u32(a[0] + 8, a[1]);
            Ret::default()
        });
        e.register(SIMPLE_LIST_NEXT, |e, a| returns(e.mem.u32(a[0] + 4)));
        e.register(SIMPLE_LIST_ITEM, |_, a| returns(a[0]));
        e.register(READ_WORD, |e, a| returns(e.mem.u32(a[0])));
        e.register(MEMSET, |e, a| {
            for offset in 0..a[2] {
                e.mem.set_u8(a[0] + offset, a[1] as u8);
            }
            Ret::default()
        });
        e.register(BS_EXTRA_DATA_INIT, |e, a| {
            e.mem.set_u8(a[0] + 4, a[1] as u8);
            e.mem.set_u32(a[0] + 8, 0);
            returns(a[0])
        });
        e.register(OPERATOR_NEW, |e, a| returns(e.mem.alloc(a[0])));
        stub(&mut e, OPERATOR_DELETE);
        stub(&mut e, LOCK);
        stub(&mut e, UNLOCK);
        e.register(ASSIGN_HANDLE, |e, a| {
            e.mem.set_u32(a[0], a[1]);
            returns(a[0])
        });
        for &(address, extra_type) in CONSTRUCTORS {
            e.register_double(address, move |e, a| {
                e.mem.set_u32(a[0], VTABLE);
                e.mem.set_u8(a[0] + 4, extra_type);
                e.mem.set_u32(a[0] + 8, 0);
                if let Some(word) = a.get(1) {
                    e.mem.set_u32(a[0] + 0x0c, *word);
                }
                returns(a[0])
            });
        }
        e.map(0x0101_2000, 0x1000);
        e.map(0x011d_e000, 0x1000);
        e.set_global::<f64>(ZERO_DOUBLE, 0.0);
        e.set_global::<u32>(PLAYER_SINGLETON, 0x7777);
        e
    }

    fn new_list(e: &mut Engine) -> Ptr<ExtraDataList> {
        e.new_object()
    }

    /// Adds an extra data of `extra_type` holding `word` at +0x0C to the list
    /// through `AddExtra`.
    fn add(
        e: &mut Engine,
        list: Ptr<ExtraDataList>,
        extra_type: u8,
        word: u32,
    ) -> Ptr<BSExtraData> {
        let extra: Ptr<BSExtraData> = Ptr::new(e.mem.alloc(0x40));
        e.mem.set_u32(extra.addr(), VTABLE);
        e.set(extra, BSExtraData::cEtype, extra_type);
        e.mem.set_u32(extra.addr() + 0x0c, word);
        add_extra(e, list, extra);
        extra
    }

    fn chain_types(e: &Engine, list: Ptr<ExtraDataList>) -> Vec<u8> {
        let mut types = vec![];
        let mut current: Ptr<BSExtraData> = e.get(list, ExtraDataList::pHead).cast();
        while !current.is_null() {
            types.push(e.get(current, BSExtraData::cEtype));
            current = e.get(current, BSExtraData::pNext).cast();
        }
        types
    }

    fn word_of(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
        let extra = find_extra(e, list, extra_type);
        assert!(!extra.is_null());
        e.mem.u32(extra.addr() + 0x0c)
    }

    fn calls_to(log: &Log, address: u32) -> Vec<Vec<u32>> {
        log.iter()
            .filter(|(callee, _)| *callee == address)
            .map(|(_, args)| args.clone())
            .collect()
    }

    /// Runs `call` with the call log on and returns the log.
    fn logged(e: &mut Engine, call: impl FnOnce(&mut Engine)) -> Log {
        e.call_log = Some(vec![]);
        call(e);
        e.call_log.take().unwrap()
    }

    /// A word getter: 0 without the extra data, the word at +0x0C with it.
    fn check_word_getter(address: u32, extra_type: u8) {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(address, &args![list]).u32(), 0);
        add(&mut e, list, 0x01, 0x1111);
        assert_eq!(e.call(address, &args![list]).u32(), 0);
        add(&mut e, list, extra_type, 0xabcd_0001);
        assert_eq!(e.call(address, &args![list]).u32(), 0xabcd_0001);
    }

    /// A word setter: builds the extra data on the first call (one 0x10-byte
    /// allocation, its constructor), updates it on the second, removes (and
    /// deletes) it on `remove_value`.
    fn check_word_setter(address: u32, extra_type: u8, construct: u32, remove_value: u32) {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0x4000u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, construct).len(), 1);
        assert_eq!(chain_types(&e, list), vec![extra_type]);
        assert_eq!(word_of(&mut e, list, extra_type), 0x4000);

        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0x5000u32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert!(calls_to(&log, construct).is_empty());
        assert_eq!(chain_types(&e, list), vec![extra_type]);
        assert_eq!(word_of(&mut e, list, extra_type), 0x5000);

        let extra = find_extra(&mut e, list, extra_type);
        let log = logged(&mut e, |e| {
            e.call(address, &args![list, remove_value]);
        });
        assert_eq!(calls_to(&log, DESTRUCTOR), vec![vec![extra.addr(), 1]]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn merchant_container_getter() {
        check_word_getter(0x0042_1400, EXTRA_MERCHANT_CONTAINER);
    }

    #[test]
    fn merchant_container_setter() {
        check_word_setter(
            0x0042_1430,
            EXTRA_MERCHANT_CONTAINER,
            MERCHANT_CONTAINER_INIT,
            0,
        );
    }

    #[test]
    fn lev_crea_mod_getter_defaults_to_one_and_zero() {
        let mut e = engine();
        e.register(LEV_CREA_MOD_GET_FLOAT, |_, _| Ret {
            st0: 2.5,
            ..Ret::default()
        });
        e.register(LEV_CREA_MOD_GET_WORD, |_, _| returns(7));
        let list = new_list(&mut e);
        let out: Ptr = Ptr::new(e.mem.alloc(8));
        e.mem.set_u32(out.addr(), 0xdead);
        e.mem.set_u32(out.addr() + 4, 0xbeef);
        let log = logged(&mut e, |e| {
            e.call(0x0042_14f0, &args![list, out, out.addr() + 4]);
        });
        assert_eq!(e.mem.f32(out.addr()), 1.0);
        assert_eq!(e.mem.u32(out.addr() + 4), 0);
        assert!(calls_to(&log, LEV_CREA_MOD_GET_FLOAT).is_empty());

        let extra = add(&mut e, list, EXTRA_LEV_CREA_MOD, 0);
        let log = logged(&mut e, |e| {
            e.call(0x0042_14f0, &args![list, out, out.addr() + 4]);
        });
        assert_eq!(e.mem.f32(out.addr()), 2.5);
        assert_eq!(e.mem.u32(out.addr() + 4), 7);
        assert_eq!(
            calls_to(&log, LEV_CREA_MOD_GET_FLOAT),
            vec![vec![extra.addr()]]
        );
        assert_eq!(
            calls_to(&log, LEV_CREA_MOD_GET_WORD),
            vec![vec![extra.addr()]]
        );
    }

    #[test]
    fn lev_crea_mod_setter_removes_on_four() {
        check_word_setter(0x0042_1540, EXTRA_LEV_CREA_MOD, LEV_CREA_MOD_INIT, 4);
    }

    #[test]
    fn no_rumors_setter_stores_a_byte_and_never_removes() {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1600, &args![list, 1u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        // The constructor receives the byte.
        let constructed = calls_to(&log, NO_RUMORS_INIT);
        assert_eq!(constructed.len(), 1);
        assert_eq!(constructed[0][1], 1);
        assert_eq!(chain_types(&e, list), vec![EXTRA_NO_RUMORS]);
        assert_eq!(word_of(&mut e, list, EXTRA_NO_RUMORS), 1);

        // An existing extra data gets only the byte at +0x0C.
        let extra = find_extra(&mut e, list, EXTRA_NO_RUMORS);
        e.mem.set_u32(extra.addr() + 0x0c, 0x1122_3344);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1600, &args![list, 0u32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(word_of(&mut e, list, EXTRA_NO_RUMORS), 0x1122_3300);
        assert_eq!(chain_types(&e, list), vec![EXTRA_NO_RUMORS]);
    }

    #[test]
    fn no_rumors_remover_deletes_the_extra_data() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = add(&mut e, list, EXTRA_NO_RUMORS, 1);
        let log = logged(&mut e, |e| {
            e.call(0x0042_16b0, &args![list]);
        });
        assert_eq!(calls_to(&log, REMOVE_EXTRA), vec![vec![list.addr(), 0x4e]]);
        assert_eq!(calls_to(&log, DESTRUCTOR), vec![vec![extra.addr(), 1]]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn leveled_creature_presence_is_has_extra() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(!e.call(0x0042_16d0, &args![list]).bool());
        add(&mut e, list, 0x01, 0);
        assert!(!e.call(0x0042_16d0, &args![list]).bool());
        add(&mut e, list, EXTRA_LEVELED_CREATURE, 0);
        let log = logged(&mut e, |e| {
            assert!(e.call(0x0042_16d0, &args![list]).bool());
        });
        assert_eq!(calls_to(&log, HAS_EXTRA), vec![vec![list.addr(), 0x2e]]);
    }

    #[test]
    fn lev_crea_original_base_getter() {
        check_word_getter(0x0042_16f0, EXTRA_LEVELED_CREATURE);
    }

    #[test]
    fn leveled_creature_second_word_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0042_1720, &args![list]).u32(), 0);
        let extra = add(&mut e, list, EXTRA_LEVELED_CREATURE, 0x11);
        e.mem.set_u32(extra.addr() + 0x10, 0x22);
        assert_eq!(e.call(0x0042_1720, &args![list]).u32(), 0x22);
    }

    #[test]
    fn leveled_creature_setter() {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1750, &args![list, 0x11u32, 0x22u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x14]]);
        assert_eq!(calls_to(&log, LEVELED_CREATURE_INIT).len(), 1);
        assert_eq!(word_of(&mut e, list, EXTRA_LEVELED_CREATURE), 0x11);
        let extra = find_extra(&mut e, list, EXTRA_LEVELED_CREATURE);
        assert_eq!(e.mem.u32(extra.addr() + 0x10), 0x22);

        // An existing extra data is updated in place.
        let log = logged(&mut e, |e| {
            e.call(0x0042_1750, &args![list, 0x33u32, 0x44u32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(e.mem.u32(extra.addr() + 0x0c), 0x33);
        assert_eq!(e.mem.u32(extra.addr() + 0x10), 0x44);

        // A zero in either word removes it.
        e.call(0x0042_1750, &args![list, 0x33u32, 0u32]);
        assert!(chain_types(&e, list).is_empty());
        e.call(0x0042_1750, &args![list, 0x33u32, 0x44u32]);
        assert_eq!(chain_types(&e, list), vec![EXTRA_LEVELED_CREATURE]);
        e.call(0x0042_1750, &args![list, 0u32, 0x44u32]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn detach_time_getter() {
        check_word_getter(0x0042_1820, EXTRA_CELL_DETACH_TIME);
    }

    #[test]
    fn detach_time_setter() {
        check_word_setter(0x0042_1850, EXTRA_CELL_DETACH_TIME, DETACH_TIME_INIT, 0);
    }

    #[test]
    fn seen_data_getter() {
        check_word_getter(0x0042_1910, EXTRA_SEEN_DATA);
    }

    #[test]
    fn seen_data_setter_deletes_the_old_object() {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1940, &args![list, 0x4000u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, SEEN_DATA_INIT).len(), 1);
        assert_eq!(word_of(&mut e, list, EXTRA_SEEN_DATA), 0x4000);
        assert!(calls_to(&log, DESTRUCTOR).is_empty());

        // The old word is an object: it is deleted before the replacement.
        let old: Ptr = Ptr::new(e.mem.alloc(8));
        e.mem.set_u32(old.addr(), VTABLE);
        let extra = find_extra(&mut e, list, EXTRA_SEEN_DATA);
        e.mem.set_u32(extra.addr() + 0x0c, old.addr());
        let log = logged(&mut e, |e| {
            e.call(0x0042_1940, &args![list, 0x5000u32]);
        });
        assert_eq!(calls_to(&log, DESTRUCTOR), vec![vec![old.addr(), 1]]);
        assert_eq!(word_of(&mut e, list, EXTRA_SEEN_DATA), 0x5000);

        // A null old word is not deleted; a null value removes.
        e.mem.set_u32(extra.addr() + 0x0c, 0);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1940, &args![list, 0x6000u32]);
        });
        assert!(calls_to(&log, DESTRUCTOR).is_empty());
        assert_eq!(word_of(&mut e, list, EXTRA_SEEN_DATA), 0x6000);
        e.call(0x0042_1940, &args![list, 0u32]);
        assert!(chain_types(&e, list).is_empty());
    }

    /// Float getter: 0.0 without the extra data, the float with it; a
    /// signalling NaN comes out quiet (x87 load).
    fn check_float_getter(address: u32, extra_type: u8) {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(address, &args![list]).f32(), 0.0);
        let extra = add(&mut e, list, extra_type, 2.5f32.to_bits());
        assert_eq!(e.call(address, &args![list]).f32(), 2.5);
        e.mem.set_u32(extra.addr() + 0x0c, 0x7f80_0001);
        let quiet = e.call(address, &args![list]).f32();
        assert_eq!(quiet.to_bits() & 0x7fc0_0000, 0x7fc0_0000);
    }

    /// Float setter: builds, updates, stores a NaN, and removes on 0.0 and
    /// -0.0.
    fn check_float_setter(address: u32, extra_type: u8) {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 1.5f32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(chain_types(&e, list), vec![extra_type]);
        assert_eq!(word_of(&mut e, list, extra_type), 1.5f32.to_bits());

        let log = logged(&mut e, |e| {
            e.call(address, &args![list, -4.25f32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(word_of(&mut e, list, extra_type), (-4.25f32).to_bits());

        // A signalling NaN is not zero, and is stored quiet.
        e.call(address, &args![list, f32::from_bits(0x7f80_0001)]);
        assert_eq!(chain_types(&e, list), vec![extra_type]);
        assert_eq!(word_of(&mut e, list, extra_type), 0x7fc0_0001);

        let extra = find_extra(&mut e, list, extra_type);
        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0.0f32]);
        });
        assert_eq!(calls_to(&log, DESTRUCTOR), vec![vec![extra.addr(), 1]]);
        assert!(chain_types(&e, list).is_empty());
        add(&mut e, list, extra_type, 1.0f32.to_bits());
        e.call(address, &args![list, -0.0f32]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn north_rotation_getter() {
        check_float_getter(0x0042_1a40, EXTRA_NORTH_ROTATION);
    }

    #[test]
    fn north_rotation_setter() {
        check_float_setter(0x0042_1a70, EXTRA_NORTH_ROTATION);
    }

    #[test]
    fn x_target_getter() {
        check_word_getter(0x0042_1b40, EXTRA_X_TARGET);
    }

    #[test]
    fn x_target_setter() {
        check_word_setter(0x0042_1b70, EXTRA_X_TARGET, X_TARGET_INIT, 0);
    }

    #[test]
    fn encounter_zone_getter() {
        check_word_getter(0x0042_1c30, EXTRA_ENCOUNTER_ZONE);
    }

    #[test]
    fn encounter_zone_setter() {
        check_word_setter(0x0042_1c60, EXTRA_ENCOUNTER_ZONE, ENCOUNTER_ZONE_INIT, 0);
    }

    #[test]
    fn emittance_source_getter() {
        check_word_getter(0x0042_1d20, EXTRA_EMITTANCE_SOURCE);
    }

    #[test]
    fn emittance_source_setter() {
        check_word_setter(
            0x0042_1d50,
            EXTRA_EMITTANCE_SOURCE,
            EMITTANCE_SOURCE_INIT,
            0,
        );
    }

    #[test]
    fn multibound_reference_getter() {
        check_word_getter(0x0042_1e10, EXTRA_MULTIBOUND_REF);
    }

    #[test]
    fn multibound_reference_setter() {
        check_word_setter(0x0042_1e40, EXTRA_MULTIBOUND_REF, MULTIBOUND_REF_INIT, 0);
    }

    #[test]
    fn multibound_data_getter() {
        check_word_getter(0x0042_1f00, EXTRA_MULTIBOUND_DATA);
    }

    #[test]
    fn multibound_data_setter_frees_the_old_block() {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1f30, &args![list, 0x4000u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, MULTIBOUND_DATA_INIT).len(), 1);
        assert_eq!(word_of(&mut e, list, EXTRA_MULTIBOUND_DATA), 0x4000);
        assert!(calls_to(&log, OPERATOR_DELETE).is_empty());

        // The old block is freed before it is replaced.
        let log = logged(&mut e, |e| {
            e.call(0x0042_1f30, &args![list, 0x5000u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_DELETE), vec![vec![0x4000]]);
        assert_eq!(word_of(&mut e, list, EXTRA_MULTIBOUND_DATA), 0x5000);

        // A null old word is not freed.
        let extra = find_extra(&mut e, list, EXTRA_MULTIBOUND_DATA);
        e.mem.set_u32(extra.addr() + 0x0c, 0);
        let log = logged(&mut e, |e| {
            e.call(0x0042_1f30, &args![list, 0x6000u32]);
        });
        assert!(calls_to(&log, OPERATOR_DELETE).is_empty());

        // A null value removes the extra data (without freeing the block).
        let log = logged(&mut e, |e| {
            e.call(0x0042_1f30, &args![list, 0u32]);
        });
        assert!(calls_to(&log, OPERATOR_DELETE).is_empty());
        assert!(chain_types(&e, list).is_empty());
    }

    /// A handle getter reads the word the handle at +0x0C points at.
    fn check_handle_getter(address: u32, extra_type: u8) {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(address, &args![list]).u32(), 0);
        let extra = add(&mut e, list, extra_type, 0x6001);
        let log = logged(&mut e, |e| {
            assert_eq!(e.call(address, &args![list]).u32(), 0x6001);
        });
        assert_eq!(calls_to(&log, READ_WORD), vec![vec![extra.addr() + 0x0c]]);
    }

    /// A handle setter: the new extra data is constructed, assigned the
    /// handle, then added; an existing one is only assigned; zero removes.
    fn check_handle_setter(address: u32, extra_type: u8, construct: u32) {
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0x4000u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, construct).len(), 1);
        let extra = find_extra(&mut e, list, extra_type);
        assert_eq!(
            calls_to(&log, ASSIGN_HANDLE),
            vec![vec![extra.addr() + 0x0c, 0x4000]]
        );
        let order: Vec<u32> = log.iter().map(|(callee, _)| *callee).collect();
        let assigned = order.iter().position(|a| *a == ASSIGN_HANDLE).unwrap();
        let added = order.iter().position(|a| *a == ADD_EXTRA).unwrap();
        assert!(assigned < added);
        assert_eq!(word_of(&mut e, list, extra_type), 0x4000);

        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0x5000u32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(
            calls_to(&log, ASSIGN_HANDLE),
            vec![vec![extra.addr() + 0x0c, 0x5000]]
        );

        let log = logged(&mut e, |e| {
            e.call(address, &args![list, 0u32]);
        });
        assert!(calls_to(&log, ASSIGN_HANDLE).is_empty());
        assert_eq!(calls_to(&log, DESTRUCTOR), vec![vec![extra.addr(), 1]]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn multibound_getter_reads_the_handle() {
        check_handle_getter(0x0042_2020, EXTRA_MULTIBOUND);
    }

    #[test]
    fn multibound_setter_assigns_the_handle() {
        check_handle_setter(0x0042_2050, EXTRA_MULTIBOUND, MULTIBOUND_INIT);
    }

    #[test]
    fn occlusion_plane_getter_reads_the_handle() {
        check_handle_getter(0x0042_2120, EXTRA_OCCLUSION_PLANE);
    }

    #[test]
    fn occlusion_plane_setter_assigns_the_handle() {
        check_handle_setter(0x0042_2150, EXTRA_OCCLUSION_PLANE, OCCLUSION_PLANE_INIT);
    }

    #[test]
    fn radius_setter() {
        check_float_setter(0x0042_2220, EXTRA_RADIUS);
        // The new extra data is built with the radius.
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_2220, &args![list, 3.0f32]);
        });
        let extra = find_extra(&mut e, list, EXTRA_RADIUS);
        assert_eq!(
            calls_to(&log, BS_EXTRA_DATA_INIT),
            vec![vec![extra.addr(), 0x5c]]
        );
        assert_eq!(e.mem.u32(extra.addr()), VTABLE_EXTRA_RADIUS);
    }

    #[test]
    fn radius_constructor() {
        let mut e = engine();
        let block: Ptr = Ptr::new(e.mem.alloc(0x10));
        let result = e.call(0x0042_22f0, &args![block, 12.5f32]).ptr::<()>();
        assert_eq!(result, block);
        assert_eq!(e.mem.u32(block.addr()), 0x0101_5208);
        assert_eq!(e.mem.u8(block.addr() + 4), 0x5c);
        assert_eq!(e.mem.f32(block.addr() + 0x0c), 12.5);
    }

    #[test]
    fn radius_getter() {
        check_float_getter(0x0042_2320, EXTRA_RADIUS);
    }

    #[test]
    fn radiation_setter() {
        check_float_setter(0x0042_2350, EXTRA_RADIATION);
        let mut e = engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_2350, &args![list, 3.0f32]);
        });
        let extra = find_extra(&mut e, list, EXTRA_RADIATION);
        assert_eq!(
            calls_to(&log, BS_EXTRA_DATA_INIT),
            vec![vec![extra.addr(), 0x5d]]
        );
        assert_eq!(e.mem.u32(extra.addr()), VTABLE_EXTRA_RADIATION);
    }

    #[test]
    fn radiation_constructor() {
        let mut e = engine();
        let block: Ptr = Ptr::new(e.mem.alloc(0x10));
        let result = e.call(0x0042_2420, &args![block, -0.5f32]).ptr::<()>();
        assert_eq!(result, block);
        assert_eq!(e.mem.u32(block.addr()), 0x0101_5214);
        assert_eq!(e.mem.u8(block.addr() + 4), 0x5d);
        assert_eq!(e.mem.f32(block.addr() + 0x0c), -0.5);
    }

    #[test]
    fn radiation_getter() {
        check_float_getter(0x0042_2450, EXTRA_RADIATION);
    }

    /// Doubles for the follower list: `LIST_CONTAINS` is true for the item
    /// 0x5555, `LIST_ADD_HEAD` records the items added.
    fn follower_engine() -> (Engine, Added) {
        let mut e = engine();
        e.register(FOLLOWER_INIT, |e, a| {
            e.mem.set_u32(a[0], VTABLE);
            e.mem.set_u8(a[0] + 4, EXTRA_FOLLOWER);
            e.mem.set_u32(a[0] + 8, 0);
            e.mem.set_u32(a[0] + 0x0c, 0xaaaa);
            returns(a[0])
        });
        e.register(LIST_CONTAINS, |e, a| {
            returns((e.mem.u32(a[1]) == 0x5555) as u32)
        });
        let added = Rc::new(RefCell::new(vec![]));
        let record = added.clone();
        e.register_double(LIST_ADD_HEAD, move |e, a| {
            record.borrow_mut().push((a[0], e.mem.u32(a[1])));
            Ret::default()
        });
        (e, added)
    }

    #[test]
    fn add_follower_builds_the_extra_data_and_adds_new_followers() {
        let (mut e, added) = follower_engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_2480, &args![list, 0x1234u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, FOLLOWER_INIT).len(), 1);
        assert_eq!(chain_types(&e, list), vec![EXTRA_FOLLOWER]);
        assert_eq!(*added.borrow(), vec![(0xaaaa, 0x1234)]);
        // The test is made on the same list and a stack slot with the item.
        assert_eq!(calls_to(&log, LIST_CONTAINS)[0][0], 0xaaaa);

        // The extra data exists now: nothing is built, a second follower is
        // added, a follower already in the list is not.
        let log = logged(&mut e, |e| {
            e.call(0x0042_2480, &args![list, 0x2345u32]);
            e.call(0x0042_2480, &args![list, 0x5555u32]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(*added.borrow(), vec![(0xaaaa, 0x1234), (0xaaaa, 0x2345)]);
    }

    #[test]
    fn add_follower_ignores_the_player() {
        let (mut e, added) = follower_engine();
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_2480, &args![list, 0x7777u32]);
        });
        // Only the top-level call: nothing else happens.
        assert_eq!(log.len(), 1);
        assert!(chain_types(&e, list).is_empty());
        assert!(added.borrow().is_empty());
    }

    #[test]
    fn follower_membership_test() {
        let (mut e, _) = follower_engine();
        let list = new_list(&mut e);
        // No extra data: false, without testing a list.
        let log = logged(&mut e, |e| {
            assert!(!e.call(0x0042_2550, &args![list, 0x5555u32]).bool());
        });
        assert!(calls_to(&log, LIST_CONTAINS).is_empty());
        add(&mut e, list, EXTRA_FOLLOWER, 0xaaaa);
        let log = logged(&mut e, |e| {
            assert!(e.call(0x0042_2550, &args![list, 0x5555u32]).bool());
            assert!(!e.call(0x0042_2550, &args![list, 0x1111u32]).bool());
        });
        let tests = calls_to(&log, LIST_CONTAINS);
        assert_eq!(tests.len(), 2);
        assert!(tests.iter().all(|a| a[0] == 0xaaaa));
    }

    #[test]
    fn friend_hit_builds_the_extra_data_then_adds_a_hit() {
        let mut e = engine();
        stub(&mut e, FRIEND_HITS_ADD_HIT);
        e.register(FRIEND_HITS_GET_HIT_COUNT, |_, _| returns(3));
        let list = new_list(&mut e);
        let log = logged(&mut e, |e| {
            e.call(0x0042_2590, &args![list]);
        });
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x1c]]);
        assert_eq!(calls_to(&log, FRIEND_HITS_INIT).len(), 1);
        let extra = find_extra(&mut e, list, EXTRA_FRIEND_HITS);
        assert_eq!(
            calls_to(&log, FRIEND_HITS_ADD_HIT),
            vec![vec![extra.addr()]]
        );
        assert_eq!(
            calls_to(&log, FRIEND_HITS_GET_HIT_COUNT),
            vec![vec![extra.addr()]]
        );

        // Existing extra data: nothing built.
        let log = logged(&mut e, |e| {
            e.call(0x0042_2590, &args![list]);
        });
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(calls_to(&log, FRIEND_HITS_ADD_HIT).len(), 1);
    }

    #[test]
    fn friend_hit_count_getter() {
        let mut e = engine();
        e.register(FRIEND_HITS_GET_HIT_COUNT, |_, _| returns(3));
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0042_2640, &args![list]).u32(), 0);
        let extra = add(&mut e, list, EXTRA_FRIEND_HITS, 0);
        let log = logged(&mut e, |e| {
            assert_eq!(e.call(0x0042_2640, &args![list]).u32(), 3);
        });
        assert_eq!(
            calls_to(&log, FRIEND_HITS_GET_HIT_COUNT),
            vec![vec![extra.addr()]]
        );
    }
}
