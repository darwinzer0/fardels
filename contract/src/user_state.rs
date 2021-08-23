use crate::state::{
    get_bin_data, set_bin_data, PREFIX_ACCOUNTS, PREFIX_ACCOUNT_THUMBNAIL_IMGS, PREFIX_BANNED,
    PREFIX_DEACTIVATED, PREFIX_HANDLES, PREFIX_REGISTERED_ADDRESSES, PREFIX_VIEWING_KEY,
};
use crate::viewing_key::ViewingKey;
use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, ReadonlyStorage, StdResult, Storage, StdError,};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

//
// User accounts
//   b"account" | {owner canonical addr} -> StoredAccount
//   b"account-img" | {owner canonical addr} -> img
//

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Account {
    pub owner: HumanAddr,
    pub handle: String,
    pub description: String,
    pub view_settings: String,
    pub private_settings: String,
}

impl Account {
    pub fn into_stored<A: Api>(self, api: &A) -> StdResult<StoredAccount> {
        let account = StoredAccount {
            owner: api.canonical_address(&self.owner)?,
            handle: self.handle.as_bytes().to_vec(),
            description: self.description.as_bytes().to_vec(),
            view_settings: self.view_settings.as_bytes().to_vec(),
            private_settings: self.private_settings.as_bytes().to_vec(),
        };
        Ok(account)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredAccount {
    pub owner: CanonicalAddr,
    pub handle: Vec<u8>,
    pub description: Vec<u8>,
    pub view_settings: Vec<u8>,
    pub private_settings: Vec<u8>,
}

impl StoredAccount {
    pub fn into_humanized<A: Api>(self, api: &A) -> StdResult<Account> {
        let account = Account {
            owner: api.human_address(&self.owner)?,
            handle: String::from_utf8(self.handle).ok().unwrap_or_default(),
            description: String::from_utf8(self.description).ok().unwrap_or_default(),
            view_settings: String::from_utf8(self.view_settings)
                .ok()
                .unwrap_or_default(),
            private_settings: String::from_utf8(self.private_settings)
                .ok()
                .unwrap_or_default(),
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
pub fn delete_handle_map<S: Storage>(store: &mut S, handle: String) {
    let mut store = PrefixedStorage::new(PREFIX_HANDLES, store);
    store.remove(handle.as_bytes())
}

pub fn get_account_for_handle<S: ReadonlyStorage>(
    store: &S,
    handle: &String,
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
pub fn get_account_img<S: ReadonlyStorage>(store: &S, owner: &CanonicalAddr) -> StdResult<Vec<u8>> {
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
pub fn is_deactivated<S: ReadonlyStorage>(store: &S, account: &CanonicalAddr) -> bool {
    let store = ReadonlyPrefixedStorage::new(PREFIX_DEACTIVATED, store);
    get_bin_data(&store, &account.as_slice()).unwrap_or_else(|_| false)
}

//
// Banned accounts
//   b"banned" | {owner canonical addr} -> bool
//

pub fn store_account_ban<S: Storage>(
    storage: &mut S,
    account: &CanonicalAddr,
    banned: bool,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_BANNED, storage);
    set_bin_data(&mut storage, &account.as_slice(), &banned)
}

// returns true is account is banned
pub fn is_banned<S: ReadonlyStorage>(storage: &S, account: &CanonicalAddr) -> bool {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_BANNED, storage);
    get_bin_data(&storage, &account.as_slice()).unwrap_or_else(|_| false)
}

//
// Address append store (for admin to iterate over users for indexing)
//   b"addresses" |  {appendstore idx} -> owner canonical address
//

pub fn address_list_add<S: Storage>(storage: &mut S, address: &CanonicalAddr) -> StdResult<u32> {
    let mut storage = PrefixedStorage::new(PREFIX_REGISTERED_ADDRESSES, storage);
    let mut storage = AppendStoreMut::<CanonicalAddr, _>::attach_or_create(&mut storage)?;
    storage.push(address)?;
    Ok(storage.len())
}

pub fn get_registered_address<S: ReadonlyStorage>(
    storage: &S,
    idx: u32,
) -> StdResult<CanonicalAddr> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_REGISTERED_ADDRESSES, storage);

    // Try to access the storage of addresses using contract
    // If it doesn't exist yet, return an empty list.
    let storage = if let Some(result) = AppendStore::<CanonicalAddr, _>::attach(&storage) {
        result?
    } else {
        return Err(StdError::generic_err("Error accessing storage of addresses"));
    };

    storage.get_at(idx)
}

pub fn get_registered_addresses<S: ReadonlyStorage>(
    storage: &S,
    start: u32,
    count: u32,
) -> StdResult<Vec<CanonicalAddr>> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_REGISTERED_ADDRESSES, storage);

    // Try to access the storage of addresses using contract
    // If it doesn't exist yet, return an empty list.
    let storage = if let Some(result) = AppendStore::<CanonicalAddr, _>::attach(&storage) {
        result?
    } else {
        return Ok(vec![]);
    };

    // Take `count` addresses starting from the `start` address
    // addresses from the start.
    let address_iter = storage.iter().skip(start as _).take(count as _);
    // Convert to HumanAddr
    let addresses: StdResult<Vec<CanonicalAddr>> = address_iter.map(|address| address).collect();
    addresses
}

pub fn get_total_number_registered_accounts<S: ReadonlyStorage>(storage: &S) -> StdResult<u32> {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_REGISTERED_ADDRESSES, storage);

    // Try to access the storage of addresses using contract
    // If it doesn't exist yet, return 0.
    let storage = if let Some(result) = AppendStore::<CanonicalAddr, _>::attach(&storage) {
        result?
    } else {
        return Ok(0_u32);
    };
    Ok(storage.len())
}
