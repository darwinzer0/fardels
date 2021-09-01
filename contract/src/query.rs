use crate::fardel_state::{
    get_fardel_by_global_id, get_fardel_by_hash, get_fardel_img, get_fardel_owner, get_fardels,
    get_global_id_by_hash, get_number_of_fardels, get_sealed_status, get_total_fardel_count,
    is_fardel_hidden, Fardel, get_fardel_unpack_count, is_fardel_removed,
};
use crate::msg::{
    CommentResponse, FardelBatchResponse, FardelResponse, PendingApprovalResponse, QueryAnswer,
    RegisteredAccountsResponse, ResponseStatus, ResponseStatus::Failure, ResponseStatus::Success,
};
use crate::social_state::{
    get_comments, get_downvotes, get_follower_count, get_followers, get_following,
    get_number_of_comments, get_number_of_followers, get_number_of_following, get_rating,
    get_upvotes, is_following,
};
use crate::state::ReadonlyConfig;
use crate::tx_state::{get_purchase_txs, get_sale_txs, get_number_of_sales, get_number_of_purchases, PurchaseTx, SaleTx};
use crate::unpack_state::{
    get_number_of_unpacked_by_unpacker, get_pending_approvals_from_start,
    get_pending_unpacked_status_by_fardel_id, get_unpacked_by_unpacker,
    get_unpacked_status_by_fardel_id, UnpackedFardel,
};
use crate::user_state::{
    get_account, get_account_for_handle, get_account_img, get_registered_addresses, is_banned,
    is_deactivated, Account, get_total_number_registered_accounts, get_registered_address,
};
use cosmwasm_std::{
    to_binary, Api, Extern, HumanAddr, Querier, QueryResult, StdError, Storage, Uint128,
};

pub fn query_get_profile<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    handle: String,
) -> QueryResult {
    let status: ResponseStatus = Success;
    let address = get_account_for_handle(&deps.storage, &handle)?;
    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }
    let account = get_account(&deps.storage, &address)?.into_humanized(&deps.api)?;
    let img = get_account_img(&deps.storage, &address).unwrap_or_else(|_| vec![]);
    let img_str = String::from_utf8(img).unwrap();
    let follower_count = get_follower_count(&deps.storage, &address) as i32;
    let answer = QueryAnswer::GetProfile {
        status,
        handle: Some(handle),
        description: Some(account.description),
        view_settings: Some(account.view_settings),
        img: Some(img_str),
        follower_count,
    };
    to_binary(&answer)
}

pub fn query_get_profile_by_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    idx: i32,
) -> QueryResult {
    let mut status: ResponseStatus = Success;
    let mut handle: Option<String> = None;
    let mut description: Option<String> = None;
    let mut view_settings: Option<String> = None;
    let mut img: Option<String> = None;
    let mut follower_count: i32 = 0;
    if idx < 0 {
        status = Failure;
    } else {
        let number_of_accounts = get_total_number_registered_accounts(&deps.storage)?;
        let mut which: u32 = idx as u32;
        if number_of_accounts <= which {
            which = which % number_of_accounts;
        }
        let user_address = get_registered_address(&deps.storage, which)?;
        if !is_banned(&deps.storage, &user_address) && !is_deactivated(&deps.storage, &user_address) {
            let account = get_account(&deps.storage, &user_address)?.into_humanized(&deps.api)?;
            let img_vec = get_account_img(&deps.storage, &user_address).unwrap_or_else(|_| vec![]);
            let img_str = String::from_utf8(img_vec).unwrap();
            follower_count = get_follower_count(&deps.storage, &user_address) as i32;
            handle = Some(account.handle);
            description = Some(account.description);
            view_settings = Some(account.view_settings);
            img = Some(img_str);            
        } else {
            status = Failure;
        }
    }

    let answer = QueryAnswer::GetProfileByIndex {
        status,
        handle,
        description,
        img,
        view_settings,
        follower_count,
    };
    to_binary(&answer)
}

pub fn query_is_handle_available<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    handle: String,
) -> QueryResult {
    let response = match get_account_for_handle(&deps.storage, &handle) {
        Ok(_) => true,
        Err(_) => false,
    };
    let answer = QueryAnswer::IsHandleAvailable { response };
    to_binary(&answer)
}

