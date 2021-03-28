use std::convert::TryFrom;
use cosmwasm_std::{
    to_binary, Api, Binary, Coin, Env, Extern, HandleResponse, CanonicalAddr, HumanAddr, 
    InitResponse, Querier, 
    StdError, CosmosMsg, BankMsg, 
    StdResult, Storage, QueryResult, Uint128
};
use secret_toolkit::crypto::sha_256;
use crate::msg::{HandleMsg, HandleAnswer, InitMsg, QueryMsg, QueryAnswer, ResponseStatus, 
    ResponseStatus::Success, ResponseStatus::Failure, FardelResponse};
use crate::state::{Config, Constants, ReadonlyConfig, 
    Account, get_account, get_account_for_handle, map_handle_to_account, delete_handle_map,
    store_account, store_account_img, get_account_img,
    Fardel, get_fardel_by_id, get_fardel_owner, seal_fardel, store_fardel, get_fardels,
    store_following, get_following, remove_following,
    get_unpacked_status_by_fardel_id, store_unpack, 
    get_upvotes, get_downvotes, upvote_fardel, downvote_fardel, 
    get_comments, comment_on_fardel,
    read_viewing_key, write_viewing_key};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};

/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

// maximum cost of a fardel in uscrt
pub const DEFAULT_MAX_COST: u128 = 5000000_u128;
pub const DEFAULT_MAX_PUBLIC_MESSAGE_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_THUMBNAIL_IMG_SIZE: u32 = 65536_u32;
pub const DEFAULT_MAX_CONTENTS_TEXT_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_IPFS_CID_LEN: u16 = 128_u16;
pub const DEFAULT_MAX_CONTENTS_PASSPHRASE_LEN: u16 = 64_u16;
pub const DEFAULT_MAX_HANDLE_LEN: u16 = 64_u16;
pub const DEFAULT_MAX_DESCRIPTION_LEN: u16 = 280_u16;

pub const DENOM: &str = "uscrt";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let admin = msg.admin.unwrap_or_else(|| env.message.sender);

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);

    let max_cost = match msg.max_cost{
        Some(v) => v.u128(),
        None => DEFAULT_MAX_COST
    };

    let max_public_message_len = valid_max_public_message_len(msg.max_public_message_len)?;
    let max_thumbnail_img_size = valid_max_thumbnail_img_size(msg.max_thumbnail_img_size)?;
    let max_contents_text_len = valid_max_contents_text_len(msg.max_contents_text_len)?;
    let max_ipfs_cid_len = valid_max_ipfs_cid_len(msg.max_ipfs_cid_len)?;
    let max_contents_passphrase_len = valid_max_contents_passphrase_len(msg.max_contents_passphrase_len)?;
    let max_handle_len = valid_max_handle_len(msg.max_handle_len)?;
    let max_description_len = valid_max_description_len(msg.max_description_len)?;

    let mut config = Config::from_storage(&mut deps.storage);
    config.set_constants(&Constants {
        admin,
        max_cost,
        max_public_message_len,
        max_thumbnail_img_size,
        max_contents_text_len,
        max_ipfs_cid_len,
        max_contents_passphrase_len,
        max_handle_len,
        max_description_len,
        prng_seed: prng_seed_hashed.to_vec(),
    })?;

    Ok(InitResponse::default())
}

//
// init helper functions
//

// limit the max public message size to values in 1..65535, default 280 bytes
fn valid_max_public_message_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_public_message_len"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_public_message_len")))
            }
        },
        None => Ok(DEFAULT_MAX_PUBLIC_MESSAGE_LEN)
    }
}

// limit the max thumbnail img size in bytes to u32, default 64K
fn valid_max_thumbnail_img_size(val: Option<i32>) -> StdResult<u32> {
    match val {
        Some(v) => {
            u32::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_thumbnail_img_size")))
        },
        None => Ok(DEFAULT_MAX_THUMBNAIL_IMG_SIZE)
    }
}

