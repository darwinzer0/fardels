use crate::fardel_state::get_fardel_owner;
use crate::state::{
    get_bin_data, set_bin_data, PREFIX_BLOCKED, PREFIX_COMMENTS, PREFIX_DELETED_COMMENTS,
    PREFIX_DOWNVOTES, PREFIX_FOLLOWERS, PREFIX_FOLLOWER_COUNT, PREFIX_FOLLOWING, PREFIX_LINK,
    PREFIX_RATED, PREFIX_UPVOTES, PREFIX_VEC,
};
use crate::user_state::{get_account, get_account_for_handle, Account};
use cosmwasm_std::{Api, CanonicalAddr, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::{Deserialize, Serialize};

//
// Following / Follower
//
//   b"following" | {owner canonical addr} | b"link" | {followed canonical addr} -> v_index
//   b"following" | {owner canonical addr} | b"vec" | {appendstore index} -> Following (active = true means following)
//   b"followers" | {owner canonical addr} | b"link" | {follower canonical addr} -> v_index
//   b"followers" | {owner canonical addr} | b"vec" | {appendstore index} -> Follower (active = true means follower)
//   b"follower-count" | {canonical addr} -> count of (active) followers of this account
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
    increment_follower_count(storage, &followed_addr);

    Ok(())
}

pub fn is_following<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> bool {
    let link_storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK],
        storage,
    );
    let result: StdResult<u32> = get_bin_data(&link_storage, &followed_addr.as_slice());
    match result {
        Ok(index) => {
            let vec_storage = ReadonlyPrefixedStorage::multilevel(
                &[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC],
                storage,
            );
            if let Some(vec_store_result) = AppendStore::<Following, _>::attach(&vec_storage) {
                let following = vec_store_result.unwrap().get_at(index);
                match following {
                    Ok(f) => return f.active,
                    Err(_) => return false,
                }
            } else {
                return false;
            };
        }
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
    let mut vec_storage =
        PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK],
        storage,
    );

    // get current idx or add to end
    let idx: u32 =
        get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage_len);
    // prepare following relationship
    let following = Following {
        who: followed_addr.clone(),
        active: true,
    };

    // now store the following relation

    let mut vec_storage =
        PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let mut vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;

    if idx == vec_storage_len {
        vec_storage.push(&following)?;
    } else {
        vec_storage.set_at(idx, &following)?;
    }

    let mut link_storage =
        PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK], storage);
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
    let mut vec_storage = PrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC],
        storage,
    );
    let vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK],
        storage,
    );

    let idx: u32 =
        get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage_len);
    let follower = Follower {
        who: owner.clone(),
        active: true,
    };

    let mut vec_storage = PrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC],
        storage,
    );
    let mut vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    if idx == vec_storage_len {
        vec_storage.push(&follower)?;
    } else {
        vec_storage.set_at(idx, &follower)?;
    }

    let mut link_storage = PrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK],
        storage,
    );
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
    decrement_follower_count(storage, &followed_addr);

    Ok(())
}

fn delete_following_relation<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    followed_addr: &CanonicalAddr,
) -> StdResult<()> {
    let mut vec_storage =
        PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
    let vec_storage = AppendStoreMut::<Following, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_LINK],
        storage,
    );
    // update following relation, active = false

    // get current idx and set to false or ignore if not following
    let idx: u32 =
        get_bin_data(&link_storage, followed_addr.as_slice()).unwrap_or_else(|_| vec_storage_len);

    let mut vec_storage =
        PrefixedStorage::multilevel(&[PREFIX_FOLLOWING, &owner.as_slice(), PREFIX_VEC], storage);
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
    let mut vec_storage = PrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC],
        storage,
    );
    let vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;
    let vec_storage_len = vec_storage.len();

    let link_storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_LINK],
        storage,
    );
    let idx: u32 =
        get_bin_data(&link_storage, owner.as_slice()).unwrap_or_else(|_| vec_storage_len);

    let mut vec_storage = PrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, &followed_addr.as_slice(), PREFIX_VEC],
        storage,
    );
    let mut vec_storage = AppendStoreMut::<Follower, _>::attach_or_create(&mut vec_storage)?;

    if idx < vec_storage_len {
        let mut follower: Follower = vec_storage.get_at(idx)?;
        follower.active = false;
        vec_storage.set_at(idx, &follower)?;
    }
    Ok(())
}

// returns a vec of handles
pub fn get_following<A: Api, S: Storage>(
    api: &A,
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<String>> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWING, owner.as_slice(), PREFIX_VEC],
        storage,
    );

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
            let followed_account: Account = get_account(storage, &followed)
                .unwrap()
                .into_humanized(api)
                .unwrap();
            followed_account.handle
        })
        .collect();
    Ok(result)
}

// returns number following including non-active -- for pagination
pub fn get_number_of_following<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> u32 {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWING, owner.as_slice(), PREFIX_VEC],
        storage,
    );

    // Try to access the storage of following for the account.
    // If it doesn't exist yet, return 0.
    if let Some(result) = AppendStore::<Following, _>::attach(&store) {
        return result.unwrap().len();
    } else {
        return 0;
    };
}

pub fn get_followers<A: Api, S: Storage>(
    api: &A,
    storage: &S,
    owner: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<Vec<String>> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, owner.as_slice(), PREFIX_VEC],
        storage,
    );

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
            let follower_account: Account = get_account(storage, &follower)
                .unwrap()
                .into_humanized(api)
                .unwrap();
            follower_account.handle
        })
        .collect();
    Ok(result)
}

