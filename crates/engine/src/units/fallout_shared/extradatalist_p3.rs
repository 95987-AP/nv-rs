//! `fallout shared/extradatalist.cpp` (Xbox PDB source unit), part 3: its functions from `0041db00` up to
//! (not including) `00421400` in FalloutNV.exe 1.4.0.525, translated
//! (docs/ENGINE_CRATE.md). The unit's shared layouts and helpers are in
//! [`super::extradatalist`]; anything public there may be used here.
//!
//! Translated so far, in address order: the first 40 functions of the range,
//! `0041db00` to `0041eb60`: the accessors of the extra data of the enable
//! state parent (type `0x37`) and children (`0x38`), the item dropper
//! (`0x39`), the dropped item list (`0x3A`), the water type (`3`), the
//! type `0x3B` marker, the ash pile reference (`0x89`), the linked
//! reference (`0x51`) and its children (`0x52`), the open/close activate
//! reference (`0x6C`) and the activate reference (`0x53`, up to its flag
//! getter). The next session continues at `0041eba0`.
//!
//! The type numbers are `EXTRA_DATA_TYPE` of the Xbox PDB (`0x37`
//! `EXTRA_ENABLESTATEPARENT`, `0x3B` `EXTRA_TELEPORTMARKER`, `0x51`
//! `EXTRA_LINKED_REF`, ...). The functions of the same unit outside this
//! file (`GetExtraData` `00410220`, `AddExtra` `0040ff60`, the two
//! `RemoveExtra` `00410020` and `00410140`, the node constructor `00414010`)
//! are called by address, like every other callee. The compiler's
//! exception-unwinding frames (the `FS:[0]` chains of the functions that
//! allocate) are not translated.

#[allow(unused_imports)]
use super::extradatalist::*;
use super::extradataobjects::{
    ExtraActivateRef, ExtraAshPileRef, ExtraEnableStateChildren, ExtraEnableStateParent,
    ExtraLinkedRef, ExtraLinkedRefChildren, ExtraRandomTeleportMarker, RefActivateData,
};
#[allow(unused_imports)]
use crate::prelude::*;
use crate::types::BSSimpleList;

// ---------------------------------------------------------------------------
// Layouts

layout! {
    /// `ExtraItemDropper` (Xbox PDB), type `0x39`, 0x10 bytes.
    pub struct ExtraItemDropper: 0x10 {
        /// `pDropper` (Xbox PDB): `TESObjectREFR*`.
        0x0C pDropper: Ptr,
    }

    /// `ExtraDroppedItemList` (Xbox PDB), type `0x3A`, 0x14 bytes.
    pub struct ExtraDroppedItemList: 0x14 {
        /// `DroppedItemList` (Xbox PDB): `BSSimpleList<TESObjectREFR *>`.
        0x0C DroppedItemList: Inline<BSSimpleList>,
    }

    /// `ExtraCellWaterType` (Xbox PDB), type 3, 0x10 bytes.
    pub struct ExtraCellWaterType: 0x10 {
        /// `pWater` (Xbox PDB): `TESWaterForm*`.
        0x0C pWater: Ptr,
    }

    /// `ExtraOpenCloseActivateRef` (Xbox PDB), type `0x6C`, 0x10 bytes.
    pub struct ExtraOpenCloseActivateRef: 0x10 {
        /// `pActivateRef` (Xbox PDB): `TESObjectREFR*`.
        0x0C pActivateRef: Ptr,
    }
}

// ---------------------------------------------------------------------------
// Constants

/// `ExtraEnableStateParent` (`EXTRA_ENABLESTATEPARENT`).
const EXTRA_ENABLE_STATE_PARENT: u8 = 0x37;
/// `ExtraEnableStateChildren` (`EXTRA_ENABLESTATECHILDREN`).
const EXTRA_ENABLE_STATE_CHILDREN: u8 = 0x38;
/// `ExtraItemDropper` (`EXTRA_ITEMDROPPER`).
const EXTRA_ITEM_DROPPER_TYPE: u8 = 0x39;
/// `ExtraDroppedItemList` (`EXTRA_DROPPEDITEMLIST`).
const EXTRA_DROPPED_ITEM_LIST: u8 = 0x3a;
/// `ExtraCellWaterType` (`EXTRA_WATERTYPE`).
const EXTRA_WATER_TYPE: u8 = 0x03;
/// `ExtraRandomTeleportMarker` (`EXTRA_TELEPORTMARKER`).
const EXTRA_TELEPORT_MARKER: u8 = 0x3b;
/// `ExtraAshPileRef` (`EXTRA_ASHPILE_REF`).
const EXTRA_ASH_PILE_REF: u8 = 0x89;
/// `ExtraLinkedRef` (`EXTRA_LINKED_REF`).
const EXTRA_LINKED_REF_TYPE: u8 = 0x51;
/// `ExtraLinkedRefChildren` (`EXTRA_LINKED_REF_CHILDREN`).
const EXTRA_LINKED_REF_CHILDREN_TYPE: u8 = 0x52;
/// `ExtraActivateRef` (`EXTRA_ACTIVATE_REF`).
const EXTRA_ACTIVATE_REF_TYPE: u8 = 0x53;
/// `ExtraOpenCloseActivateRef` (`EXTRA_OPENCLOSEACTIVATE_REF`).
const EXTRA_OPEN_CLOSE_ACTIVATE_REF_TYPE: u8 = 0x6c;

/// `operator new(size)`.
const OPERATOR_NEW: u32 = 0x0040_1000;
/// `operator delete(block)` (cdecl).
const OPERATOR_DELETE: u32 = 0x0040_1030;
/// `BaseExtraList::GetExtraData(type)` (`00410220`).
const GET_EXTRA_DATA: u32 = 0x0041_0220;
/// `BaseExtraList::AddExtra(extra)` (`0040ff60`).
const ADD_EXTRA: u32 = 0x0040_ff60;
/// `BaseExtraList::RemoveExtra(extra, destroy)` (`00410020`).
const REMOVE_EXTRA: u32 = 0x0041_0020;
/// `BaseExtraList::RemoveExtra_ov2(type)` (`00410140`).
const REMOVE_EXTRA_BY_TYPE: u32 = 0x0041_0140;
/// `BSExtraData::BSExtraData(type)` (`0040ec80`): base vtable, type, null next.
const BS_EXTRA_DATA_INIT: u32 = 0x0040_ec80;
/// `00414010`: constructor of a two-word list node (item and next null; the
/// engine map files it as `BSSimpleList<REF_ACTIVATE_DATA_P>::AddHead`).
const LIST_NODE_INIT: u32 = 0x0041_4010;
/// `BSSimpleList::AddHead(&item)` (`005ae3d0`); `this` is the head node.
const LIST_ADD_HEAD: u32 = 0x005a_e3d0;
/// Whether the list at `this` holds an item equal to the word at the address
/// given (`005f65d0`).
const LIST_CONTAINS: u32 = 0x005f_65d0;
/// Removes from the list at `this` the first node whose item equals the word
/// at the address given (`00905330`).
const LIST_REMOVE_ITEM: u32 = 0x0090_5330;
/// `BSSimpleList::IsEmpty`: no item and no next (`008256d0`).
const LIST_IS_EMPTY: u32 = 0x0082_56d0;
/// `MOV EAX,ECX`: the address of a list node is the address of its item slot
/// (`006815c0`).
const LIST_ITEM_SLOT: u32 = 0x0068_15c0;
/// `MOV EAX,[ECX+4]`: the next node of a list node (`00726070`).
const LIST_NEXT: u32 = 0x0072_6070;
/// `BGSSaveFormBuffer::GetForm` (`007af430`, `MOV EAX,[ECX+0x20]`): on a
/// reference, its base form.
const REFERENCE_BASE_FORM: u32 = 0x007a_f430;
/// `TESForm::GetFormType` (`00401170`, `MOVZX EAX,[ECX+4]`).
const FORM_TYPE: u32 = 0x0040_1170;
/// Form type the dropped item search looks for.
const FORM_TYPE_DROPPED_ITEM_MATCH: u32 = 0x28;
/// Search in an `ExtraActivateRef` (`00433a00`): the `REF_ACTIVATE_DATA` of
/// its parent list whose reference is the argument, or null.
const ACTIVATE_REF_FIND: u32 = 0x0043_3a00;

/// Constructors of the extra data (`this` = the new block, no other
/// argument), in `extradataobjects.cpp` and `extradatacell.cpp`.
const ENABLE_STATE_CHILDREN_INIT: u32 = 0x0043_33a0;
const ITEM_DROPPER_INIT: u32 = 0x0043_5920;
const DROPPED_ITEM_LIST_INIT: u32 = 0x0043_5950;
const WATER_TYPE_INIT: u32 = 0x0040_f370;
const TELEPORT_MARKER_INIT: u32 = 0x0043_34b0;
const ASH_PILE_REF_INIT: u32 = 0x0043_36c0;
const LINKED_REF_INIT: u32 = 0x0043_3640;
const LINKED_REF_CHILDREN_INIT: u32 = 0x0043_3530;
const ACTIVATE_REF_INIT: u32 = 0x0043_38b0;

/// Vtable set by the constructor `0041e750`.
const VTABLE_EXTRA_OPEN_CLOSE_ACTIVATE_REF: u32 = 0x0101_51a8;

// ---------------------------------------------------------------------------
// Helpers

/// `list->GetExtraData(extra_type)`.
fn find_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> Ptr<BSExtraData> {
    e.call(GET_EXTRA_DATA, &args![list, extra_type as u32])
        .ptr()
}

/// `list->AddExtra(extra)`.
fn add_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra: Ptr<BSExtraData>) {
    e.call(ADD_EXTRA, &args![list, extra]);
}

/// `list->RemoveExtra(extra, true)`: unlinks and deletes the extra data.
fn remove_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra: Ptr<BSExtraData>) {
    e.call(REMOVE_EXTRA, &args![list, extra, 1u32]);
}

