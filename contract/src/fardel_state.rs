use crate::contract::DENOM;
use crate::state::{
    get_bin_data, set_bin_data, KEY_FARDEL_COUNT, PREFIX_FARDELS, PREFIX_FARDEL_NUM_UNPACKS,
    PREFIX_FARDEL_THUMBNAIL_IMGS, PREFIX_HASH_ID_MAPPINGS, PREFIX_HIDDEN, PREFIX_REMOVED,
    PREFIX_ID_FARDEL_MAPPINGS, PREFIX_SEALED,
};
use crate::unpack_state::store_unpack;
use cosmwasm_std::{CanonicalAddr, Coin, ReadonlyStorage, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

//
// Fardels
//
// are stored using multilevel prefixed + appendstore keys:
//    b"fardels" | {owner canonical addr} | {appendstore index} -> Fardel
//
//  plus an additional mapping is stored to allow getting by global_id:
//    b"id-to-fardel" | {global fardel id} -> GlobalIdToFardelMapping(owner, index)
//
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Fardel {
    pub global_id: Uint128,
    pub hash_id: Uint128,
    pub public_message: String,
    pub tags: Vec<String>,
    pub contents_data: String,
    pub cost: Coin,
    pub countable: u16,
    pub approval_req: bool,
    pub seal_time: u64,
    pub timestamp: u64,
}

impl Fardel {
    pub fn into_stored(self) -> StdResult<StoredFardel> {
        let stored_tags = self
            .tags
            .iter()
            .map(|tag| tag.as_bytes().to_vec())
            .collect();
        let fardel = StoredFardel {
            global_id: self.global_id.u128(),
            hash_id: self.hash_id.u128(),
            public_message: self.public_message.as_bytes().to_vec(),
            tags: stored_tags,
            contents_data: self.contents_data.as_bytes().to_vec(),
            cost: self.cost.amount.u128(),
            countable: self.countable,
            approval_req: self.approval_req,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }

    pub fn sold_out<S: ReadonlyStorage>(self, storage: &S) -> bool {
        if self.countable > 0 {
            let num_unpacks =
                get_fardel_unpack_count(storage, self.global_id.u128()).unwrap_or_else(|_| 0_u64);
            if num_unpacks >= self.countable.into() {
                return true;
            }
        }
        return false;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredFardel {
    pub global_id: u128,
    pub hash_id: u128,
    pub public_message: Vec<u8>,
    pub tags: Vec<Vec<u8>>,
    pub contents_data: Vec<u8>,
    pub cost: u128,
    pub countable: u16,
    pub approval_req: bool,
    pub seal_time: u64,
    pub timestamp: u64,
}

impl StoredFardel {
    pub fn into_humanized(self) -> StdResult<Fardel> {
        let humanized_tags = self
            .tags
            .iter()
            .map(|tag| String::from_utf8(tag.clone()).ok().unwrap_or_default())
            .collect();
        let fardel = Fardel {
            global_id: Uint128(self.global_id),
            hash_id: Uint128(self.hash_id),
            public_message: String::from_utf8(self.public_message)
                .ok()
                .unwrap_or_default(),
            tags: humanized_tags,
            contents_data: String::from_utf8(self.contents_data)
                .ok()
                .unwrap_or_default(),
            cost: Coin {
                amount: Uint128(self.cost),
                denom: DENOM.to_string(),
            },
            countable: self.countable,
            approval_req: self.approval_req,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalIdFardelMapping {
    pub owner: CanonicalAddr,
    pub index: u32,
}

pub fn get_total_fardel_count<S: ReadonlyStorage>(storage: &S) -> u128 {
    get_bin_data(storage, KEY_FARDEL_COUNT).unwrap_or_else(|_| 0_u128)
}

pub fn store_fardel<S: Storage>(
    store: &mut S,
    hash_id: u128,
    owner: &CanonicalAddr,
    public_message: Vec<u8>,
    tags: Vec<Vec<u8>>,
    contents_data: Vec<u8>,
    cost: u128,
    countable: u16,
    approval_req: bool,
    seal_time: u64,
    timestamp: u64,
) -> StdResult<u128> {
    //let mut config = Config::from_storage(store);
    let global_id: u128 = get_total_fardel_count(store);

    let fardel = StoredFardel {
        global_id: global_id.clone(),
        hash_id: hash_id.clone(),
        public_message: public_message.clone(),
        tags: tags.clone(),
        contents_data: contents_data.clone(),
        cost,
        countable,
        approval_req,
        seal_time,
        timestamp,
    };

    let index: u32 = append_fardel(store, &owner, fardel.clone())?;
    map_global_id_to_fardel(store, global_id.clone(), &owner, index)?;
    map_hash_id_to_global_id(store, hash_id, global_id.clone())?;
    store_fardel_unpack_count(store, global_id.clone(), 0_u64)?;

    // automatically unpack for the owner
    // TODO: make so can just view own fardels without unpacking
    store_unpack(store, &owner, global_id.clone())?;

    // update global fardel count
    set_bin_data(store, KEY_FARDEL_COUNT, &(global_id + 1))?;

    Ok(global_id)
}

// returns the index of the appended fardel
fn append_fardel<S: Storage>(
    store: &mut S,
    owner: &CanonicalAddr,
    fardel: StoredFardel,
) -> StdResult<u32> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_FARDELS, owner.as_slice()], store);
    let mut storage = AppendStoreMut::<StoredFardel, _>::attach_or_create(&mut storage)?;
    let idx = storage.len();
    storage.push(&fardel)?;
    Ok(idx)
}

// stores a mapping from global fardel id to fardel in prefixed storage
fn map_global_id_to_fardel<S: Storage>(
    store: &mut S,
    global_id: u128,
    owner: &CanonicalAddr,
    index: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_ID_FARDEL_MAPPINGS, store);
    let mapping = GlobalIdFardelMapping {
        owner: owner.clone(),
        index,
    };
    set_bin_data(&mut storage, &global_id.to_be_bytes(), &mapping)
}

//
// Stores the number of times the fardel has been unpacked
//   Pending unpacks count, but cancel will decrement the count again
//   Or always 0 in uncountable fardels.
//   b"fardel-unpack-count" | {fardel global_id} | {count}
//
pub fn store_fardel_unpack_count<S: Storage>(
    storage: &mut S,
    fardel_id: u128,
    new_unpack_count: u64,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_FARDEL_NUM_UNPACKS, storage);
    set_bin_data(&mut storage, &fardel_id.to_be_bytes(), &new_unpack_count)
}

pub fn get_fardel_unpack_count<S: ReadonlyStorage>(storage: &S, fardel_id: u128) -> StdResult<u64> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_FARDEL_NUM_UNPACKS, storage);
    get_bin_data(&storage, &fardel_id.to_be_bytes())
}

