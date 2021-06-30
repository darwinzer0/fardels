use std::any::type_name;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use cosmwasm_std::{Api, Coin, CanonicalAddr, HumanAddr, Storage, StdError, StdResult, Uint128, 
    ReadonlyStorage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use crate::contract::DENOM;
use crate::viewing_key::ViewingKey;
use crate::msg::Fee;

// Globals
pub static PREFIX_CONFIG: &[u8] = b"config";
pub const KEY_CONSTANTS: &[u8] = b"constants";
pub const KEY_FARDEL_COUNT: &[u8] = b"fardel-count";
pub const KEY_COMMISSION_BALANCE: &[u8] = b"commission";

// Fardel
pub const PREFIX_FARDELS: &[u8] = b"fardel";
pub const PREFIX_FARDEL_THUMBNAIL_IMGS: &[u8] = b"fardel-img";
pub const PREFIX_ID_FARDEL_MAPPINGS: &[u8] = b"id-to-fardel";
pub const PREFIX_HASH_ID_MAPPINGS: &[u8] = b"hash-to-id";
pub const PREFIX_SEALED: &[u8] = b"sealed";
pub const PREFIX_FARDEL_NEXT_PACKAGE: &[u8] = b"next";

// Fardel unpacking
pub const PREFIX_UNPACKED: &[u8] = b"unpacked";
pub const PREFIX_ID_UNPACKED_MAPPINGS: &[u8] = b"id-to-unpacked";
pub const PREFIX_PENDING_UNPACK: &[u8] = b"pending";
pub const PREFIX_PENDING_START: &[u8] = b"pending-start";
pub const PREFIX_ID_PENDING_UNPACKED_MAPPINGS: &[u8] = b"id-to-pending";

// Fardel rating/comments
pub const PREFIX_RATED: &[u8] = b"rated";
pub const PREFIX_UPVOTES: &[u8] = b"upvotes";
pub const PREFIX_DOWNVOTES: &[u8] = b"downvotes";
pub const PREFIX_COMMENTS: &[u8] = b"comments";
pub const PREFIX_DELETED_COMMENTS: &[u8] = b"del-comment";

// Following
pub const PREFIX_FOLLOWING: &[u8] = b"following";
pub const PREFIX_FOLLOWERS: &[u8] = b"followers";
pub const PREFIX_LINK: &[u8] = b"link";
pub const PREFIX_VEC: &[u8] = b"vec";

// Blocked
pub const PREFIX_BLOCKED: &[u8] = b"blocked";

// Accounts
pub const PREFIX_ACCOUNTS: &[u8] = b"account";
pub const PREFIX_ACCOUNT_THUMBNAIL_IMGS: &[u8] = b"account-img";
pub const PREFIX_HANDLES: &[u8] = b"handle";
pub const PREFIX_VIEWING_KEY: &[u8] = b"viewingkey";
pub const PREFIX_DEACTIVATED: &[u8] = b"deactived";

// Banned accounts
pub const PREFIX_BANNED: &[u8] = b"banned";

// Completed transactions
pub const PREFIX_COMPLETED_TX: &[u8] = b"tx";

//
// CONFIG
//

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {
    pub admin: CanonicalAddr,
    pub transaction_fee: StoredFee,
    pub max_query_page_size: u16,

    // fardel settings
    // maximum cost of a fardel
    pub max_cost: u128,
    pub max_public_message_len: u16,
    pub max_tag_len: u8,
    pub max_number_of_tags: u8,
    pub max_fardel_img_size: u32,
    pub max_contents_data_len: u16,

    // user settings
    pub max_handle_len: u16,
    pub max_profile_img_size: u32,
    pub max_description_len: u16,
    
    pub prng_seed: Vec<u8>,
}

pub struct ReadonlyConfig<'a, S: ReadonlyStorage> {
    storage: ReadonlyPrefixedStorage<'a, S>,
}

impl<'a, S: ReadonlyStorage> ReadonlyConfig<'a, S> {
    pub fn from_storage(storage: &'a S) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(PREFIX_CONFIG, storage),
        }
    }

    fn as_readonly(&self) -> ReadonlyConfigImpl<ReadonlyPrefixedStorage<S>> {
        ReadonlyConfigImpl(&self.storage)
    }

    pub fn constants(&self) -> StdResult<Constants> {
        self.as_readonly().constants()
    }
}

pub struct Config<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}

impl<'a, S: Storage> Config<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(PREFIX_CONFIG, storage),
        }
    }

    fn as_readonly(&self) -> ReadonlyConfigImpl<PrefixedStorage<S>> {
        ReadonlyConfigImpl(&self.storage)
    }

    pub fn constants(&self) -> StdResult<Constants> {
        self.as_readonly().constants()
    }

    pub fn set_constants(&mut self, constants: &Constants) -> StdResult<()> {
        set_bin_data(&mut self.storage, KEY_CONSTANTS, constants)
    }
}