/// `list->RemoveExtra(extra_type)`.
fn remove_extra_by_type(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) {
    e.call(REMOVE_EXTRA_BY_TYPE, &args![list, extra_type as u32]);
}

/// `new T`: a block of `size` bytes built by the constructor at `construct`
/// (`this` = the block). As in the code, a failed allocation gives null and
/// the constructor is not run.
fn new_object(e: &mut Engine, size: u32, construct: u32) -> u32 {
    let block = e.call(OPERATOR_NEW, &args![size]).u32();
    if block == 0 {
        0
    } else {
        e.call(construct, &args![block]).u32()
    }
}

/// The word at +0x0C of the first extra data of `extra_type`, or 0.
fn extra_word_or_zero(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        0
    } else {
        e.mem.u32(extra.addr() + 0x0c)
    }
}

/// The address of the list at +0x0C of the first extra data of `extra_type`,
/// or null.
fn extra_list_or_null(
    e: &mut Engine,
    list: Ptr<ExtraDataList>,
    extra_type: u8,
) -> Ptr<BSSimpleList> {
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        Ptr::NULL
    } else {
        extra.byte_add(0x0c).cast()
    }
}

/// The shape of the setters of a reference (water, teleport marker, linked
/// reference): a null `value` deletes the extra data of `extra_type` (by
/// type); otherwise the word at +0x0C of the existing one is overwritten, or
/// a new `0x10`-byte one is built by `construct`, given the value, and
/// added.
fn set_word_or_remove_by_type(
    e: &mut Engine,
    list: Ptr<ExtraDataList>,
    extra_type: u8,
    construct: u32,
    value: u32,
) {
    if value == 0 {
        remove_extra_by_type(e, list, extra_type);
        return;
    }
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        let block = new_object(e, 0x10, construct);
        e.mem.set_u32(block + 0x0c, value);
        add_extra(e, list, Ptr::new(block));
    } else {
        e.mem.set_u32(extra.addr() + 0x0c, value);
    }
}

/// The shape of the adders to a reference list: when `item` is not null,
/// finds the extra data of `extra_type` or builds a `0x14`-byte one with
/// `construct` and adds it, then puts `item` at the head of the list at
/// +0x0C unless the list already holds it.
fn add_to_reference_list(
    e: &mut Engine,
    list: Ptr<ExtraDataList>,
    extra_type: u8,
    construct: u32,
    item: u32,
) {
    if item == 0 {
        return;
    }
    let mut extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        extra = Ptr::new(new_object(e, 0x14, construct));
        add_extra(e, list, extra);
    }
    let head = extra.addr() + 0x0c;
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), item);
        if !e.call(LIST_CONTAINS, &args![head, slot]).bool() {
            e.call(LIST_ADD_HEAD, &args![head, slot]);
        }
    });
}

/// The shape of the removers from a reference list: when `item` is not
/// null and the list has an extra data of `extra_type`, removes `item` from
/// the list at +0x0C, and deletes the extra data if that left the list empty.
fn remove_from_reference_list(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8, item: u32) {
    if item == 0 {
        return;
    }
    let extra = find_extra(e, list, extra_type);
    if extra.is_null() {
        return;
    }
    let head = extra.addr() + 0x0c;
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), item);
        e.call(LIST_REMOVE_ITEM, &args![head, slot]);
    });
    if e.call(LIST_IS_EMPTY, &args![head]).bool() {
        remove_extra(e, list, extra);
    }
}

/// A `float` loaded and stored through the x87 stack (`FLD`/`FSTP`): the
/// same bits, except that a signalling NaN comes out quiet.
fn x87_float(value: f32) -> f32 {
    let bits = value.to_bits();
    let is_nan = bits & 0x7f80_0000 == 0x7f80_0000 && bits & 0x007f_ffff != 0;
    f32::from_bits(if is_nan { bits | 0x0040_0000 } else { bits })
}

/// A new `REF_ACTIVATE_DATA` (8 bytes, node constructor `00414010`): its
/// address, or 0 when the allocation fails (then the code stores through the
/// null pointer all the same; the model has no failing allocation).
fn new_activate_data(e: &mut Engine) -> u32 {
    let block = e.call(OPERATOR_NEW, &args![8u32]).u32();
    if block == 0 {
        0
    } else {
        e.call(LIST_NODE_INIT, &args![block]).u32()
    }
}

/// Stores `reference` in `entry` (`pActivateRef`), then puts the entry at the
/// head of the `ParentList` of `extra` (`AddHead` is given the address of a
/// local holding the entry).
fn push_activate_parent(e: &mut Engine, extra: Ptr<BSExtraData>, entry: u32, reference: u32) {
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), entry);
        e.mem.set_u32(entry, reference);
        e.call(LIST_ADD_HEAD, &args![extra.addr() + 0x0c, slot]);
    });
}

// Translated from 0041db00 (decompiled, FalloutNV.exe 1.4.0.525)
/// Flag bit 0 of `cFlags` of the type `0x37` extra data
/// (`ExtraEnableStateParent`, `0041db30`), false when the list has none. No
/// Xbox PDB name.
pub fn fn_0041db00(e: &mut Engine, this: Ptr<ExtraDataList>) -> bool {
    let extra = find_extra(e, this, EXTRA_ENABLE_STATE_PARENT);
    if extra.is_null() {
        false
    } else {
        fn_0041db30(e, extra.cast())
    }
}

// Translated from 0041db30 (decompiled, FalloutNV.exe 1.4.0.525)
/// Bit 0 of `cFlags` (+0x10) of an `ExtraEnableStateParent`.
pub fn fn_0041db30(e: &mut Engine, this: Ptr<ExtraEnableStateParent>) -> bool {
    e.get(this, ExtraEnableStateParent::cFlags) & 1 != 0
}

// Translated from 0041db50 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets or clears (`value`) flag bit 0 of the type `0x37` extra data
/// (`0041db80`); does nothing when the list has none.
pub fn fn_0041db50(e: &mut Engine, this: Ptr<ExtraDataList>, value: bool) {
    let extra = find_extra(e, this, EXTRA_ENABLE_STATE_PARENT);
    if !extra.is_null() {
        fn_0041db80(e, extra.cast(), value);
    }
}

// Translated from 0041db80 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets (`value` true) or clears bit 0 of `cFlags` (+0x10) of an
/// `ExtraEnableStateParent`.
pub fn fn_0041db80(e: &mut Engine, this: Ptr<ExtraEnableStateParent>, value: bool) {
    let flags = e.get(this, ExtraEnableStateParent::cFlags);
    let flags = if value { flags | 1 } else { flags & 0xfe };
    e.set(this, ExtraEnableStateParent::cFlags, flags);
}