pub fn query_get_fardel_by_id<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &Option<HumanAddr>,
    fardel_id: Uint128,
) -> QueryResult {
    let fardel_id = fardel_id.u128();
    let fardel = get_fardel_by_hash(&deps.storage, fardel_id)?;

    let fardel = match fardel {
        Some(fardel) => fardel,
        None => {
            return Err(StdError::generic_err("Fardel not found."));
        }
    };

    let global_id = fardel.global_id.u128();

    let removed = is_fardel_removed(&deps.storage, global_id);
    if removed {
        return Err(StdError::generic_err("Fardel not found."));
    }

    let upvotes: i32 = get_upvotes(&deps.storage, global_id) as i32;
    let downvotes: i32 = get_downvotes(&deps.storage, global_id) as i32;
    let number_of_comments = get_number_of_comments(&deps.storage, global_id) as i32;

    // unpacked parts
    let mut contents_data: Option<String> = None;
    let mut unpacked = false;
    let mut pending_unpack = false;

    let owner = get_fardel_owner(&deps.storage, global_id)?;
    let banned = is_banned(&deps.storage, &owner);
    let deactivated = is_deactivated(&deps.storage, &owner);
    let hidden = is_fardel_hidden(&deps.storage, global_id);
    let mut rating: Option<bool> = None;

    if address.is_some() {
        let unpacker_address = address.clone().unwrap();
        let unpacker = &deps.api.canonical_address(&unpacker_address)?;
        let unpacked_status = get_unpacked_status_by_fardel_id(&deps.storage, unpacker, global_id);
        
        if unpacked_status.unpacked {
            contents_data = Some(fardel.contents_data);
            unpacked = true;
        } else if get_pending_unpacked_status_by_fardel_id(&deps.storage, unpacker, global_id).value {
            pending_unpack = true;
        } else if banned || deactivated || hidden {
            return Err(StdError::generic_err("Fardel not found."));
        }

        match get_rating(&deps.storage, &unpacker, global_id) {
            Ok(r) => rating = Some(r),
            Err(_) => {}
        };
    } else if banned || deactivated || hidden {
        return Err(StdError::generic_err("Fardel not found."));
    }

    let timestamp: i32 = fardel.timestamp as i32;
    let mut seal_time: Option<i32> = None;
    if fardel.seal_time > 0 {
        seal_time = Some(fardel.seal_time as i32);
    }
    let sealed = get_sealed_status(&deps.storage, global_id);
    let img = get_fardel_img(&deps.storage, global_id);
    let mut remaining: Option<i32> = None;
    if fardel.countable > 0 {
        let unpack_count = get_fardel_unpack_count(&deps.storage, global_id).unwrap_or_else(|_| 0_u64) as u16;
        remaining = Some((fardel.countable - unpack_count) as i32);
    }

    let fardel_owner = get_fardel_owner(&deps.storage, global_id).unwrap();
    let carrier = get_account(&deps.storage, &fardel_owner).unwrap().into_humanized(&deps.api).unwrap().handle;

    let fardel_response = FardelResponse {
        id: fardel.hash_id,
        carrier,
        public_message: fardel.public_message,
        tags: fardel.tags,
        cost: fardel.cost.amount,
        unpacked,
        pending_unpack,
        upvotes,
        downvotes,
        number_of_comments,
        remaining,
        seal_time,
        sealed,
        timestamp,
        contents_data,
        rating,
        img,
    };
    let answer = QueryAnswer::GetFardelById {
        fardel: fardel_response,
    };
    to_binary(&answer)
}

