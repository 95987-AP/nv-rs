//! `fallout shared/extradatalist.cpp` (Xbox PDB source unit), part 5: its functions from `0042dd90` up to
//! (not including) `ffffffff` in FalloutNV.exe 1.4.0.525, translated
//! (docs/ENGINE_CRATE.md). The unit's shared layouts and helpers are in
//! [`super::extradatalist`]; anything public there may be used here.
//!
//! First session (40 functions, `0042dd90` to `0042ea50`): the hot key,
//! info general topic, talking actor, model swap, navmesh portal, weapon mod
//! slot / is-modding, faction changes, dismembered limbs, actor cause and
//! combat style extra data (getters, setters, removers, and the small
//! constructors and destructors the compiler placed here), with the
//! `ExtraDataList` function at `0042dd90` that drops the saved animation,
//! havok data and last finished sequence. The next session continues at
//! `0042eb10`.
//!
//! The extra data type numbers are `EXTRA_DATA_TYPE` of the Xbox PDB
//! (`0x4A` `EXTRA_HOT_KEY`, `0x4D` `EXTRA_INFO_GENERAL_TOPIC`, ...). The
//! compiler's exception-unwinding frames (the `FS:[0]` chains of the setters
//! that call `new`) are not translated. Every setter allocates with
//! `operator new` and, as the code does, carries a failed allocation (a null
//! block) on to `AddExtra`.

#[allow(unused_imports)]
use super::extradatalist::*;
#[allow(unused_imports)]
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Constants

/// `operator new(size)`.
const OPERATOR_NEW: u32 = 0x0040_1000;
/// `operator delete(block)` (`platform`).
const OPERATOR_DELETE: u32 = 0x0040_1030;
/// `BSExtraData::BSExtraData(type)`: sets the base vtable, the type, and a
/// null next.
const BS_EXTRA_DATA_INIT: u32 = 0x0040_ec80;
/// `BSExtraData::~BSExtraData` (`0040ecb0`).
const BS_EXTRA_DATA_DESTROY: u32 = 0x0040_ecb0;
/// Count of the non-null items of a `BSSimpleList` (`005ae380`).
const LIST_COUNT: u32 = 0x005a_e380;

/// `ExtraDataList::RemoveSavedAnimation` (`00422aa0`).
const REMOVE_SAVED_ANIMATION: u32 = 0x0042_2aa0;
/// `ExtraDataList::RemoveSavedHavokData` (`00422c20`).
const REMOVE_SAVED_HAVOK_DATA: u32 = 0x0042_2c20;
/// `ExtraDataList::RemoveLastFinishedSequence` (`00422920`).
const REMOVE_LAST_FINISHED_SEQUENCE: u32 = 0x0042_2920;
/// `MOV EAX,[ECX+0x2C]; MOV EDX,[ESP+4]; MOV [EDX],EAX`: copies the word at
/// +0x2C of `this` (a form's flags word) into the slot it is given and
/// returns that slot (`0042ce30`).
const COPY_FLAGS_WORD: u32 = 0x0042_ce30;
/// Whether `[this] & mask` is nonzero (`004280f0`, `this` a pointer to a
/// flags word).
const FLAGS_TEST: u32 = 0x0042_80f0;
/// The routine `fn_0042e010` forwards to, with the words it reads from the
/// `MenuTopic` (`0083df80`, `this` = the `MenuTopic`, four words).
const MENU_TOPIC_FORWARD: u32 = 0x0083_df80;

/// Constructor of the type `0x4A` extra data (`ExtraHotKey`, `00432380`,
/// `this` = the new block, then the hot key byte zero-extended).
const EXTRA_HOT_KEY_INIT: u32 = 0x0043_2380;
/// Constructor of the type `0x4D` extra data (`ExtraInfoGeneralTopic`,
/// `00432470`).
const EXTRA_INFO_GENERAL_TOPIC_INIT: u32 = 0x0043_2470;
/// `ExtraTalkingActor::ExtraTalkingActor` (Xbox PDB, `00436b20`; two words).
const EXTRA_TALKING_ACTOR_INIT: u32 = 0x0043_6b20;
/// Copy of the 8-byte value at +0x0C of the navmesh portal record
/// (`0069a690`, `this` = the copy).
const NAVMESH_PORTAL_COPY: u32 = 0x0069_a690;
/// Constructor of the type `0x8D` extra data without argument (`0042cc40`).
const EXTRA_WEAPON_MOD_SLOTS_INIT: u32 = 0x0042_cc40;
/// Reads the byte at +0x0C of the type `0x8D` extra data (`00424940`).
const WEAPON_MOD_FLAGS_READ: u32 = 0x0042_4940;
/// Constructor of the type `0x5E` extra data (`00436cb0`).
const EXTRA_FACTION_CHANGES_INIT: u32 = 0x0043_6cb0;
/// `ExtraDismemberedLimbs::ExtraDismemberedLimbs` (Xbox PDB, `00430200`).
const EXTRA_DISMEMBERED_LIMBS_INIT: u32 = 0x0043_0200;
/// Constructor of the type `0x60` extra data (`ExtraActorCause`, `0042ca90`).
const EXTRA_ACTOR_CAUSE_INIT: u32 = 0x0042_ca90;
/// `NiPointer<ActorCause>::operator=` (Xbox PDB, `0042f780`; `this` = the
/// smart pointer, then the new pointer).
const ACTOR_CAUSE_POINTER_ASSIGN: u32 = 0x0042_f780;
/// Constructor of the type `0x69` extra data (`ExtraCombatStyle`, `0042eb10`).
const EXTRA_COMBAT_STYLE_INIT: u32 = 0x0042_eb10;

/// Extra data types (`EXTRA_DATA_TYPE` of the Xbox PDB) of this part.
const EXTRA_USED_MARKERS: u8 = 0x12;
const EXTRA_HOT_KEY: u8 = 0x4a;
const EXTRA_INFO_GENERAL_TOPIC: u8 = 0x4d;
const EXTRA_TALKING_ACTOR: u8 = 0x55;
const EXTRA_NAVMESH_PORTAL: u8 = 0x5a;
const EXTRA_MODEL_SWAP: u8 = 0x5b;
const EXTRA_FACTION_CHANGES: u8 = 0x5e;
const EXTRA_DISMEMBERED_LIMBS: u8 = 0x5f;
const EXTRA_ACTOR_CAUSE: u8 = 0x60;
const EXTRA_COMBAT_STYLE: u8 = 0x69;
const EXTRA_WEAPON_MOD_SLOTS: u8 = 0x8d;
const EXTRA_WEAPON_IS_MODDING: u8 = 0x8e;

/// Vtables set by the constructors of this part.
const VTABLE_EXTRA_MODEL_SWAP: u32 = 0x0101_5980;
const VTABLE_EXTRA_WEAPON_MOD_SLOTS: u32 = 0x0101_59a4;
const VTABLE_EXTRA_WEAPON_IS_MODDING: u32 = 0x0101_59bc;

// ---------------------------------------------------------------------------
// Helpers

/// The first extra data of `extra_type` in the list (`GetExtraData`).
fn find_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> Ptr<BSExtraData> {
    base_extra_list_get_extra_data(e, list.cast(), extra_type)
}

/// `new` and construct: allocates `size` bytes and runs `construct` on the
/// block, or gives null when the allocation failed.
fn new_object(e: &mut Engine, size: u32, construct: impl FnOnce(&mut Engine, u32) -> u32) -> u32 {
    let block = e.call(OPERATOR_NEW, &args![size]).u32();
    if block == 0 {
        0
    } else {
        construct(e, block)
    }
}

/// A new extra data of `size` bytes, built by the constructor at `construct`
/// (`this` = the block, then `construct_args`). Returns the extra data
/// (null when the allocation failed).
fn build_extra(
    e: &mut Engine,
    size: u32,
    construct: u32,
    construct_args: &[u32],
) -> Ptr<BSExtraData> {
    Ptr::new(new_object(e, size, |e, block| {
        let mut words = vec![block];
        words.extend_from_slice(construct_args);
        e.call(construct, &words).u32()
    }))
}

/// `AddExtra` of the list.
fn add_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra: Ptr<BSExtraData>) {
    base_extra_list_add_extra(e, list.cast(), extra);
}

/// The shape of the getters of a pointer or word: the word at +0x0C of the
/// first extra data of `extra_type`, or 0 when the list has none.
fn extra_word(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        0
    } else {
        e.mem.u32(extra.addr() + 0x0c)
    }
}