// Translated from 0041dbd0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::ShouldPopInWhenEnabledByParent` (Xbox PDB): flag bit 1 of
/// `cFlags` of the type `0x37` extra data (`0041dc00`), false when the list
/// has none.
pub fn extra_data_list_should_pop_in_when_enabled_by_parent(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> bool {
    let extra = find_extra(e, this, EXTRA_ENABLE_STATE_PARENT);
    if extra.is_null() {
        false
    } else {
        fn_0041dc00(e, extra.cast())
    }
}

// Translated from 0041dc00 (decompiled, FalloutNV.exe 1.4.0.525)
/// Bit 1 of `cFlags` (+0x10) of an `ExtraEnableStateParent`.
pub fn fn_0041dc00(e: &mut Engine, this: Ptr<ExtraEnableStateParent>) -> bool {
    e.get(this, ExtraEnableStateParent::cFlags) & 2 != 0
}

// Translated from 0041dc20 (decompiled, FalloutNV.exe 1.4.0.525)
/// Flag bit 2 of `cFlags` of the type `0x37` extra data (`0041dc50`), false
/// when the list has none. No Xbox PDB name.
pub fn fn_0041dc20(e: &mut Engine, this: Ptr<ExtraDataList>) -> bool {
    let extra = find_extra(e, this, EXTRA_ENABLE_STATE_PARENT);
    if extra.is_null() {
        false
    } else {
        fn_0041dc50(e, extra.cast())
    }
}

// Translated from 0041dc50 (decompiled, FalloutNV.exe 1.4.0.525)
/// Bit 2 of `cFlags` (+0x10) of an `ExtraEnableStateParent`.
pub fn fn_0041dc50(e: &mut Engine, this: Ptr<ExtraEnableStateParent>) -> bool {
    e.get(this, ExtraEnableStateParent::cFlags) & 4 != 0
}

// Translated from 0041dc70 (decompiled, FalloutNV.exe 1.4.0.525)
/// Stores `flags` in the whole `cFlags` byte of the type `0x37` extra data;
/// does nothing when the list has none. No Xbox PDB name.
pub fn fn_0041dc70(e: &mut Engine, this: Ptr<ExtraDataList>, flags: u8) {
    let extra: Ptr<ExtraEnableStateParent> = find_extra(e, this, EXTRA_ENABLE_STATE_PARENT).cast();
    if !extra.is_null() {
        e.set(extra, ExtraEnableStateParent::cFlags, flags);
    }
}

// Translated from 0041dca0 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `ChildList` (+0x0C, the inline head node) of the type `0x38` extra
/// data (`ExtraEnableStateChildren`), or null when the list has none.
pub fn fn_0041dca0(e: &mut Engine, this: Ptr<ExtraDataList>) -> Ptr<BSSimpleList> {
    let extra: Ptr<ExtraEnableStateChildren> =
        find_extra(e, this, EXTRA_ENABLE_STATE_CHILDREN).cast();
    if extra.is_null() {
        Ptr::NULL
    } else {
        extra.at(ExtraEnableStateChildren::ChildList)
    }
}

// Translated from 0041dcd0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds `child` to the `ChildList` of the type `0x38` extra data
/// (`ExtraEnableStateChildren`, `0x14` bytes, `004333a0`), which it builds
/// and adds to the list when there is none, unless the child is already in
/// the list. A null `child` does nothing. No Xbox PDB name.
pub fn fn_0041dcd0(e: &mut Engine, this: Ptr<ExtraDataList>, child: u32) {
    add_to_reference_list(
        e,
        this,
        EXTRA_ENABLE_STATE_CHILDREN,
        ENABLE_STATE_CHILDREN_INIT,
        child,
    );
}

// Translated from 0041dda0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Removes `child` from the `ChildList` of the type `0x38` extra data and
/// deletes the extra data when the list is then empty. A null `child`, or no
/// such extra data, does nothing. No Xbox PDB name.
pub fn fn_0041dda0(e: &mut Engine, this: Ptr<ExtraDataList>, child: u32) {
    remove_from_reference_list(e, this, EXTRA_ENABLE_STATE_CHILDREN, child);
}

// Translated from 0041de00 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetItemDropper` (Xbox PDB): `pDropper` of the type `0x39`
/// extra data (`ExtraItemDropper`), or 0.
pub fn extra_data_list_get_item_dropper(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or_zero(e, this, EXTRA_ITEM_DROPPER_TYPE)
}

// Translated from 0041de40 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::AddDroppedItem` (Xbox PDB; the body sets `pDropper`): a
/// null `dropper` deletes the type `0x39` extra data (by type); otherwise the
/// extra data is found or built (`0x10` bytes, `00435920`) and added, and
/// `dropper` is stored in it (after the add, for a new one).
pub fn extra_data_list_add_dropped_item(e: &mut Engine, this: Ptr<ExtraDataList>, dropper: u32) {
    if dropper == 0 {
        remove_extra_by_type(e, this, EXTRA_ITEM_DROPPER_TYPE);
        return;
    }
    let mut extra = find_extra(e, this, EXTRA_ITEM_DROPPER_TYPE);
    if extra.is_null() {
        extra = Ptr::new(new_object(e, 0x10, ITEM_DROPPER_INIT));
        add_extra(e, this, extra);
    }
    e.set(extra.cast(), ExtraItemDropper::pDropper, Ptr::new(dropper));
}

// Translated from 0041df00 (decompiled, FalloutNV.exe 1.4.0.525)
/// Walks the dropped item list (`DroppedItemList` of the type `0x3A` extra
/// data) and returns the first reference whose base form (`007af430`) has
/// form type `0x28` (`00401170`). The walk stops with 0 at the first node
/// whose item is null, and ends with 0 when the list has none or no match.
/// No Xbox PDB name.
pub fn fn_0041df00(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let mut found = 0;
    let extra = find_extra(e, this, EXTRA_DROPPED_ITEM_LIST);
    if extra.is_null() {
        return 0;
    }
    let mut node = extra.addr() + 0x0c;
    while found == 0 && node != 0 {
        let slot = e.call(LIST_ITEM_SLOT, &args![node]).u32();
        let item = e.mem.u32(slot);
        if item == 0 {
            return 0;
        }
        let form = e.call(REFERENCE_BASE_FORM, &args![item]).u32();
        let form_type = e.call(FORM_TYPE, &args![form]).u32();
        if form_type == FORM_TYPE_DROPPED_ITEM_MATCH {
            found = item;
        }
        node = e.call(LIST_NEXT, &args![node]).u32();
    }
    found
}

// Translated from 0041df90 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetDroppedItemList` (Xbox PDB): the `DroppedItemList`
/// (+0x0C, the inline head node) of the type `0x3A` extra data, or null.
pub fn extra_data_list_get_dropped_item_list(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSSimpleList> {
    extra_list_or_null(e, this, EXTRA_DROPPED_ITEM_LIST)
}

// Translated from 0041dfd0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveDroppedItemList` (Xbox PDB): deletes the type `0x3A`
/// extra data (by type) when the list has one.
pub fn extra_data_list_remove_dropped_item_list(e: &mut Engine, this: Ptr<ExtraDataList>) {
    let extra = find_extra(e, this, EXTRA_DROPPED_ITEM_LIST);
    if !extra.is_null() {
        remove_extra_by_type(e, this, EXTRA_DROPPED_ITEM_LIST);
    }
}

// Translated from 0041e000 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds `item` to the `DroppedItemList` of the type `0x3A` extra data
/// (`0x14` bytes, `00435950`, built and added when there is none), unless the
/// list already holds it. A null `item` does nothing. No Xbox PDB name.
pub fn fn_0041e000(e: &mut Engine, this: Ptr<ExtraDataList>, item: u32) {
    add_to_reference_list(
        e,
        this,
        EXTRA_DROPPED_ITEM_LIST,
        DROPPED_ITEM_LIST_INIT,
        item,
    );
}

// Translated from 0041e0d0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveDroppedItem` (Xbox PDB): removes `item` from the
/// `DroppedItemList` of the type `0x3A` extra data and deletes the extra data
/// when the list is then empty.
pub fn extra_data_list_remove_dropped_item(e: &mut Engine, this: Ptr<ExtraDataList>, item: u32) {
    remove_from_reference_list(e, this, EXTRA_DROPPED_ITEM_LIST, item);
}

// Translated from 0041e130 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetWaterType` (Xbox PDB): `pWater` of the type 3 extra data
/// (`ExtraCellWaterType`), or 0.
pub fn extra_data_list_get_water_type(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    extra_word_or_zero(e, this, EXTRA_WATER_TYPE)
}

// Translated from 0041e160 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets `pWater` of the type 3 extra data: a null `water` deletes it (by
/// type); otherwise the existing one is overwritten, or a `0x10`-byte one
/// (`0040f370`) is built, given the water and added. No Xbox PDB name.
pub fn fn_0041e160(e: &mut Engine, this: Ptr<ExtraDataList>, water: u32) {
    set_word_or_remove_by_type(e, this, EXTRA_WATER_TYPE, WATER_TYPE_INIT, water);
}

// Translated from 0041e220 (decompiled, FalloutNV.exe 1.4.0.525)
/// `pMarker` of the type `0x3B` extra data (`ExtraRandomTeleportMarker`), or
/// 0. No Xbox PDB name.
pub fn fn_0041e220(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra: Ptr<ExtraRandomTeleportMarker> = find_extra(e, this, EXTRA_TELEPORT_MARKER).cast();
    if extra.is_null() {
        0
    } else {
        e.get(extra, ExtraRandomTeleportMarker::pMarker).addr()
    }
}

// Translated from 0041e250 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets `pMarker` of the type `0x3B` extra data: a null `marker` deletes it
/// (by type); otherwise the existing one is overwritten, or a `0x10`-byte one
/// (`004334b0`) is built, given the marker and added. No Xbox PDB name.
pub fn fn_0041e250(e: &mut Engine, this: Ptr<ExtraDataList>, marker: u32) {
    set_word_or_remove_by_type(e, this, EXTRA_TELEPORT_MARKER, TELEPORT_MARKER_INIT, marker);
}

// Translated from 0041e310 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetAshPileRef` (Xbox PDB): `pAshPileRef` of the type
/// `0x89` extra data (`ExtraAshPileRef`), or 0.
pub fn extra_data_list_get_ash_pile_ref(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra: Ptr<ExtraAshPileRef> = find_extra(e, this, EXTRA_ASH_PILE_REF).cast();
    if extra.is_null() {
        0
    } else {
        e.get(extra, ExtraAshPileRef::pAshPileRef).addr()
    }
}

// Translated from 0041e340 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets `pAshPileRef` of the type `0x89` extra data. The extra data is looked
/// up first: a null `reference` deletes it (the object itself, if any); a
/// non-null one is stored in the existing extra data, or in a `0x10`-byte one
/// (`004336c0`) built and added first. No Xbox PDB name.
pub fn fn_0041e340(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) {
    let mut extra = find_extra(e, this, EXTRA_ASH_PILE_REF);
    if reference == 0 {
        if !extra.is_null() {
            remove_extra(e, this, extra);
        }
        return;
    }
    if extra.is_null() {
        extra = Ptr::new(new_object(e, 0x10, ASH_PILE_REF_INIT));
        add_extra(e, this, extra);
    }
    e.set(
        extra.cast(),
        ExtraAshPileRef::pAshPileRef,
        Ptr::new(reference),
    );
}

// Translated from 0041e410 (decompiled, FalloutNV.exe 1.4.0.525)
/// `pLinkedRef` of the type `0x51` extra data (`ExtraLinkedRef`), or 0. No
/// Xbox PDB name.
pub fn fn_0041e410(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra: Ptr<ExtraLinkedRef> = find_extra(e, this, EXTRA_LINKED_REF_TYPE).cast();
    if extra.is_null() {
        0
    } else {
        e.get(extra, ExtraLinkedRef::pLinkedRef).addr()
    }
}

// Translated from 0041e440 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets `pLinkedRef` of the type `0x51` extra data: a null `reference`
/// deletes it (by type); otherwise the existing one is overwritten, or a
/// `0x10`-byte one (`00433640`) is built, given the reference and added. No
/// Xbox PDB name.
pub fn fn_0041e440(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) {
    set_word_or_remove_by_type(e, this, EXTRA_LINKED_REF_TYPE, LINKED_REF_INIT, reference);
}

// Translated from 0041e500 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::GetLinkedRefChildren` (Xbox PDB): the `ChildList` (+0x0C,
/// the inline head node) of the type `0x52` extra data
/// (`ExtraLinkedRefChildren`), or null.
pub fn extra_data_list_get_linked_ref_children(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
) -> Ptr<BSSimpleList> {
    let extra: Ptr<ExtraLinkedRefChildren> =
        find_extra(e, this, EXTRA_LINKED_REF_CHILDREN_TYPE).cast();
    if extra.is_null() {
        Ptr::NULL
    } else {
        extra.at(ExtraLinkedRefChildren::ChildList)
    }
}

// Translated from 0041e530 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds `child` to the `ChildList` of the type `0x52` extra data
/// (`0x14` bytes, `00433530`, built and added when there is none), unless the
/// list already holds it. A null `child` does nothing. No Xbox PDB name.
pub fn fn_0041e530(e: &mut Engine, this: Ptr<ExtraDataList>, child: u32) {
    add_to_reference_list(
        e,
        this,
        EXTRA_LINKED_REF_CHILDREN_TYPE,
        LINKED_REF_CHILDREN_INIT,
        child,
    );
}

// Translated from 0041e600 (decompiled, FalloutNV.exe 1.4.0.525)
/// Removes `child` from the `ChildList` of the type `0x52` extra data and
/// deletes the extra data when the list is then empty. No Xbox PDB name.
pub fn fn_0041e600(e: &mut Engine, this: Ptr<ExtraDataList>, child: u32) {
    remove_from_reference_list(e, this, EXTRA_LINKED_REF_CHILDREN_TYPE, child);
}

// Translated from 0041e660 (decompiled, FalloutNV.exe 1.4.0.525)
/// `pActivateRef` of the type `0x6C` extra data (`ExtraOpenCloseActivateRef`),
/// or 0. No Xbox PDB name.
pub fn fn_0041e660(e: &mut Engine, this: Ptr<ExtraDataList>) -> u32 {
    let extra: Ptr<ExtraOpenCloseActivateRef> =
        find_extra(e, this, EXTRA_OPEN_CLOSE_ACTIVATE_REF_TYPE).cast();
    if extra.is_null() {
        0
    } else {
        e.get(extra, ExtraOpenCloseActivateRef::pActivateRef).addr()
    }
}

// Translated from 0041e690 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::SetOpenCloseActivateRef` (Xbox PDB): a null `reference`
/// deletes the type `0x6C` extra data (by type); otherwise the existing one
/// is overwritten, or a `0x10`-byte one is built with the constructor of this
/// file (`0041e750`), given the reference and added.
pub fn extra_data_list_set_open_close_activate_ref(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
    reference: u32,
) {
    if reference == 0 {
        remove_extra_by_type(e, this, EXTRA_OPEN_CLOSE_ACTIVATE_REF_TYPE);
        return;
    }
    let extra = find_extra(e, this, EXTRA_OPEN_CLOSE_ACTIVATE_REF_TYPE);
    if extra.is_null() {
        let block = e.call(OPERATOR_NEW, &args![0x10u32]).u32();
        let built = if block == 0 {
            0
        } else {
            fn_0041e750(e, Ptr::new(block)).addr()
        };
        e.mem.set_u32(built + 0x0c, reference);
        add_extra(e, this, Ptr::new(built));
    } else {
        e.mem.set_u32(extra.addr() + 0x0c, reference);
    }
}

// Translated from 0041e750 (decompiled, FalloutNV.exe 1.4.0.525)
/// Constructor of `ExtraOpenCloseActivateRef` (type `0x6C`): the base
/// constructor (`0040ec80`) with the type, the vtable `010151a8`, and a null
/// `pActivateRef`. Returns `this`.
pub fn fn_0041e750(
    e: &mut Engine,
    this: Ptr<ExtraOpenCloseActivateRef>,
) -> Ptr<ExtraOpenCloseActivateRef> {
    e.call(
        BS_EXTRA_DATA_INIT,
        &args![this, EXTRA_OPEN_CLOSE_ACTIVATE_REF_TYPE as u32],
    );
    e.mem
        .set_u32(this.addr(), VTABLE_EXTRA_OPEN_CLOSE_ACTIVATE_REF);
    e.set(this, ExtraOpenCloseActivateRef::pActivateRef, Ptr::NULL);
    this
}

// Translated from 0041e780 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `ParentList` (+0x0C, the inline head node) of the type `0x53` extra
/// data (`ExtraActivateRef`), or null. No Xbox PDB name.
pub fn fn_0041e780(e: &mut Engine, this: Ptr<ExtraDataList>) -> Ptr<BSSimpleList> {
    let extra: Ptr<ExtraActivateRef> = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE).cast();
    if extra.is_null() {
        Ptr::NULL
    } else {
        extra.at(ExtraActivateRef::ParentList)
    }
}

// Translated from 0041e7b0 (decompiled, FalloutNV.exe 1.4.0.525)
/// The `REF_ACTIVATE_DATA` of the type `0x53` extra data's parent list whose
/// reference is `reference` (`00433a00`), or 0 when the list has no such
/// extra data or the parent is not in it. No Xbox PDB name.
pub fn fn_0041e7b0(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) -> u32 {
    let extra = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE);
    if extra.is_null() {
        0
    } else {
        e.call(ACTIVATE_REF_FIND, &args![extra, reference]).u32()
    }
}

