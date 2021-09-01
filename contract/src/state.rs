use crate::msg::Fee;
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

// Globals
pub const PREFIX_CONFIG: &[u8] = b"config";
pub const KEY_CONSTANTS: &[u8] = b"constants";
// to change admin SetAdmin must be called 3 times with the same address
//   to help prevent accidental change to wrong address
pub const KEY_NEW_ADMIN: &[u8] = b"new-admin";
pub const KEY_NEW_ADMIN_COUNT: &[u8] = b"new-admin-count";
pub const KEY_FARDEL_COUNT: &[u8] = b"fardel-count";
pub const KEY_FROZEN: &[u8] = b"frozen";

// Fardel
pub const PREFIX_FARDELS: &[u8] = b"fardel";
pub const PREFIX_FARDEL_THUMBNAIL_IMGS: &[u8] = b"fardel-img";
pub const PREFIX_ID_FARDEL_MAPPINGS: &[u8] = b"id-to-fardel";
pub const PREFIX_HASH_ID_MAPPINGS: &[u8] = b"hash-to-id";
pub const PREFIX_SEALED: &[u8] = b"sealed";
pub const PREFIX_HIDDEN: &[u8] = b"hidden";
pub const PREFIX_REMOVED: &[u8] = b"removed";
pub const PREFIX_FARDEL_NUM_UNPACKS: &[u8] = b"fardel-unpack-count";

// Fardel unpacking
pub const PREFIX_UNPACKED: &[u8] = b"unpacked";
pub const PREFIX_ID_UNPACKED_MAPPINGS: &[u8] = b"id-to-unpacked";
// pending unpacks indexed by owner of the fardel
pub const PREFIX_PENDING_APPROVAL: &[u8] = b"pending";
pub const PREFIX_PENDING_START: &[u8] = b"pending-start";
// pending unpacks indexed by the unpacker of the fardel
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
pub const PREFIX_FOLLOWER_COUNT: &[u8] = b"follower-count";

// Blocked
pub const PREFIX_BLOCKED: &[u8] = b"blocked";

// Accounts
pub const PREFIX_ACCOUNTS: &[u8] = b"account";
pub const PREFIX_ACCOUNT_THUMBNAIL_IMGS: &[u8] = b"account-img";
pub const PREFIX_HANDLES: &[u8] = b"handle";
pub const PREFIX_VIEWING_KEY: &[u8] = b"viewingkey";
pub const PREFIX_DEACTIVATED: &[u8] = b"deactived";

// Registered addresses
pub const PREFIX_REGISTERED_ADDRESSES: &[u8] = b"addresses";

// Banned accounts
pub const PREFIX_BANNED: &[u8] = b"banned";

// Completed transactions
pub const PREFIX_SALE_TX: &[u8] = b"sale-tx";
pub const PREFIX_PURCHASE_TX: &[u8] = b"purchase-tx";

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
    pub max_view_settings_len: u16,
    pub max_private_settings_len: u16,

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
// New admin
//
pub fn set_new_admin<S: Storage>(storage: &mut S, new_admin: &CanonicalAddr) -> StdResult<()> {
    set_bin_data(storage, KEY_NEW_ADMIN, &new_admin)
}

pub fn get_new_admin<S: Storage>(storage: &S) -> StdResult<CanonicalAddr> {
    get_bin_data(storage, KEY_NEW_ADMIN)
}

pub fn set_new_admin_count<S: Storage>(storage: &mut S, new_admin_count: u8) -> StdResult<()> {
    set_bin_data(storage, KEY_NEW_ADMIN_COUNT, &new_admin_count)
}

pub fn get_new_admin_count<S: Storage>(storage: &S) -> u8 {
    get_bin_data(storage, KEY_NEW_ADMIN_COUNT).unwrap_or_else(|_| 0_u8)
}

//
// Frozen status
//
pub fn set_frozen<S: Storage>(storage: &mut S, value: bool) -> StdResult<()> {
    set_bin_data(storage, KEY_FROZEN, &value)
}

pub fn is_frozen<S: ReadonlyStorage>(storage: &S) -> bool {
    get_bin_data(storage, KEY_FROZEN).unwrap_or_else(|_| false)
}

//
// Bin data storage setters and getters
//

pub fn set_bin_data<T: Serialize, S: Storage>(
    storage: &mut S,
    key: &[u8],
    data: &T,
) -> StdResult<()> {
    let bin_data =
        bincode2::serialize(&data).map_err(|e| StdError::serialize_err(type_name::<T>(), e))?;
    storage.set(key, &bin_data);
    Ok(())
}

pub fn get_bin_data<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<T> {
    let bin_data = storage.get(key);
    match bin_data {
        None => Err(StdError::not_found("Key not found in storage")),
        Some(bin_data) => Ok(bincode2::deserialize::<T>(&bin_data)
            .map_err(|e| StdError::serialize_err(type_name::<T>(), e))?),
    }
}