pub fn query_get_fardels<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &Option<HumanAddr>,
    handle: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let account = get_account_for_handle(&deps.storage, &handle)?;
    let banned = is_banned(&deps.storage, &account);
    let deactivated = is_deactivated(&deps.storage, &account);

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;
    let fardels: Vec<Fardel> =
        get_fardels(&deps.storage, &account, page, page_size).unwrap_or_else(|_| vec![]);

    let mut fardels_response: Vec<FardelResponse> = vec![];
    if fardels.len() > 0 {
        fardels_response = fardels
            .iter()
            .filter(|fardel| {
                let global_id = fardel.global_id.u128();
                let mut unpacked = false;
                let mut pending_unpack = false;

                let removed = is_fardel_removed(&deps.storage, global_id);

                let hidden = is_fardel_hidden(&deps.storage, global_id);
                if address.is_some() {
                    let unpacker_address = address.clone().unwrap();
                    let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
                    if get_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id)
                        .unpacked
                    {
                        unpacked = true;
                    } else if get_pending_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id).value {
                        pending_unpack = true;
                    }
                }
                !removed && (!(banned || deactivated || hidden) || unpacked || pending_unpack)
            })
            .map(|fardel| {
                let global_id = fardel.global_id.u128();
                let upvotes: i32 = get_upvotes(&deps.storage, global_id) as i32;
                let downvotes: i32 = get_downvotes(&deps.storage, global_id) as i32;

                let number_of_comments = get_number_of_comments(&deps.storage, global_id) as i32;

                // unpacked parts
                let mut contents_data: Option<String> = None;
                let mut unpacked = false;
                let mut pending_unpack = false;
                let mut rating: Option<bool> = None;

                if address.is_some() {
                    let unpacker_address = address.clone().unwrap();
                    let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
                    if get_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id)
                        .unpacked
                    {
                        contents_data = Some(fardel.contents_data.clone());
                        unpacked = true;
                    } else if get_pending_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id).value {
                        pending_unpack = true;
                    }

                    match get_rating(&deps.storage, &unpacker, global_id) {
                        Ok(r) => rating = Some(r),
                        Err(_) => {}
                    };
                }

                let timestamp: i32 = fardel.timestamp as i32;
                let mut seal_time: Option<i32> = None;
                if fardel.seal_time > 0 {
                    seal_time = Some(fardel.seal_time as i32);
                }
                let sealed = get_sealed_status(&deps.storage, global_id);
                let img = get_fardel_img(&deps.storage, global_id);

                let mut remaining: Option<i32> = None;
                if fardel.countable > 0 {
                    let unpack_count = get_fardel_unpack_count(&deps.storage, global_id).unwrap_or_else(|_| 0_u64) as u16;
                    remaining = Some((fardel.countable - unpack_count) as i32);
                }

                let fardel_owner = get_fardel_owner(&deps.storage, global_id).unwrap();
                let carrier = get_account(&deps.storage, &fardel_owner).unwrap().into_humanized(&deps.api).unwrap().handle;

                FardelResponse {
                    id: fardel.hash_id,
                    carrier,
                    public_message: fardel.public_message.clone(),
                    tags: fardel.tags.clone(),
                    cost: fardel.cost.amount,
                    unpacked,
                    pending_unpack,
                    upvotes,
                    downvotes,
                    number_of_comments,
                    remaining,
                    seal_time,
                    sealed,
                    timestamp,
                    contents_data,
                    rating,
                    img,
                }
            })
            .collect();
    }
    let total_count = get_number_of_fardels(&deps.storage, &account) as i32;
    let answer = QueryAnswer::GetFardels {
        fardels: fardels_response,
        total_count,
    };
    to_binary(&answer)
}