/// This struct refactors out the readonly methods that we need for `Config` and `ReadonlyConfig`
/// in a way that is generic over their mutability.
///
/// This was the only way to prevent code duplication of these methods because of the way
/// that `ReadonlyPrefixedStorage` and `PrefixedStorage` are implemented in `cosmwasm-std`
struct ReadonlyConfigImpl<'a, S: ReadonlyStorage>(&'a S);

impl<'a, S: ReadonlyStorage> ReadonlyConfigImpl<'a, S> {
    fn constants(&self) -> StdResult<Constants> {
        let consts_bytes = self
            .0
            .get(KEY_CONSTANTS)
            .ok_or_else(|| StdError::generic_err("no constants stored in configuration"))?;
        bincode2::deserialize::<Constants>(&consts_bytes)
            .map_err(|e| StdError::serialize_err(type_name::<Constants>(), e))
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoredFee {
    pub commission_rate_nom: u128,
    pub commission_rate_denom: u128,
}

impl StoredFee {
    pub fn into_humanized(self) -> StdResult<Fee> {
        let fee = Fee {
            commission_rate_nom: Uint128(self.commission_rate_nom),
            commission_rate_denom: Uint128(self.commission_rate_denom),
        };
        Ok(fee)
    }
}

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
    pub contents_data: Vec<String>,
    pub cost: Coin,
    pub countable: bool,
    pub approval_req: bool,
    pub seal_time: u64,
    pub timestamp: u64,
}

impl Fardel {
    pub fn into_stored(self) -> StdResult<StoredFardel> {
        let stored_tags = self.tags.iter().map(|tag|
            tag.as_bytes().to_vec()
        ).collect();
        let stored_contents_data = self.contents_data.iter().map(|package|
            package.as_bytes().to_vec()
        ).collect();
        let fardel = StoredFardel {
            global_id: self.global_id.u128(),
            hash_id: self.hash_id.u128(),
            public_message: self.public_message.as_bytes().to_vec(),
            tags: stored_tags,
            contents_data: stored_contents_data,
            cost: self.cost.amount.u128(),
            countable: self.countable,
            approval_req: self.approval_req,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }

    pub fn number_of_packages(self) -> u16 {
        self.contents_data.len() as u16
    }

    pub fn number_of_packages_left<S: ReadonlyStorage>(self, storage: &S) -> u16 {
        let next_package = get_fardel_next_package(storage, self.global_id.u128()).unwrap_or_else(|_| 0_u16);
        let total = self.contents_data.len() as u16;
        0_u16.min(total - next_package)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredFardel {
    pub global_id: u128,
    pub hash_id: u128,
    pub public_message: Vec<u8>,
    pub tags: Vec<Vec<u8>>,
    pub contents_data: Vec<Vec<u8>>,
    pub cost: u128,
    pub countable: bool,
    pub approval_req: bool,
    pub seal_time: u64,
    pub timestamp: u64,
}

impl StoredFardel {
    pub fn into_humanized(self) -> StdResult<Fardel> {
        let humanized_tags = self.tags.iter().map(|tag|
            String::from_utf8(tag.clone()).ok().unwrap_or_default()
        ).collect();
        let humanized_contents_data = self.contents_data.iter().map(|package|
            String::from_utf8(package.clone()).ok().unwrap_or_default()
        ).collect();
        let fardel = Fardel {
            global_id: Uint128(self.global_id),
            hash_id: Uint128(self.hash_id),
            public_message: String::from_utf8(self.public_message).ok().unwrap_or_default(),
            tags: humanized_tags,
            contents_data: humanized_contents_data,
            cost: Coin { amount: Uint128(self.cost), denom: DENOM.to_string() },
            countable: self.countable,
            approval_req: self.approval_req,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }

    pub fn number_of_packages(self) -> u16 {
        self.contents_data.len() as u16
    }

    pub fn number_of_packages_left<S: ReadonlyStorage>(self, storage: &S) -> u16 {
        let next_package = get_fardel_next_package(storage, self.global_id).unwrap_or_else(|_| 0_u16);
        let total = self.contents_data.len() as u16;
        0_u16.min(total - next_package)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalIdFardelMapping {
    pub owner: CanonicalAddr,
    pub index: u32,
}

pub fn get_total_fardel_count<S: ReadonlyStorage>(
    storage: &S
) -> u128 {
    get_bin_data(storage, KEY_FARDEL_COUNT).unwrap_or_else(|_| 0_u128)
}

pub fn store_fardel<S: Storage>(
    store: &mut S,
    hash_id: u128,
    owner: &CanonicalAddr,
    public_message: Vec<u8>,
    tags: Vec<Vec<u8>>,
    contents_data: Vec<Vec<u8>>,
    cost: u128,
    countable: bool,
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
    store_fardel_next_package(store, global_id.clone(), 0_u16)?;

    // automatically unpack for the owner
    // TODO: make so can just view own fardels without unpacking
    for (i, _) in contents_data.iter().enumerate() {
        store_unpack(store, &owner, global_id.clone(), i as u16)?;
    }

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
        index
    };
    set_bin_data(&mut storage, &global_id.to_be_bytes(), &mapping)
}

//
// Stores the current index of the next package in countable fardels,
//   Or always 0 in uncountable fardels.
//   b"next" | {fardel global_id} | {next package idx}
//
pub fn store_fardel_next_package<S: Storage>(
    store: &mut S,
    fardel_id: u128,
    next_package: u16,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_FARDEL_NEXT_PACKAGE, store);
    set_bin_data(&mut storage, &fardel_id.to_be_bytes(), &next_package)
}

pub fn get_fardel_next_package<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
) -> StdResult<u16> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_FARDEL_NEXT_PACKAGE, storage);
    get_bin_data(&storage, &fardel_id.to_be_bytes())
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

pub fn get_fardel_by_id<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
) -> StdResult<Option<Fardel>> {
    let mapping_store = ReadonlyPrefixedStorage::new(PREFIX_ID_FARDEL_MAPPINGS, storage);
    let mapping: GlobalIdFardelMapping = get_bin_data(&mapping_store, &fardel_id.to_be_bytes())?;
    //let mapping: GlobalIdFardelMapping = match get_bin_data(&mapping_store, &fardel_id.to_be_bytes()) {
    //    Ok(m) => m,
    //    _ => { return Err(StdError::generic_err(format!("could not get fardel mapping for global id {}", fardel_id))); }
    //};

    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FARDELS, mapping.owner.as_slice()], storage);
    // Try to access the storage of fardels for the account.
    // If it doesn't exist yet, return None.
    let store = if let Some(result) = AppendStore::<StoredFardel, _>::attach(&store) {
        result?
    } else {
        return Ok(None);
    };

