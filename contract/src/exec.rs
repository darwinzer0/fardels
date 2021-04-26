use cosmwasm_std::{
    to_binary, Api, Binary, Coin, Env, Extern, HandleResponse, Querier, 
    CosmosMsg, BankMsg, HumanAddr,
    StdError, StdResult, Storage, Uint128
};
use crate::msg::{
    HandleAnswer, ResponseStatus, 
    ResponseStatus::Success, ResponseStatus::Failure, Fee,
};
use crate::state::{Config, ReadonlyConfig,
    Account, get_account, get_account_for_handle, map_handle_to_account, delete_handle_map,
    store_account, store_account_img, store_account_ban,
    Fardel, get_fardel_by_id, get_fardel_owner, seal_fardel, store_fardel, 
    store_following, remove_following,
    store_account_deactivated,
    get_unpacked_status_by_fardel_id, store_unpack, 
    upvote_fardel, downvote_fardel, comment_on_fardel,
    write_viewing_key, get_commission_balance,
};
use crate::validation::{
    valid_max_public_message_len, valid_max_thumbnail_img_size, valid_max_contents_data_len, 
    valid_max_handle_len, valid_max_tag_len, valid_max_number_of_tags,
    valid_max_description_len, valid_max_query_page_size,
};
use crate::viewing_key::{ViewingKey};
use crate::contract::DENOM;

// admin-only functions

pub fn try_set_constants<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    transaction_fee: Option<Fee>, 
    max_query_page_size: Option<i32>, 
    max_cost: Option<Uint128>, 
    max_public_message_len: Option<i32>, 
    max_tag_len: Option<i32>, 
    max_number_of_tags: Option<i32>,
    max_fardel_img_size: Option<i32>, 
    max_contents_data_len: Option<i32>, 
    max_handle_len: Option<i32>,
    max_profile_img_size: Option<i32>, 
    max_description_len: Option<i32>,
) -> StdResult<HandleResponse> {
    let mut config = Config::from_storage(&mut deps.storage);
    let mut constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    if transaction_fee.is_some() {
        constants.transaction_fee = transaction_fee.unwrap();
    }
    if max_query_page_size.is_some() {
        constants.max_query_page_size = valid_max_query_page_size(max_query_page_size)?;
    }
    if max_cost.is_some() {
        constants.max_cost = max_cost.unwrap().u128();
    }
    if max_public_message_len.is_some() {
        constants.max_public_message_len = valid_max_public_message_len(max_public_message_len)?;
    }
    if max_tag_len.is_some() {
        constants.max_tag_len = valid_max_tag_len(max_tag_len)?;
    }
    if max_number_of_tags.is_some() {
        constants.max_number_of_tags = valid_max_number_of_tags(max_number_of_tags)?;
    }
    if max_fardel_img_size.is_some() {
        constants.max_fardel_img_size = valid_max_thumbnail_img_size(max_fardel_img_size)?;
    }
    if max_contents_data_len.is_some() {
        constants.max_contents_data_len = valid_max_contents_data_len(max_contents_data_len)?;
    }
    if max_handle_len.is_some() {
        constants.max_handle_len = valid_max_handle_len(max_handle_len)?;
    }
    if max_description_len.is_some() {
        constants.max_description_len = valid_max_description_len(max_description_len)?;
    }
    if max_profile_img_size.is_some() {
        constants.max_profile_img_size = valid_max_thumbnail_img_size(max_profile_img_size)?;
    }
    config.set_constants(&constants);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetConstants { status: Success })?),
    })
}

pub fn try_change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_admin: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config = Config::from_storage(&mut deps.storage);
    let mut constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    constants.admin = deps.api.canonical_address(&new_admin)?;

    config.set_constants(&constants);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?),
    })
}

pub fn try_store_ban<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: Option<String>,
    address: Option<HumanAddr>,
    banned: bool,
) -> StdResult<HandleResponse> {
    let mut status = Success;
    let mut msg = None;

    let mut config = Config::from_storage(&mut deps.storage);
    let mut constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    // check if address given first
    if address.is_some() {
        store_account_ban(
            &mut deps.storage, 
            &deps.api.canonical_address(&address.unwrap())?, 
            banned
        );
    } else if handle.is_some() { // otherwise use handle
        store_account_ban(
            &mut deps.storage,
            &get_account_for_handle(&deps.storage, &handle.unwrap())?,
            banned
        );
    } else {
        status = Failure;
        msg = Some(String::from("No handle or address given."));
    }

    if banned {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Ban { status, msg })?),
        })
    } else {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Unban { status, msg })?),
        })
    }
}