pub fn query_get_comments<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &Option<HumanAddr>,
    fardel_id: Uint128,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let fardel_id = fardel_id.u128();
    let fardel = get_fardel_by_hash(&deps.storage, fardel_id)?;
    let fardel = match fardel {
        Some(fardel) => fardel,
        None => {
            return Err(StdError::generic_err("Fardel not found."));
        }
    };
    let global_id = fardel.global_id.u128();

    let removed = is_fardel_removed(&deps.storage, global_id);
    if removed {
        return Err(StdError::generic_err("Fardel not found."));
    }

    // make sure it is not hidden
    let mut unpacked = false;
    let owner = get_fardel_owner(&deps.storage, global_id)?;
    let banned = is_banned(&deps.storage, &owner);
    let deactivated = is_deactivated(&deps.storage, &owner);
    let hidden = is_fardel_hidden(&deps.storage, global_id);
    if address.is_some() {
        let unpacker_address = address.clone().unwrap();
        let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
        unpacked = get_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id).unpacked;
    }
    if (banned || deactivated || hidden) && !unpacked {
        return Err(StdError::generic_err("Fardel not found."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    // get last page_size comments
    let comments: Vec<CommentResponse> = get_comments(&deps.storage, global_id, page, page_size)?
        .iter()
        .map(|c| {
            let commenter_account = get_account(&deps.storage, &c.commenter).unwrap();
            let mut response_fardel_id: Option<Uint128> = None;
            let mut response_comment_id: Option<i32> = None;
            if address.is_some() {
                let commenter_address = address.clone().unwrap();
                let sender = deps.api.canonical_address(&commenter_address).unwrap();
                if sender == c.commenter {
                    response_fardel_id = Some(fardel.hash_id);
                    response_comment_id = Some(c.idx as i32);
                }
            }
            CommentResponse {
                text: String::from_utf8(c.text.clone()).ok().unwrap(),
                handle: String::from_utf8(commenter_account.handle).ok().unwrap(),
                fardel_id: response_fardel_id,
                comment_id: response_comment_id,
            }
        })
        .collect();

    let answer = QueryAnswer::GetComments { comments };
    to_binary(&answer)
}

// Authenticated queries

pub fn query_get_sale_transactions<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let txs: Vec<SaleTx> =
        get_sale_txs(&deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let total_count = get_number_of_sales(&deps.storage, &address)? as i32;
    
    let response = QueryAnswer::GetSaleTransactions { txs, total_count };
    to_binary(&response)
}

pub fn query_get_purchase_transactions<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let txs: Vec<PurchaseTx> =
        get_purchase_txs(&deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let total_count = get_number_of_purchases(&deps.storage, &address)? as i32;

    let response = QueryAnswer::GetPurchaseTransactions { txs, total_count };
    to_binary(&response)
}

pub fn query_get_handle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
) -> QueryResult {
    let mut status: ResponseStatus = Success;
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let mut handle: Option<String> = None;
    let mut private_settings: Option<String> = None;

    let account: Option<Account> = match get_account(&deps.storage, &address) {
        Ok(acc) => Some(acc.into_humanized(&deps.api)?),
        Err(_) => {
            status = Failure;
            None
        }
    };

    if status == Success {
        let account = account.unwrap();
        handle = Some(account.handle);
        private_settings = Some(account.private_settings);
    }

    let answer = QueryAnswer::GetHandle {
        status,
        handle,
        private_settings,
    };
    to_binary(&answer)
}

pub fn query_get_following<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let following: Vec<String> = get_following(&deps.api, &deps.storage, &address, page, page_size)
        .unwrap_or_else(|_| vec![]);
    let total_count = get_number_of_following(&deps.storage, &address) as i32;
    let response = QueryAnswer::GetFollowing {
        following,
        total_count,
    };
    to_binary(&response)
}

pub fn query_is_following<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    handle: String,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let followed_addr = get_account_for_handle(&deps.storage, &handle)?;
    let following = is_following(&deps.storage, &address, &followed_addr);
    let response = QueryAnswer::IsFollowing {
        response: following,
    };
    to_binary(&response)
}

pub fn query_get_followers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let followers: Vec<String> = get_followers(&deps.api, &deps.storage, &address, page, page_size)
        .unwrap_or_else(|_| vec![]);
    let total_count = get_number_of_followers(&deps.storage, &address) as i32;
    let response = QueryAnswer::GetFollowers {
        followers,
        total_count,
    };
    to_binary(&response)
}

pub fn query_is_pending_unpack<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    fardel_id: Uint128,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let global_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;

    let pending_unpack =
        get_pending_unpacked_status_by_fardel_id(&deps.storage, &address, global_id);
    let response = QueryAnswer::IsPendingUnpack {
        response: pending_unpack.value,
    };
    to_binary(&response)
}