    //let stored_fardel: StoredFardel = store.get_at(mapping.index)?;
    let stored_fardel: StoredFardel = match store.get_at(mapping.index) {
        Ok(f) => f,
        _ => { return Err(StdError::generic_err(format!("could not get stored fardel for global id {}", fardel_id)));}, 
    };
    let fardel: Fardel = stored_fardel.into_humanized()?;
    Ok(Some(fardel))
}

pub fn get_global_id_by_hash<S: ReadonlyStorage>(
    storage: &S,
    hash: u128,
) -> StdResult<u128> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    get_bin_data(&storage, &hash.to_be_bytes())
}

pub fn get_fardel_by_hash<S: ReadonlyStorage>(
    store: &S,
    hash: u128,
) -> StdResult<Option<Fardel>> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, store);
    let global_id: u128 = match get_bin_data(&storage, &hash.to_be_bytes()) {
        Ok(id) => id,
        _ => { return Err(StdError::generic_err(format!("could not get global_id for hash_id {}", hash))); }
    };
    get_fardel_by_id(store, global_id)
}

pub fn get_fardel_owner_by_hash<S: ReadonlyStorage>(
    storage: &S,
    hash: u128,
) -> StdResult<CanonicalAddr> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    let global_id: u128 = get_bin_data(&store, &hash.to_be_bytes())?;
    get_fardel_owner(storage, global_id)
}

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

//
//  Sealed fardels
//
//    b"sealed" | {global fardel id} -> true/false
//       value == true means it is sealed, value == false OR no record in storage means not sealed
//

pub fn seal_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_SEALED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &true)
}

pub fn unseal_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_SEALED, store);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &false)
}

// get sealed status of a given fardel
//  true means sealed, false means not sealed
pub fn get_sealed_status<S: Storage>(
    store: &S, 
    fardel_id: u128,
) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_SEALED, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
// Fardel Thumbnail Img
//

// stores a thumbnail img for fardel in prefixed storage
pub fn store_fardel_img<S: Storage>(
    store: &mut S,
    fardel_id: u128,
    img: Vec<u8>,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_FARDEL_THUMBNAIL_IMGS, store);
    set_bin_data(&mut storage, &fardel_id.to_be_bytes(), &img)
}

// gets a thumbnail img for fardel in prefixed storage
pub fn get_fardel_img<S: Storage>(
    store: &S,
    fardel_id: u128,
) -> String {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_FARDEL_THUMBNAIL_IMGS, store);
    let img_vec = get_bin_data(&storage, &fardel_id.to_be_bytes()).unwrap_or_else(|_| vec![]);
    String::from_utf8(img_vec).unwrap()
}

//
// User accounts
//

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Account {
    pub owner: HumanAddr,
    pub handle: String,
    pub description: String,
}