pub fn increment_fardel_unpack_count<S: Storage>(storage: &mut S, fardel_id: u128) -> Option<u64> {
    let unpack_count = get_fardel_unpack_count(storage, fardel_id).unwrap_or_else(|_| 0_u64);
    let result = store_fardel_unpack_count(storage, fardel_id, unpack_count + 1);
    match result {
        Ok(_) => Some(unpack_count + 1),
        Err(_) => None,
    }
}

pub fn decrement_fardel_unpack_count<S: Storage>(storage: &mut S, fardel_id: u128) -> Option<u64> {
    let unpack_count = get_fardel_unpack_count(storage, fardel_id).unwrap_or_else(|_| 0_u64);
    if unpack_count < 1 {
        return None;
    }
    let result = store_fardel_unpack_count(storage, fardel_id, unpack_count - 1);
    match result {
        Ok(_) => Some(unpack_count - 1),
        Err(_) => None,
    }
}

// Stores a mapping from hash id to global_id in prefixed storage
//   users see hash ids only.
//   Admin can access fardels directly by global (sequential) id for indexing.
fn map_hash_id_to_global_id<S: Storage>(
    store: &mut S,
    hash_id: u128,
    global_id: u128,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, store);
    set_bin_data(&mut storage, &hash_id.to_be_bytes(), &global_id)
}

pub fn get_fardel_owner<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
) -> StdResult<CanonicalAddr> {
    let mapping_store = ReadonlyPrefixedStorage::new(PREFIX_ID_FARDEL_MAPPINGS, storage);
    let mapping: GlobalIdFardelMapping = get_bin_data(&mapping_store, &fardel_id.to_be_bytes())?;
    Ok(mapping.owner)
}