pub fn set_follower_count<S: Storage>(
    storage: &mut S,
    account: &CanonicalAddr,
    value: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PREFIX_FOLLOWER_COUNT, storage);
    set_bin_data(&mut storage, account.as_slice(), &value)
}

// number of active followers
pub fn get_follower_count<S: ReadonlyStorage>(storage: &S, account: &CanonicalAddr) -> u32 {
    let storage = ReadonlyPrefixedStorage::new(PREFIX_FOLLOWER_COUNT, storage);
    get_bin_data(&storage, account.as_slice()).unwrap_or_else(|_| 0_u32)
}

pub fn increment_follower_count<S: Storage>(
    storage: &mut S,
    account: &CanonicalAddr,
) -> Option<u32> {
    let follower_count = get_follower_count(storage, &account);
    let result = set_follower_count(storage, &account, follower_count + 1);
    match result {
        Ok(_) => Some(follower_count + 1),
        Err(_) => None,
    }
}

pub fn decrement_follower_count<S: Storage>(
    storage: &mut S,
    account: &CanonicalAddr,
) -> Option<u32> {
    let follower_count = get_follower_count(storage, &account);
    if follower_count < 1 {
        return None;
    }
    let result = set_follower_count(storage, &account, follower_count - 1);
    match result {
        Ok(_) => Some(follower_count - 1),
        Err(_) => None,
    }
}

// returns number followers including non-active -- for pagination
pub fn get_number_of_followers<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> u32 {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_FOLLOWERS, owner.as_slice(), PREFIX_VEC],
        storage,
    );

    // Try to access the storage of followers for the account.
    // If it doesn't exist yet, return 0.
    if let Some(result) = AppendStore::<Follower, _>::attach(&store) {
        return result.unwrap().len();
    } else {
        return 0;
    };
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
    let mut store =
        PrefixedStorage::multilevel(&[PREFIX_BLOCKED, blocker_addr.as_slice()], storage);
    set_bin_data(&mut store, &blocked_addr.as_slice(), &blocked)
}

// returns true if blocked_addr is blocked by blocker_addr
pub fn is_blocked_by<S: ReadonlyStorage>(
    storage: &S,
    blocker_addr: &CanonicalAddr,
    blocked_addr: &CanonicalAddr,
) -> bool {
    let storage =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_BLOCKED, blocker_addr.as_slice()], storage);
    get_bin_data(&storage, &blocked_addr.as_slice()).unwrap_or_else(|_| false)
}

//
// Fardel rating and comments
//   each user can only upvote or downvote a fardel once, and they must have unpacked it
//   there are no limits on number of comments made but each costs gas
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

pub fn has_rated<S: ReadonlyStorage>(storage: &S, rater: &CanonicalAddr, fardel_id: u128) -> bool {
    let storage = ReadonlyPrefixedStorage::multilevel(&[PREFIX_RATED, rater.as_slice()], storage);
    let result: StdResult<bool> = get_bin_data(&storage, &fardel_id.to_be_bytes());
    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn remove_rated<S: Storage>(storage: &mut S, rater: &CanonicalAddr, fardel_id: u128) {
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

pub fn add_upvote_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_UPVOTES, store);
    let upvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(upvotes + 1))
}

pub fn subtract_upvote_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_UPVOTES, store);
    let upvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(upvotes - 1))
}

pub fn get_upvotes<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> u32 {
    let store = ReadonlyPrefixedStorage::new(PREFIX_UPVOTES, store);
    get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32)
}

pub fn add_downvote_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_DOWNVOTES, store);
    let downvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(downvotes + 1))
}

pub fn subtract_downvote_fardel<S: Storage>(store: &mut S, fardel_id: u128) -> StdResult<()> {
    let mut store = PrefixedStorage::new(PREFIX_DOWNVOTES, store);
    let downvotes: u32 = get_bin_data(&store, &fardel_id.to_be_bytes()).unwrap_or_else(|_| 0_u32);
    set_bin_data(&mut store, &fardel_id.to_be_bytes(), &(downvotes - 1))
}

pub fn get_downvotes<S: ReadonlyStorage>(store: &S, fardel_id: u128) -> u32 {
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
    let mut store =
        PrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);
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
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);

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
            Ok(IndexedComment {
                commenter: comment.commenter.clone(),
                text: comment.text,
                idx: idx as u32,
            })
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
pub fn get_number_of_comments<S: ReadonlyStorage>(storage: &S, fardel_id: u128) -> u32 {
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);

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
    let store =
        ReadonlyPrefixedStorage::multilevel(&[PREFIX_COMMENTS, &fardel_id.to_be_bytes()], storage);
    // Try to access the storage of comments for the fardel.
    // If it doesn't exist yet, return 0.
    let store = if let Some(result) = AppendStore::<Comment, _>::attach(&store) {
        result?
    } else {
        return Err(StdError::generic_err(
            "no comment at that index for that fardel.",
        ));
    };
    store.get_at(comment_id)
}

pub fn delete_comment<S: Storage>(
    storage: &mut S,
    fardel_id: u128,
    comment_id: u32,
) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(
        &[PREFIX_DELETED_COMMENTS, &fardel_id.to_be_bytes()],
        storage,
    );
    set_bin_data(&mut storage, &comment_id.to_be_bytes(), &true)
}

fn comment_is_deleted<S: ReadonlyStorage>(storage: &S, fardel_id: u128, comment_id: u32) -> bool {
    let storage = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_DELETED_COMMENTS, &fardel_id.to_be_bytes()],
        storage,
    );
    get_bin_data(&storage, &comment_id.to_be_bytes()).unwrap_or_else(|_| false)
}
