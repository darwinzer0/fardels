use crate::fardel_state::get_fardel_by_global_id;
use crate::state::{PREFIX_PURCHASE_TX, PREFIX_SALE_TX};
use crate::user_state::get_account;
use cosmwasm_std::{debug_print, CanonicalAddr, ReadonlyStorage, StdResult, Storage, Uint128};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

//
// Sale transaction record
//
//  b"sale-tx" | {owner canonical address} | appendstore | Tx
//
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SaleTx {
    pub fardel_id: Uint128,
    pub handle: String,
    pub amount: Uint128,
    pub fee: Uint128,
    pub timestamp: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredSaleTx {
    pub fardel_id: u128,
    pub unpacker: CanonicalAddr,
    pub amount: u128,
    pub fee: u128,
    pub timestamp: u64,
}

impl StoredSaleTx {
    pub fn into_humanized<S: ReadonlyStorage>(self, storage: &S) -> StdResult<SaleTx> {
        let fardel = get_fardel_by_global_id(storage, self.fardel_id)?.unwrap();
        let unpacker = get_account(storage, &self.unpacker)?;
        let tx = SaleTx {
            fardel_id: fardel.hash_id,
            handle: String::from_utf8(unpacker.handle).ok().unwrap_or_default(),
            amount: Uint128(self.amount),
            fee: Uint128(self.fee),
            timestamp: self.timestamp as i32,
        };
        Ok(tx)
    }
}

// returns the index of the appended fardel
pub fn append_sale_tx<S: Storage>(
    storage: &mut S,
    owner: CanonicalAddr,
    unpacker: CanonicalAddr,
    fardel_id: u128,
    amount: u128,
    fee: u128,
    timestamp: u64,
) -> StdResult<u32> {
    let mut store = PrefixedStorage::multilevel(&[PREFIX_SALE_TX, owner.as_slice()], storage);
    let mut store = AppendStoreMut::<StoredSaleTx, _>::attach_or_create(&mut store)?;
    let tx = StoredSaleTx {
        fardel_id,
        unpacker,
        amount,
        fee,
        timestamp,
    };
    store.push(&tx)?;
    Ok(store.len() - 1)
}

pub fn get_sale_txs<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<SaleTx>> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_SALE_TX, owner.as_slice()], storage);

    // Try to access the storage of txs for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<StoredSaleTx, _>::attach(&store) {
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
    let txs: StdResult<Vec<SaleTx>> = tx_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(storage)).and_then(|x| x))
        .collect();
    txs
}

pub fn get_number_of_sales<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
) -> StdResult<u32> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_SALE_TX, owner.as_slice()], storage);

    // Try to access the storage of sale txs for the account.
    // If it doesn't exist yet, return an empty list.
    if let Some(result) = AppendStore::<StoredSaleTx, _>::attach(&store) {
        return Ok(result?.len());
    } else {
        return Ok(0_u32);
    };
}

//
// Purchase transaction record
//
//  b"purchase-tx" | {unpacker canonical address} | appendstore | Tx
//
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PurchaseTx {
    pub fardel_id: Uint128,
    pub handle: String,
    pub amount: Uint128,
    pub fee: Uint128,
    pub timestamp: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredPurchaseTx {
    pub fardel_id: u128,
    pub owner: CanonicalAddr,
    pub amount: u128,
    pub fee: u128,
    pub timestamp: u64,
}

impl StoredPurchaseTx {
    pub fn into_humanized<S: ReadonlyStorage>(self, storage: &S) -> StdResult<PurchaseTx> {
        let fardel = get_fardel_by_global_id(storage, self.fardel_id)?.unwrap();
        let owner = get_account(storage, &self.owner)?;
        let tx = PurchaseTx {
            fardel_id: fardel.hash_id,
            handle: String::from_utf8(owner.handle).ok().unwrap_or_default(),
            amount: Uint128(self.amount),
            fee: Uint128(self.fee),
            timestamp: self.timestamp as i32,
        };
        Ok(tx)
    }
}

// returns the index of the appended fardel
pub fn append_purchase_tx<S: Storage>(
    storage: &mut S,
    owner: CanonicalAddr,
    unpacker: CanonicalAddr,
    fardel_id: u128,
    amount: u128,
    fee: u128,
    timestamp: u64,
) -> StdResult<u32> {
    let mut store =
        PrefixedStorage::multilevel(&[PREFIX_PURCHASE_TX, unpacker.as_slice()], storage);
    let mut store = AppendStoreMut::<StoredPurchaseTx, _>::attach_or_create(&mut store)?;
    let tx = StoredPurchaseTx {
        fardel_id,
        owner,
        amount,
        fee,
        timestamp,
    };
    store.push(&tx)?;
    Ok(store.len() - 1)
}

pub fn get_purchase_txs<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<PurchaseTx>> {
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_PURCHASE_TX, unpacker.as_slice()], storage);

    // Try to access the storage of txs for the account.
    // If it doesn't exist yet, return an empty list.
    let store = if let Some(result) = AppendStore::<StoredPurchaseTx, _>::attach(&store) {
        result?
    } else {
        return Ok(vec![]);
    };

    debug_print!("{}", page);
    debug_print!("{}", page_size); 
    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let tx_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);
    // The `and_then` here flattens the `StdResult<StdResult<Tx>>` to an `StdResult<Tx>`
    let txs: StdResult<Vec<PurchaseTx>> = tx_iter
        .map(|tx| tx.map(|tx| tx.into_humanized(storage)).and_then(|x| {
            debug_print!("{:?}", x); 
            x
        }))
        .collect();
    txs
}

pub fn get_number_of_purchases<S: ReadonlyStorage>(
    storage: &S,
    unpacker: &CanonicalAddr,
) -> StdResult<u32> {
    let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_PURCHASE_TX, unpacker.as_slice()], storage);

    // Try to access the storage of purchase txs for the account.
    // If it doesn't exist yet, return an empty list.
    if let Some(result) = AppendStore::<StoredPurchaseTx, _>::attach(&store) {
        return Ok(result?.len());
    } else {
        return Ok(0_u32);
    };
}