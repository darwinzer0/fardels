use cosmwasm_std::{
    to_binary, Api, Binary, Coin, Env, Extern, HandleResponse, Querier, 
    CosmosMsg, BankMsg, 
    StdResult, Storage, Uint128
};
use crate::msg::{
    HandleAnswer, ResponseStatus, 
    ResponseStatus::Success, ResponseStatus::Failure,
};
use crate::state::{ReadonlyConfig, 
    Account, get_account, get_account_for_handle, map_handle_to_account, delete_handle_map,
    store_account, store_account_img,
    Fardel, get_fardel_by_id, get_fardel_owner, seal_fardel, store_fardel, 
    store_following, remove_following,
    get_unpacked_status_by_fardel_id, store_unpack, upvote_fardel, downvote_fardel, comment_on_fardel,
    write_viewing_key
};
use crate::viewing_key::{ViewingKey};
use crate::contract::DENOM;

pub fn try_register<S: Storage, A: Api, Q: Querier>(
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

pub fn try_set_profile_thumbnail_img<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    img: Binary,
) -> StdResult<HandleResponse> {
    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let img: Vec<u8> = img.0;
    if img.len() as u32 > constants.max_profile_img_size {
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

pub fn try_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
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

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
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

pub fn try_deactivate<S: Storage, A: Api, Q: Querier>(
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

pub fn try_carry_fardel<S: Storage, A: Api, Q: Querier>(
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

pub fn try_seal_fardel<S: Storage, A: Api, Q: Querier>(
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

pub fn try_follow<S: Storage, A: Api, Q: Querier>(
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

pub fn try_unfollow<S: Storage, A: Api, Q: Querier>(
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

pub fn try_rate_fardel<S: Storage, A: Api, Q: Querier>(
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

pub fn try_comment_on_fardel<S: Storage, A: Api, Q: Querier>(
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

pub fn try_unpack_fardel<S: Storage, A: Api, Q: Querier>(
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