// Translated from 0041e7f0 (decompiled, FalloutNV.exe 1.4.0.525)
/// Adds `reference` as an activate parent: a new `REF_ACTIVATE_DATA` (8
/// bytes, node constructor `00414010`, `pActivateRef` = `reference`) goes to
/// the head of the `ParentList` of the type `0x53` extra data (`0x20` bytes,
/// `004338b0`, built and added when there is none), unless the existing extra
/// data already has an entry for `reference` (`00433a00`). A null `reference`
/// does nothing. No Xbox PDB name.
pub fn fn_0041e7f0(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) {
    if reference == 0 {
        return;
    }
    let extra = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE);
    let extra = if extra.is_null() {
        let built = Ptr::new(new_object(e, 0x20, ACTIVATE_REF_INIT));
        add_extra(e, this, built);
        built
    } else {
        if e.call(ACTIVATE_REF_FIND, &args![extra, reference]).u32() != 0 {
            return;
        }
        extra
    };
    let entry = new_activate_data(e);
    push_activate_parent(e, extra, entry, reference);
}

// Translated from 0041e960 (decompiled, FalloutNV.exe 1.4.0.525)
/// `ExtraDataList::RemoveActivateParent` (Xbox PDB): finds the entry for
/// `reference` in the `ParentList` of the type `0x53` extra data (`00433a00`),
/// takes it out of the list (`00905330`), deletes it, and deletes the extra
/// data (by type) when the list is then empty. A null `reference`, no such
/// extra data or no entry does nothing.
pub fn extra_data_list_remove_activate_parent(
    e: &mut Engine,
    this: Ptr<ExtraDataList>,
    reference: u32,
) {
    if reference == 0 {
        return;
    }
    let extra = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE);
    if extra.is_null() {
        return;
    }
    let entry = e.call(ACTIVATE_REF_FIND, &args![extra, reference]).u32();
    if entry == 0 {
        return;
    }
    let head = extra.addr() + 0x0c;
    e.with_stack(4, |e, slot| {
        e.mem.set_u32(slot.addr(), entry);
        e.call(LIST_REMOVE_ITEM, &args![head, slot]);
    });
    e.call(OPERATOR_DELETE, &args![entry]);
    if e.call(LIST_IS_EMPTY, &args![head]).bool() {
        remove_extra_by_type(e, this, EXTRA_ACTIVATE_REF_TYPE);
    }
}

// Translated from 0041e9e0 (decompiled, FalloutNV.exe 1.4.0.525)
/// `fActivateDelay` (+4) of the `REF_ACTIVATE_DATA` for `reference` in the
/// type `0x53` extra data (`00433a00`), or 0.0 when there is none. Returned
/// in `ST0` (`FLD`, so a signalling NaN comes out quiet). No Xbox PDB name.
pub fn fn_0041e9e0(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32) -> f32 {
    let extra = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE);
    if !extra.is_null() {
        let entry: Ptr<RefActivateData> = e.call(ACTIVATE_REF_FIND, &args![extra, reference]).ptr();
        if !entry.is_null() {
            return x87_float(e.get(entry, RefActivateData::fActivateDelay));
        }
    }
    0.0
}

// Translated from 0041ea30 (decompiled, FalloutNV.exe 1.4.0.525)
/// Sets the activate delay of the entry for `reference` in the type `0x53`
/// extra data: the extra data is built (`0x20` bytes, `004338b0`) and added
/// when there is none; when it has no entry for `reference` (`00433a00`), a
/// new `REF_ACTIVATE_DATA` (8 bytes, `00414010`) goes to the head of the
/// `ParentList` and gets `pActivateRef` = `reference`; then `delay` is
/// stored in the entry (`FLD`/`FSTP`, so a signalling NaN is stored quiet).
/// A null `reference` does nothing. No Xbox PDB name.
pub fn fn_0041ea30(e: &mut Engine, this: Ptr<ExtraDataList>, reference: u32, delay: f32) {
    if reference == 0 {
        return;
    }
    let mut extra = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE);
    let mut entry = 0;
    if extra.is_null() {
        extra = Ptr::new(new_object(e, 0x20, ACTIVATE_REF_INIT));
        add_extra(e, this, extra);
    } else {
        entry = e.call(ACTIVATE_REF_FIND, &args![extra, reference]).u32();
    }
    if entry == 0 {
        entry = new_activate_data(e);
        e.with_stack(4, |e, slot| {
            e.mem.set_u32(slot.addr(), entry);
            e.call(LIST_ADD_HEAD, &args![extra.addr() + 0x0c, slot]);
        });
        e.mem.set_u32(entry, reference);
    }
    e.set(
        Ptr::<RefActivateData>::new(entry),
        RefActivateData::fActivateDelay,
        x87_float(delay),
    );
}

// Translated from 0041eb60 (decompiled, FalloutNV.exe 1.4.0.525)
/// Bit 0 of `cActivateFlags` (+0x14) of the type `0x53` extra data, false
/// when the list has none. No Xbox PDB name.
pub fn fn_0041eb60(e: &mut Engine, this: Ptr<ExtraDataList>) -> bool {
    let extra: Ptr<ExtraActivateRef> = find_extra(e, this, EXTRA_ACTIVATE_REF_TYPE).cast();
    !extra.is_null() && e.get(extra, ExtraActivateRef::cActivateFlags) & 1 != 0
}