impl Account {
    pub fn into_stored<A: Api>(self, api: &A) -> StdResult<StoredAccount> {
        let account = StoredAccount {
            owner: api.canonical_address(&self.owner)?,
            handle: self.handle.as_bytes().to_vec(),
            description: self.description.as_bytes().to_vec(),
        };
        Ok(account)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredAccount {
    pub owner: CanonicalAddr,
    pub handle: Vec<u8>,
    pub description: Vec<u8>,
}

impl StoredAccount {
    pub fn into_humanized<A: Api>(self, api: &A) -> StdResult<Account> {
        let account = Account {
            owner: api.human_address(&self.owner)?,
            handle: String::from_utf8(self.handle).ok().unwrap_or_default(),
            description: String::from_utf8(self.description).ok().unwrap_or_default(),
        };
        Ok(account)
    }
}

pub fn store_account<S: Storage>(
    store: &mut S,
    account: StoredAccount,
    owner: &CanonicalAddr,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_ACCOUNTS, store);
    set_bin_data(&mut store, &owner.as_slice(), &account)
}

pub fn get_account<S: ReadonlyStorage>(
    store: &S,
    owner: &CanonicalAddr,
) -> StdResult<StoredAccount> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_ACCOUNTS, store);
    get_bin_data(&store, &owner.as_slice())
}

//
// Handle to account mapping -- allows look up by handle, not address
//

pub fn map_handle_to_account<S: Storage>(
    store: &mut S, 
    owner: &CanonicalAddr, 
    handle: String,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_HANDLES, store);
    set_bin_data(&mut store, handle.as_bytes(), &owner)
}

// this is meant to be called after handle has been changed in account
pub fn delete_handle_map<S: Storage>(
    store: &mut S,
    handle: String,
) {
    let mut store = PrefixedStorage::new(PREFIX_HANDLES, store);
    store.remove(handle.as_bytes())
}

pub fn get_account_for_handle<S: Storage>(
    store: &S, 
    handle: &String
) -> StdResult<CanonicalAddr> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_HANDLES, store);
    get_bin_data(&store, handle.as_bytes())
}

//
// Account Thumbnail Img
//

// stores a thumbnail img for account in prefixed storage
pub fn store_account_img<S: Storage>(
    store: &mut S,
    owner: &CanonicalAddr,
    img: Vec<u8>,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_ACCOUNT_THUMBNAIL_IMGS, store);
    set_bin_data(&mut store, &owner.as_slice(), &img)
}

// gets a thumbnail img for account in prefixed storage
pub fn get_account_img<S: Storage>(
    store: &S,
    owner: &CanonicalAddr,
) -> StdResult<Vec<u8>> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_ACCOUNT_THUMBNAIL_IMGS, store);
    get_bin_data(&store, &owner.as_slice())
}

//
// Viewing Keys
//
pub fn write_viewing_key<S: Storage>(store: &mut S, owner: &CanonicalAddr, key: &ViewingKey) {
    let mut user_key_store = PrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.set(owner.as_slice(), &key.to_hashed());
}

pub fn read_viewing_key<S: Storage>(store: &S, owner: &CanonicalAddr) -> Option<Vec<u8>> {
    let user_key_store = ReadonlyPrefixedStorage::new(PREFIX_VIEWING_KEY, store);
    user_key_store.get(owner.as_slice())
}

//
// Deactivated accounts
//

pub fn store_account_deactivated<S: Storage>(
    store: &mut S,
    account: &CanonicalAddr,
    deactivated: bool,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_DEACTIVATED, store);
    set_bin_data(&mut store, &account.as_slice(), &deactivated)
}

// returns true is account is deactivated
pub fn is_deactivated<S: Storage>(
    store: &S,
    account: &CanonicalAddr,
) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_DEACTIVATED, store);
    get_bin_data(&store, &account.as_slice()).unwrap_or_else(|_| false)
}

//
// Banned accounts
//

pub fn store_account_ban<S: Storage>(
    store: &mut S,
    account: &CanonicalAddr,
    banned: bool,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_BANNED, store);
    set_bin_data(&mut store, &account.as_slice(), &banned)
}

// returns true is account is banned
pub fn is_banned<S: Storage>(
    store: &S,
    account: &CanonicalAddr,
) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_BANNED, store);
    get_bin_data(&store, &account.as_slice()).unwrap_or_else(|_| false)
}

//
// Following / Follower
//
//   b"following" | {owner canonical addr} | b"link" | {followed canonical addr} -> v_index
//   b"following" | {owner canonical addr} | b"vec" | {appendstore index} -> Following (active = true means following)
//   b"followers" | {owner canonical addr} | b"link" | {follower canonical addr} -> v_index
//   b"followers" | {owner canonical addr} | b"vec" | {appendstore index} -> Follower (active = true means follower)
//
// addresses are saved rather than handles, in case followed user changes handle
//

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Following {
    pub who: CanonicalAddr,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Follower {
    pub who: CanonicalAddr,
    pub active: bool,
}

pub fn store_following<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let followed_addr = get_account_for_handle(storage, &handle)?;

    save_following_relation(storage, &owner, &followed_addr)?;
    save_follower_relation(storage, &owner, &followed_addr)?;
    Ok(())
}