// limit the max contents text to values in 1..65535, default 280 bytes
fn valid_max_contents_text_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_contents_text_len"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_contents_text_len")))
            }
        },
        None => Ok(DEFAULT_MAX_CONTENTS_TEXT_LEN)
    }
}

// limit the max IPFS CID length to values in 1..65535, default 128 bytes
fn valid_max_ipfs_cid_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_ipfs_cid_len"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_ipfs_cid_len")))
            }
        },
        None => Ok(DEFAULT_MAX_IPFS_CID_LEN)
    }
}

// limit the max contents passphrase length (in bytes) to values in 16..65535, default 64 bytes
fn valid_max_contents_passphrase_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 16 {
                Err(StdError::generic_err("invalid_max_contents_passphrase_length"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_contents_passphrase_length")))
            }
        },
        None => Ok(DEFAULT_MAX_CONTENTS_PASSPHRASE_LEN)
    }
}

// limit the max handle length (in bytes) to values in 8..65535, default 64 bytes
fn valid_max_handle_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 8 {
                Err(StdError::generic_err("invalid_max_handle_length"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_handle_length")))
            }
        },
        None => Ok(DEFAULT_MAX_HANDLE_LEN)
    }
}

// limit the max description length (in bytes) to values in 1..65535, default 280 bytes
fn valid_max_description_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_description_length"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_description_length")))
            }
        },
        None => Ok(DEFAULT_MAX_DESCRIPTION_LEN)
    }
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        // Account 
        HandleMsg::Register { handle, description, .. } => 
          try_register(deps, env, handle, description),
        HandleMsg::SetProfileThumbnailImg { img, .. } =>
          try_set_profile_thumbnail_img(deps, env, img),
        //HandleMsg::RegisterAndGenerateViewingKey { handle, entropy, .. } => 
        //  try_register_and_generate_viewing_key(deps, env, handle, entropy),
        //HandleMsg::RegisterAndSetViewingKey { handle, key, .. } => 
        //  try_register_and_set_viewing_key(deps, env, handle, key),
        HandleMsg::GenerateViewingKey { entropy, .. } => 
          try_generate_viewing_key(deps, env, entropy),
        HandleMsg::SetViewingKey { key, .. } => 
          try_set_viewing_key(deps, env, key),
        HandleMsg::Deactivate { .. } => 
          try_deactivate(deps, env),

        // My fardels
        HandleMsg::CarryFardel { public_message, contents_text, ipfs_cid, passphrase, cost, .. } =>
            try_carry_fardel(deps, env, public_message, contents_text, ipfs_cid, passphrase, cost),
        HandleMsg::SealFardel { fardel_id, .. } =>
            try_seal_fardel(deps, env, fardel_id), 

        // Other fardels
        HandleMsg::Follow { handle, .. } =>
            try_follow(deps, env, handle),
        HandleMsg::Unfollow { handle, .. } =>
            try_unfollow(deps, env, handle),
        HandleMsg::RateFardel { fardel_id, rating, .. } => 
            try_rate_fardel(deps, env, fardel_id, rating),
        HandleMsg::CommentOnFardel { fardel_id, comment, rating, .. } => 
            try_comment_on_fardel(deps, env, fardel_id, comment, rating),
        HandleMsg::UnpackFardel { fardel_id, .. } => 
            try_unpack_fardel(deps, env, fardel_id),
    }
}

//
//  handle helper functions
//

fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
    description: String,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
    let handle = handle.trim().to_owned();

    if handle.as_bytes().len() > constants.max_handle_len.into() {
        // if handle is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Handle is too long."));
    } else if description.as_bytes().len() > constants.max_description_len.into() {
        // if description is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Description is too long."));
    } else {
        match get_account_for_handle(&deps.storage, &handle) {
            Ok(_) => {
                status = Failure;
                msg = Some(String::from("Handle is already in use."))
            },
            Err(_) => {
                let message_sender = deps.api.canonical_address(&env.message.sender)?;
                // check if previously registered
                match get_account(&mut deps.storage, &message_sender) {
                    Ok(stored_account) => {
                        // yes, deactivate old handle if it is different
                        let account = stored_account.into_humanized(&deps.api)?;
                        let old_handle = account.handle;
                        if !handle.eq(&old_handle) {
                            delete_handle_map(&mut deps.storage, old_handle);
                        }
                    },
                    _ => { }
                }
                let stored_account = Account {
                    owner: env.message.sender,
                    handle: handle.clone(),
                    description,
                }.into_stored(&deps.api)?;
                map_handle_to_account(&mut deps.storage, &message_sender, handle.clone())?;
                store_account(&mut deps.storage, stored_account, &message_sender)?;
            }
        }        
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Register { status, msg })?),
    })
}

