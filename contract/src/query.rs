use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, 
    StdError,
    StdResult, Storage, QueryResult, Uint128
};
use crate::msg::{
    QueryAnswer, ResponseStatus, CommentResponse,
    ResponseStatus::Success, ResponseStatus::Failure, FardelResponse, PendingUnpackResponse,
};
use crate::state::{
    ReadonlyConfig,
    get_account, get_account_for_handle,
    get_account_img,
    Fardel, get_fardel_by_id, get_fardel_by_hash,
    get_fardels, get_sealed_status,
    get_following, get_followers,
    get_unpacked_status_by_fardel_id, 
    get_upvotes, get_downvotes, 
    get_comments, get_number_of_comments,
    Tx, get_txs,
    get_unpacked_by_unpacker,
    get_pending_unpacks_from_start,
};

pub fn query_get_profile<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    handle: String,
) -> QueryResult {
    let status: ResponseStatus = Success;
    let account = get_account_for_handle(&deps.storage, &handle)?;
    let description = get_account(&deps.storage, &account)?.into_humanized(&deps.api)?.description;
    let img = get_account_img(&deps.storage, &account).unwrap_or_else(|_| vec![]);
    let bin_img = Binary::from(img.as_slice());
    let answer = QueryAnswer::GetProfile {
        status,
        handle: Some(handle),
        description: Some(description),
        img: Some(bin_img),
    };
    to_binary(&answer)
}