/// This part's translated functions, by exe address.
pub fn funcs() -> Vec<(u32, AbiFn)> {
    vec![
        entry!(0x0041db00, fn_0041db00(Ptr<ExtraDataList>) -> bool),
        entry!(0x0041db30, fn_0041db30(Ptr<ExtraEnableStateParent>) -> bool),
        entry!(0x0041db50, fn_0041db50(Ptr<ExtraDataList>, bool)),
        entry!(0x0041db80, fn_0041db80(Ptr<ExtraEnableStateParent>, bool)),
        entry!(
            0x0041dbd0,
            extra_data_list_should_pop_in_when_enabled_by_parent(Ptr<ExtraDataList>) -> bool
        ),
        entry!(0x0041dc00, fn_0041dc00(Ptr<ExtraEnableStateParent>) -> bool),
        entry!(0x0041dc20, fn_0041dc20(Ptr<ExtraDataList>) -> bool),
        entry!(0x0041dc50, fn_0041dc50(Ptr<ExtraEnableStateParent>) -> bool),
        entry!(0x0041dc70, fn_0041dc70(Ptr<ExtraDataList>, u8)),
        entry!(
            0x0041dca0,
            fn_0041dca0(Ptr<ExtraDataList>) -> Ptr<BSSimpleList>
        ),
        entry!(0x0041dcd0, fn_0041dcd0(Ptr<ExtraDataList>, u32)),
        entry!(0x0041dda0, fn_0041dda0(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0041de00,
            extra_data_list_get_item_dropper(Ptr<ExtraDataList>) -> u32
        ),
        entry!(
            0x0041de40,
            extra_data_list_add_dropped_item(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x0041df00, fn_0041df00(Ptr<ExtraDataList>) -> u32),
        entry!(
            0x0041df90,
            extra_data_list_get_dropped_item_list(Ptr<ExtraDataList>) -> Ptr<BSSimpleList>
        ),
        entry!(
            0x0041dfd0,
            extra_data_list_remove_dropped_item_list(Ptr<ExtraDataList>)
        ),
        entry!(0x0041e000, fn_0041e000(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0041e0d0,
            extra_data_list_remove_dropped_item(Ptr<ExtraDataList>, u32)
        ),
        entry!(
            0x0041e130,
            extra_data_list_get_water_type(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x0041e160, fn_0041e160(Ptr<ExtraDataList>, u32)),
        entry!(0x0041e220, fn_0041e220(Ptr<ExtraDataList>) -> u32),
        entry!(0x0041e250, fn_0041e250(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0041e310,
            extra_data_list_get_ash_pile_ref(Ptr<ExtraDataList>) -> u32
        ),
        entry!(0x0041e340, fn_0041e340(Ptr<ExtraDataList>, u32)),
        entry!(0x0041e410, fn_0041e410(Ptr<ExtraDataList>) -> u32),
        entry!(0x0041e440, fn_0041e440(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0041e500,
            extra_data_list_get_linked_ref_children(Ptr<ExtraDataList>) -> Ptr<BSSimpleList>
        ),
        entry!(0x0041e530, fn_0041e530(Ptr<ExtraDataList>, u32)),
        entry!(0x0041e600, fn_0041e600(Ptr<ExtraDataList>, u32)),
        entry!(0x0041e660, fn_0041e660(Ptr<ExtraDataList>) -> u32),
        entry!(
            0x0041e690,
            extra_data_list_set_open_close_activate_ref(Ptr<ExtraDataList>, u32)
        ),
        entry!(
            0x0041e750,
            fn_0041e750(Ptr<ExtraOpenCloseActivateRef>) -> Ptr<ExtraOpenCloseActivateRef>
        ),
        entry!(
            0x0041e780,
            fn_0041e780(Ptr<ExtraDataList>) -> Ptr<BSSimpleList>
        ),
        entry!(0x0041e7b0, fn_0041e7b0(Ptr<ExtraDataList>, u32) -> u32),
        entry!(0x0041e7f0, fn_0041e7f0(Ptr<ExtraDataList>, u32)),
        entry!(
            0x0041e960,
            extra_data_list_remove_activate_parent(Ptr<ExtraDataList>, u32)
        ),
        entry!(0x0041e9e0, fn_0041e9e0(Ptr<ExtraDataList>, u32) -> f32),
        entry!(0x0041ea30, fn_0041ea30(Ptr<ExtraDataList>, u32, f32)),
        entry!(0x0041eb60, fn_0041eb60(Ptr<ExtraDataList>) -> bool),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    type Log = Vec<(u32, Vec<u32>)>;

    /// A fake list: the extra data of type `t` sits in the word at
    /// `list + 0x100 + 4 * t` (the doubles of `GetExtraData`, `AddExtra` and
    /// the two `RemoveExtra` read and write it).
    fn table_slot(list: u32, extra_type: u32) -> u32 {
        list + 0x100 + 4 * extra_type
    }

    fn returns(value: u32) -> Ret {
        Ret {
            eax: value,
            ..Ret::default()
        }
    }

    /// Makes the constructor at `address` a double that sets the type byte
    /// of the new block and returns it.
    fn constructor_double(e: &mut Engine, address: u32, extra_type: u8) {
        e.register_double(address, move |e, a| {
            e.mem.set_u8(a[0] + 4, extra_type);
            returns(a[0])
        });
    }

    fn engine() -> Engine {
        let mut e = Engine::new();
        e.register(GET_EXTRA_DATA, |e, a| {
            returns(e.mem.u32(table_slot(a[0], a[1])))
        });
        e.register(ADD_EXTRA, |e, a| {
            let extra_type = e.mem.u8(a[1] + 4) as u32;
            e.mem.set_u32(table_slot(a[0], extra_type), a[1]);
            returns(a[1])
        });
        e.register(REMOVE_EXTRA, |e, a| {
            let extra_type = e.mem.u8(a[1] + 4) as u32;
            e.mem.set_u32(table_slot(a[0], extra_type), 0);
            Ret::default()
        });
        e.register(REMOVE_EXTRA_BY_TYPE, |e, a| {
            e.mem.set_u32(table_slot(a[0], a[1]), 0);
            Ret::default()
        });
        e.register(OPERATOR_NEW, |e, a| returns(e.mem.alloc(a[0])));
        e.register(OPERATOR_DELETE, |_, _| Ret::default());
        e.register(BS_EXTRA_DATA_INIT, |e, a| {
            e.mem.set_u8(a[0] + 4, a[1] as u8);
            e.mem.set_u32(a[0] + 8, 0);
            returns(a[0])
        });
        // The list doubles work on the inline head node of an extra data;
        // an add puts the item in the head node, which is all the tests need.
        e.register(LIST_CONTAINS, |e, a| {
            let wanted = e.mem.u32(a[1]);
            let mut node = a[0];
            let mut found = false;
            while node != 0 {
                found |= e.mem.u32(node) == wanted;
                node = e.mem.u32(node + 4);
            }
            returns(found as u32)
        });
        e.register(LIST_ADD_HEAD, |e, a| {
            let item = e.mem.u32(a[1]);
            e.mem.set_u32(a[0], item);
            Ret::default()
        });
        e.register(LIST_REMOVE_ITEM, |e, a| {
            let wanted = e.mem.u32(a[1]);
            if e.mem.u32(a[0]) == wanted {
                e.mem.set_u32(a[0], 0);
            }
            Ret::default()
        });
        e.register(LIST_IS_EMPTY, |e, a| {
            returns((e.mem.u32(a[0]) == 0 && e.mem.u32(a[0] + 4) == 0) as u32)
        });
        e.register(LIST_ITEM_SLOT, |_, a| returns(a[0]));
        e.register(LIST_NEXT, |e, a| returns(e.mem.u32(a[0] + 4)));
        constructor_double(&mut e, ENABLE_STATE_CHILDREN_INIT, 0x38);
        constructor_double(&mut e, ITEM_DROPPER_INIT, 0x39);
        constructor_double(&mut e, DROPPED_ITEM_LIST_INIT, 0x3a);
        constructor_double(&mut e, WATER_TYPE_INIT, 0x03);
        constructor_double(&mut e, TELEPORT_MARKER_INIT, 0x3b);
        constructor_double(&mut e, ASH_PILE_REF_INIT, 0x89);
        constructor_double(&mut e, LINKED_REF_INIT, 0x51);
        constructor_double(&mut e, LINKED_REF_CHILDREN_INIT, 0x52);
        constructor_double(&mut e, ACTIVATE_REF_INIT, 0x53);
        e
    }

    fn new_list(e: &mut Engine) -> Ptr<ExtraDataList> {
        Ptr::new(e.mem.alloc(0x400))
    }

    /// An extra data of the given type, put in the list's table.
    fn put_extra(e: &mut Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
        let extra = e.mem.alloc(0x40);
        e.mem.set_u8(extra + 4, extra_type);
        e.mem
            .set_u32(table_slot(list.addr(), extra_type as u32), extra);
        extra
    }

    fn extra_of(e: &Engine, list: Ptr<ExtraDataList>, extra_type: u8) -> u32 {
        e.mem.u32(table_slot(list.addr(), extra_type as u32))
    }

    fn calls_to(log: &Log, address: u32) -> Vec<Vec<u32>> {
        log.iter()
            .filter(|(callee, _)| *callee == address)
            .map(|(_, args)| args.clone())
            .collect()
    }

    // 0041db00, 0041db30, 0041db50, 0041db80, 0041dbd0, 0041dc00, 0041dc20,
    // 0041dc50, 0041dc70: the flags of the enable state parent.

    #[test]
    fn flag_bit_zero_getter_reads_the_flags_or_gives_false() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(!e.call(0x0041db00, &args![list]).bool());
        let extra = put_extra(&mut e, list, 0x37);
        assert!(!e.call(0x0041db00, &args![list]).bool());
        e.mem.set_u8(extra + 0x10, 0b110);
        assert!(!e.call(0x0041db00, &args![list]).bool());
        e.mem.set_u8(extra + 0x10, 0b001);
        assert!(e.call(0x0041db00, &args![list]).bool());
    }

    #[test]
    fn flag_bit_zero_of_the_extra_data_is_its_low_bit() {
        let mut e = engine();
        let extra: Ptr<ExtraEnableStateParent> = e.new_object();
        e.set(extra, ExtraEnableStateParent::cFlags, 0xfe);
        assert!(!e.call(0x0041db30, &args![extra]).bool());
        e.set(extra, ExtraEnableStateParent::cFlags, 0x01);
        assert!(e.call(0x0041db30, &args![extra]).bool());
    }

    #[test]
    fn flag_bit_zero_setter_goes_through_the_extra_data_setter() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call(0x0041db50, &args![list, true]);
        let extra = put_extra(&mut e, list, 0x37);
        e.mem.set_u8(extra + 0x10, 0b1000_0110);
        e.call_log = Some(vec![]);
        e.call(0x0041db50, &args![list, true]);
        assert_eq!(e.mem.u8(extra + 0x10), 0b1000_0111);
        e.call(0x0041db50, &args![list, false]);
        assert_eq!(e.mem.u8(extra + 0x10), 0b1000_0110);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, GET_EXTRA_DATA).len(), 2);
    }

    #[test]
    fn flag_bit_zero_setter_on_the_extra_data_keeps_the_other_bits() {
        let mut e = engine();
        let extra: Ptr<ExtraEnableStateParent> = e.new_object();
        e.set(extra, ExtraEnableStateParent::cFlags, 0b0101_0100);
        e.call(0x0041db80, &args![extra, true]);
        assert_eq!(e.get(extra, ExtraEnableStateParent::cFlags), 0b0101_0101);
        e.call(0x0041db80, &args![extra, false]);
        assert_eq!(e.get(extra, ExtraEnableStateParent::cFlags), 0b0101_0100);
        e.call(0x0041db80, &args![extra, false]);
        assert_eq!(e.get(extra, ExtraEnableStateParent::cFlags), 0b0101_0100);
    }

    #[test]
    fn pop_in_when_enabled_by_parent_is_bit_one() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(!e.call(0x0041dbd0, &args![list]).bool());
        let extra = put_extra(&mut e, list, 0x37);
        e.mem.set_u8(extra + 0x10, 0b101);
        assert!(!e.call(0x0041dbd0, &args![list]).bool());
        e.mem.set_u8(extra + 0x10, 0b010);
        assert!(e.call(0x0041dbd0, &args![list]).bool());
    }

    #[test]
    fn flag_bit_one_of_the_extra_data() {
        let mut e = engine();
        let extra: Ptr<ExtraEnableStateParent> = e.new_object();
        e.set(extra, ExtraEnableStateParent::cFlags, 0b101);
        assert!(!e.call(0x0041dc00, &args![extra]).bool());
        e.set(extra, ExtraEnableStateParent::cFlags, 0b010);
        assert!(e.call(0x0041dc00, &args![extra]).bool());
    }

    #[test]
    fn flag_bit_two_getter_reads_the_flags_or_gives_false() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(!e.call(0x0041dc20, &args![list]).bool());
        let extra = put_extra(&mut e, list, 0x37);
        e.mem.set_u8(extra + 0x10, 0b011);
        assert!(!e.call(0x0041dc20, &args![list]).bool());
        e.mem.set_u8(extra + 0x10, 0b100);
        assert!(e.call(0x0041dc20, &args![list]).bool());
    }

    #[test]
    fn flag_bit_two_of_the_extra_data() {
        let mut e = engine();
        let extra: Ptr<ExtraEnableStateParent> = e.new_object();
        e.set(extra, ExtraEnableStateParent::cFlags, 0b011);
        assert!(!e.call(0x0041dc50, &args![extra]).bool());
        e.set(extra, ExtraEnableStateParent::cFlags, 0b100);
        assert!(e.call(0x0041dc50, &args![extra]).bool());
    }

    #[test]
    fn flags_setter_replaces_the_whole_byte_or_does_nothing() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call(0x0041dc70, &args![list, 0x55u32]);
        let extra = put_extra(&mut e, list, 0x37);
        e.mem.set_u8(extra + 0x10, 0xff);
        e.call(0x0041dc70, &args![list, 0x12u32]);
        assert_eq!(e.mem.u8(extra + 0x10), 0x12);
        // Only the byte is stored.
        e.call(0x0041dc70, &args![list, 0x1234u32]);
        assert_eq!(e.mem.u8(extra + 0x10), 0x34);
    }

    // 0041dca0, 0041dcd0, 0041dda0: the children of the enable state parent.

    #[test]
    fn enable_state_children_getter_gives_the_inline_list_or_null() {
        let mut e = engine();
        let list = new_list(&mut e);
        let none = e.call(0x0041dca0, &args![list]).ptr::<BSSimpleList>();
        assert!(none.is_null());
        let extra = put_extra(&mut e, list, 0x38);
        let head = e.call(0x0041dca0, &args![list]).ptr::<BSSimpleList>();
        assert_eq!(head.addr(), extra + 0x0c);
    }

    #[test]
    fn add_enable_state_child_builds_the_extra_data_and_adds_the_child() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041dcd0, &args![list, 0xaaa0u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x38);
        assert_ne!(extra, 0);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x14]]);
        assert_eq!(calls_to(&log, ENABLE_STATE_CHILDREN_INIT).len(), 1);
        assert_eq!(calls_to(&log, ADD_EXTRA), vec![vec![list.addr(), extra]]);
        assert_eq!(calls_to(&log, LIST_ADD_HEAD).len(), 1);
        assert_eq!(calls_to(&log, LIST_ADD_HEAD)[0][0], extra + 0x0c);
        assert_eq!(e.mem.u32(extra + 0x0c), 0xaaa0);
    }

    #[test]
    fn add_enable_state_child_skips_a_child_already_in_the_list() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x38);
        e.mem.set_u32(extra + 0x0c, 0xaaa0);
        e.call_log = Some(vec![]);
        e.call(0x0041dcd0, &args![list, 0xaaa0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(calls_to(&log, LIST_CONTAINS).len(), 1);
        assert!(calls_to(&log, LIST_ADD_HEAD).is_empty());
    }

    #[test]
    fn add_enable_state_child_ignores_a_null_child() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041dcd0, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(extra_of(&e, list, 0x38), 0);
    }

    #[test]
    fn remove_enable_state_child_deletes_the_extra_data_when_empty() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x38);
        e.mem.set_u32(extra + 0x0c, 0xaaa0);
        e.call_log = Some(vec![]);
        e.call(0x0041dda0, &args![list, 0xaaa0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, LIST_REMOVE_ITEM)[0][0], extra + 0x0c);
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA),
            vec![vec![list.addr(), extra, 1]]
        );
        assert_eq!(extra_of(&e, list, 0x38), 0);
    }

    #[test]
    fn remove_enable_state_child_keeps_the_extra_data_when_not_empty() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x38);
        e.mem.set_u32(extra + 0x0c, 0xaaa0);
        e.mem.set_u32(extra + 0x10, 0x1000);
        e.call_log = Some(vec![]);
        e.call(0x0041dda0, &args![list, 0xbbb0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, REMOVE_EXTRA).is_empty());
        assert_eq!(extra_of(&e, list, 0x38), extra);
    }

    #[test]
    fn remove_enable_state_child_needs_a_child_and_an_extra_data() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041dda0, &args![list, 0xaaa0u32]);
        e.call(0x0041dda0, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, LIST_REMOVE_ITEM).is_empty());
        assert!(calls_to(&log, LIST_IS_EMPTY).is_empty());
    }

    // 0041de00, 0041de40: the item dropper.

    #[test]
    fn item_dropper_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041de00, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x39);
        e.mem.set_u32(extra + 0x0c, 0x7777);
        assert_eq!(e.call(0x0041de00, &args![list]).u32(), 0x7777);
    }

    #[test]
    fn add_dropped_item_with_null_removes_by_type() {
        let mut e = engine();
        let list = new_list(&mut e);
        put_extra(&mut e, list, 0x39);
        e.call_log = Some(vec![]);
        e.call(0x0041de40, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 0x39]]
        );
        assert_eq!(extra_of(&e, list, 0x39), 0);
    }

    #[test]
    fn add_dropped_item_builds_the_extra_data_then_stores_the_dropper() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041de40, &args![list, 0x7777u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x39);
        assert_ne!(extra, 0);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, ITEM_DROPPER_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x7777);
    }

    #[test]
    fn add_dropped_item_overwrites_an_existing_dropper() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x39);
        e.mem.set_u32(extra + 0x0c, 0x1111);
        e.call_log = Some(vec![]);
        e.call(0x0041de40, &args![list, 0x7777u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(e.mem.u32(extra + 0x0c), 0x7777);
    }

    // 0041df00: the search of the dropped item list.

    /// A reference whose base form has the given form type.
    fn reference_with_form_type(e: &mut Engine, form_type: u8) -> u32 {
        let form = e.mem.alloc(0x20);
        e.mem.set_u8(form + 4, form_type);
        let reference = e.mem.alloc(0x40);
        e.mem.set_u32(reference + 0x20, form);
        reference
    }

    fn engine_with_form_doubles() -> Engine {
        let mut e = engine();
        e.register(REFERENCE_BASE_FORM, |e, a| returns(e.mem.u32(a[0] + 0x20)));
        e.register(FORM_TYPE, |e, a| returns(e.mem.u8(a[0] + 4) as u32));
        e
    }

    #[test]
    fn dropped_item_search_without_the_extra_data_gives_zero() {
        let mut e = engine_with_form_doubles();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041df00, &args![list]).u32(), 0);
    }

    #[test]
    fn dropped_item_search_returns_the_first_item_of_form_type_0x28() {
        let mut e = engine_with_form_doubles();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        let other = reference_with_form_type(&mut e, 0x10);
        let wanted = reference_with_form_type(&mut e, 0x28);
        let later = reference_with_form_type(&mut e, 0x28);
        let second = e.mem.alloc(8);
        let third = e.mem.alloc(8);
        e.mem.set_u32(extra + 0x0c, other);
        e.mem.set_u32(extra + 0x10, second);
        e.mem.set_u32(second, wanted);
        e.mem.set_u32(second + 4, third);
        e.mem.set_u32(third, later);
        e.call_log = Some(vec![]);
        assert_eq!(e.call(0x0041df00, &args![list]).u32(), wanted);
        let log = e.call_log.take().unwrap();
        // The walk stops at the first match: the third node is not read.
        assert_eq!(calls_to(&log, REFERENCE_BASE_FORM).len(), 2);
    }

    #[test]
    fn dropped_item_search_without_a_match_gives_zero() {
        let mut e = engine_with_form_doubles();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        let only = reference_with_form_type(&mut e, 0x11);
        e.mem.set_u32(extra + 0x0c, only);
        assert_eq!(e.call(0x0041df00, &args![list]).u32(), 0);
    }

    #[test]
    fn dropped_item_search_stops_at_a_null_item() {
        let mut e = engine_with_form_doubles();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        let wanted = reference_with_form_type(&mut e, 0x28);
        let second = e.mem.alloc(8);
        e.mem.set_u32(extra + 0x10, second);
        e.mem.set_u32(second, wanted);
        // The head item is null: the later node is never reached.
        assert_eq!(e.call(0x0041df00, &args![list]).u32(), 0);
    }

    // 0041df90, 0041dfd0, 0041e000, 0041e0d0: the dropped item list.

    #[test]
    fn dropped_item_list_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(e
            .call(0x0041df90, &args![list])
            .ptr::<BSSimpleList>()
            .is_null());
        let extra = put_extra(&mut e, list, 0x3a);
        assert_eq!(e.call(0x0041df90, &args![list]).u32(), extra + 0x0c);
    }

    #[test]
    fn remove_dropped_item_list_only_when_there_is_one() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041dfd0, &args![list]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, REMOVE_EXTRA_BY_TYPE).is_empty());
        put_extra(&mut e, list, 0x3a);
        e.call_log = Some(vec![]);
        e.call(0x0041dfd0, &args![list]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 0x3a]]
        );
        assert_eq!(extra_of(&e, list, 0x3a), 0);
    }

    #[test]
    fn add_to_dropped_item_list_builds_the_extra_data_and_adds_the_item() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e000, &args![list, 0x4242u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x3a);
        assert_ne!(extra, 0);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x14]]);
        assert_eq!(calls_to(&log, DROPPED_ITEM_LIST_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x4242);
    }

    #[test]
    fn add_to_dropped_item_list_skips_an_item_already_there() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        e.mem.set_u32(extra + 0x0c, 0x4242);
        e.call_log = Some(vec![]);
        e.call(0x0041e000, &args![list, 0x4242u32]);
        e.call(0x0041e000, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, LIST_ADD_HEAD).is_empty());
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
    }

    #[test]
    fn remove_dropped_item_deletes_the_extra_data_when_empty() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        e.mem.set_u32(extra + 0x0c, 0x4242);
        e.call_log = Some(vec![]);
        e.call(0x0041e0d0, &args![list, 0x4242u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA),
            vec![vec![list.addr(), extra, 1]]
        );
    }

    #[test]
    fn remove_dropped_item_keeps_a_non_empty_list() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x3a);
        e.mem.set_u32(extra + 0x0c, 0x4242);
        e.call_log = Some(vec![]);
        e.call(0x0041e0d0, &args![list, 0x9999u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, REMOVE_EXTRA).is_empty());
        assert_eq!(extra_of(&e, list, 0x3a), extra);
    }

    // 0041e130, 0041e160: the water type.

    #[test]
    fn water_type_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e130, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x03);
        e.mem.set_u32(extra + 0x0c, 0x6060);
        assert_eq!(e.call(0x0041e130, &args![list]).u32(), 0x6060);
    }

    #[test]
    fn water_type_setter_stores_the_value_before_adding_a_new_extra_data() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.register(ADD_EXTRA, |e, a| {
            // The value is already in the block when it is added.
            assert_eq!(e.mem.u32(a[1] + 0x0c), 0x6060);
            e.mem.set_u32(table_slot(a[0], 3), a[1]);
            Ret::default()
        });
        e.call_log = Some(vec![]);
        e.call(0x0041e160, &args![list, 0x6060u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, WATER_TYPE_INIT).len(), 1);
        assert_eq!(e.mem.u32(extra_of(&e, list, 3) + 0x0c), 0x6060);
    }

    #[test]
    fn water_type_setter_overwrites_or_removes() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x03);
        e.call_log = Some(vec![]);
        e.call(0x0041e160, &args![list, 0x7070u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert_eq!(e.mem.u32(extra + 0x0c), 0x7070);
        e.call_log = Some(vec![]);
        e.call(0x0041e160, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 3]]
        );
        assert_eq!(extra_of(&e, list, 3), 0);
    }

    // 0041e220, 0041e250: the type 0x3B marker.

    #[test]
    fn marker_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e220, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x3b);
        e.mem.set_u32(extra + 0x0c, 0x3131);
        assert_eq!(e.call(0x0041e220, &args![list]).u32(), 0x3131);
    }

    #[test]
    fn marker_setter_builds_overwrites_and_removes() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e250, &args![list, 0x3131u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x3b);
        assert_eq!(calls_to(&log, TELEPORT_MARKER_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x3131);
        e.call(0x0041e250, &args![list, 0x3232u32]);
        assert_eq!(extra_of(&e, list, 0x3b), extra);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x3232);
        e.call_log = Some(vec![]);
        e.call(0x0041e250, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 0x3b]]
        );
    }

    // 0041e310, 0041e340: the ash pile reference.

    #[test]
    fn ash_pile_ref_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e310, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x89);
        e.mem.set_u32(extra + 0x0c, 0x8989);
        assert_eq!(e.call(0x0041e310, &args![list]).u32(), 0x8989);
    }

    #[test]
    fn ash_pile_ref_setter_adds_before_it_stores() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.register(ADD_EXTRA, |e, a| {
            // The value is stored after the add in this function.
            assert_eq!(e.mem.u32(a[1] + 0x0c), 0);
            e.mem.set_u32(table_slot(a[0], 0x89), a[1]);
            Ret::default()
        });
        e.call_log = Some(vec![]);
        e.call(0x0041e340, &args![list, 0x8989u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x89);
        assert_eq!(calls_to(&log, ASH_PILE_REF_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x8989);
    }

    #[test]
    fn ash_pile_ref_setter_overwrites_or_deletes_the_object() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x89);
        e.call(0x0041e340, &args![list, 0x9090u32]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x9090);
        e.call_log = Some(vec![]);
        e.call(0x0041e340, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA),
            vec![vec![list.addr(), extra, 1]]
        );
        // With no extra data a null reference does nothing at all.
        e.call_log = Some(vec![]);
        e.call(0x0041e340, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, REMOVE_EXTRA).is_empty());
        assert!(calls_to(&log, REMOVE_EXTRA_BY_TYPE).is_empty());
    }

    // 0041e410, 0041e440: the linked reference.

    #[test]
    fn linked_ref_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e410, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x51);
        e.mem.set_u32(extra + 0x0c, 0x5151);
        assert_eq!(e.call(0x0041e410, &args![list]).u32(), 0x5151);
    }

    #[test]
    fn linked_ref_setter_builds_overwrites_and_removes() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e440, &args![list, 0x5151u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x51);
        assert_eq!(calls_to(&log, LINKED_REF_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x5151);
        e.call(0x0041e440, &args![list, 0x5252u32]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x5252);
        e.call(0x0041e440, &args![list, 0u32]);
        assert_eq!(extra_of(&e, list, 0x51), 0);
    }

    // 0041e500, 0041e530, 0041e600: the linked reference children.

    #[test]
    fn linked_ref_children_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(e
            .call(0x0041e500, &args![list])
            .ptr::<BSSimpleList>()
            .is_null());
        let extra = put_extra(&mut e, list, 0x52);
        assert_eq!(e.call(0x0041e500, &args![list]).u32(), extra + 0x0c);
    }

    #[test]
    fn add_linked_ref_child_builds_the_extra_data_and_adds_the_child() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e530, &args![list, 0x5353u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x52);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x14]]);
        assert_eq!(calls_to(&log, LINKED_REF_CHILDREN_INIT), vec![vec![extra]]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x5353);
        // The same child again, and a null one, change nothing.
        e.call_log = Some(vec![]);
        e.call(0x0041e530, &args![list, 0x5353u32]);
        e.call(0x0041e530, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, LIST_ADD_HEAD).is_empty());
    }

    #[test]
    fn remove_linked_ref_child_deletes_the_extra_data_when_empty() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x52);
        e.mem.set_u32(extra + 0x0c, 0x5353);
        e.call_log = Some(vec![]);
        e.call(0x0041e600, &args![list, 0x5353u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA),
            vec![vec![list.addr(), extra, 1]]
        );
        assert_eq!(extra_of(&e, list, 0x52), 0);
    }

    // 0041e660, 0041e690, 0041e750: the open/close activate reference.

    #[test]
    fn open_close_activate_ref_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e660, &args![list]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x6c);
        e.mem.set_u32(extra + 0x0c, 0x6c6c);
        assert_eq!(e.call(0x0041e660, &args![list]).u32(), 0x6c6c);
    }

    #[test]
    fn open_close_activate_ref_setter_builds_with_the_constructor_of_this_file() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.register(ADD_EXTRA, |e, a| {
            assert_eq!(e.mem.u32(a[1] + 0x0c), 0x6c6c);
            e.mem.set_u32(table_slot(a[0], 0x6c), a[1]);
            Ret::default()
        });
        e.call_log = Some(vec![]);
        e.call(0x0041e690, &args![list, 0x6c6cu32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x6c);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x10]]);
        assert_eq!(calls_to(&log, BS_EXTRA_DATA_INIT), vec![vec![extra, 0x6c]]);
        assert_eq!(e.mem.u32(extra), 0x0101_51a8);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x6c6c);
    }

    #[test]
    fn open_close_activate_ref_setter_overwrites_or_removes() {
        let mut e = engine();
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x6c);
        e.call(0x0041e690, &args![list, 0x6d6du32]);
        assert_eq!(e.mem.u32(extra + 0x0c), 0x6d6d);
        e.call_log = Some(vec![]);
        e.call(0x0041e690, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 0x6c]]
        );
        assert_eq!(extra_of(&e, list, 0x6c), 0);
    }

    #[test]
    fn open_close_activate_ref_constructor() {
        let mut e = engine();
        let extra: Ptr<ExtraOpenCloseActivateRef> = e.new_object();
        e.mem.set_u32(extra.addr() + 0x0c, 0xdead);
        let result = e
            .call(0x0041e750, &args![extra])
            .ptr::<ExtraOpenCloseActivateRef>();
        assert_eq!(result, extra);
        assert_eq!(e.mem.u8(extra.addr() + 4), 0x6c);
        assert_eq!(e.mem.u32(extra.addr()), 0x0101_51a8);
        assert_eq!(
            e.get(extra, ExtraOpenCloseActivateRef::pActivateRef),
            Ptr::NULL
        );
    }

    // 0041e780, 0041e7b0, 0041e7f0, 0041e960, 0041e9e0, 0041ea30, 0041eb60:
    // the activate reference.

    /// Doubles `00433a00` as a search of the first entry of the extra data's
    /// parent list (an entry is `[reference, delay]`, found when its first
    /// word is the reference; the test lists are one entry long).
    fn activate_find_double(e: &mut Engine) {
        e.register(ACTIVATE_REF_FIND, |e, a| {
            let entry = e.mem.u32(a[0] + 0x0c);
            let found = entry != 0 && e.mem.u32(entry) == a[1];
            returns(if found { entry } else { 0 })
        });
    }

    #[test]
    fn parent_list_getter() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(e
            .call(0x0041e780, &args![list])
            .ptr::<BSSimpleList>()
            .is_null());
        let extra = put_extra(&mut e, list, 0x53);
        assert_eq!(e.call(0x0041e780, &args![list]).u32(), extra + 0x0c);
    }

    #[test]
    fn parent_entry_finder_asks_the_extra_data() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e7b0, &args![list, 0x5000u32]).u32(), 0);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.call_log = Some(vec![]);
        assert_eq!(e.call(0x0041e7b0, &args![list, 0x5000u32]).u32(), entry);
        assert_eq!(e.call(0x0041e7b0, &args![list, 0x5001u32]).u32(), 0);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, ACTIVATE_REF_FIND).len(), 2);
        assert_eq!(calls_to(&log, ACTIVATE_REF_FIND)[0], vec![extra, 0x5000]);
    }

    #[test]
    fn add_activate_parent_builds_the_extra_data_and_the_entry() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e7f0, &args![list, 0x5000u32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x53);
        assert_ne!(extra, 0);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x20], vec![0x08]]);
        assert_eq!(calls_to(&log, ACTIVATE_REF_INIT), vec![vec![extra]]);
        // A new extra data is not searched.
        assert!(calls_to(&log, ACTIVATE_REF_FIND).is_empty());
        let entry = e.mem.u32(extra + 0x0c);
        assert_ne!(entry, 0);
        assert_eq!(e.mem.u32(entry), 0x5000);
        assert_eq!(e.mem.u32(entry + 4), 0);
        assert_eq!(calls_to(&log, LIST_NODE_INIT).len(), 1);
    }

    #[test]
    fn add_activate_parent_adds_an_entry_only_for_a_new_parent() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.call_log = Some(vec![]);
        e.call(0x0041e7f0, &args![list, 0x5000u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert!(calls_to(&log, LIST_ADD_HEAD).is_empty());
        e.call_log = Some(vec![]);
        e.call(0x0041e7f0, &args![list, 0x6000u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x08]]);
        assert_eq!(calls_to(&log, LIST_ADD_HEAD).len(), 1);
        assert!(calls_to(&log, ADD_EXTRA).is_empty());
        let added = e.mem.u32(extra + 0x0c);
        assert_ne!(added, entry);
        assert_eq!(e.mem.u32(added), 0x6000);
    }

    #[test]
    fn add_activate_parent_ignores_a_null_reference() {
        let mut e = engine();
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041e7f0, &args![list, 0u32]);
        assert_eq!(e.call_log.take().unwrap().len(), 1);
    }

    #[test]
    fn remove_activate_parent_deletes_the_entry_and_the_empty_extra_data() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.call_log = Some(vec![]);
        e.call(0x0041e960, &args![list, 0x5000u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, LIST_REMOVE_ITEM)[0][0], extra + 0x0c);
        assert_eq!(calls_to(&log, OPERATOR_DELETE), vec![vec![entry]]);
        assert_eq!(
            calls_to(&log, REMOVE_EXTRA_BY_TYPE),
            vec![vec![list.addr(), 0x53]]
        );
        assert_eq!(extra_of(&e, list, 0x53), 0);
    }

    #[test]
    fn remove_activate_parent_keeps_a_non_empty_list_and_ignores_unknown_parents() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.mem.set_u32(extra + 0x10, 0x1234);
        e.call_log = Some(vec![]);
        e.call(0x0041e960, &args![list, 0x5000u32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, OPERATOR_DELETE), vec![vec![entry]]);
        assert!(calls_to(&log, REMOVE_EXTRA_BY_TYPE).is_empty());
        e.call_log = Some(vec![]);
        e.call(0x0041e960, &args![list, 0x7000u32]);
        e.call(0x0041e960, &args![list, 0u32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_DELETE).is_empty());
        assert!(calls_to(&log, LIST_REMOVE_ITEM).is_empty());
    }

    #[test]
    fn activate_delay_getter() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        assert_eq!(e.call(0x0041e9e0, &args![list, 0x5000u32]).f32(), 0.0);
        let extra = put_extra(&mut e, list, 0x53);
        assert_eq!(e.call(0x0041e9e0, &args![list, 0x5000u32]).f32(), 0.0);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(entry + 4, 2.5f32.to_bits());
        e.mem.set_u32(extra + 0x0c, entry);
        assert_eq!(e.call(0x0041e9e0, &args![list, 0x5000u32]).f32(), 2.5);
        assert_eq!(e.call(0x0041e9e0, &args![list, 0x5001u32]).f32(), 0.0);
        // A signalling NaN comes out of the x87 load quiet.
        e.mem.set_u32(entry + 4, 0x7f80_0001);
        let result = e.call(0x0041e9e0, &args![list, 0x5000u32]).f32();
        assert_eq!(result.to_bits(), 0x7fc0_0001);
    }

    #[test]
    fn set_activate_delay_builds_everything_for_a_new_parent() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041ea30, &args![list, 0x5000u32, 1.5f32]);
        let log = e.call_log.take().unwrap();
        let extra = extra_of(&e, list, 0x53);
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x20], vec![0x08]]);
        assert_eq!(calls_to(&log, ACTIVATE_REF_INIT), vec![vec![extra]]);
        let entry = e.mem.u32(extra + 0x0c);
        assert_eq!(e.mem.u32(entry), 0x5000);
        assert_eq!(f32::from_bits(e.mem.u32(entry + 4)), 1.5);
    }

    #[test]
    fn set_activate_delay_updates_an_existing_entry() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.call_log = Some(vec![]);
        e.call(0x0041ea30, &args![list, 0x5000u32, 4.0f32]);
        let log = e.call_log.take().unwrap();
        assert!(calls_to(&log, OPERATOR_NEW).is_empty());
        assert!(calls_to(&log, LIST_ADD_HEAD).is_empty());
        assert_eq!(f32::from_bits(e.mem.u32(entry + 4)), 4.0);
    }

    #[test]
    fn set_activate_delay_adds_an_entry_for_a_new_parent_of_an_existing_extra_data() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        let extra = put_extra(&mut e, list, 0x53);
        e.call_log = Some(vec![]);
        e.call(0x0041ea30, &args![list, 0x6000u32, 0.25f32]);
        let log = e.call_log.take().unwrap();
        assert_eq!(calls_to(&log, OPERATOR_NEW), vec![vec![0x08]]);
        assert!(calls_to(&log, ADD_EXTRA).is_empty());
        let entry = e.mem.u32(extra + 0x0c);
        assert_eq!(e.mem.u32(entry), 0x6000);
        assert_eq!(f32::from_bits(e.mem.u32(entry + 4)), 0.25);
    }

    #[test]
    fn set_activate_delay_ignores_a_null_reference_and_quiets_a_nan() {
        let mut e = engine();
        activate_find_double(&mut e);
        let list = new_list(&mut e);
        e.call_log = Some(vec![]);
        e.call(0x0041ea30, &args![list, 0u32, 1.0f32]);
        assert_eq!(e.call_log.take().unwrap().len(), 1);
        let extra = put_extra(&mut e, list, 0x53);
        let entry = e.mem.alloc(8);
        e.mem.set_u32(entry, 0x5000);
        e.mem.set_u32(extra + 0x0c, entry);
        e.call(
            0x0041ea30,
            &args![list, 0x5000u32, f32::from_bits(0x7f80_0001)],
        );
        assert_eq!(e.mem.u32(entry + 4), 0x7fc0_0001);
    }

    #[test]
    fn activate_flag_getter_is_bit_zero_of_the_activate_flags() {
        let mut e = engine();
        let list = new_list(&mut e);
        assert!(!e.call(0x0041eb60, &args![list]).bool());
        let extra = put_extra(&mut e, list, 0x53);
        e.mem.set_u8(extra + 0x14, 0b10);
        assert!(!e.call(0x0041eb60, &args![list]).bool());
        e.mem.set_u8(extra + 0x14, 0b11);
        assert!(e.call(0x0041eb60, &args![list]).bool());
    }
}