pub fn is_following<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> bool {
    let link_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
    let result: StdResult<u32> = get_bin_data(&link_storage, &followed_addr.as_slice());
    match result {
        Ok(index) => {
            let vec_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
            if let Some(vec_store_result) = AppendStore::<Following, _>::attach(&vec_storage) {
                let following = vec_store_result.unwrap().get_at(index);
                match following {
                    Ok(f) => return f.active,
                    Err(_) => return false,
                }
            } else {
                return false;
            };
        },
        Err(_) => false,
    }
}

fn save_following_relation<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> StdResult<()> {
    // save following relation

    // get length of following appendstore
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);

    // get current idx or add to end
    let idx: u32 = get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage_len);
    // prepare following relationship
    let following = Following {
        who: followed_addr.clone(),
        active: true,
    };

    // now store the following relation

    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;
    
    if idx == vec_storage_len {
        vec_storage.push(&following)?;
    } else {
        vec_storage.set_at(idx, &following)?;
    }

    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
    if idx == vec_storage_len {
        set_bin_data(&mut link_storage, followed_addr.as_slice(), &idx)?;
    }
    Ok(())
}

fn save_follower_relation<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> StdResult<()> {
    // save follower relation
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();
    
    let link_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK], storage);
     
    let idx: u32 = get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage_len);
    let follower = Follower {
        who: owner.clone(),
        active: true,
    };

    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    if idx == vec_storage_len {
        vec_storage.push(&follower)?;
    } else {
        vec_storage.set_at(idx, &follower)?;
    }
    
    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK], storage);
    if idx == vec_storage_len {
        set_bin_data(&mut link_storage, owner.as_slice(), &idx)?;
    }
    
    Ok(())
}

pub fn remove_following<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let followed_addr = get_account_for_handle(storage, &handle)?;

    delete_following_relation(storage, &owner, &followed_addr)?;
    delete_follower_relation(storage, &owner, &followed_addr)?;

    Ok(())
}

fn delete_following_relation<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> StdResult<()> {
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
    // update following relation, active = false

    // get current idx and set to false or ignore if not following
    let idx: u32 = get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage_len);

    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;

    if idx < vec_storage_len {
        let mut following: Following = vec_storage.get_at(idx)?;
        following.active = false;
        vec_storage.set_at(idx, &following)?;
    } 

    Ok(())
}

fn delete_follower_relation<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> StdResult<()> {
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK], storage);
    let idx: u32 = get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage_len);

    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;

    if idx < vec_storage_len {
        let mut follower: Follower = vec_storage.get_at(idx)?;
        follower.active = false;
        vec_storage.set_at(idx, &follower)?;
    } 
    Ok(())
}

// returns a vec of handles
pub fn get_following<A:Api, S: Storage>(
    api: &A,
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<String>>{
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWING, owner.as_slice(), PREFIX_VEC], storage);

    // Try to access the storage of following for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<Following, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` following starting from the latest following, potentially skipping `page * page_size`
    // following from the start. Also filters non-active following.
    let following_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    let result = following_iter
        .filter(|following| following.clone().as_ref().unwrap().active)
        .map(|following| { 
            let followed = following.unwrap().who;
            let followed_account: Account = get_account(storage, &followed).unwrap().into_humanized(api).unwrap();
            followed_account.handle
        }).collect();
    Ok(result)
}

pub fn get_followers<A:Api, S: Storage>(
    api: &A,
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<String>>{
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, owner.as_slice(), PREFIX_VEC], storage);

    // Try to access the storage of followers for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<Follower, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` following starting from the latest followers, potentially skipping `page * page_size`
    // followers from the start. Also filters non-active followers.
    let follower_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    let result = follower_iter
        .filter(|follower| follower.clone().as_ref().unwrap().active)
        .map(|follower| { 
            let follower = follower.unwrap().who;
            let follower_account: Account = get_account(storage, &follower).unwrap().into_humanized(api).unwrap();
            follower_account.handle
        }).collect();
    Ok(result)
}

//
// Blocked accounts
//
// are stored using multilevel prefixed keys:
//     b"blocked" | {blocker canonical addr} | {blocked canonical addr} -> bool
//

pub fn store_account_block<S: Storage>(
    storage: &mut S,
    blocker_addr: &CanonicalAddr,
    blocked_addr: &CanonicalAddr,
    blocked: bool,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_BLOCKED, blocker_addr.as_slice()], storage);
    set_bin_data(&mut store, &blocked_addr.as_slice(), &blocked)
}

// returns true if blocked_addr is blocked by blocker_addr
pub fn is_blocked_by<S: ReadonlyStorage>(
    storage: &S,
    blocker_addr: &CanonicalAddr,
    blocked_addr: &CanonicalAddr,
) -> bool {
    let storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_BLOCKED, blocker_addr.as_slice()], storage);
    get_bin_data(&storage, &blocked_addr.as_slice()).unwrap_or_else(|_| false)
}

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
    pub package_idx: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnpackedStatus {
    pub unpacked: bool,
    pub package_idx: u16,
}