pub fn try_draw_commission<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: Option<HumanAddr>,
    amount: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let mut status = Success;
    let mut msg = None;

    let mut config = Config::from_storage(&mut deps.storage);
    let mut constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    let address = match address {
        Some(a) => a,
        None => deps.api.human_address(&constants.admin)?,
    };

    let amount = match amount {
        Some(a) => a.u128(),
        None => get_commission_balance(&deps.storage),
    };

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        from_address: env.contract.address.clone(),
        to_address: address,
        amount: vec![Coin {
            denom: DENOM.to_string(),
            amount: Uint128(amount),
        }],
    }));

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::DrawCommission { status, msg, amount: Uint128(amount), address })?),
    })
}

// all user functions

pub fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
    description: Option<String>,
    img: Option<Binary>,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let description = description.unwrap_or_else(|| String::from(""));

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

                // if profile img sent store this as well
                if img.is_some() {
                    let img: Vec<u8> = img.unwrap().0;
                    if img.len() as u32 > constants.max_profile_img_size {
                        status = Failure;
                        msg = Some(String::from("Account registered, but profile image is too large."));
                    } else {
                        store_account_img(&mut deps.storage, &message_sender, img)?;
                    }
                }
            }
        }        
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Register { status, msg })?),
    })
}

pub fn try_set_handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
    let handle = handle.trim().to_owned();

    if handle.as_bytes().len() > constants.max_handle_len.into() {
        // if handle is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Handle is too long."));
    } else {
        match get_account_for_handle(&deps.storage, &handle) {
            Ok(_) => {
                status = Failure;
                msg = Some(String::from("Handle is already in use."))
            },
            Err(_) => {
                let message_sender = deps.api.canonical_address(&env.message.sender)?;
                let mut description = String::from("");

                // check if previously registered
                match get_account(&mut deps.storage, &message_sender) {
                    Ok(stored_account) => {
                        // yes, deactivate old handle if it is different
                        let account = stored_account.into_humanized(&deps.api)?;
                        let old_handle = account.handle;
                        description = account.description;
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
        data: Some(to_binary(&HandleAnswer::SetHandle { status, msg })?),
    })
}

pub fn try_set_description<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    description: String,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    if description.as_bytes().len() > constants.max_description_len.into() {
        // if description is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Description is too long."));
    } else {
        let message_sender = deps.api.canonical_address(&env.message.sender)?;
        match get_account(&mut deps.storage, &message_sender) {
            Ok(stored_account) => {
                let account = stored_account.into_humanized(&deps.api)?;
                let stored_account = Account {
                    owner: env.message.sender,
                    handle: account.handle,
                    description,
                }.into_stored(&deps.api)?;
                store_account(&mut deps.storage, stored_account, &message_sender)?;
            },
            _ => {
                status = Failure;
                msg = Some(String::from("Account has not been registered, yet."));
            }
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetDescription { status, msg })?),
    })
}

pub fn try_set_profile_img<S: Storage, A: Api, Q: Querier>(
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
        msg = Some(String::from("Profile image is too large."));
    } else {
        store_account_img(&mut deps.storage, &message_sender, img)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetProfileImg { status, msg })?),
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
    let mut status = Success;
    let mut msg = None;

    if key.as_bytes().len() < 8 {
        status = Failure;
        msg = Some(String::from("Key is too short."));
    } else {
        let vk = ViewingKey(key);

        let message_sender = deps.api.canonical_address(&env.message.sender)?;

        write_viewing_key(&mut deps.storage, &message_sender, &vk);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { 
            status,
            msg,
        })?),
    })
}

pub fn try_deactivate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deactivated: bool,
) -> StdResult<HandleResponse> {
    let mut status = Success;
    let mut msg = None;

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    store_account_deactivated(&mut deps.storage, &message_sender, deactivated)?;

    if deactivated {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Deactivate { status, msg })?),
        })
    } else {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Reactivate { status, msg })?),
        })
    }
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