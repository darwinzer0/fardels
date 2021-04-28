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

// Fardel unpacking
pub const PREFIX_UNPACKED: &[u8] = b"unpacked";
pub const PREFIX_ID_UNPACKED_MAPPINGS: &[u8] = b"id-to-unpacked";

// Fardel rating/comments
pub const PREFIX_UPVOTES: &[u8] = b"upvotes";
pub const PREFIX_DOWNVOTES: &[u8] = b"downvotes";
pub const PREFIX_COMMENTS: &[u8] = b"comments";

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

// Pending transactions
pub const PREFIX_PENDING_TX: &[u8] = b"pending-tx";

// Completed transactions
pub const PREFIX_COMPLETED_TX: &[u8] = b"tx";

//
// CONFIG
//

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {
    pub admin: CanonicalAddr,
    pub transaction_fee: Fee,
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

    pub fn fardel_count(&self) -> u128 {
        self.as_readonly().fardel_count()
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

    pub fn fardel_count(&self) -> u128 {
        self.as_readonly().fardel_count()
    }

    pub fn set_fardel_count(&mut self, count: u128) -> StdResult<()> {
        set_bin_data(&mut self.storage, KEY_FARDEL_COUNT, &count)
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

    pub fn fardel_count(&self) -> u128 {
        get_bin_data(self.0, KEY_FARDEL_COUNT).unwrap_or_default()
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
    pub next_package: u16,
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
            next_package: self.next_package,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }

    pub fn number_of_packages(self) -> u16 {
        self.contents_data.len() as u16
    }

    pub fn number_of_packages_left(self) -> u16 {
        let total = self.contents_data.len() as u16;
        0_u16.min(total - self.next_package)
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
    pub next_package: u16,
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
            next_package: self.next_package,
            seal_time: self.seal_time,
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }

    pub fn number_of_packages(self) -> u16 {
        self.contents_data.len() as u16
    }

    pub fn number_of_packages_left(self) -> u16 {
        let total = self.contents_data.len() as u16;
        0_u16.min(total - self.next_package)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GlobalIdFardelMapping {
    pub owner: CanonicalAddr,
    pub index: u32,
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
    next_package: u16,
    seal_time: u64,
    timestamp: u64,
) -> StdResult<()> {
    let mut config = Config::from_storage(store);
    let global_id = config.fardel_count();
    config.set_fardel_count(global_id + 1)?;

    let fardel = StoredFardel {
        global_id,
        hash_id,
        public_message: public_message.clone(),
        tags: tags.clone(),
        contents_data: contents_data.clone(),
        cost,
        countable,
        approval_req,
        next_package,
        seal_time,
        timestamp,
    };

    let index = append_fardel(store, &owner, fardel.clone())?;
    map_global_id_to_fardel(store, global_id, &owner, index)?;
    map_hash_id_to_global_id(store, hash_id, global_id)?;
    // automatically unpack for the owner
    store_unpack(store, &owner, global_id)?;
    Ok(())
}

// returns the index of the appended fardel
fn append_fardel<S: Storage>(
    store: &mut S,
    owner: &CanonicalAddr,
    fardel: StoredFardel,
) -> StdResult<u32> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_FARDELS, owner.as_slice()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    store.push(&fardel)?;
    Ok(store.len() - 1)
}

// stores a mapping from global fardel id to fardel in prefixed storage
fn map_global_id_to_fardel<S: Storage>(
    store: &mut S,
    global_id: u128,
    owner: &CanonicalAddr,
    index: u32,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_ID_FARDEL_MAPPINGS, store);
    let mapping = GlobalIdFardelMapping {
        owner: owner.clone(),
        index
    };
    set_bin_data(&mut store, &global_id.to_be_bytes(), &mapping)
}

// Stores a mapping from hash id to global_id in prefixed storage
//   users see hash ids only. 
//   Admin can access fardels directly by global (sequential) id for indexing.
fn map_hash_id_to_global_id<S: Storage>(
    storage: &mut S,
    hash_id: u128,
    global_id: u128,
) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    set_bin_data(&mut store, &hash_id.to_be_bytes(), &global_id)
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

    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_FARDELS, mapping.owner.as_slice()], storage);
    // Try to access the storage of fardels for the account.
    // If it doesn't exist yet, return None.
    let store = if let Some(result) = AppendStore::<StoredFardel, _>::attach(&store) {
        result?
    } else {
        return Ok(None);
    };

    let stored_fardel: StoredFardel = store.get_at(mapping.index)?;
    let fardel: Fardel = stored_fardel.into_humanized()?;
    Ok(Some(fardel))
}

pub fn get_fardel_by_hash<S: ReadonlyStorage>(
    storage: &S,
    hash: u128,
) -> StdResult<Option<Fardel>> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_HASH_ID_MAPPINGS, storage);
    let global_id: u128 = get_bin_data(&storage, &hash.to_be_bytes())?;
    get_fardel_by_id(&storage, global_id)
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