pub fn store_unpack<S: Storage>(
    storage: &mut S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
    package_idx: u16,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);
    let mut store = AppendStoreMut::<UnpackedFardel, _>::attach_or_create(&mut store)?;
    
    let unpacked_fardel = UnpackedFardel {
        fardel_id,
        package_idx,
    };
    store.push(&unpacked_fardel)?;
    let unpacked_status = UnpackedStatus {
        unpacked: true,
        package_idx: package_idx,
    };
    map_global_id_to_unpacked_by_unpacker(storage, fardel_id, unpacker, unpacked_status)
}

// stores a mapping from global fardel id to unpacked status in prefixed storage
fn map_global_id_to_unpacked_by_unpacker<S: Storage>(
    store: &mut S,
    global_id: u128,
    unpacker: &CanonicalAddr,
    value: UnpackedStatus,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()], store);
    set_bin_data(&mut storage, &global_id.to_be_bytes(), &value)
}

// get the unpacked status of a fardel for a given unpacker canonical address
pub fn get_unpacked_status_by_fardel_id<S: Storage>(
    storage: &S, 
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> UnpackedStatus {
    let mapping_store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()], storage);
    get_bin_data(&mapping_store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| UnpackedStatus{
        unpacked: false,
        package_idx: 0,
    })
}

// gets a list of unpacked fardels for a given unpacker canonical address
pub fn get_unpacked_by_unpacker<S: Storage>(
    storage: &S, 
    unpacker: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<u128>> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);

    // Try to access the storage of unpacked for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<u128, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` unpacked starting from the latest unpacked, potentially skipping `page * page_size`
    // unpacked from the start.
    let unpacked_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    // The `and_then` here flattens the `StdResult<StdResult<u128>>` to an `StdResult<u128>`
    let unpacked: StdResult<Vec<u128>> = unpacked_iter
        .map(|fardel_id| fardel_id)
        .collect();
    unpacked
}

//
// Pending unpacked fardels
//
// are stored in appendstore for each owner
//   b"pending" | {owner canonical addr} | {appendstore idx}
// with
//   b"pending-start" | {owner canonical addr} | index
//

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingUnpack {
    pub fardel_id: u128,
    pub package_idx: u16,
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

pub fn get_pending_start<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
) -> u32 {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_PENDING_START, storage);
    get_bin_data(&storage, owner.as_slice()).unwrap_or_else(|_| 0_u32)
}

pub fn store_pending_unpack<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
    package_idx: u16,
    sent_funds: Coin,
    timestamp: u64,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_PENDING_UNPACK, owner.as_slice()], storage);
    let mut store = AppendStoreMut::<PendingUnpack, _>::attach_or_create(&mut store)?;
    
    let pending_unpack = PendingUnpack {
        fardel_id,
        package_idx,
        unpacker: unpacker.clone(),
        coin: sent_funds,
        timestamp,
        canceled: false,
    };
    store.push(&pending_unpack)?;
    let pending_unpack_idx = store.len() - 1;
    map_global_id_to_pending_unpacked_by_unpacker(storage, fardel_id, unpacker, pending_unpack_idx, true)
}

// gets an individual pending unpacked fardel for a given owner canonical address
pub fn get_pending_unpack<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    idx: u32,
) -> StdResult<PendingUnpack> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_PENDING_UNPACK, owner.as_slice()], storage);
    // Try to access the storage of pending unpacks for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<PendingUnpack, _>::attach(&store) {
        result?
    } else {
        return Err(StdError::generic_err("no pending unpacks for this owner"));
    };
    store.get_at(idx)
}

// gets a list of pending unpacked fardels for a given owner canonical address
pub fn get_pending_unpacks_from_start<S: ReadonlyStorage>(
    storage: &S, 
    owner: &CanonicalAddr,
    number: u32,
) -> StdResult<Vec<PendingUnpack>> {
    let start = get_pending_start(storage, owner);
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_PENDING_UNPACK, owner.as_slice()], storage);

    // Try to access the storage of unpacked for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<PendingUnpack, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `number` unpacked starting from the latest unpacked, potentially skipping `start`
    // unpacked from the start.
    let unpacked_iter = store
        .iter()
        .skip(start as _)
        .take(number as _);
    let unpacked: StdResult<Vec<PendingUnpack>> = unpacked_iter
        .map(|pending_unpack| pending_unpack)
        .collect();
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
    let mut store = PrefixedStorage::multilevel(&[PREFIX_ID_PENDING_UNPACKED_MAPPINGS, unpacker.as_slice()], store);
    let my_pending_unpack = MyPendingUnpack {
        pending_unpack_idx,
        value,
    };
    set_bin_data(&mut store, &global_id.to_be_bytes(), &my_pending_unpack)
}

