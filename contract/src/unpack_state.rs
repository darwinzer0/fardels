use crate::state::{
    get_bin_data, set_bin_data, PREFIX_ID_PENDING_UNPACKED_MAPPINGS, PREFIX_ID_UNPACKED_MAPPINGS,
    PREFIX_PENDING_APPROVAL, PREFIX_PENDING_START, PREFIX_UNPACKED,
};
use cosmwasm_std::{CanonicalAddr, Coin, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

//
// Unpacked fardels
//
// are stored using multilevel prefixed + appendstore keys:
//    b"unpacked" | {unpacker canonical addr} | {appendstore index} -> global fardel id
//
//  plus an additional mapping is stored to allow getting unpacked status and package_idx by global_id:
//    b"id-to-unpacked" | {unpacker canonical addr} | {global fardel id} -> true/false
//       value == true means it is unpacked, value == false OR no record in storage means packed
//

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnpackedFardel {
    pub fardel_id: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnpackedStatus {
    pub unpacked: bool,
}

pub fn store_unpack<S: Storage>(
    storage: &mut S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> StdResult<()> {
    push_unpack(storage, unpacker, fardel_id)?;
    let unpacked_status = UnpackedStatus { unpacked: true };
    map_global_id_to_unpacked_by_unpacker(storage, fardel_id, unpacker, unpacked_status)
}

fn push_unpack<S: Storage>(
    storage: &mut S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);
    let mut storage = AppendStoreMut::<UnpackedFardel, _>::attach_or_create(&mut storage)?;

    let unpacked_fardel = UnpackedFardel { fardel_id };
    storage.push(&unpacked_fardel)
}

// stores a mapping from global fardel id to unpacked status in prefixed storage
fn map_global_id_to_unpacked_by_unpacker<S: Storage>(
    storage: &mut S,
    global_id: u128,
    unpacker: &CanonicalAddr,
    value: UnpackedStatus,
) -> StdResult<()> {
    let mut storage =
        PrefixedStorage::multilevel(&[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()], storage);
    set_bin_data(&mut storage, &global_id.to_be_bytes(), &value)
}

// get the unpacked status of a fardel for a given unpacker canonical address
pub fn get_unpacked_status_by_fardel_id<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> UnpackedStatus {
    let mapping_store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()],
        storage,
    );
    get_bin_data(&mapping_store, &fardel_id.to_be_bytes())
        .unwrap_or_else(|_| UnpackedStatus { unpacked: false })
}