/// `if (extra = GetExtraData(type)) RemoveExtra(extra, true)`.
fn remove_if_present(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) {
    let extra = find_extra(e, list, extra_type);
    if !extra.is_null() {
        base_extra_list_remove_extra(e, list.cast(), extra, true);
    }
}

// Translated from 0042dd90 (decompiled, FalloutNV.exe 1.4.0.525)
/// A function of `ExtraDataList` (`this`) called with a form: runs
/// `RemoveSavedAnimation` (`00422aa0`), `RemoveSavedHavokData` (`00422c20`)
/// and `RemoveLastFinishedSequence` (`00422920`), then, when bit 31 of the
/// form's flags word (the word at +0x2C, copied through `0042ce30` and tested
/// by `004280f0`) is set, removes the type `0x12` extra data
/// (`EXTRA_USEDMARKERS` in the Xbox enum). Its only caller is `005629a0`.
pub fn fn_0042dd90(e: &mut Engine, this: Ptr<ExtraDataList>, form: u32) {
    e.call(REMOVE_SAVED_ANIMATION, &args![this]);
    e.call(REMOVE_SAVED_HAVOK_DATA, &args![this]);
    e.call(REMOVE_LAST_FINISHED_SEQUENCE, &args![this]);
    let flagged = e.with_stack(4, |e, copy| {
        let copied = e.call(COPY_FLAGS_WORD, &args![form, copy]).u32();
        e.call(FLAGS_TEST, &args![copied, 0x8000_0000u32]).bool()
    });
    if flagged {
        base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_USED_MARKERS);
    }
}

// Translated from 0042dde0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetHotKey` (Xbox PDB): stores `hot_key` (a byte at +0x0C)
/// in the type `0x4A` extra data (`ExtraHotKey`), or builds one (`0x10`
/// bytes, `00432380`, given the byte zero-extended) and adds it.
pub fn extra_data_list_set_hot_key(e: &mut Engine, this: Ptr<ExtraDataList>, hot_key: u8) {
    let extra = find_extra(e, this, EXTRA_HOT_KEY);
    if extra.is_null() {
        let extra = build_extra(e, 0x10, EXTRA_HOT_KEY_INIT, &[hot_key as u32]);
        add_extra(e, this, extra);
    } else {
        e.mem.set_u8(extra.addr() + 0x0c, hot_key);
    }
}

// Translated from 0042de90 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetHotKey` (Xbox PDB): the byte at +0x0C of the type
/// `0x4A` extra data, or `0xFF` when the list has none.
pub fn extra_data_list_get_hot_key(e: &mut Engine, this: Ptr<ExtraDataList>) -> u8 {
    let extra = find_extra(e, this, EXTRA_HOT_KEY);
    if extra.is_null() {
        0xff
    } else {
        e.mem.u8(extra.addr() + 0x0c)
    }
}

// Translated from 0042dec0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveHotKey` (Xbox PDB): `RemoveExtra` by type `0x4A`.
pub fn extra_data_list_remove_hot_key(e: &mut Engine, this: Ptr<ExtraDataList>) {
    base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_HOT_KEY);
}

// Translated from 0042dee0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetInfoGeneralTopic` (Xbox PDB): stores `topic`
/// (`pInfoGen`, a `MenuTopic*` at +0x0C) in the type `0x4D` extra data
/// (`ExtraInfoGeneralTopic`), or builds one (`0x10` bytes, `00432470`) and
/// adds it.
pub fn extra_data_list_set_info_general_topic(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
    topic: u32,
) {
    let extra = find_extra(e, this, EXTRA_INFO_GENERAL_TOPIC);
    if extra.is_null() {
        let extra = build_extra(e, 0x10, EXTRA_INFO_GENERAL_TOPIC_INIT, &[topic]);
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(extra.addr() + 0x0c, topic);
    }
}

// Translated from 0042df90 (decompiled, FalloutNV.exe 1.4.0.525)
/// The topic of the type `0x4D` extra data (the word at +0x0C, a
/// `MenuTopic*`), or 0 when there is none. A non-null topic that
/// `fn_0042dff0` finds empty is also passed to `fn_0042e010` with `argument`.
pub fn fn_0042df90(e: &mut Engine, this: Ptr<ExtraDataList>, argument: u32) -> u32 {
    let extra = find_extra(e, this, EXTRA_INFO_GENERAL_TOPIC);
    if extra.is_null() {
        return 0;
    }
    let topic = e.mem.u32(extra.addr() + 0x0c);
    if topic != 0 && fn_0042dff0(e, Ptr::new(topic)) {
        fn_0042e010(e, Ptr::new(topic), argument);
    }
    topic
}

// Translated from 0042dff0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Whether the `BSSimpleList` at +0x0C of the `MenuTopic` has no item
/// (its count, `005ae380`, is 0).
pub fn fn_0042dff0(e: &mut Engine, this: Ptr) -> bool {
    e.call(LIST_COUNT, &args![this.addr() + 0x0c]).u32() == 0
}

// Translated from 0042e010 (decompiled, FalloutNV.exe 1.4.0.525)
/// Calls `0083df80` on the `MenuTopic` with the words at +0x14, +0x28 and
/// +0x18 of it, then `argument`.
pub fn fn_0042e010(e: &mut Engine, this: Ptr, argument: u32) {
    let first = e.mem.u32(this.addr() + 0x14);
    let second = e.mem.u32(this.addr() + 0x28);
    let third = e.mem.u32(this.addr() + 0x18);
    e.call(
        MENU_TOPIC_FORWARD,
        &args![this, first, second, third, argument],
    );
}

// Translated from 0042e040 (decompiled, FalloutNV.exe 1.4.0.525)
/// Remover of the type `0x4D` extra data: `RemoveExtra` by type.
pub fn fn_0042e040(e: &mut Engine, this: Ptr<ExtraDataList>) {
    base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_INFO_GENERAL_TOPIC);
}

// Translated from 0042e060 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetTalkingActorExtra` (Xbox PDB): deletes an existing type
/// `0x55` extra data (`ExtraTalkingActor`), then always builds a new one
/// (`0x10` bytes, `00436b20`, given `first` and `second`) and adds it.
pub fn extra_data_list_set_talking_actor_extra(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
    first: u32,
    second: u32,
) {
    remove_if_present(e, this, EXTRA_TALKING_ACTOR);
    let extra = build_extra(e, 0x10, EXTRA_TALKING_ACTOR_INIT, &[first, second]);
    add_extra(e, this, extra);
}

// Translated from 0042e110 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetTalkingActorExtra` (Xbox PDB): the type `0x55` extra
/// data, or null.
pub fn extra_data_list_get_talking_actor_extra(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSExtraData> {
    find_extra(e, this, EXTRA_TALKING_ACTOR)
}

// Translated from 0042e130 (decompiled, FalloutNV.exe 1.4.0.525)
/// Remover of the type `0x55` extra data: `RemoveExtra` by type.
pub fn fn_0042e130(e: &mut Engine, this: Ptr<ExtraDataList>) {
    base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_TALKING_ACTOR);
}

// Translated from 0042e150 (decompiled, FalloutNV.exe 1.4.0.525)
/// Setter of the type `0x5B` extra data (`ExtraModelSwap`: `pModelSwap` at
/// +0x0C and `pModelSwapForm` at +0x10, Xbox PDB): stores both words in an
/// existing one, or builds one (`0x14` bytes, `fn_0042e210`) and adds it.
pub fn fn_0042e150(e: &mut Engine, this: Ptr<ExtraDataList>, model_swap: u32, swap_form: u32) {
    let extra = find_extra(e, this, EXTRA_MODEL_SWAP);
    if extra.is_null() {
        let extra = Ptr::new(new_object(e, 0x14, |e, block| {
            fn_0042e210(e, Ptr::new(block), model_swap, swap_form).addr()
        }));
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(extra.addr() + 0x0c, model_swap);
        e.mem.set_u32(extra.addr() + 0x10, swap_form);
    }
}

// Translated from 0042e210 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of `ExtraModelSwap` (Xbox PDB): a `BSExtraData` of type `0x5B`
/// with `pModelSwap` and `pModelSwapForm` set. Returns `this`.
pub fn fn_0042e210(e: &mut Engine, this: Ptr, model_swap: u32, swap_form: u32) -> Ptr {
    e.call(BS_EXTRA_DATA_INIT, &args![this, EXTRA_MODEL_SWAP as u32]);
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_MODEL_SWAP);
    e.mem.set_u32(this.addr() + 0x0c, model_swap);
    e.mem.set_u32(this.addr() + 0x10, swap_form);
    this
}