// get the pending unpacked status of a fardel for a given unpacker canonical address
pub fn get_pending_unpacked_status_by_fardel_id<S: Storage>(
    storage: &S, 
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> MyPendingUnpack {
    let mapping_store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_ID_PENDING_UNPACKED_MAPPINGS, unpacker.as_slice()], storage);
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
        let mut pending_unpack = get_pending_unpack(storage, &owner, my_pending_unpack.pending_unpack_idx)?;
        pending_unpack.canceled = true;
        let mut store = PrefixedStorage::multilevel(&[PREFIX_PENDING_UNPACK, owner.as_slice()], storage);
        let mut store = AppendStoreMut::<PendingUnpack, _>::attach_or_create(&mut store)?;
        // update element to canceled
        store.set_at(my_pending_unpack.pending_unpack_idx, &pending_unpack)?;
        map_global_id_to_pending_unpacked_by_unpacker(storage, fardel_id, unpacker, my_pending_unpack.pending_unpack_idx, false)
    } else {
        return Err(StdError::generic_err("cannot cancel unpack that is not pending."));
    }
}

//
// Fardel rating and comments
//   there are no limits on number of downvotes/upvotes/comments made but each costs gas
//
// Record of whether an address has rated a fardel with upvote or downvote
//    b"rated" | {rater canonical addr} | {fardel_id} -> bool
//
// Upvotes are stored using prefixed storage:
//    b"upvotes" | {fardel_id} -> upvote count
//
// Downvotes are stored using prefixed storage:
//    b"downvotes" | {fardel_id} -> downvote count
//
// Comments are stored using multilevel prefixed + appendstore keys: 
//    b"comments" | {fardel id} | {appendstore index} -> Comment
//
pub fn set_rated<S: Storage>(
    storage: &mut S,
    rater: &CanonicalAddr,
    fardel_id: u128,
    rating: bool,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_RATED, rater.as_slice()], storage);
    set_bin_data(&mut storage, &fardel_id.to_be_bytes(), &rating)
}

pub fn has_rated<S: ReadonlyStorage>(
    storage: &S,
    rater: &CanonicalAddr,
    fardel_id: u128,
) -> bool {
    let storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_RATED, rater.as_slice()], storage);
    let result: StdResult<bool> = get_bin_data(&storage, &fardel_id.to_be_bytes());
    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn remove_rated<S: Storage>(
    storage: &mut S,
    rater: &CanonicalAddr,
    fardel_id: u128,
) {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_RATED, rater.as_slice()], storage);
    storage.remove(&fardel_id.to_be_bytes())
}

pub fn get_rating<S: ReadonlyStorage>(
    storage: &S,
    rater: &CanonicalAddr,
    fardel_id: u128,
) -> StdResult<bool> {
    let storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_RATED, rater.as_slice()], storage);
    get_bin_data(&storage, &fardel_id.to_be_bytes())
}

pub fn add_upvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_UPVOTES, store);
    let upvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(upvotes + 1))
}

pub fn subtract_upvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_UPVOTES, store);
    let upvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(upvotes - 1))
}

pub fn get_upvotes<S: Storage>(
    store: &S,
    fardel_id: u128,
) -> u32 {
    let store = ReadonlyPrefixedStorage::new(PREFIX_UPVOTES, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32)
}

pub fn add_downvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_DOWNVOTES, store);
    let downvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(downvotes + 1))
}

pub fn subtract_downvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_DOWNVOTES, store);
    let downvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(downvotes - 1))
}

pub fn get_downvotes<S: Storage>(
    store: &S,
    fardel_id: u128,
) -> u32 {
    let store = ReadonlyPrefixedStorage::new(PREFIX_DOWNVOTES, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Comment {
    pub commenter: CanonicalAddr,
    pub text: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexedComment {
    pub commenter: CanonicalAddr,
    pub text: Vec<u8>,
    pub idx: u32,
}

pub fn comment_on_fardel<S: Storage>(
    storage: &mut S,
    commenter: &CanonicalAddr,
    fardel_id: u128,
    text: String,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);
    let mut store = AppendStoreMut::<Comment, _>::attach_or_create(&mut store)?;
    let comment = Comment {
        commenter: commenter.clone(),
        text: text.as_bytes().to_vec(),
    };
    store.push(&comment)
}

// get paginated comments for a given fardel
pub fn get_comments<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<IndexedComment>> {
    let fardel_owner = get_fardel_owner(storage, fardel_id)?;
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);

    // Try to access the storage of comments for the fardel.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<Comment, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` comments starting from the latest comment, potentially skipping `page * page_size`
    // comments from the start. Add in the index of the comment for the fardel.
    let comments_iter = store
        .iter()
        .enumerate()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    let comments: StdResult<Vec<IndexedComment>> = comments_iter
        .map(|(idx, comment)| {
            let comment = comment.unwrap();
            Ok(
                IndexedComment {
                    commenter: comment.commenter.clone(),
                    text: comment.text,
                    idx: idx as u32,
                }
            )
        })
        .filter(|comment| {
            let comment = comment.as_ref().unwrap();
            // check if deleted
            let deleted = comment_is_deleted(storage, fardel_id, comment.idx);
            // check if blocked
            let blocked = is_blocked_by(storage, &fardel_owner, &comment.commenter);
            !deleted && !blocked
        })
        .collect();
    comments
}

// get total number of comments for a fardel
pub fn get_number_of_comments<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
) -> u32 {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);

    // Try to access the storage of comments for the fardel.
    // If it doesn't exist yet, return 0.
    let store_result = if let Some(result) = AppendStore::<Comment, _>::attach(&store) {
        result
    } else {
        return 0_u32;
    };
    if store_result.is_ok() {
        store_result.unwrap().len()
    } else {
        0_u32
    }
}

