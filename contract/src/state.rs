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

// Globals
pub static PREFIX_CONFIG: &[u8] = b"config";
pub const KEY_CONSTANTS: &[u8] = b"constants";
pub const KEY_FARDEL_COUNT: &[u8] = b"fardel-count";

// Fardel
pub const PREFIX_FARDELS: &[u8] = b"fardel";
pub const PREFIX_ID_FARDEL_MAPPINGS: &[u8] = b"id-to-fardel";
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

// Accounts
pub const PREFIX_ACCOUNTS: &[u8] = b"account";
pub const PREFIX_ACCOUNT_THUMBNAIL_IMGS: &[u8] = b"account-img";
pub const PREFIX_HANDLES: &[u8] = b"handle";
pub const PREFIX_VIEWING_KEY: &[u8] = b"viewingkey";

// Transactions

//
// CONFIG
//

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {
    //todo: change to vec
    pub admin: HumanAddr,
    // fardel settings
    // maximum cost of a fardel
    pub max_cost: u128,
    pub max_public_message_len: u16,
    pub max_tag_len: u8,
    pub max_number_of_tags: u8,
    pub max_fardel_img_size: u32,
    pub max_contents_data_len: u16,
    //pub max_contents_text_len: u16,
    //pub max_ipfs_cid_len: u16,
    //pub max_contents_passphrase_len: u16,
    // user settings
    pub max_handle_len: u16,
    pub max_description_len: u16,
    pub max_profile_img_size: u32,
    
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
    pub public_message: String,
    pub contents_text: String,
    pub ipfs_cid: String,
    pub passphrase: String,
    pub cost: Coin,
    pub timestamp: u64,
}

impl Fardel {
    pub fn into_stored(self) -> StdResult<StoredFardel> {
        let fardel = StoredFardel {
            global_id: self.global_id.u128(),
            public_message: self.public_message.as_bytes().to_vec(),
            contents_text: self.contents_text.as_bytes().to_vec(),
            ipfs_cid: self.ipfs_cid.as_bytes().to_vec(),
            passphrase: self.passphrase.as_bytes().to_vec(),
            cost: self.cost.amount.u128(),
            timestamp: self.timestamp,
        };
        Ok(fardel)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredFardel {
    pub global_id: u128,
    pub public_message: Vec<u8>,
    pub contents_text: Vec<u8>,
    pub ipfs_cid: Vec<u8>,
    pub passphrase: Vec<u8>,
    pub cost: u128,
    pub timestamp: u64,
}

impl StoredFardel {
    pub fn into_humanized(self) -> StdResult<Fardel> {
        let fardel = Fardel {
            global_id: Uint128(self.global_id),
            public_message: String::from_utf8(self.public_message).ok().unwrap_or_default(),
            contents_text: String::from_utf8(self.contents_text).ok().unwrap_or_default(),
            ipfs_cid: String::from_utf8(self.ipfs_cid).ok().unwrap_or_default(),
            passphrase: String::from_utf8(self.passphrase).ok().unwrap_or_default(),
            cost: Coin { amount: Uint128(self.cost), denom: DENOM.to_string() },
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

pub fn store_fardel<S: Storage>(
    store: &mut S,
    owner: &CanonicalAddr,
    public_message: Vec<u8>,
    contents_text: Vec<u8>,
    ipfs_cid: Vec<u8>,
    passphrase: Vec<u8>,
    cost: u128,
    timestamp: u64,
) -> StdResult<()> {
    let mut config = Config::from_storage(store);
    let global_id = config.fardel_count();
    config.set_fardel_count(global_id + 1)?;

    let fardel = StoredFardel {
        global_id,
        public_message: public_message.clone(),
        contents_text: contents_text.clone(),
        ipfs_cid: ipfs_cid.clone(),
        passphrase: passphrase.clone(),
        cost,
        timestamp,
    };

    let index = append_fardel(store, &owner, fardel.clone())?;
    map_global_id_to_fardel(store, global_id, &owner, index)?;
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
pub fn get_sealed_status<S: Storage>(
    store: &S, 
    fardel_id: u128,
) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_SEALED, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| false)
}

//
// Following
//
// are stored as a Vec<HumanAddr>, not ideal for long following lists but quick and dirty
// TODO: make smarter eg.
//   b"following" | {owner canonical addr} | {appendstore index} -> {followed canonical addr}
//
// addresses are saved rather than handles, in case followed user changes handle
//

pub fn store_following<A: Api, S: Storage>(
    api: &A,
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWING, storage);
    let mut following: Vec<HumanAddr> = get_bin_data(&store, owner.as_slice()).unwrap_or_else(|_| vec![]);
    let followed: HumanAddr = api.human_address(&get_account_for_handle(storage, &handle)?)?;
    following.push(followed);
    let mut store = PrefixedStorage::new(PREFIX_FOLLOWING, storage);
    set_bin_data(&mut store, owner.as_slice(), &following)
}

pub fn remove_following<A: Api, S: Storage>(
    api: &A,
    storage: &mut S,
    owner: &CanonicalAddr,
    handle: String,
) -> StdResult<()> {
    let store = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWING, storage);
    let mut following: Vec<HumanAddr> = get_bin_data(&store, owner.as_slice()).unwrap_or_else(|_| vec![]);
    let followed: HumanAddr = api.human_address(&get_account_for_handle(storage, &handle)?)?;
    following.retain(|x| x != &followed);
    let mut store = PrefixedStorage::new(PREFIX_FOLLOWING, storage);
    set_bin_data(&mut store, owner.as_slice(), &following)
}

// returns a vec of handles
pub fn get_following<A:Api, S: Storage>(
    api: &A,
    storage: &S,
    owner: &CanonicalAddr,
) -> StdResult<Vec<String>>{
    let store = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWING, storage);
    let following: Vec<HumanAddr> = get_bin_data(&store, owner.as_slice()).unwrap_or_else(|_| vec![]);
    let result = following.iter().map(|followed| {
        let followed: CanonicalAddr = api.canonical_address(&followed).unwrap();
        let followed_account: Account = get_account(storage, &followed).unwrap().into_humanized(api).unwrap();
        followed_account.handle
    }).collect();
    Ok(result)
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