// gets a list of unpacked fardels for a given unpacker canonical address
pub fn get_unpacked_by_unpacker<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<UnpackedFardel>> {
    let storage =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);

    // Try to access the storage of unpacked for the account.
    // If it doesn't exist yet, return an empty list.
    let storage = if let Some(result) = AppendStore::<UnpackedFardel, _>::attach(&storage) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` unpacked starting from the latest unpacked, potentially skipping `page * page_size`
    // unpacked from the start.
    let unpacked_iter = storage
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    let unpacked: StdResult<Vec<UnpackedFardel>> = unpacked_iter
        .map(|unpacked_fardel| unpacked_fardel)
        .collect();
    unpacked
}

// gets number of unpacked fardels for a given unpacker canonical address
pub fn get_number_of_unpacked_by_unpacker<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
) -> u32 {
    let storage =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);

    // Try to access the storage of unpacked for the account.
    // If it doesn't exist yet, return an empty list.
    if let Some(result) = AppendStore::<UnpackedFardel, _>::attach(&storage) {
        return result.unwrap().len();
    } else {
        return 0_u32;
    };
}

//
// Pending unpack approvals
//
// are stored in appendstore for each owner
//   b"pending" | {owner canonical addr} | {appendstore idx}
// with
//   b"pending-start" | {owner canonical addr} | index
// and
//   b"pending-unpacks" | {unpacker canonical addr} | {appendstore idx}
//

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingUnpackApproval {
    pub fardel_id: u128,
    pub unpacker: CanonicalAddr,
    pub coin: Coin,
    pub timestamp: u64,
    pub canceled: bool,
}

pub fn set_pending_start<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    idx: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_PENDING_START, storage);
    set_bin_data(&mut storage, owner.as_slice(), &idx)
}

pub fn get_pending_start<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> u32 {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_PENDING_START, storage);
    get_bin_data(&storage, owner.as_slice()).unwrap_or_else(|_| 0_u32)
}

pub fn store_pending_unpack<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
    sent_funds: Coin,
    timestamp: u64,
) -> StdResult<()> {
    let mut store =
        PrefixedStorage::multilevel(&[PREFIX_PENDING_APPROVAL, owner.as_slice()], storage);
    let mut store = AppendStoreMut::<PendingUnpackApproval, _>::attach_or_create(&mut store)?;

    let pending_unpack = PendingUnpackApproval {
        fardel_id,
        unpacker: unpacker.clone(),
        coin: sent_funds,
        timestamp,
        canceled: false,
    };
    store.push(&pending_unpack)?;
    let pending_unpack_idx = store.len() - 1;
    map_global_id_to_pending_unpacked_by_unpacker(
        storage,
        fardel_id,
        unpacker,
        pending_unpack_idx,
        true,
    )
}

// gets an individual pending unpack approval for a given owner canonical address
pub fn get_pending_unpack_approval<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    idx: u32,
) -> StdResult<PendingUnpackApproval> {
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_PENDING_APPROVAL, owner.as_slice()], storage);
    // Try to access the storage of pending unpacks for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<PendingUnpackApproval, _>::attach(&store) {
        result?
    } else {
        return Err(StdError::generic_err("no pending unpacks for this owner"));
    };
    store.get_at(idx)
}

// gets a list of pending unpack approvals for a given owner canonical address
pub fn get_pending_approvals_from_start<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    number: u32,
) -> StdResult<Vec<PendingUnpackApproval>> {
    let start = get_pending_start(storage, owner);
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_PENDING_APPROVAL, owner.as_slice()], storage);

    // Try to access the storage of unpacked for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<PendingUnpackApproval, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `number` unpacked starting from the latest unpacked, potentially skipping `start`
    // unpacked from the start.
    let unpacked_iter = store.iter().skip(start as _).take(number as _);
    let unpacked: StdResult<Vec<PendingUnpackApproval>> =
        unpacked_iter.map(|pending_unpack| pending_unpack).collect();
    unpacked
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MyPendingUnpack {
    pub pending_unpack_idx: u32,
    pub value: bool,
}

// stores a mapping from global fardel id to pending unpacked status in prefixed storage
fn map_global_id_to_pending_unpacked_by_unpacker<S: Storage>(
    store: &mut S,
    global_id: u128,
    unpacker: &CanonicalAddr,
    pending_unpack_idx: u32,
    value: bool,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(
        &[PREFIX_ID_PENDING_UNPACKED_MAPPINGS, unpacker.as_slice()],
        store,
    );
    let my_pending_unpack = MyPendingUnpack {
        pending_unpack_idx,
        value,
    };
    set_bin_data(&mut store, &global_id.to_be_bytes(), &my_pending_unpack)
}

// get the pending unpacked status of a fardel for a given unpacker canonical address
pub fn get_pending_unpacked_status_by_fardel_id<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> MyPendingUnpack {
    let mapping_store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_ID_PENDING_UNPACKED_MAPPINGS, unpacker.as_slice()],
        storage,
    );
    get_bin_data(&mapping_store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| MyPendingUnpack {
        pending_unpack_idx: 0,
        value: false,
    })
}

pub fn cancel_pending_unpack<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> StdResult<()> {
    let my_pending_unpack = get_pending_unpacked_status_by_fardel_id(storage, &unpacker, fardel_id);
    if my_pending_unpack.value {
        let mut pending_unpack_approval =
            get_pending_unpack_approval(storage, &owner, my_pending_unpack.pending_unpack_idx)?;
        pending_unpack_approval.canceled = true;
        let mut store =
            PrefixedStorage::multilevel(&[PREFIX_PENDING_APPROVAL, owner.as_slice()], storage);
        let mut store = AppendStoreMut::<PendingUnpackApproval, _>::attach_or_create(&mut store)?;
        // update element to canceled
        store.set_at(
            my_pending_unpack.pending_unpack_idx,
            &pending_unpack_approval,
        )?;
        map_global_id_to_pending_unpacked_by_unpacker(
            storage,
            fardel_id,
            unpacker,
            my_pending_unpack.pending_unpack_idx,
            false,
        )
    } else {
        return Err(StdError::generic_err(
            "Cannot cancel unpack that is not pending.",
        ));
    }
}