pub fn get_comment_by_id<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
    comment_id: u32,
) -> StdResult<Comment> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);
    // Try to access the storage of comments for the fardel.
    // If it doesn't exist yet, return 0.
    let store = if let Some(result) = AppendStore::<Comment, _>::attach(&store) {
        result?
    } else {
        return Err(StdError::generic_err("no comment at that index for that fardel."));
    };
    store.get_at(comment_id)
}

pub fn delete_comment<S: Storage>(
    storage: &mut S,
    fardel_id: u128,
    comment_id: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_DELETED_COMMENTS, &fardel_id.to_be_bytes()], storage);
    set_bin_data(&mut storage, &comment_id.to_be_bytes(), &true)    
}

fn comment_is_deleted<S: ReadonlyStorage>(
    storage: &S,
    fardel_id: u128,
    comment_id: u32,
) -> bool {
    let storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_DELETED_COMMENTS, &fardel_id.to_be_bytes()], storage);
    get_bin_data(&storage, &comment_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
// Commission balance
//
pub fn get_commission_balance<S: Storage>(
    storage: &S,
) -> u128 {
    get_bin_data(storage, KEY_COMMISSION_BALANCE).unwrap_or_else(|_| 0_u128)
}

pub fn add_to_commission_balance<S: Storage>(
    storage: &mut S,
    amount: u128,
) -> StdResult<()> {
    let current_amount = get_commission_balance(storage);
    set_bin_data(storage, KEY_COMMISSION_BALANCE, &(current_amount + amount))
}

pub fn subtract_from_commission_balance<S: Storage>(
    storage: &mut S,
    amount: u128,
) -> StdResult<()> {
    let current_amount = get_commission_balance(storage);
    if current_amount < amount {
        return Err(StdError::generic_err("Cannot subtract more than current commission amount."));
    }
    set_bin_data(storage, KEY_COMMISSION_BALANCE, &(current_amount - amount))
}

//
// Transaction record
//
//  b"tx" | {owner canonical address} | appendstore | Tx
//
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Tx {
    pub fardel_id: Uint128,
    pub package_idx: i32,
    pub handle: String,
    pub amount: Uint128,
    pub fee: Uint128,
    pub timestamp: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredTx {
    pub fardel_id: u128,
    pub package_idx: u32,
    pub unpacker: CanonicalAddr,
    pub amount: u128,
    pub fee: u128,
    pub timestamp: u64,
}

impl StoredTx {
    pub fn into_humanized<S: ReadonlyStorage>(self, storage: &S) -> StdResult<Tx> {
        let fardel = get_fardel_by_id(storage, self.fardel_id)?.unwrap();
        let unpacker = get_account(storage, &self.unpacker)?;
        let tx = Tx {
            fardel_id: fardel.hash_id,
            package_idx: self.package_idx as i32,
            handle: String::from_utf8(unpacker.handle).ok().unwrap_or_default(),
            amount: Uint128(self.amount),
            fee: Uint128(self.fee),
            timestamp: self.timestamp as i32,
        };
        Ok(tx)
    }
}

// returns the index of the appended fardel
fn append_tx<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    unpacker: CanonicalAddr,
    fardel_id: u128,
    package_idx: u32,
    amount: u128,
    fee: u128,
    timestamp: u64,
) -> StdResult<u32> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_COMPLETED_TX, owner.as_slice()], storage);
    let mut store = AppendStoreMut::<StoredTx, _>::attach_or_create(&mut store)?;
    let tx = StoredTx {
        fardel_id,
        package_idx,
        unpacker,
        amount,
        fee,
        timestamp,
    };
    store.push(&tx)?;
    Ok(store.len() - 1)
}

pub fn get_txs<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<Tx>> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMPLETED_TX, owner.as_slice()], storage);

    // Try to access the storage of txs for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<StoredTx, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let tx_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    // The `and_then` here flattens the `StdResult<StdResult<Tx>>` to an `StdResult<Tx>`
    let txs: StdResult<Vec<Tx>> = tx_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(storage)).and_then(|x| x))
        .collect();
    txs
}

//
// Bin data storage setters and getters
//

fn set_bin_data<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], data: &T) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;

    storage.set(key, &bin_data);
    Ok(())
}

fn get_bin_data<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    let bin_data = storage.get(key);

    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}