pub fn query_is_handle_available<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    handle: String,
) -> QueryResult {
    let response = match get_account_for_handle(&deps.storage, &handle) {
        Ok(_) => true,
        Err(_) => false
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
        Some(fardel) => { fardel },
        None => { 
            return Err(StdError::generic_err("Fardel not found.",));
        }
    };
    let global_id = fardel.global_id.u128();

    let upvotes: i32 = get_upvotes(&deps.storage, global_id) as i32;
    let downvotes: i32 = get_downvotes(&deps.storage, global_id) as i32;

    // get last 10 comments
    let comments: Vec<CommentResponse> = get_comments(&deps.storage, global_id, 0_u32, 10_u32)?
        .iter()
        .map(|c| {
            let commenter_account = get_account(&deps.storage, &c.commenter).unwrap();
            let mut response_fardel_id: Option<Uint128> = None;
            let mut response_comment_id: Option<i32> = None;
            if address.is_some() {
                let unpacker_address = address.clone().unwrap();
                let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
                if unpacker == c.commenter {
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
    let number_of_comments = get_number_of_comments(&deps.storage, global_id) as i32;

    // unpacked parts
    let mut contents_data: Option<String> = None;
    let mut unpacked = false;

    if address.is_some() {
        let unpacker_address = address.clone().unwrap();
        let unpacker = &deps.api.canonical_address(&unpacker_address)?;
        let unpacked_status = get_unpacked_status_by_fardel_id(&deps.storage, unpacker, global_id);
        if unpacked_status.unpacked {
            contents_data = Some(fardel.contents_data[unpacked_status.package_idx as usize].clone());
            unpacked = true;
        }
    }

    let timestamp: i32 = fardel.timestamp as i32;
    let mut seal_time: Option<i32> = None;
    if fardel.seal_time > 0 {
        seal_time = Some(fardel.seal_time as i32);
    }
    let sealed = get_sealed_status(&deps.storage, global_id);
    let fardel_response = FardelResponse {
        id: fardel.hash_id,
        public_message: fardel.public_message,
        tags: fardel.tags,
        cost: fardel.cost.amount,
        unpacked,
        upvotes,
        downvotes,
        number_of_comments,
        comments,
        seal_time,
        sealed,
        timestamp,
        contents_data,
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

    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;
    let fardels: Vec<Fardel> = get_fardels(&deps.storage, &account, page, page_size).unwrap_or_else(|_| vec![]);

    let mut fardels_response: Vec<FardelResponse> = vec![];
    if fardels.len() > 0 {
        fardels_response = fardels
            .iter()
            .map(|fardel| {
                let global_id = fardel.global_id.u128();
                let upvotes: i32 = get_upvotes(&deps.storage, global_id) as i32;
                let downvotes: i32 = get_downvotes(&deps.storage, global_id) as i32;

                // get last 10 comments
                let comments: Vec<CommentResponse> = get_comments(&deps.storage, global_id, 0_u32, 10_u32).unwrap()
                    .iter()
                    .map(|c| {
                        let commenter_account = get_account(&deps.storage, &c.commenter).unwrap();
                        let mut response_fardel_id: Option<Uint128> = None;
                        let mut response_comment_id: Option<i32> = None;
                        if address.is_some() {
                            let unpacker_address = address.clone().unwrap();
                            let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
                            if unpacker == c.commenter {
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
                let number_of_comments = get_number_of_comments(&deps.storage, global_id) as i32;

                // unpacked parts
                let mut contents_data: Option<String> = None;
                let mut unpacked = false;

                if address.is_some() {
                    let unpacker_address = address.clone().unwrap();
                    let unpacker = deps.api.canonical_address(&unpacker_address).unwrap();
                    let unpacked_status = get_unpacked_status_by_fardel_id(&deps.storage, &unpacker, global_id);
                    if unpacked_status.unpacked {
                        contents_data = Some(fardel.contents_data[unpacked_status.package_idx as usize].clone());
                        unpacked = true;
                    }
                }

                let timestamp: i32 = fardel.timestamp as i32;
                let mut seal_time: Option<i32> = None;
                if fardel.seal_time > 0 {
                    seal_time = Some(fardel.seal_time as i32);
                }
                let sealed = get_sealed_status(&deps.storage, global_id);

                FardelResponse {
                    id: fardel.hash_id,
                    public_message: fardel.public_message.clone(),
                    tags: fardel.tags.clone(),
                    cost: fardel.cost.amount,
                    unpacked,
                    upvotes,
                    downvotes,
                    number_of_comments,
                    comments,
                    seal_time,
                    sealed,
                    timestamp,
                    contents_data,
                }
            })
            .collect();
    }
    let answer = QueryAnswer::GetFardels {
        fardels: fardels_response,
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
        Some(fardel) => { fardel },
        None => { 
            return Err(StdError::generic_err("Fardel not found.",));
        }
    };
    let global_id = fardel.global_id.u128();

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

pub fn query_get_transactions<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let txs: Vec<Tx> = get_txs(&deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let response = QueryAnswer::GetTransactions { txs };
    to_binary(&response)
}

pub fn query_get_handle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
) -> StdResult<Binary> {
    let mut status: ResponseStatus = Success;
    let address = deps.api.canonical_address(account)?;
    
    let handle: Option<String> = match get_account(&deps.storage, &address) {
        Ok(acc) => Some(acc.into_humanized(&deps.api)?.handle),
        Err(_) => {
            status = Failure;
            None
        }
    };

    let answer = QueryAnswer::GetHandle { status, handle };
    to_binary(&answer)
}

pub fn query_get_following<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let following: Vec<String> = get_following(&deps.api, &deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let response = QueryAnswer::GetFollowing { following };
    to_binary(&response)
}

pub fn query_get_followers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let followers: Vec<String> = get_followers(&deps.api, &deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let response = QueryAnswer::GetFollowers { followers };
    to_binary(&response)
}

pub fn query_get_unpacked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let page = page.unwrap_or_else(|| 0_i32) as u32;
    let page_size = page_size.unwrap_or_else(|| 10_i32) as u32;

    let unpacked_ids: Vec<u128> = get_unpacked_by_unpacker(&deps.storage, &address, page, page_size).unwrap_or_else(|_| vec![]);
    let mut fardels: Vec<FardelResponse> = vec![];
    for unpack_id in unpacked_ids {
        let fardel = get_fardel_by_id(&deps.storage, unpack_id)?;
        if fardel.is_some() {
            let fardel = fardel.unwrap();
            let upvotes: i32 = get_upvotes(&deps.storage, unpack_id) as i32;
            let downvotes: i32 = get_downvotes(&deps.storage, unpack_id) as i32;
            // don't get comments
            let comments: Vec<CommentResponse> = vec![];

            let number_of_comments = get_number_of_comments(&deps.storage, unpack_id) as i32;

            // we know they are unpacked but we need package_idx
            let unpacked_status = get_unpacked_status_by_fardel_id(&deps.storage, &address, unpack_id);
            let contents_data = Some(fardel.contents_data[unpacked_status.package_idx as usize].clone());

            let timestamp: i32 = fardel.timestamp as i32;
            let mut seal_time: Option<i32> = None;
            if fardel.seal_time > 0 {
                seal_time = Some(fardel.seal_time as i32);
            }
            let sealed = get_sealed_status(&deps.storage, unpack_id);

            fardels.push(FardelResponse {
                id: fardel.hash_id,
                public_message: fardel.public_message.clone(),
                tags: fardel.tags,
                cost: fardel.cost.amount,
                unpacked: true,
                upvotes,
                downvotes,
                number_of_comments,
                comments,
                seal_time,
                sealed,
                timestamp,
                contents_data,
            });
        }
    }
    let response = QueryAnswer::GetUnpacked { fardels };
    to_binary(&response)
}

// get pending unpacks for fardel owner
pub fn query_get_pending_unpacks<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    number: Option<i32>,
) -> StdResult<Binary> {
    let owner = deps.api.canonical_address(account)?;
    let number = number.unwrap_or_else(|| 100_i32) as u32;

    //let pending_start = get_pending_start(&deps.storage, &owner);
    let pending_unpacks = get_pending_unpacks_from_start(&deps.storage, &owner, number)?;
    let mut pending: Vec<PendingUnpackResponse> = vec![];
    for pu in pending_unpacks {
        let fardel = get_fardel_by_id(&deps.storage, pu.fardel_id)?;
        if fardel.is_some() {
            let fardel = fardel.unwrap();
            let account = get_account(&deps.storage, &pu.unpacker)?;
            let handle = account.into_humanized(&deps.api)?.handle;
            pending.push(
                PendingUnpackResponse {
                    fardel_id: fardel.hash_id,
                    handle,
                }
            );
        }
    }
    let response = QueryAnswer::GetPendingUnpacks { pending };
    to_binary(&response)
}

// get pending unpacks for fardels sender is unpacking

// get fardels batch by global id -- public data only -- for admin only!
pub fn query_get_fardels_batch<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    start: Option<Uint128>,
    count: Option<Uint128>,
) -> QueryResult {
    let config = ReadonlyConfig::from_storage(&deps.storage);
    let constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(address)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    let start = start.unwrap_or_else(|| Uint128(0)).u128();
    let count = count.unwrap_or_else(|| Uint128(10)).u128();

    let mut fardels: Vec<FardelResponse> = vec![];
    for idx in start..(start+count) {
        let fardel: Option<Fardel> = get_fardel_by_id(&deps.storage, idx)?;
        if fardel.is_some() {
            let fardel = fardel.unwrap();
            let upvotes: i32 = get_upvotes(&deps.storage, idx) as i32;
            let downvotes: i32 = get_downvotes(&deps.storage, idx) as i32;

            let comments: Vec<CommentResponse> = vec![];
            let number_of_comments = get_number_of_comments(&deps.storage, idx) as i32;
            let contents_data: Option<String> = None;
            let unpacked = false;
            let timestamp: i32 = fardel.timestamp as i32;
            let mut seal_time: Option<i32> = None;
            if fardel.seal_time > 0 {
                seal_time = Some(fardel.seal_time as i32);
            }
            let sealed = get_sealed_status(&deps.storage, idx);

            fardels.push (
                FardelResponse {
                    id: fardel.hash_id,
                    public_message: fardel.public_message.clone(),
                    tags: fardel.tags,
                    cost: fardel.cost.amount,
                    unpacked,
                    upvotes,
                    downvotes,
                    number_of_comments,
                    comments,
                    seal_time,
                    sealed,
                    timestamp,
                    contents_data,
                }
            );
        }
    }

    let answer = QueryAnswer::GetFardelsBatch {
        fardels,
    };
    to_binary(&answer)
}