// Translated from 0042e250 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetModelSwap` (Xbox PDB): `pModelSwap` (the word at +0x0C)
/// of the type `0x5B` extra data, or 0.
pub fn extra_data_list_get_model_swap(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word(e, this, EXTRA_MODEL_SWAP)
}

// Translated from 0042e280 (decompiled, FalloutNV.exe 1.4.0.525)
/// Remover of the type `0x5B` extra data: `RemoveExtra` by type.
pub fn fn_0042e280(e: &mut Engine, this: Ptr<ExtraDataList>) {
    base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_MODEL_SWAP);
}

// Translated from 0042e2a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetNavMeshPortal` (Xbox PDB): the type `0x5A` extra data
/// (`EXTRA_NAVMESH_PORTAL`), or null.
pub fn extra_data_list_get_nav_mesh_portal(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSExtraData> {
    find_extra(e, this, EXTRA_NAVMESH_PORTAL)
}

// Translated from 0042e2c0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Setter of the type `0x5A` extra data (the engine map leaves it unnamed;
/// `ExtraDataList::GetNavMeshPortal` and `RemoveNavMeshPortal` are its
/// neighbours): when the list has none, builds one (`0x14` bytes) whose
/// 8-byte member is a copy (`0069a690`) of the 8 bytes at +0x0C of `portal`,
/// handed by value to its constructor (`fn_00416ad0`), and adds it. An
/// existing extra data is left as it is.
pub fn fn_0042e2c0(e: &mut Engine, this: Ptr<ExtraDataList>, portal: u32) {
    let extra = find_extra(e, this, EXTRA_NAVMESH_PORTAL);
    if !extra.is_null() {
        return;
    }
    let extra = Ptr::new(new_object(e, 0x14, |e, block| {
        e.with_stack(8, |e, copy| {
            e.call(NAVMESH_PORTAL_COPY, &args![copy, portal + 0x0c]);
            let low = e.mem.u32(copy.addr());
            let high = e.mem.u32(copy.addr() + 4);
            fn_00416ad0(e, Ptr::new(block), low, high).addr()
        })
    }));
    add_extra(e, this, extra);
}

// Translated from 0042e380 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetWeaponModSlot` (Xbox PDB): ORs the bits `slot` into the
/// byte at +0x0C of the type `0x8D` extra data (`ExtraWeaponModFlags`,
/// `cWeaponModsActive`). With none in the list and a nonzero `slot`, builds
/// one (`0x10` bytes, `fn_0042e450`), ORs the bits in, and adds it. With none
/// and `slot` 0 the code still calls `fn_0042e480` on the null extra data,
/// which reads memory at +0x0C of address 0 (the game would fault).
pub fn extra_data_list_set_weapon_mod_slot(e: &mut Engine, this: Ptr<ExtraDataList>, slot: u8) {
    let extra = find_extra(e, this, EXTRA_WEAPON_MOD_SLOTS);
    if extra.is_null() && slot != 0 {
        let extra: Ptr = Ptr::new(new_object(e, 0x10, |e, block| {
            fn_0042e450(e, Ptr::new(block), slot).addr()
        }));
        fn_0042e480(e, extra, slot);
        add_extra(e, this, extra.cast());
    } else {
        fn_0042e480(e, extra.cast(), slot);
    }
}

// Translated from 0042e450 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of `ExtraWeaponModFlags` (Xbox PDB), type `0x8D`: sets
/// `cWeaponModsActive` (byte at +0x0C) to `slot`. Returns `this`.
pub fn fn_0042e450(e: &mut Engine, this: Ptr, slot: u8) -> Ptr {
    e.call(
        BS_EXTRA_DATA_INIT,
        &args![this, EXTRA_WEAPON_MOD_SLOTS as u32],
    );
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_WEAPON_MOD_SLOTS);
    e.mem.set_u8(this.addr() + 0x0c, slot);
    this
}

// Translated from 0042e480 (decompiled, FalloutNV.exe 1.4.0.525)
/// ORs `bits` into the byte at +0x0C of the extra data.
pub fn fn_0042e480(e: &mut Engine, this: Ptr, bits: u8) {
    let flags = e.mem.u8(this.addr() + 0x0c);
    e.mem.set_u8(this.addr() + 0x0c, flags | bits);
}

// Translated from 0042e4a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets the byte at +0x0C of the type `0x8D` extra data to `value`
/// (`fn_0042e680`); with none in the list, builds one (`0x10` bytes,
/// `0042cc40`), sets the byte, and adds it.
pub fn fn_0042e4a0(e: &mut Engine, this: Ptr<ExtraDataList>, value: u8) {
    let extra = find_extra(e, this, EXTRA_WEAPON_MOD_SLOTS);
    if extra.is_null() {
        let extra: Ptr = build_extra(e, 0x10, EXTRA_WEAPON_MOD_SLOTS_INIT, &[]).cast();
        fn_0042e680(e, extra, value);
        add_extra(e, this, extra.cast());
    } else {
        fn_0042e680(e, extra.cast(), value);
    }
}

// Translated from 0042e560 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetWeaponModFlags` (Xbox PDB): the byte of the type `0x8D`
/// extra data (read by `00424940`), or 0 when the list has none.
pub fn extra_data_list_get_weapon_mod_flags(e: &mut Engine, this: Ptr<ExtraDataList>) -> u8 {
    let extra = find_extra(e, this, EXTRA_WEAPON_MOD_SLOTS);
    if extra.is_null() {
        0
    } else {
        e.call(WEAPON_MOD_FLAGS_READ, &args![extra]).u8()
    }
}

// Translated from 0042e5a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetIsModding` (Xbox PDB): stores `modding` (a byte at
/// +0x0C, `fn_0042e680`) in the type `0x8E` extra data
/// (`EXTRA_WEAPON_IS_MODDING`), or builds one (`0x10` bytes, `fn_0042e650`)
/// and adds it.
pub fn extra_data_list_set_is_modding(e: &mut Engine, this: Ptr<ExtraDataList>, modding: u8) {
    let extra = find_extra(e, this, EXTRA_WEAPON_IS_MODDING);
    if extra.is_null() {
        let extra = Ptr::new(new_object(e, 0x10, |e, block| {
            fn_0042e650(e, Ptr::new(block), modding).addr()
        }));
        add_extra(e, this, extra);
    } else {
        fn_0042e680(e, extra.cast(), modding);
    }
}

// Translated from 0042e650 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of the type `0x8E` extra data: sets the byte at +0x0C to
/// `modding`. Returns `this`.
pub fn fn_0042e650(e: &mut Engine, this: Ptr, modding: u8) -> Ptr {
    e.call(
        BS_EXTRA_DATA_INIT,
        &args![this, EXTRA_WEAPON_IS_MODDING as u32],
    );
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_WEAPON_IS_MODDING);
    e.mem.set_u8(this.addr() + 0x0c, modding);
    this
}

// Translated from 0042e680 (decompiled, FalloutNV.exe 1.4.0.525)
/// Stores `value` in the byte at +0x0C of the extra data.
pub fn fn_0042e680(e: &mut Engine, this: Ptr, value: u8) {
    e.mem.set_u8(this.addr() + 0x0c, value);
}

// Translated from 0042e6a0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Scalar deleting destructor of the type `0x8E` extra data: the destructor
/// (`fn_0042e6d0`), then `operator delete` when bit 0 of `flags` is set.
/// Returns `this`.
pub fn fn_0042e6a0(e: &mut Engine, this: Ptr, flags: u32) -> Ptr {
    fn_0042e6d0(e, this);
    if flags & 1 != 0 {
        e.call(OPERATOR_DELETE, &args![this]);
    }
    this
}

// Translated from 0042e6d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Destructor of the type `0x8E` extra data: sets its vtable, then runs
/// `BSExtraData::~BSExtraData` (`0040ecb0`).
pub fn fn_0042e6d0(e: &mut Engine, this: Ptr) {
    e.mem.set_u32(this.addr(), VTABLE_EXTRA_WEAPON_IS_MODDING);
    e.call(BS_EXTRA_DATA_DESTROY, &args![this]);
}

// Translated from 0042e6f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveIsModding` (Xbox PDB): deletes the type `0x8E`
/// extra data if the list has one.
pub fn extra_data_list_remove_is_modding(e: &mut Engine, this: Ptr<ExtraDataList>) {
    remove_if_present(e, this, EXTRA_WEAPON_IS_MODDING);
}

// Translated from 0042e730 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveNavMeshPortal` (Xbox PDB): deletes the type `0x5A`
/// extra data if the list has one.
pub fn extra_data_list_remove_nav_mesh_portal(e: &mut Engine, this: Ptr<ExtraDataList>) {
    remove_if_present(e, this, EXTRA_NAVMESH_PORTAL);
}