fn try_set_profile_thumbnail_img<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    img: Binary,
) -> StdResult<HandleResponse> {
    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let img: Vec<u8> = img.0;
    if img.len() as u32 > constants.max_thumbnail_img_size {
        status = Failure;
        msg = Some(String::from("Thumbnail image is too large."));
    } else {
        store_account_img(&mut deps.storage, &message_sender, img)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetProfileThumbnailImg { status, msg })?),
    })
}

//fn try_register_and_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
//    deps: &mut Extern<S, A, Q>,
//    env: Env,
//    handle: String,
//    entropy: String,
//) -> StdResult<HandleResponse> {
//    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
//    let prng_seed = constants.prng_seed;
//
//    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());
//
//    let message_sender = deps.api.canonical_address(&env.message.sender)?;
//
//    // todo store handle and key
//
//    Ok(HandleResponse {
//        messages: vec![],
//        log: vec![],
//        data: Some(to_binary(&HandleAnswer::RegisterAndGenerateViewingKey { 
//            status: Success,
//            key: Some(key),
//        })?),
//    })
//}

//fn try_register_and_set_viewing_key<S: Storage, A: Api, Q: Querier>(
//    deps: &mut Extern<S, A, Q>,
//    env: Env,
//    handle: String,
//    key: String,
//) -> StdResult<HandleResponse> {
//    let vk = ViewingKey(key);
//
//    let message_sender = deps.api.canonical_address(&env.message.sender)?;
//
//    // todo store handle and key
//
//    Ok(HandleResponse {
//        messages: vec![],
//        log: vec![],
//        data: Some(to_binary(&HandleAnswer::RegisterAndSetViewingKey { 
//            status: Success,
//        })?),
//    })
//}

fn try_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
    let prng_seed = constants.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenerateViewingKey { 
            key,
        })?),
    })
}

fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> StdResult<HandleResponse> {
    let vk = ViewingKey(key);

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &vk);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { 
            status: Success,
        })?),
    })
}

fn try_deactivate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    //TODO
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deactivate { status: Success })?),
    })
}

fn try_carry_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    public_message: String,
    contents_text: String,
    ipfs_cid: String,
    passphrase: String,
    cost: Uint128,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let mut fardel_id: Option<Uint128> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    if (public_message.as_bytes().len() > constants.max_public_message_len.into()) ||
       (contents_text.as_bytes().len() > constants.max_contents_text_len.into()) ||
       (ipfs_cid.as_bytes().len() > constants.max_ipfs_cid_len.into()) ||
       (passphrase.as_bytes().len() > constants.max_contents_passphrase_len.into()) ||
       (cost.u128() > constants.max_cost) {
        status = Failure;
        msg = Some(String::from("Invalid Fardel data"));
    } else {
        let message_sender = deps.api.canonical_address(&env.message.sender)?;

        let fardel = Fardel {
            // global_id will be overwritten in store_fardel, just a placeholder
            global_id: Uint128(0),
            public_message,
            contents_text,
            ipfs_cid,
            passphrase,
            cost: Coin {
                amount: cost,
                denom: DENOM.to_string(),
            },
            timestamp: env.block.time,
        }.into_stored()?;
    
        store_fardel(
            &mut deps.storage, &message_sender, 
            fardel.public_message, fardel.contents_text, 
            fardel.ipfs_cid, fardel.passphrase, fardel.cost,
            fardel.timestamp,
        )?;
        let config = ReadonlyConfig::from_storage(&deps.storage);
        fardel_id = Some(Uint128(config.fardel_count() - 1));
    }
    
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CarryFardel {
            status,
            msg,
            fardel_id, 
        })?),
    })
}