pub fn get_account<S: Storage>(
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

pub fn store_following<A: Api, S: Storage>(
    api: &A,
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let followed_addr = get_account_for_handle(storage, &handle)?;

    // save following relation

    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::attach_or_create(&mut vec_storage)?;
    
    // get current idx or add to end
    let idx: u32 = get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage.len());
    let following = Following {
        who: followed_addr,
        active: true,
    };
    if idx == vec_storage.len() {
        vec_storage.push(&following);
        set_bin_data(&mut link_storage, followed_addr.as_slice(), &idx)?;
    } else {
        vec_storage.set_at(idx, &following)?;
    }
    
    // save follower relation

    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK], storage);
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::attach_or_create(&mut vec_storage)?;
 
    let idx: u32 = get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage.len());
    let follower = Follower {
        who: owner.clone(),
        active: true,
    };
    if idx == vec_storage.len() {
        vec_storage.push(&follower);
        set_bin_data(&mut link_storage, owner.as_slice(), &idx)?;
    } else {
        vec_storage.set_at(idx, &follower)?;
    }
    Ok(())

    //let store = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWING, storage);
    //let mut following: Vec<HumanAddr> = get_bin_data(&store, owner.as_slice()).unwrap_or_else(|_| vec![]);
    //let followed: HumanAddr = api.human_address(&get_account_for_handle(storage, &handle)?)?;
    //following.push(followed);
    //let mut store = PrefixedStorage::new(PREFIX_FOLLOWING, storage);
    //set_bin_data(&mut store, owner.as_slice(), &following)
}

pub fn remove_following<A: Api, S: Storage>(
    api: &A,
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let followed_addr = get_account_for_handle(storage, &handle)?;

    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::attach_or_create(&mut vec_storage)?;

    // update following relation, active = false

    // get current idx and set to false or ignore if not following
    let idx: u32 = get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage.len());
    if idx < vec_storage.len() {
        let mut following: Following = vec_storage.get_at(idx)?;
        following.active = false;
        vec_storage.set_at(idx, &following)?;
    } 

    // update follower relation, active = false

    let mut link_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK], storage);
    let mut vec_storage = PrefixedStorage::multilevel(&[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::attach_or_create(&mut vec_storage)?;
 
    let idx: u32 = get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage.len());
    if idx < vec_storage.len() {
        let mut follower: Follower = vec_storage.get_at(idx)?;
        follower.active = false;
        vec_storage.set_at(idx, &follower)?;
    } 
    Ok(())

    //let store = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWING, storage);
    //let mut following: Vec<HumanAddr> = get_bin_data(&store, owner.as_slice()).unwrap_or_else(|_| vec![]);
    //let followed: HumanAddr = api.human_address(&get_account_for_handle(storage, &handle)?)?;
    //following.retain(|x| x != &followed);
    //let mut store = PrefixedStorage::new(PREFIX_FOLLOWING, storage);
    //set_bin_data(&mut store, owner.as_slice(), &following)
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
        .filter(|following| following.unwrap().active)
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
        .filter(|follower| follower.unwrap().active)
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
pub fn is_blocked_by<S: Storage>(
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
//    b"unpacked" | {owner canonical addr} | {appendstore index} -> global fardel id
//
//  plus an additional mapping is stored to allow getting unpacked status by global_id:
//    b"id-to-unpacked" | {unpacker canonical addr} | {global fardel id} -> true/false
//       value == true means it is unpacked, value == false OR no record in storage means packed
//

pub fn store_unpack<S: Storage>(
    storage: &mut S,
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_UNPACKED, unpacker.as_slice()], storage);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    store.push(&fardel_id)?;
    map_global_id_to_unpacked_by_unpacker(storage, fardel_id, unpacker, true)
}

// stores a mapping from global fardel id to unpacked status in prefixed storage
fn map_global_id_to_unpacked_by_unpacker<S: Storage>(
    store: &mut S,
    global_id: u128,
    unpacker: &CanonicalAddr,
    value: bool,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()], store);
    set_bin_data(&mut store, &global_id.to_be_bytes(), &value)
}

// get the unpacked status of a fardel for a given unpacker canonical address
pub fn get_unpacked_status_by_fardel_id<S: Storage>(
    storage: &S, 
    unpacker: &CanonicalAddr,
    fardel_id: u128,
) -> bool {
    let mapping_store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_ID_UNPACKED_MAPPINGS, unpacker.as_slice()], storage);
    get_bin_data(&mapping_store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
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
// Fardel rating and comments
//   there are no limits on number of downvotes/upvotes/comments made but each costs gas
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
pub fn upvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_UPVOTES, store);
    let upvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(upvotes + 1))
}

pub fn get_upvotes<S: Storage>(
    store: &S,
    fardel_id: u128,
) -> u32 {
    let store = ReadonlyPrefixedStorage::new(PREFIX_UPVOTES, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32)
}

pub fn downvote_fardel<S: Storage>(
    store: &mut S,
    fardel_id: u128,
) -> StdResult <()> {
    let mut store = PrefixedStorage::new(PREFIX_DOWNVOTES, store);
    let downvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(downvotes + 1))
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

pub fn comment_on_fardel<S: Storage>(
    store: &mut S,
    commenter: &CanonicalAddr,
    fardel_id: u128,
    text: String,
) -> StdResult<()> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], store);
    let mut store = AppendStoreMut::attach_or_create(&mut store)?;
    let comment = Comment {
        commenter: commenter.clone(),
        text: text.as_bytes().to_vec(),
    };
    store.push(&comment)
}

pub fn get_comments<S: Storage>(
    store: &S,
    fardel_id: u128,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<Comment>> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], store);

    // Try to access the storage of comments for the fardel.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<Comment, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `page_size` comments starting from the latest comment, potentially skipping `page * page_size`
    // comments from the start.
    let comments_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    let comments: StdResult<Vec<Comment>> = comments_iter
        .map(|comment| comment)
        .collect();
    comments
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