pub fn query_get_unpacked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> QueryResult {
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let unpacked_fardels: Vec<UnpackedFardel> =
        get_unpacked_by_unpacker(&deps.storage, &address, page, page_size)
            .unwrap_or_else(|_| vec![]);
    let mut fardels: Vec<FardelResponse> = vec![];
    for unpacked in unpacked_fardels {
        let unpack_id = unpacked.fardel_id;
        let fardel = get_fardel_by_global_id(&deps.storage, unpack_id)?;
        let fardel_owner = get_fardel_owner(&deps.storage, unpack_id)?;
        if fardel.is_some() && fardel_owner != address {
            let fardel = fardel.unwrap();
            let upvotes: i32 = get_upvotes(&deps.storage, unpack_id) as i32;
            let downvotes: i32 = get_downvotes(&deps.storage, unpack_id) as i32;

            let number_of_comments = get_number_of_comments(&deps.storage, unpack_id) as i32;

            let timestamp: i32 = fardel.timestamp as i32;
            let mut seal_time: Option<i32> = None;
            if fardel.seal_time > 0 {
                seal_time = Some(fardel.seal_time as i32);
            }
            let sealed = get_sealed_status(&deps.storage, unpack_id);
            let img = get_fardel_img(&deps.storage, unpack_id);

            let mut remaining: Option<i32> = None;
            if fardel.countable > 0 {
                let unpack_count = get_fardel_unpack_count(&deps.storage, unpack_id).unwrap_or_else(|_| 0_u64) as u16;
                remaining = Some((fardel.countable - unpack_count) as i32);
            }

            let fardel_owner = get_fardel_owner(&deps.storage, unpack_id).unwrap();
            let carrier = get_account(&deps.storage, &fardel_owner).unwrap().into_humanized(&deps.api).unwrap().handle;

            let mut rating: Option<bool> = None;
            match get_rating(&deps.storage, &address, unpack_id) {
                Ok(r) => rating = Some(r),
                Err(_) => {}
            };

            fardels.push(FardelResponse {
                id: fardel.hash_id,
                carrier,
                public_message: fardel.public_message.clone(),
                tags: fardel.tags,
                cost: fardel.cost.amount,
                unpacked: true,
                pending_unpack: false,
                upvotes,
                downvotes,
                number_of_comments,
                remaining,
                seal_time,
                sealed,
                timestamp,
                contents_data: Some(fardel.contents_data),
                rating,
                img,
            });
        }
    }
    let unpacks_count = get_number_of_unpacked_by_unpacker(&deps.storage, &address) as i32;
    let fardels_count = get_number_of_fardels(&deps.storage, &address) as i32;
    let total_count = unpacks_count - fardels_count; // don't include own fardels in # of unpacks
    let response = QueryAnswer::GetUnpacked {
        fardels,
        total_count,
    };
    to_binary(&response)
}

// get user's current rating for a fardel
pub fn query_get_rating<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    fardel_id: Uint128,
) -> QueryResult {
    let mut rating: Option<bool> = None;
    let address = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &address) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let global_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;
    match get_rating(&deps.storage, &address, global_id) {
        Ok(r) => rating = Some(r),
        Err(_) => {}
    };
    let response = QueryAnswer::GetRating { rating };
    to_binary(&response)
}

// get pending approvals of unpacks for the given fardel owner
pub fn query_get_pending_approvals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    number: Option<i32>,
) -> QueryResult {
    let owner = deps.api.canonical_address(account)?;

    if is_banned(&deps.storage, &owner) {
        return Err(StdError::generic_err("Account has been banned."));
    } else if is_deactivated(&deps.storage, &owner) {
        return Err(StdError::generic_err("Account has been deactivated."));
    }

    let number = number.unwrap_or_else(|| 100_i32) as u32;

    let pending_unpacks = get_pending_approvals_from_start(&deps.storage, &owner, number)?;
    let mut pending: Vec<PendingApprovalResponse> = vec![];
    for pu in pending_unpacks {
        let fardel = get_fardel_by_global_id(&deps.storage, pu.fardel_id)?;
        if fardel.is_some() {
            let fardel = fardel.unwrap();
            let account = get_account(&deps.storage, &pu.unpacker)?;
            let handle = account.into_humanized(&deps.api)?.handle;
            pending.push(PendingApprovalResponse {
                fardel_id: fardel.hash_id,
                handle,
                canceled: pu.canceled,
            });
        }
    }
    let response = QueryAnswer::GetPendingApprovals { pending };
    to_binary(&response)
}