fn try_seal_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let fardel_id = fardel_id.u128();

    match get_fardel_by_id(&deps.storage, fardel_id) {
        Ok(_) => {
            let owner = deps.api.human_address(&get_fardel_owner(&deps.storage, fardel_id)?)?;
            if owner.eq(&env.message.sender) {
                seal_fardel(&mut deps.storage, fardel_id)?;
            } else {
                status = Failure;
                msg = Some(String::from("You are not the owner of that fardel."))
            }
        },
        _ => {
            status = Failure;
            msg = Some(String::from("No Fardel with given id."));
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SealFardel {
            status,
            msg,
        })?),
    })
}

fn try_follow<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
) -> StdResult<HandleResponse> {
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    store_following(&deps.api, &mut deps.storage, &message_sender, handle)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Follow { 
            status: Success,
        })?),
    })
}

fn try_unfollow<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
) -> StdResult<HandleResponse> {
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    remove_following(&deps.api, &mut deps.storage, &message_sender, handle)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Follow { 
            status: Success,
        })?),
    })
}

fn try_rate_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
    rating: bool,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    let fardel_id = fardel_id.u128();

    if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, fardel_id) {
        // fardel has been unpacked by the user
        if rating {
            upvote_fardel(&mut deps.storage, fardel_id)?;
        } else {
            downvote_fardel(&mut deps.storage, fardel_id)?;
        }
    } else {
        // fardel has not been unpacked by the user
        status = Failure;
        msg = Some(String::from("Cannot rate fardel until you have unpacked it."))
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RateFardel { 
            status, msg
        })?),
    })
}

fn try_comment_on_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
    comment: String,
    rating: Option<bool>,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    let fardel_id = fardel_id.u128();

    if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, fardel_id) {
        // fardel has been unpacked by the user
        // add comment
        comment_on_fardel(&mut deps.storage, &message_sender, fardel_id, comment)?;

        // handle rating if it is here
        match rating {
            Some(r) => {
                if r {
                    upvote_fardel(&mut deps.storage, fardel_id)?;
                } else {
                    downvote_fardel(&mut deps.storage, fardel_id)?;
                }
            },
            _ => {}
        }
    } else {
        // fardel has not been unpacked by the user
        status = Failure;
        msg = Some(String::from("Cannot comment or rate fardel until you have unpacked it."))
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CommentOnFardel { 
            status, msg
        })?),
    })
}