// gets a fardel by global id
pub fn get_fardel_by_global_id<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
) -> StdResult<Option<Fardel>> {
    let mapping_store = ReadonlyPrefixedStorage::new(PREFIX_ID_FARDEL_MAPPINGS, storage);
    let mapping: GlobalIdFardelMapping = get_bin_data(&mapping_store, &fardel_id.to_be_bytes())?;
    //let mapping: GlobalIdFardelMapping = match get_bin_data(&mapping_store, &fardel_id.to_be_bytes()) {
    //    Ok(m) => m,
    //    _ => { return Err(StdError::generic_err(format!("could not get fardel mapping for global id {}", fardel_id))); }
    //};

    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_FARDELS, mapping.owner.as_slice()], storage);
    // Try to access the storage of fardels for the account.
    // If it doesn't exist yet, return None.
    let store = if let Some(result) = AppendStore::<StoredFardel, _>::attach(&store) {
        result?
    } else {
        return Ok(None);
    };

    let stored_fardel: StoredFardel = match store.get_at(mapping.index) {
        Ok(f) => f,
        _ => {
            return Err(StdError::generic_err(format!(
                "Could not get stored fardel for global id {}",
                fardel_id
            )));
        }
    };
    let fardel: Fardel = stored_fardel.into_humanized()?;
    Ok(Some(fardel))
}

pub fn get_global_id_by_hash<S: ReadonlyStorage>(storage: &S, hash: u128) -> StdResult<u128> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    get_bin_data(&storage, &hash.to_be_bytes())
}

pub fn get_fardel_by_hash<S: ReadonlyStorage>(store: &S, hash: u128) -> StdResult<Option<Fardel>> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, store);
    let global_id: u128 = match get_bin_data(&storage, &hash.to_be_bytes()) {
        Ok(id) => id,
        _ => {
            return Err(StdError::generic_err(format!(
                "could not get global_id for hash_id {}",
                hash
            )));
        }
    };
    get_fardel_by_global_id(store, global_id)
}

/* not implemented
pub fn get_fardel_owner_by_hash<S: ReadonlyStorage>(
    storage: &S,
    hash: u128,
) -> StdResult<CanonicalAddr> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    let global_id: u128 = get_bin_data(&store, &hash.to_be_bytes())?;
    get_fardel_owner(storage, global_id)
}
*/

pub fn get_fardels<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<Fardel>> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FARDELS, owner.as_slice()], storage);

    // Try to access the storage of fardels for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<StoredFardel, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` fardels starting from the latest fardel, potentially skipping `page * page_size`
    // fardels from the start.
    let fardel_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    // The `and_then` here flattens the `StdResult<StdResult<Fardel>>` to an `StdResult<Fardel>`
    let fardels: StdResult<Vec<Fardel>> = fardel_iter
        .map(|fardel| fardel.map(|fardel| fardel.into_humanized()).and_then(|x| x))
        .collect();
    fardels
}

// returns total number of fardels for user
pub fn get_number_of_fardels<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> u32 {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FARDELS, owner.as_slice()], storage);

    // Try to access the storage of fardels for the account.
    // If it doesn't exist yet, return 0.
    if let Some(result) = AppendStore::<StoredFardel, _>::attach(&store) {
        return result.unwrap().len();
    } else {
        return 0;
    };
}

//
//  Sealed fardels
//
//    b"sealed" | {global fardel id} -> true/false
//       value == true means it is sealed, value == false OR no record in storage means not sealed
//

pub fn seal_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_SEALED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &true)
}

/* not implemented
pub fn unseal_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_SEALED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &false)
}
*/

// get sealed status of a given fardel
//  true means sealed, false means not sealed
pub fn get_sealed_status<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_SEALED, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
//  Hidden fardels
//
//    b"hidden" | {global fardel id} -> true/false
//       value == true means it is hidden, value == false OR no record in storage means not hidden
//

pub fn hide_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_HIDDEN, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &true)
}

pub fn unhide_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_HIDDEN, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &false)
}

// get hidden status of a given fardel
//  true means hidden, false means not hidden
pub fn is_fardel_hidden<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_HIDDEN, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
//  Removed fardels
//
//    b"removed" | {global fardel id} -> true/false
//       value == true means it is removed, value == false OR no record in storage means not removed
//

pub fn remove_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_REMOVED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &true)
}

pub fn unremove_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_REMOVED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &false)
}

// get removed status of a given fardel
//  true means removed, false means not removed
pub fn is_fardel_removed<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_REMOVED, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
// Fardel Thumbnail Img
//

// stores a thumbnail img for fardel in prefixed storage
pub fn store_fardel_img<S: Storage>(store: &mut S, fardel_id: u128, img: Vec<u8>) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_FARDEL_THUMBNAIL_IMGS, store);
    set_bin_data(&mut storage, &fardel_id.to_be_bytes(), &img)
}

// gets a thumbnail img for fardel in prefixed storage
pub fn get_fardel_img<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> String {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_FARDEL_THUMBNAIL_IMGS, store);
    let img_vec = get_bin_data(&storage, &fardel_id.to_be_bytes()).unwrap_or_else(|_| vec![]);
    String::from_utf8(img_vec).unwrap()
}