// get fardels batch by global id -- public data only -- for admin only!
pub fn query_get_fardels_batch<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    start: Option<Uint128>,
    count: Option<Uint128>,
) -> QueryResult {
    let config = ReadonlyConfig::from_storage(&deps.storage);
    let constants = config.constants()?;

    // permission check - admin only
    if deps.api.canonical_address(address)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    let start = start.unwrap_or_else(|| Uint128(0)).u128();
    let count = count.unwrap_or_else(|| Uint128(10)).u128();

    let mut fardels: Vec<FardelBatchResponse> = vec![];
    let mut end = start + count;
    let total = get_total_fardel_count(&deps.storage);
    if end > total {
        end = total;
    }

    for idx in start..end {
        let owner = get_fardel_owner(&deps.storage, idx)?;
        let banned = is_banned(&deps.storage, &owner);
        let deactivated = is_deactivated(&deps.storage, &owner);
        let removed = is_fardel_removed(&deps.storage, idx);
        let hidden = is_fardel_hidden(&deps.storage, idx);
        // ignore if fardel is hidden or user is banned or deactivated
        if !(banned || deactivated || removed || hidden) {
            let fardel: Option<Fardel> = get_fardel_by_global_id(&deps.storage, idx)?;
            if fardel.is_some() {
                let fardel = fardel.unwrap();
                let upvotes: i32 = get_upvotes(&deps.storage, idx) as i32;
                let downvotes: i32 = get_downvotes(&deps.storage, idx) as i32;

                let number_of_comments = get_number_of_comments(&deps.storage, idx) as i32;
                let timestamp: i32 = fardel.timestamp as i32;
                let mut seal_time: Option<i32> = None;
                if fardel.seal_time > 0 {
                    seal_time = Some(fardel.seal_time as i32);
                }
                let sealed = get_sealed_status(&deps.storage, idx);
                let img = get_fardel_img(&deps.storage, idx);
                let account = get_account(&deps.storage, &owner)?.into_humanized(&deps.api)?;

                let mut remaining: Option<i32> = None;
                if fardel.countable > 0 {
                    let unpack_count = get_fardel_unpack_count(&deps.storage, idx).unwrap_or_else(|_| 0_u64) as u16;
                    remaining = Some((fardel.countable - unpack_count) as i32);
                }

                // fardel batch only gets public data
                fardels.push(FardelBatchResponse {
                    global_id: Uint128(idx),
                    hash_id: fardel.hash_id,
                    public_message: fardel.public_message.clone(),
                    tags: fardel.tags,
                    cost: fardel.cost.amount,
                    upvotes,
                    downvotes,
                    number_of_comments,
                    remaining,
                    seal_time,
                    sealed,
                    timestamp,
                    img,
                    owner: account.owner,
                    handle: account.handle,
                });
            }
        }
    }

    let answer = QueryAnswer::GetFardelsBatch { fardels };
    to_binary(&answer)
}

// get all registered accounts -- public data only
pub fn query_get_registered_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    start: Option<i32>,
    count: Option<i32>,
) -> QueryResult {
    let config = ReadonlyConfig::from_storage(&deps.storage);
    let constants = config.constants()?;

    // permission check - admin only
    if deps.api.canonical_address(address)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    let start = start.unwrap_or_else(|| 0_i32) as u32;
    let count = count.unwrap_or_else(|| 10_i32) as u32;

    let accounts: Vec<RegisteredAccountsResponse> =
        get_registered_addresses(&deps.storage, start, count)?
            .iter()
            .map(|a| {
                let banned = is_banned(&deps.storage, &a);
                let deactivated = is_deactivated(&deps.storage, &a);
                let account = get_account(&deps.storage, &a)
                    .unwrap()
                    .into_humanized(&deps.api)
                    .unwrap();
                let img = get_account_img(&deps.storage, &a).unwrap_or_else(|_| vec![]);
                let img_str = String::from_utf8(img).unwrap();
                RegisteredAccountsResponse {
                    address: deps.api.human_address(a).unwrap(),
                    banned,
                    deactivated,
                    handle: Some(account.handle),
                    description: Some(account.description),
                    view_settings: Some(account.view_settings),
                    img: Some(img_str),
                }
            })
            .collect();
    let total_registered = get_total_number_registered_accounts(&deps.storage)? as i32;
    let answer = QueryAnswer::GetRegisteredAccounts { accounts, total_registered };
    to_binary(&answer)
}