fn try_unpack_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let mut contents_text: Option<String> = None;
    let mut ipfs_cid: Option<String> = None;
    let mut passphrase: Option<String> = None;
    let mut cost: u128 = 0;
    let fardel_id = fardel_id.u128();
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let sent_coins = env.message.sent_funds;
    if sent_coins[0].denom != DENOM {
        status = Failure;
        msg = Some(String::from("Wrong denomination."))
    } else {
        let fardel = get_fardel_by_id(&deps.storage, fardel_id);
        match fardel {
            Ok(fardel) => {
                match fardel {
                    Some(f) => {
                        cost = f.cost.amount.u128();
                        let sent_amount: u128 = sent_coins[0].amount.u128();
                        if sent_amount != cost {
                            status = Failure;
                            msg = Some(String::from("Didn't send correct number of coins to unpack."))
                        } else {
                            if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, fardel_id) {
                                status = Failure;
                                msg = Some(String::from("You have already unpacked this fardel."));
                            } else {
                                store_unpack(&mut deps.storage, &message_sender, fardel_id)?;
                                contents_text = Some(f.contents_text);
                                ipfs_cid = Some(f.ipfs_cid);
                                passphrase = Some(f.passphrase);
                            }
                        }
                    },
                    None => {
                        status = Failure;
                        msg = Some(String::from("Fardel is not available to unpack."));
                    }
                }
            }
            Err(_) => {
                status = Failure;
                msg = Some(String::from("Fardel is not available to unpack."));
            }
        }
    }
    
    let mut messages: Vec<CosmosMsg> = vec![];

    if status == Success {
        let fardel_owner = deps.api.human_address(&get_fardel_owner(&deps.storage, fardel_id)?)?;
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: fardel_owner,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128(cost),
            }],
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UnpackFardel { 
            status,
            msg,
            contents_text,
            ipfs_cid,
            passphrase,
        })?),
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    match msg {
        QueryMsg::GetProfile { handle } =>
            query_get_profile(deps, handle),
        QueryMsg::IsHandleAvailable { handle } => 
            query_is_handle_available(deps, handle),
        QueryMsg::GetFardelById { fardel_id } =>
            query_get_fardel_by_id(deps, fardel_id),
        QueryMsg::GetFardels { handle, page, page_size } =>
            query_get_fardels(deps, handle, page, page_size),
        _ => authenticated_queries(deps, msg),
    }
}

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                // Base
                QueryMsg::GetHandle { address, .. } => query_get_handle(&deps, &address),
                QueryMsg::GetFollowing { address, .. } => query_get_following(&deps, &address),
                QueryMsg::GetFardelByIdAuth { address, fardel_id, .. } => 
                    query_get_fardel_by_id_auth(&deps, &address, fardel_id),
                QueryMsg::GetFardelsAuth { address, handle, page, page_size, .. } =>
                    query_get_fardels_auth(&deps, &address, handle, page, page_size),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Ok(to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })?)
}

fn query_get_profile<S: Storage, A: Api, Q: Querier>(
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

fn query_is_handle_available<S: Storage, A: Api, Q: Querier>(
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

fn query_get_fardel_by_id<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    fardel_id: Uint128,
) -> QueryResult {
    let fardel_id = fardel_id.u128();
    let fardel = get_fardel_by_id(&deps.storage, fardel_id)?;
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
        packed: true,
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

fn query_get_fardel_by_id_auth<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    fardel_id: Uint128,
) -> QueryResult {
    let fardel_id = fardel_id.u128();
    let fardel = get_fardel_by_id(&deps.storage, fardel_id)?;
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
    let mut packed = true;

    let unpacker = &deps.api.canonical_address(address)?;
    if get_unpacked_status_by_fardel_id(&deps.storage, unpacker, fardel_id) {
        contents_text = Some(fardel.contents_text);
        ipfs_cid = Some(fardel.ipfs_cid);
        passphrase = Some(fardel.passphrase);
        packed = false;
    }

    let timestamp: i32 = fardel.timestamp as i32;

    let fardel_response = FardelResponse {
        id: Uint128(fardel_id),
        public_message: fardel.public_message,
        cost: fardel.cost.amount,
        packed,
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

fn query_get_fardels<S: Storage, A: Api, Q: Querier>(
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
                    packed: true,
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

fn query_get_fardels_auth<S: Storage, A: Api, Q: Querier>(
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
                let mut packed = true;

                let unpacker = &deps.api.canonical_address(address).unwrap();
                if get_unpacked_status_by_fardel_id(&deps.storage, unpacker, fardel_id) {
                    contents_text = Some(fardel.contents_text.clone());
                    ipfs_cid = Some(fardel.ipfs_cid.clone());
                    passphrase = Some(fardel.passphrase.clone());
                    packed = false;
                }

                FardelResponse {
                    id: Uint128(fardel_id),
                    public_message: fardel.public_message.clone(),
                    cost: fardel.cost.amount,
                    packed,
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

fn query_get_handle<S: Storage, A: Api, Q: Querier>(
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

fn query_get_following<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
) -> StdResult<Binary> {
    let address = deps.api.canonical_address(account)?;
    let following: Vec<String> = get_following(&deps.api, &deps.storage, &address).unwrap_or_else(|_| vec![]);
    let response = QueryAnswer::GetFollowing { following };
    to_binary(&response)
}

