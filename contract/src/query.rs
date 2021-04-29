use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, 
    StdError,
    StdResult, Storage, QueryResult, Uint128
};
use crate::msg::{
    QueryAnswer, ResponseStatus, 
    ResponseStatus::Success, ResponseStatus::Failure, FardelResponse
};
use crate::state::{get_account, get_account_for_handle,
    get_account_img,
    Fardel, get_fardel_by_id, get_fardel_by_hash,
    get_fardels,
    get_following, get_followers,
    get_unpacked_status_by_fardel_id, 
    get_upvotes, get_downvotes, 
    get_comments,
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
    let has_ipfs_cid: bool = fardel.ipfs_cid.len() > 0;

    let upvotes: i32 = get_upvotes(&deps.storage, fardel_id) as i32;
    let downvotes: i32 = get_downvotes(&deps.storage, fardel_id) as i32;

    // get last 10 comments, TODO: pagination
    let comments: Vec<String> = get_comments(&deps.storage, fardel_id, 0_u32, 10_u32)?
                                    .iter()
                                    .map(|c| String::from_utf8(c.text.clone()).ok().unwrap())
                                    .collect();
    // unpacked parts
    let contents_text: Option<String> = None;
    let ipfs_cid: Option<String> = None;
    let passphrase: Option<String> = None;
    let timestamp: i32 = fardel.timestamp as i32;
    let fardel_response = FardelResponse {
        id: Uint128(fardel_id),
        public_message: fardel.public_message,
        cost: fardel.cost.amount,
        unpacked: false,
        has_ipfs_cid,
        upvotes,
        downvotes,
        comments,
        contents_text,
        ipfs_cid,
        passphrase,
        timestamp,
    };
    let answer = QueryAnswer::GetFardelById {
        fardel: fardel_response,
    };
    to_binary(&answer)
}

pub fn query_get_fardel_by_id_auth<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
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
    let has_ipfs_cid: bool = fardel.ipfs_cid.len() > 0;

    let upvotes: i32 = get_upvotes(&deps.storage, fardel_id) as i32;
    let downvotes: i32 = get_downvotes(&deps.storage, fardel_id) as i32;

    // get last 10 comments, TODO: pagination
    let comments: Vec<String> = get_comments(&deps.storage, fardel_id, 0_u32, 10_u32)?
                                    .iter()
                                    .map(|c| String::from_utf8(c.text.clone()).ok().unwrap())
                                    .collect();
    // unpacked parts
    let mut contents_text: Option<String> = None;
    let mut ipfs_cid: Option<String> = None;
    let mut passphrase: Option<String> = None;
    let mut unpacked = false;

    let unpacker = &deps.api.canonical_address(address)?;
    if get_unpacked_status_by_fardel_id(&deps.storage, unpacker, fardel_id) {
        contents_text = Some(fardel.contents_text);
        ipfs_cid = Some(fardel.ipfs_cid);
        passphrase = Some(fardel.passphrase);
        unpacked = true;
    }

    let timestamp: i32 = fardel.timestamp as i32;

    let fardel_response = FardelResponse {
        id: Uint128(fardel_id),
        public_message: fardel.public_message,
        cost: fardel.cost.amount,
        unpacked,
        has_ipfs_cid,
        upvotes,
        downvotes,
        comments,
        contents_text,
        ipfs_cid,
        passphrase,
        timestamp,
    };
    let answer = QueryAnswer::GetFardelById {
        fardel: fardel_response,
    };
    to_binary(&answer)
}

pub fn query_get_fardels<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
                let has_ipfs_cid: bool = fardel.ipfs_cid.len() > 0;
                let fardel_id = fardel.global_id.u128();
                let upvotes: i32 = get_upvotes(&deps.storage, fardel_id) as i32;
                let downvotes: i32 = get_downvotes(&deps.storage, fardel_id) as i32;

                // get last 10 comments, TODO: pagination
                let comments: Vec<String> = get_comments(&deps.storage, fardel_id, 0_u32, 10_u32).unwrap()
                    .iter()
                    .map(|c| String::from_utf8(c.text.clone()).ok().unwrap())
                    .collect();

                // unpacked parts
                let contents_text: Option<String> = None;
                let ipfs_cid: Option<String> = None;
                let passphrase: Option<String> = None;
                let timestamp: i32 = fardel.timestamp as i32;
                FardelResponse {
                    id: Uint128(fardel_id),
                    public_message: fardel.public_message.clone(),
                    cost: fardel.cost.amount,
                    unpacked: true,
                    has_ipfs_cid,
                    upvotes,
                    downvotes,
                    comments,
                    contents_text,
                    ipfs_cid,
                    passphrase,
                    timestamp,
                }
            })
            .collect();
    }
    let answer = QueryAnswer::GetFardels {
        fardels: fardels_response,
    };
    to_binary(&answer)
}

pub fn query_get_fardels_auth<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
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
                let has_ipfs_cid: bool = fardel.ipfs_cid.len() > 0;
                let fardel_id = fardel.global_id.u128();
                let upvotes: i32 = get_upvotes(&deps.storage, fardel_id) as i32;
                let downvotes: i32 = get_downvotes(&deps.storage, fardel_id) as i32;

                // get last 10 comments, TODO: pagination
                let comments: Vec<String> = get_comments(&deps.storage, fardel_id, 0_u32, 10_u32).unwrap()
                    .iter()
                    .map(|c| String::from_utf8(c.text.clone()).ok().unwrap())
                    .collect();
                //let comments: Vec<String> = vec![];

                // unpacked parts
                let mut contents_text: Option<String> = None;
                let mut ipfs_cid: Option<String> = None;
                let mut passphrase: Option<String> = None;
                let timestamp: i32 = fardel.timestamp as i32;
                let mut unpacked = false;

                let unpacker = &deps.api.canonical_address(address).unwrap();
                if get_unpacked_status_by_fardel_id(&deps.storage, unpacker, fardel_id) {
                    contents_text = Some(fardel.contents_text.clone());
                    ipfs_cid = Some(fardel.ipfs_cid.clone());
                    passphrase = Some(fardel.passphrase.clone());
                    unpacked = true;
                }

                FardelResponse {
                    id: Uint128(fardel_id),
                    public_message: fardel.public_message.clone(),
                    cost: fardel.cost.amount,
                    unpacked,
                    has_ipfs_cid,
                    upvotes,
                    downvotes,
                    comments,
                    contents_text,
                    ipfs_cid,
                    passphrase,
                    timestamp,
                }
            })
            .collect();
    }
    let answer = QueryAnswer::GetFardels {
        fardels: fardels_response,
    };
    to_binary(&answer)
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
    let following: Vec<String> = get_following(&deps.api, &deps.storage, &address).unwrap_or_else(|_| vec![]);
    let response = QueryAnswer::GetFollowing { following };
    to_binary(&response)
}

pub fn query_get_followers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: Option<i32>,
    page_size: Option<i32>,
) -> StdResult<Binary> {
    //TODO: FIX
    //let followers = vec![];
    let response = QueryAnswer::GetFollowers { followers };
    to_binary(&response)
}