// Translated from 0042e760 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds a type `0x5E` extra data (`EXTRA_FACTION_CHANGES` in the Xbox enum;
/// `0x10` bytes, constructor `00436cb0`) when the list has none.
pub fn fn_0042e760(e: &mut Engine, this: Ptr<ExtraDataList>) {
    let extra = find_extra(e, this, EXTRA_FACTION_CHANGES);
    if extra.is_null() {
        let extra = build_extra(e, 0x10, EXTRA_FACTION_CHANGES_INIT, &[]);
        add_extra(e, this, extra);
    }
}

// Translated from 0042e800 (decompiled, FalloutNV.exe 1.4.0.525)
/// The type `0x5E` extra data, or null.
pub fn fn_0042e800(e: &mut Engine, this: Ptr<ExtraDataList>) -> Ptr<BSExtraData> {
    find_extra(e, this, EXTRA_FACTION_CHANGES)
}

// Translated from 0042e820 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::AddDismembermentExtra` (Xbox PDB): the type `0x5F` extra
/// data (`ExtraDismemberedLimbs`), built (`0x30` bytes, `00430200`) and added
/// first when the list has none. Returns it.
pub fn extra_data_list_add_dismemberment_extra(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSExtraData> {
    let mut extra = find_extra(e, this, EXTRA_DISMEMBERED_LIMBS);
    if extra.is_null() {
        extra = build_extra(e, 0x30, EXTRA_DISMEMBERED_LIMBS_INIT, &[]);
        add_extra(e, this, extra);
    }
    extra
}

// Translated from 0042e8c0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetDismembermentExtra` (Xbox PDB): the type `0x5F` extra
/// data, or null.
pub fn extra_data_list_get_dismemberment_extra(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSExtraData> {
    find_extra(e, this, EXTRA_DISMEMBERED_LIMBS)
}

// Translated from 0042e8e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveDismembermentExtra` (Xbox PDB): deletes the type
/// `0x5F` extra data if the list has one.
pub fn extra_data_list_remove_dismemberment_extra(e: &mut Engine, this: Ptr<ExtraDataList>) {
    remove_if_present(e, this, EXTRA_DISMEMBERED_LIMBS);
}

// Translated from 0042e910 (decompiled, FalloutNV.exe 1.4.0.525)
/// The type `0x60` extra data (`EXTRA_ACTOR_CAUSE` in the Xbox enum), or
/// null.
pub fn fn_0042e910(e: &mut Engine, this: Ptr<ExtraDataList>) -> Ptr<BSExtraData> {
    find_extra(e, this, EXTRA_ACTOR_CAUSE)
}

// Translated from 0042e930 (decompiled, FalloutNV.exe 1.4.0.525)
/// Setter of the actor cause of the type `0x60` extra data
/// (`ExtraActorCause`): a null `cause` removes the extra data by type;
/// otherwise the cause is assigned (`fn_0042ea00`) to an existing one, or to
/// a new one (`0x10` bytes, `0042ca90`) that is then added.
pub fn fn_0042e930(e: &mut Engine, this: Ptr<ExtraDataList>, cause: u32) {
    if cause == 0 {
        base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_ACTOR_CAUSE);
        return;
    }
    let extra = find_extra(e, this, EXTRA_ACTOR_CAUSE);
    if extra.is_null() {
        let extra: Ptr = build_extra(e, 0x10, EXTRA_ACTOR_CAUSE_INIT, &[]).cast();
        fn_0042ea00(e, extra, cause);
        add_extra(e, this, extra.cast());
    } else {
        fn_0042ea00(e, extra.cast(), cause);
    }
}

// Translated from 0042ea00 (decompiled, FalloutNV.exe 1.4.0.525)
/// Assigns `cause` to the `NiPointer<ActorCause>` (`spActorCause`) at +0x0C
/// of the extra data (`NiPointer<ActorCause>::operator=`, `0042f780`).
pub fn fn_0042ea00(e: &mut Engine, this: Ptr, cause: u32) {
    e.call(
        ACTOR_CAUSE_POINTER_ASSIGN,
        &args![this.addr() + 0x0c, cause],
    );
}

// Translated from 0042ea20 (decompiled, FalloutNV.exe 1.4.0.525)
/// `pCombatStyle` (the word at +0x0C) of the type `0x69` extra data
/// (`ExtraCombatStyle`), or 0.
pub fn fn_0042ea20(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word(e, this, EXTRA_COMBAT_STYLE)
}

// Translated from 0042ea50 (decompiled, FalloutNV.exe 1.4.0.525)
/// Setter of the combat style of the type `0x69` extra data: a null
/// `combat_style` removes the extra data by type; otherwise it is stored in
/// an existing one, or a new one (`0x10` bytes, `0042eb10`) is built and
/// added.
pub fn fn_0042ea50(e: &mut Engine, this: Ptr<ExtraDataList>, combat_style: u32) {
    if combat_style == 0 {
        base_extra_list_remove_extra_ov2(e, this.cast(), EXTRA_COMBAT_STYLE);
        return;
    }
    let extra = find_extra(e, this, EXTRA_COMBAT_STYLE);
    if extra.is_null() {
        let extra = build_extra(e, 0x10, EXTRA_COMBAT_STYLE_INIT, &[combat_style]);
        add_extra(e, this, extra);
    } else {
        e.mem.set_u32(extra.addr() + 0x0c, combat_style);
    }
}

/// This part's translated functions, by exe address.
pub fn funcs() -> Vec<(u32, AbiFn)> {
    vec![
        entry!(0x0042dd90, fn_0042dd90(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0042dde0,
            extra_data_list_set_hot_key(Ptr<ExtraDataList>, u8)
        ),
        entry!(
            0x0042de90,
            extra_data_list_get_hot_key(Ptr<ExtraDataList>) -> u8
        ),
        entry!(
            0x0042dec0,
            extra_data_list_remove_hot_key(Ptr<ExtraDataList>)
        ),
        entry!(
            0x0042dee0,
            extra_data_list_set_info_general_topic(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x0042df90, fn_0042df90(Ptr<ExtraDataList>, u32) -> u32),
        entry!(0x0042dff0, fn_0042dff0(Ptr) -> bool),
        entry!(0x0042e010, fn_0042e010(Ptr, u32)),
        entry!(0x0042e040, fn_0042e040(Ptr<ExtraDataList>)),
        entry!(
            0x0042e060,
            extra_data_list_set_talking_actor_extra(Ptr<ExtraDataList>, u32, u32)
        ),
        entry!(
            0x0042e110,
            extra_data_list_get_talking_actor_extra(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(0x0042e130, fn_0042e130(Ptr<ExtraDataList>)),
        entry!(0x0042e150, fn_0042e150(Ptr<ExtraDataList>, u32, u32)),
        entry!(0x0042e210, fn_0042e210(Ptr, u32, u32) -> Ptr),
        entry!(
            0x0042e250,
            extra_data_list_get_model_swap(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x0042e280, fn_0042e280(Ptr<ExtraDataList>)),
        entry!(
            0x0042e2a0,
            extra_data_list_get_nav_mesh_portal(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(0x0042e2c0, fn_0042e2c0(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0042e380,
            extra_data_list_set_weapon_mod_slot(Ptr<ExtraDataList>, u8)
        ),
        entry!(0x0042e450, fn_0042e450(Ptr, u8) -> Ptr),
        entry!(0x0042e480, fn_0042e480(Ptr, u8)),
        entry!(0x0042e4a0, fn_0042e4a0(Ptr<ExtraDataList>, u8)),
        entry!(
            0x0042e560,
            extra_data_list_get_weapon_mod_flags(Ptr<ExtraDataList>) -> u8
        ),
        entry!(
            0x0042e5a0,
            extra_data_list_set_is_modding(Ptr<ExtraDataList>, u8)
        ),
        entry!(0x0042e650, fn_0042e650(Ptr, u8) -> Ptr),
        entry!(0x0042e680, fn_0042e680(Ptr, u8)),
        entry!(0x0042e6a0, fn_0042e6a0(Ptr, u32) -> Ptr),
        entry!(0x0042e6d0, fn_0042e6d0(Ptr)),
        entry!(
            0x0042e6f0,
            extra_data_list_remove_is_modding(Ptr<ExtraDataList>)
        ),
        entry!(
            0x0042e730,
            extra_data_list_remove_nav_mesh_portal(Ptr<ExtraDataList>)
        ),
        entry!(0x0042e760, fn_0042e760(Ptr<ExtraDataList>)),
        entry!(
            0x0042e800,
            fn_0042e800(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(
            0x0042e820,
            extra_data_list_add_dismemberment_extra(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(
            0x0042e8c0,
            extra_data_list_get_dismemberment_extra(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(
            0x0042e8e0,
            extra_data_list_remove_dismemberment_extra(Ptr<ExtraDataList>)
        ),
        entry!(
            0x0042e910,
            fn_0042e910(Ptr<ExtraDataList>) -> Ptr<BSExtraData>
        ),
        entry!(0x0042e930, fn_0042e930(Ptr<ExtraDataList>, u32)),
        entry!(0x0042ea00, fn_0042ea00(Ptr, u32)),
        entry!(0x0042ea20, fn_0042ea20(Ptr<ExtraDataList>) -> u32),
        entry!(0x0042ea50, fn_0042ea50(Ptr<ExtraDataList>, u32)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    type Log = Vec<(u32, Vec<u32>)>;

    /// Test vtable of the extra data (slot 0 the scalar deleting destructor).
    const VTABLE: u32 = 0x0200_0000;
    const DESTRUCTOR: u32 = 0x0200_1000;
    // The list machinery the unit's main file calls (private there).
    const LOCK: u32 = 0x0040_fbf0;
    const UNLOCK: u32 = 0x0040_fba0;
    const GET_TYPE: u32 = 0x004f_1540;
    const GET_NEXT: u32 = 0x0044_ddc0;
    const SET_NEXT: u32 = 0x0040_3550;
    const MEMSET: u32 = 0x0040_3d30;
    /// `MOV EAX,[ECX+4]`: the head of the chain, where the by-type remover
    /// starts its walk.
    const LIST_HEAD: u32 = 0x0072_6070;

    /// The constructors of the extra data these setters build: address and
    /// type. Each double sets the test vtable, the type, a null next and the
    /// words it is given at +0x0C.
    const CONSTRUCTORS: [(u32, u8); 8] = [
        (EXTRA_HOT_KEY_INIT, EXTRA_HOT_KEY),
        (EXTRA_INFO_GENERAL_TOPIC_INIT, EXTRA_INFO_GENERAL_TOPIC),
        (EXTRA_TALKING_ACTOR_INIT, EXTRA_TALKING_ACTOR),
        (EXTRA_WEAPON_MOD_SLOTS_INIT, EXTRA_WEAPON_MOD_SLOTS),
        (EXTRA_FACTION_CHANGES_INIT, EXTRA_FACTION_CHANGES),
        (EXTRA_DISMEMBERED_LIMBS_INIT, EXTRA_DISMEMBERED_LIMBS),
        (EXTRA_ACTOR_CAUSE_INIT, EXTRA_ACTOR_CAUSE),
        (EXTRA_COMBAT_STYLE_INIT, EXTRA_COMBAT_STYLE),
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

    /// An engine with working doubles for the callees of the list
    /// operations (type and next accessors, lock, `memset`), `operator new`,
    /// the `BSExtraData` constructor, and the constructors of the extra data
    /// these functions build.
    fn engine() -> Engine {
        let mut e = Engine::new();
        e.map(0x011c_3000, 0x1000);
        e.put_vtable(VTABLE, &[DESTRUCTOR]);
        stub(&mut e, DESTRUCTOR);
        e.register(GET_TYPE, |e, a| returns(e.mem.u8(a[0] + 4) as u32));
        e.register(GET_NEXT, |e, a| returns(e.mem.u32(a[0] + 8)));
        e.register(SET_NEXT, |e, a| {
            e.mem.set_u32(a[0] + 8, a[1]);
            Ret::default()
        });
        e.register(MEMSET, |e, a| {
            for offset in 0..a[2] {
                e.mem.set_u8(a[0] + offset, a[1] as u8);
            }
            Ret::default()
        });
        e.register(LIST_HEAD, |e, a| returns(e.mem.u32(a[0] + 4)));
        stub(&mut e, LOCK);
        stub(&mut e, UNLOCK);
        stub(&mut e, OPERATOR_DELETE);
        e.register(OPERATOR_NEW, |e, a| returns(e.mem.alloc(a[0])));
        e.register(BS_EXTRA_DATA_INIT, |e, a| {
            e.mem.set_u8(a[0] + 4, a[1] as u8);
            e.mem.set_u32(a[0] + 8, 0);
            returns(a[0])
        });
        for (address, extra_type) in CONSTRUCTORS {
            e.register_double(address, move |e, a| {
                e.mem.set_u32(a[0], VTABLE);
                e.mem.set_u8(a[0] + 4, extra_type);
                e.mem.set_u32(a[0] + 8, 0);
                for (index, word) in a[1..].iter().enumerate() {
                    e.mem.set_u32(a[0] + 0x0c + 4 * index as u32, *word);
                }
                returns(a[0])
            });
        }
        e
    }

    /// A list holding extra data of the given types with the given word at
    /// +0x0C, added through `AddExtra`.
    fn list_with(e: &mut Engine, items: &[(u8, u32)]) -> Ptr<ExtraDataList> {
        let list: Ptr<ExtraDataList> = e.new_object();
        for &(extra_type, word) in items {
            let extra: Ptr<BSExtraData> = Ptr::new(e.mem.alloc(0x40));
            e.mem.set_u32(extra.addr(), VTABLE);
            e.set(extra, BSExtraData::cEtype, extra_type);
            e.mem.set_u32(extra.addr() + 0x0c, word);
            base_extra_list_add_extra(e, list.cast(), extra);
        }
        list
    }

    /// The types of the chain, in order.
    fn chain_types(e: &Engine, list: Ptr<ExtraDataList>) -> Vec<u8> {
        let mut types = vec![];
        let mut current: Ptr<BSExtraData> = e.get(list, ExtraDataList::pHead).cast();
        while !current.is_null() {
            types.push(e.get(current, BSExtraData::cEtype));
            current = e.get(current, BSExtraData::pNext).cast();
        }
        types
    }

    fn payload_of(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
        let extra = find_extra(e, list, extra_type);
        assert!(!extra.is_null());
        e.mem.u32(extra.addr() + 0x0c)
    }

    /// Runs `call` with the call log on and returns the log.
    fn logged(e: &mut Engine, call: impl FnOnce(&mut Engine)) -> Log {
        e.call_log = Some(vec![]);
        call(e);
        e.call_log.take().unwrap()
    }

    fn calls_to(log: &Log, address: u32) -> Vec<Vec<u32>> {
        log.iter()
            .filter(|(callee, _)| *callee == address)
            .map(|(_, args)| args.clone())
            .collect()
    }

    /// The extra data the log shows deleted (the destructor called).
    fn deleted(log: &Log) -> Vec<u32> {
        calls_to(log, DESTRUCTOR).iter().map(|a| a[0]).collect()
    }

    /// The sizes passed to `operator new`.
    fn allocated_sizes(log: &Log) -> Vec<u32> {
        calls_to(log, OPERATOR_NEW).iter().map(|a| a[0]).collect()
    }

    /// Doubles for the callees of `fn_0042dd90`: three removers, the flags
    /// word copy and the bit test.
    fn flag_engine() -> Engine {
        let mut e = engine();
        stub(&mut e, REMOVE_SAVED_ANIMATION);
        stub(&mut e, REMOVE_SAVED_HAVOK_DATA);
        stub(&mut e, REMOVE_LAST_FINISHED_SEQUENCE);
        e.register(COPY_FLAGS_WORD, |e, a| {
            let word = e.mem.u32(a[0] + 0x2c);
            e.mem.set_u32(a[1], word);
            returns(a[1])
        });
        e.register(FLAGS_TEST, |e, a| {
            returns((e.mem.u32(a[0]) & a[1] != 0) as u32)
        });
        e
    }

    // ---- 0042dd90

    #[test]
    fn fn_0042dd90_removes_used_markers_when_the_form_flag_is_set() {
        let mut e = flag_engine();
        let form = e.mem.alloc(0x40);
        e.mem.set_u32(form + 0x2c, 0x8000_0001);
        let list = list_with(&mut e, &[(EXTRA_USED_MARKERS, 0), (0x30, 0)]);
        let extra = find_extra(&mut e, list, EXTRA_USED_MARKERS);
        let log = logged(&mut e, |e| {
            e.call(0x0042dd90, &args![list, form]);
        });
        let order: Vec<u32> = log.iter().map(|(callee, _)| *callee).collect();
        assert_eq!(
            &order[..6],
            &[
                0x0042dd90,
                REMOVE_SAVED_ANIMATION,
                REMOVE_SAVED_HAVOK_DATA,
                REMOVE_LAST_FINISHED_SEQUENCE,
                COPY_FLAGS_WORD,
                FLAGS_TEST
            ]
        );
        assert_eq!(calls_to(&log, FLAGS_TEST)[0][1], 0x8000_0000);
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert_eq!(chain_types(&e, list), vec![0x30]);
    }

    #[test]
    fn fn_0042dd90_keeps_used_markers_when_the_form_flag_is_clear() {
        let mut e = flag_engine();
        let form = e.mem.alloc(0x40);
        e.mem.set_u32(form + 0x2c, 0x7fff_ffff);
        let list = list_with(&mut e, &[(EXTRA_USED_MARKERS, 0)]);
        let log = logged(&mut e, |e| {
            e.call(0x0042dd90, &args![list, form]);
        });
        assert!(deleted(&log).is_empty());
        assert_eq!(chain_types(&e, list), vec![EXTRA_USED_MARKERS]);
    }

    // ---- hot key

    #[test]
    fn set_hot_key_builds_the_extra_data_or_stores_the_byte() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042dde0, &args![list, 0x7fu32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(calls_to(&log, EXTRA_HOT_KEY_INIT)[0][1], 0x7f);
        assert_eq!(chain_types(&e, list), vec![EXTRA_HOT_KEY]);
        assert_eq!(payload_of(&mut e, list, EXTRA_HOT_KEY) & 0xff, 0x7f);
        let log = logged(&mut e, |e| {
            e.call(0x0042dde0, &args![list, 3u32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(payload_of(&mut e, list, EXTRA_HOT_KEY) & 0xff, 3);
        assert_eq!(chain_types(&e, list), vec![EXTRA_HOT_KEY]);
    }

    #[test]
    fn get_hot_key_gives_the_byte_or_ff() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_HOT_KEY, 0x1234_5605)]);
        assert_eq!(e.call(0x0042de90, &args![list]).u8(), 0x05);
        let empty = list_with(&mut e, &[(0x30, 1)]);
        assert_eq!(e.call(0x0042de90, &args![empty]).u8(), 0xff);
    }

    #[test]
    fn remove_hot_key_deletes_it_by_type() {
        let mut e = engine();
        let list = list_with(&mut e, &[(0x30, 0), (EXTRA_HOT_KEY, 5)]);
        let extra = find_extra(&mut e, list, EXTRA_HOT_KEY);
        let log = logged(&mut e, |e| {
            e.call(0x0042dec0, &args![list]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert_eq!(chain_types(&e, list), vec![0x30]);
        let log = logged(&mut e, |e| {
            e.call(0x0042dec0, &args![list]);
        });
        assert!(deleted(&log).is_empty());
    }

    // ---- info general topic

    #[test]
    fn set_info_general_topic_builds_or_stores() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042dee0, &args![list, 0x1111u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(calls_to(&log, EXTRA_INFO_GENERAL_TOPIC_INIT)[0][1], 0x1111);
        assert_eq!(payload_of(&mut e, list, EXTRA_INFO_GENERAL_TOPIC), 0x1111);
        e.call(0x0042dee0, &args![list, 0x2222u32]);
        assert_eq!(payload_of(&mut e, list, EXTRA_INFO_GENERAL_TOPIC), 0x2222);
        assert_eq!(chain_types(&e, list), vec![EXTRA_INFO_GENERAL_TOPIC]);
    }

    #[test]
    fn fn_0042df90_forwards_only_an_empty_topic() {
        let mut e = engine();
        e.register(LIST_COUNT, |e, a| returns(e.mem.u32(a[0])));
        stub(&mut e, MENU_TOPIC_FORWARD);
        // No extra data.
        let none = list_with(&mut e, &[]);
        assert_eq!(e.call(0x0042df90, &args![none, 9u32]).u32(), 0);
        // A null topic.
        let null_topic = list_with(&mut e, &[(EXTRA_INFO_GENERAL_TOPIC, 0)]);
        assert_eq!(e.call(0x0042df90, &args![null_topic, 9u32]).u32(), 0);
        // A topic with items: returned, not forwarded.
        let topic = e.mem.alloc(0x40);
        e.mem.set_u32(topic + 0x0c, 2);
        let busy = list_with(&mut e, &[(EXTRA_INFO_GENERAL_TOPIC, topic)]);
        let log = logged(&mut e, |e| {
            assert_eq!(e.call(0x0042df90, &args![busy, 9u32]).u32(), topic);
        });
        assert!(calls_to(&log, MENU_TOPIC_FORWARD).is_empty());
        // An empty topic: forwarded with the argument.
        let empty_topic = e.mem.alloc(0x40);
        e.mem.set_u32(empty_topic + 0x14, 0xa1);
        e.mem.set_u32(empty_topic + 0x28, 0xa2);
        e.mem.set_u32(empty_topic + 0x18, 0xa3);
        let ready = list_with(&mut e, &[(EXTRA_INFO_GENERAL_TOPIC, empty_topic)]);
        let log = logged(&mut e, |e| {
            assert_eq!(e.call(0x0042df90, &args![ready, 9u32]).u32(), empty_topic);
        });
        assert_eq!(
            calls_to(&log, MENU_TOPIC_FORWARD),
            vec![vec![empty_topic, 0xa1, 0xa2, 0xa3, 9]]
        );
    }

    #[test]
    fn fn_0042dff0_tests_the_list_count() {
        let mut e = engine();
        e.register(LIST_COUNT, |e, a| returns(e.mem.u32(a[0])));
        let topic = e.mem.alloc(0x40);
        let log = logged(&mut e, |e| {
            assert!(e.call(0x0042dff0, &args![Ptr::<()>::new(topic)]).bool());
        });
        assert_eq!(calls_to(&log, LIST_COUNT), vec![vec![topic + 0x0c]]);
        e.mem.set_u32(topic + 0x0c, 3);
        assert!(!e.call(0x0042dff0, &args![Ptr::<()>::new(topic)]).bool());
    }

    #[test]
    fn fn_0042e010_passes_three_fields_and_the_argument() {
        let mut e = engine();
        stub(&mut e, MENU_TOPIC_FORWARD);
        let topic = e.mem.alloc(0x40);
        e.mem.set_u32(topic + 0x14, 1);
        e.mem.set_u32(topic + 0x18, 3);
        e.mem.set_u32(topic + 0x28, 2);
        let log = logged(&mut e, |e| {
            e.call(0x0042e010, &args![Ptr::<()>::new(topic), 4u32]);
        });
        assert_eq!(
            calls_to(&log, MENU_TOPIC_FORWARD),
            vec![vec![topic, 1, 2, 3, 4]]
        );
    }

    #[test]
    fn fn_0042e040_removes_the_info_general_topic() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_INFO_GENERAL_TOPIC, 1), (0x30, 0)]);
        e.call(0x0042e040, &args![list]);
        assert_eq!(chain_types(&e, list), vec![0x30]);
    }

    // ---- talking actor

    #[test]
    fn set_talking_actor_extra_replaces_an_existing_one() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e060, &args![list, 0x10u32, 0x20u32]);
        });
        assert!(deleted(&log).is_empty());
        assert_eq!(
            calls_to(&log, EXTRA_TALKING_ACTOR_INIT)[0][1..],
            [0x10, 0x20]
        );
        let old = find_extra(&mut e, list, EXTRA_TALKING_ACTOR);
        let log = logged(&mut e, |e| {
            e.call(0x0042e060, &args![list, 0x30u32, 0x40u32]);
        });
        assert_eq!(deleted(&log), vec![old.addr()]);
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(chain_types(&e, list), vec![EXTRA_TALKING_ACTOR]);
        assert_eq!(payload_of(&mut e, list, EXTRA_TALKING_ACTOR), 0x30);
    }

    #[test]
    fn get_talking_actor_extra_and_its_remover() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_TALKING_ACTOR, 5)]);
        let extra = e.call(0x0042e110, &args![list]).ptr::<BSExtraData>();
        assert!(!extra.is_null());
        e.call(0x0042e130, &args![list]);
        assert!(e
            .call(0x0042e110, &args![list])
            .ptr::<BSExtraData>()
            .is_null());
    }

    // ---- model swap

    #[test]
    fn model_swap_setter_getter_and_remover() {
        let mut e = engine();
        // The vtable the constructor installs, for the remover's delete.
        e.put_vtable(VTABLE_EXTRA_MODEL_SWAP, &[DESTRUCTOR]);
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e150, &args![list, 0xa0u32, 0xb0u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x14]);
        let extra = find_extra(&mut e, list, EXTRA_MODEL_SWAP);
        assert_eq!(e.mem.u32(extra.addr()), VTABLE_EXTRA_MODEL_SWAP);
        assert_eq!(e.mem.u32(extra.addr() + 0x0c), 0xa0);
        assert_eq!(e.mem.u32(extra.addr() + 0x10), 0xb0);
        assert_eq!(e.call(0x0042e250, &args![list]).u32(), 0xa0);
        let log = logged(&mut e, |e| {
            e.call(0x0042e150, &args![list, 0xc0u32, 0xd0u32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(e.mem.u32(extra.addr() + 0x0c), 0xc0);
        assert_eq!(e.mem.u32(extra.addr() + 0x10), 0xd0);
        e.call(0x0042e280, &args![list]);
        assert_eq!(e.call(0x0042e250, &args![list]).u32(), 0);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn fn_0042e210_builds_the_model_swap_extra_data() {
        let mut e = engine();
        let block = e.mem.alloc(0x14);
        let result = e
            .call(0x0042e210, &args![Ptr::<()>::new(block), 7u32, 8u32])
            .u32();
        assert_eq!(result, block);
        assert_eq!(e.mem.u8(block + 4), EXTRA_MODEL_SWAP);
        assert_eq!(e.mem.u32(block), 0x0101_5980);
        assert_eq!(e.mem.u32(block + 0x0c), 7);
        assert_eq!(e.mem.u32(block + 0x10), 8);
    }

    #[test]
    fn model_swap_getter_gives_zero_without_the_extra_data() {
        let mut e = engine();
        let list = list_with(&mut e, &[(0x30, 1)]);
        assert_eq!(e.call(0x0042e250, &args![list]).u32(), 0);
    }

    // ---- navmesh portal

    #[test]
    fn fn_0042e2c0_copies_the_portal_record_into_a_new_extra_data() {
        let mut e = engine();
        e.register(NAVMESH_PORTAL_COPY, |e, a| {
            let (low, high) = (e.mem.u32(a[1]), e.mem.u32(a[1] + 4));
            e.mem.set_u32(a[0], low);
            e.mem.set_u32(a[0] + 4, high);
            Ret::default()
        });
        let portal = e.mem.alloc(0x40);
        e.mem.set_u32(portal + 0x0c, 0x1234_5678);
        e.mem.set_u32(portal + 0x10, 0x0000_9abc);
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e2c0, &args![list, portal]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x14]);
        let extra = find_extra(&mut e, list, EXTRA_NAVMESH_PORTAL);
        assert!(!extra.is_null());
        assert_eq!(e.mem.u32(extra.addr() + 0x0c), 0x1234_5678);
        assert_eq!(e.mem.u32(extra.addr() + 0x10), 0x0000_9abc);
        // An existing one is left alone.
        e.mem.set_u32(portal + 0x0c, 0);
        let log = logged(&mut e, |e| {
            e.call(0x0042e2c0, &args![list, portal]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(e.mem.u32(extra.addr() + 0x0c), 0x1234_5678);
    }

    #[test]
    fn nav_mesh_portal_getter_and_remover() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_NAVMESH_PORTAL, 1), (0x30, 0)]);
        let extra = find_extra(&mut e, list, EXTRA_NAVMESH_PORTAL);
        assert_eq!(e.call(0x0042e2a0, &args![list]).ptr::<BSExtraData>(), extra);
        let log = logged(&mut e, |e| {
            e.call(0x0042e730, &args![list]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert!(e
            .call(0x0042e2a0, &args![list])
            .ptr::<BSExtraData>()
            .is_null());
        let log = logged(&mut e, |e| {
            e.call(0x0042e730, &args![list]);
        });
        assert!(deleted(&log).is_empty());
    }

    // ---- weapon mod slots

    #[test]
    fn set_weapon_mod_slot_builds_with_a_nonzero_slot() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e380, &args![list, 0x04u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        let extra = find_extra(&mut e, list, EXTRA_WEAPON_MOD_SLOTS);
        assert_eq!(e.mem.u32(extra.addr()), VTABLE_EXTRA_WEAPON_MOD_SLOTS);
        assert_eq!(e.mem.u8(extra.addr() + 0x0c), 0x04);
    }

    #[test]
    fn set_weapon_mod_slot_ors_into_an_existing_extra_data() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_WEAPON_MOD_SLOTS, 0x01)]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e380, &args![list, 0x04u32]);
            e.call(0x0042e380, &args![list, 0u32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(
            payload_of(&mut e, list, EXTRA_WEAPON_MOD_SLOTS) & 0xff,
            0x05
        );
    }

    #[test]
    #[should_panic]
    fn set_weapon_mod_slot_with_slot_zero_and_no_extra_data_reads_null() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        e.call(0x0042e380, &args![list, 0u32]);
    }

    #[test]
    fn fn_0042e450_builds_the_weapon_mod_flags_extra_data() {
        let mut e = engine();
        let block = e.mem.alloc(0x10);
        let result = e
            .call(0x0042e450, &args![Ptr::<()>::new(block), 0x82u32])
            .u32();
        assert_eq!(result, block);
        assert_eq!(e.mem.u8(block + 4), EXTRA_WEAPON_MOD_SLOTS);
        assert_eq!(e.mem.u32(block), 0x0101_59a4);
        assert_eq!(e.mem.u8(block + 0x0c), 0x82);
    }

    #[test]
    fn fn_0042e480_ors_the_bits() {
        let mut e = engine();
        let block = e.mem.alloc(0x10);
        e.mem.set_u8(block + 0x0c, 0x11);
        e.call(0x0042e480, &args![Ptr::<()>::new(block), 0x30u32]);
        assert_eq!(e.mem.u8(block + 0x0c), 0x31);
    }

    #[test]
    fn fn_0042e4a0_sets_the_byte_or_builds_the_extra_data() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e4a0, &args![list, 0x21u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(calls_to(&log, EXTRA_WEAPON_MOD_SLOTS_INIT).len(), 1);
        assert_eq!(
            payload_of(&mut e, list, EXTRA_WEAPON_MOD_SLOTS) & 0xff,
            0x21
        );
        // An existing byte is replaced, not ORed.
        e.call(0x0042e4a0, &args![list, 0x02u32]);
        assert_eq!(
            payload_of(&mut e, list, EXTRA_WEAPON_MOD_SLOTS) & 0xff,
            0x02
        );
        assert_eq!(chain_types(&e, list), vec![EXTRA_WEAPON_MOD_SLOTS]);
    }

    #[test]
    fn get_weapon_mod_flags_reads_through_the_helper() {
        let mut e = engine();
        e.register(WEAPON_MOD_FLAGS_READ, |e, a| {
            returns(e.mem.u8(a[0] + 0x0c) as u32)
        });
        let list = list_with(&mut e, &[(EXTRA_WEAPON_MOD_SLOTS, 0x0a)]);
        assert_eq!(e.call(0x0042e560, &args![list]).u8(), 0x0a);
        let empty = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            assert_eq!(e.call(0x0042e560, &args![empty]).u8(), 0);
        });
        assert!(calls_to(&log, WEAPON_MOD_FLAGS_READ).is_empty());
    }

    // ---- is modding

    #[test]
    fn set_is_modding_builds_or_stores_the_byte() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e5a0, &args![list, 1u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        let extra = find_extra(&mut e, list, EXTRA_WEAPON_IS_MODDING);
        assert_eq!(e.mem.u32(extra.addr()), VTABLE_EXTRA_WEAPON_IS_MODDING);
        assert_eq!(e.mem.u8(extra.addr() + 0x0c), 1);
        let log = logged(&mut e, |e| {
            e.call(0x0042e5a0, &args![list, 0u32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(e.mem.u8(extra.addr() + 0x0c), 0);
    }

    #[test]
    fn fn_0042e650_builds_the_is_modding_extra_data() {
        let mut e = engine();
        let block = e.mem.alloc(0x10);
        let result = e
            .call(0x0042e650, &args![Ptr::<()>::new(block), 1u32])
            .u32();
        assert_eq!(result, block);
        assert_eq!(e.mem.u8(block + 4), EXTRA_WEAPON_IS_MODDING);
        assert_eq!(e.mem.u32(block), 0x0101_59bc);
        assert_eq!(e.mem.u8(block + 0x0c), 1);
    }

    #[test]
    fn fn_0042e680_stores_the_byte() {
        let mut e = engine();
        let block = e.mem.alloc(0x10);
        e.mem.set_u32(block + 0x0c, 0xffff_ffff);
        e.call(0x0042e680, &args![Ptr::<()>::new(block), 0x12u32]);
        assert_eq!(e.mem.u32(block + 0x0c), 0xffff_ff12);
    }

    #[test]
    fn is_modding_scalar_deleting_destructor_deletes_only_on_bit_zero() {
        let mut e = engine();
        stub(&mut e, BS_EXTRA_DATA_DESTROY);
        let block = e.mem.alloc(0x10);
        let log = logged(&mut e, |e| {
            let result = e.call(0x0042e6a0, &args![Ptr::<()>::new(block), 0u32]);
            assert_eq!(result.u32(), block);
        });
        assert_eq!(calls_to(&log, BS_EXTRA_DATA_DESTROY), vec![vec![block]]);
        assert!(calls_to(&log, OPERATOR_DELETE).is_empty());
        assert_eq!(e.mem.u32(block), VTABLE_EXTRA_WEAPON_IS_MODDING);
        let log = logged(&mut e, |e| {
            e.call(0x0042e6a0, &args![Ptr::<()>::new(block), 3u32]);
        });
        assert_eq!(calls_to(&log, OPERATOR_DELETE), vec![vec![block]]);
    }

    #[test]
    fn fn_0042e6d0_sets_the_vtable_and_runs_the_base_destructor() {
        let mut e = engine();
        stub(&mut e, BS_EXTRA_DATA_DESTROY);
        let block = e.mem.alloc(0x10);
        let log = logged(&mut e, |e| {
            e.call(0x0042e6d0, &args![Ptr::<()>::new(block)]);
        });
        assert_eq!(e.mem.u32(block), 0x0101_59bc);
        assert_eq!(calls_to(&log, BS_EXTRA_DATA_DESTROY), vec![vec![block]]);
    }

    #[test]
    fn remove_is_modding_deletes_if_present() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_WEAPON_IS_MODDING, 1)]);
        let extra = find_extra(&mut e, list, EXTRA_WEAPON_IS_MODDING);
        let log = logged(&mut e, |e| {
            e.call(0x0042e6f0, &args![list]);
            e.call(0x0042e6f0, &args![list]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert!(chain_types(&e, list).is_empty());
    }

    // ---- faction changes (0x5E)

    #[test]
    fn fn_0042e760_adds_the_type_5e_extra_data_once() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let log = logged(&mut e, |e| {
            e.call(0x0042e760, &args![list]);
            e.call(0x0042e760, &args![list]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(chain_types(&e, list), vec![EXTRA_FACTION_CHANGES]);
        let found = e.call(0x0042e800, &args![list]).ptr::<BSExtraData>();
        assert_eq!(found, find_extra(&mut e, list, EXTRA_FACTION_CHANGES));
        let empty = list_with(&mut e, &[]);
        assert!(e
            .call(0x0042e800, &args![empty])
            .ptr::<BSExtraData>()
            .is_null());
    }

    // ---- dismembered limbs

    #[test]
    fn dismemberment_extra_is_added_returned_and_removed() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        assert!(e
            .call(0x0042e8c0, &args![list])
            .ptr::<BSExtraData>()
            .is_null());
        let log = logged(&mut e, |e| {
            e.call(0x0042e820, &args![list]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x30]);
        let extra = find_extra(&mut e, list, EXTRA_DISMEMBERED_LIMBS);
        assert!(!extra.is_null());
        let log = logged(&mut e, |e| {
            let again = e.call(0x0042e820, &args![list]).ptr::<BSExtraData>();
            assert_eq!(again, extra);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(e.call(0x0042e8c0, &args![list]).ptr::<BSExtraData>(), extra);
        let log = logged(&mut e, |e| {
            e.call(0x0042e8e0, &args![list]);
            e.call(0x0042e8e0, &args![list]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn add_dismemberment_extra_returns_the_new_extra_data() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        let returned = e.call(0x0042e820, &args![list]).ptr::<BSExtraData>();
        assert_eq!(returned, find_extra(&mut e, list, EXTRA_DISMEMBERED_LIMBS));
    }

    // ---- actor cause

    #[test]
    fn fn_0042e910_gets_the_actor_cause_extra_data() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_ACTOR_CAUSE, 1)]);
        assert_eq!(
            e.call(0x0042e910, &args![list]).ptr::<BSExtraData>(),
            find_extra(&mut e, list, EXTRA_ACTOR_CAUSE)
        );
        let empty = list_with(&mut e, &[]);
        assert!(e
            .call(0x0042e910, &args![empty])
            .ptr::<BSExtraData>()
            .is_null());
    }

    #[test]
    fn fn_0042e930_assigns_builds_or_removes() {
        let mut e = engine();
        stub(&mut e, ACTOR_CAUSE_POINTER_ASSIGN);
        let list = list_with(&mut e, &[]);
        // Null with no extra data: nothing happens.
        let log = logged(&mut e, |e| {
            e.call(0x0042e930, &args![list, 0u32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        // Non-null with none: built (0x10 bytes), assigned, added.
        let log = logged(&mut e, |e| {
            e.call(0x0042e930, &args![list, 0xcau32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        let extra = find_extra(&mut e, list, EXTRA_ACTOR_CAUSE);
        assert_eq!(
            calls_to(&log, ACTOR_CAUSE_POINTER_ASSIGN),
            vec![vec![extra.addr() + 0x0c, 0xca]]
        );
        // Non-null with one: assigned only.
        let log = logged(&mut e, |e| {
            e.call(0x0042e930, &args![list, 0xcbu32]);
        });
        assert!(allocated_sizes(&log).is_empty());
        assert_eq!(
            calls_to(&log, ACTOR_CAUSE_POINTER_ASSIGN),
            vec![vec![extra.addr() + 0x0c, 0xcb]]
        );
        // Null with one: removed.
        let log = logged(&mut e, |e| {
            e.call(0x0042e930, &args![list, 0u32]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn fn_0042ea00_assigns_the_smart_pointer_at_0c() {
        let mut e = engine();
        stub(&mut e, ACTOR_CAUSE_POINTER_ASSIGN);
        let block = e.mem.alloc(0x10);
        let log = logged(&mut e, |e| {
            e.call(0x0042ea00, &args![Ptr::<()>::new(block), 0x77u32]);
        });
        assert_eq!(
            calls_to(&log, ACTOR_CAUSE_POINTER_ASSIGN),
            vec![vec![block + 0x0c, 0x77]]
        );
    }

    // ---- combat style

    #[test]
    fn fn_0042ea20_gets_the_combat_style() {
        let mut e = engine();
        let list = list_with(&mut e, &[(EXTRA_COMBAT_STYLE, 0xabcd)]);
        assert_eq!(e.call(0x0042ea20, &args![list]).u32(), 0xabcd);
        let empty = list_with(&mut e, &[(0x30, 5)]);
        assert_eq!(e.call(0x0042ea20, &args![empty]).u32(), 0);
    }

    #[test]
    fn fn_0042ea50_stores_builds_or_removes() {
        let mut e = engine();
        let list = list_with(&mut e, &[]);
        // Null with none: nothing.
        e.call(0x0042ea50, &args![list, 0u32]);
        assert!(chain_types(&e, list).is_empty());
        // Non-null with none: built with the value.
        let log = logged(&mut e, |e| {
            e.call(0x0042ea50, &args![list, 0x51u32]);
        });
        assert_eq!(allocated_sizes(&log), vec![0x10]);
        assert_eq!(calls_to(&log, EXTRA_COMBAT_STYLE_INIT)[0][1], 0x51);
        // Non-null with one: stored.
        e.call(0x0042ea50, &args![list, 0x52u32]);
        assert_eq!(payload_of(&mut e, list, EXTRA_COMBAT_STYLE), 0x52);
        assert_eq!(chain_types(&e, list), vec![EXTRA_COMBAT_STYLE]);
        // Null with one: removed.
        let extra = find_extra(&mut e, list, EXTRA_COMBAT_STYLE);
        let log = logged(&mut e, |e| {
            e.call(0x0042ea50, &args![list, 0u32]);
        });
        assert_eq!(deleted(&log), vec![extra.addr()]);
        assert!(chain_types(&e, list).is_empty());
    }

    #[test]
    fn funcs_registers_forty_distinct_addresses_in_range() {
        let entries = funcs();
        assert_eq!(entries.len(), 40);
        let addresses: Vec<u32> = entries.iter().map(|(address, _)| *address).collect();
        let mut sorted = addresses.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(addresses, sorted);
        assert_eq!(addresses[0], 0x0042_dd90);
        assert_eq!(addresses[39], 0x0042_ea50);
    }
}
