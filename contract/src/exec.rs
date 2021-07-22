use cosmwasm_std::{
    to_binary, Api, Coin, Env, Extern, HandleResponse, Querier, 
    CosmosMsg, BankMsg, HumanAddr, CanonicalAddr,
    StdError, StdResult, Storage, Uint128
};
use twox_hash::xxh3::hash128_with_seed;
use primitive_types::U256;
use crate::u256_math::*;
use crate::msg::{
    HandleAnswer, ResponseStatus, 
    ResponseStatus::Success, ResponseStatus::Failure, Fee,
};
use crate::state::{Config, ReadonlyConfig,
    set_frozen,
    Account, get_account, get_account_for_handle, map_handle_to_account, delete_handle_map,
    store_account, store_account_img, store_account_ban, store_account_block,
    Fardel, get_fardel_by_hash, get_fardel_by_id, get_fardel_owner, seal_fardel, store_fardel, 
    get_fardel_next_package, store_fardel_next_package, store_pending_unpack,
    get_global_id_by_hash, get_total_fardel_count, store_fardel_img, 
    store_following, remove_following,
    store_account_deactivated,
    PendingUnpack, cancel_pending_unpack, get_pending_unpacked_status_by_fardel_id,
    get_unpacked_status_by_fardel_id, get_sealed_status, store_unpack, 
    get_pending_unpacks_from_start, get_pending_start, set_pending_start,
    append_sale_tx, append_purchase_tx,
    has_rated, set_rated, get_rating, remove_rated, 
    subtract_upvote_fardel, subtract_downvote_fardel,
    add_upvote_fardel, add_downvote_fardel, 
    comment_on_fardel, delete_comment, get_comment_by_id,
    write_viewing_key, get_commission_balance,
    is_blocked_by,
};
use crate::validation::{
    valid_max_public_message_len, valid_max_thumbnail_img_size, valid_max_contents_data_len, 
    valid_max_handle_len, valid_max_tag_len, valid_max_number_of_tags,
    valid_max_description_len, valid_max_query_page_size, valid_seal_time,
    has_whitespace,
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
        constants.transaction_fee = transaction_fee.unwrap().into_stored()?;
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
    config.set_constants(&constants)?;

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

    config.set_constants(&constants)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?),
    })
}

pub fn try_freeze_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);
    let constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    set_frozen(&mut deps.storage, true)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::FreezeContract { status: Success })?),
    })
}

pub fn try_unfreeze_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);
    let constants = config.constants()?;

    // permission check
    if deps.api.canonical_address(&env.message.sender)? != constants.admin {
        return Err(StdError::unauthorized());
    }

    set_frozen(&mut deps.storage, false)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UnfreezeContract { status: Success })?),
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

    let config = Config::from_storage(&mut deps.storage);
    let constants = config.constants()?;

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
        )?;
    } else if handle.is_some() { // otherwise use handle
        let account = get_account_for_handle(&deps.storage, &handle.unwrap())?;
        store_account_ban(
            &mut deps.storage,
            &account,
            banned
        )?;
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
    let status = Success;
    let msg = None;

    let config = Config::from_storage(&mut deps.storage);
    let constants = config.constants()?;

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
        to_address: address.clone(),
        amount: vec![Coin {
            denom: DENOM.to_string(),
            amount: Uint128(amount),
        }],
    }));

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::DrawCommission { status, msg, amount: Uint128(amount), address: address.clone() })?),
    })
}

// all user functions

pub fn try_register<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
    description: Option<String>,
    view_settings: Option<String>,
    private_settings: Option<String>,
    img: Option<String>,
    entropy: Option<String>,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let mut key: Option<ViewingKey> = None;
    let description = description.unwrap_or_else(|| String::from(""));
    let view_settings = view_settings.unwrap_or_else(|| String::from(""));
    let private_settings = private_settings.unwrap_or_else(|| String::from(""));

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
    let handle = handle.trim().to_owned();

    if handle.as_bytes().len() > constants.max_handle_len.into() || has_whitespace(&handle) {
        // if handle is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Handle is too long or has whitespace."));
    } else if description.as_bytes().len() > constants.max_description_len.into() {
        // if description is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Description is too long."));
    } else if view_settings.as_bytes().len() > constants.max_view_settings_len.into() {
        status = Failure;
        msg = Some(String::from("View settings are too long."));
    } else if private_settings.as_bytes().len() > constants.max_private_settings_len.into() {
        status = Failure;
        msg = Some(String::from("Private settings are too long."));
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
                        // yes, delete old handle if it is different
                        let account = stored_account.into_humanized(&deps.api)?;
                        let old_handle = account.handle;
                        if !handle.eq(&old_handle) {
                            delete_handle_map(&mut deps.storage, old_handle);
                        }
                    },
                    _ => { }
                }
                let stored_account = Account {
                    owner: env.message.sender.clone(),
                    handle: handle.clone(),
                    description,
                    view_settings,
                    private_settings,
                }.into_stored(&deps.api)?;
                map_handle_to_account(&mut deps.storage, &message_sender, handle.clone())?;
                store_account(&mut deps.storage, stored_account, &message_sender)?;

                // if profile img sent store this as well
                if img.is_some() {
                    let img: Vec<u8> = img.unwrap().as_bytes().to_vec();
                    if img.len() as u32 > constants.max_fardel_img_size {
                        status = Failure;
                        msg = Some(String::from("Account registered, but profile image is too large."));
                    } else {
                        store_account_img(&mut deps.storage, &message_sender, img)?;
                    }
                }

                // if entropy was sent then generate and return a viewing key as well
                if entropy.is_some() {
                    let prng_seed = constants.prng_seed;
                    let viewing_key = ViewingKey::new(&env, &prng_seed, (&entropy.unwrap()).as_ref());
                    write_viewing_key(&mut deps.storage, &message_sender, &viewing_key);
                    key = Some(viewing_key);
                }
            }
        }        
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Register { status, key, msg })?),
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

    if handle.as_bytes().len() > constants.max_handle_len.into() || has_whitespace(&handle) {
        // if handle is too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Handle is too long or has whitespace."));
    } else {
        match get_account_for_handle(&deps.storage, &handle) {
            Ok(_) => {
                status = Failure;
                msg = Some(String::from("Handle is already in use."))
            },
            Err(_) => {
                let message_sender = deps.api.canonical_address(&env.message.sender)?;
                let mut description = String::from("");
                let mut view_settings = String::from("");
                let mut private_settings = String::from("");

                // check if previously registered
                match get_account(&mut deps.storage, &message_sender) {
                    Ok(stored_account) => {
                        // yes, delete old handle if it is different
                        let account = stored_account.into_humanized(&deps.api)?;
                        let old_handle = account.handle;
                        description = account.description;
                        view_settings = account.view_settings;
                        private_settings = account.private_settings;
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
                    view_settings,
                    private_settings,
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
                    view_settings: account.view_settings,
                    private_settings: account.private_settings,
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

pub fn try_set_view_settings<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    view_settings: String,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    if view_settings.as_bytes().len() > constants.max_view_settings_len.into() {
        // if view settings are too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("View settings are too long."));
    } else {
        let message_sender = deps.api.canonical_address(&env.message.sender)?;
        match get_account(&mut deps.storage, &message_sender) {
            Ok(stored_account) => {
                let account = stored_account.into_humanized(&deps.api)?;
                let stored_account = Account {
                    owner: env.message.sender,
                    handle: account.handle,
                    description: account.description,
                    view_settings,
                    private_settings: account.private_settings,
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
        data: Some(to_binary(&HandleAnswer::SetViewSettings { status, msg })?),
    })
}

pub fn try_set_private_settings<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    private_settings: String,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;

    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    if private_settings.as_bytes().len() > constants.max_private_settings_len.into() {
        // if private settings are too long, set status message and do nothing else
        status = Failure;
        msg = Some(String::from("Private settings are too long."));
    } else {
        let message_sender = deps.api.canonical_address(&env.message.sender)?;
        match get_account(&mut deps.storage, &message_sender) {
            Ok(stored_account) => {
                let account = stored_account.into_humanized(&deps.api)?;
                let stored_account = Account {
                    owner: env.message.sender,
                    handle: account.handle,
                    description: account.description,
                    view_settings: account.view_settings,
                    private_settings,
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
        data: Some(to_binary(&HandleAnswer::SetPrivateSettings { status, msg })?),
    })
}

pub fn try_set_profile_img<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    img: String,
) -> StdResult<HandleResponse> {
    let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let img: Vec<u8> = img.as_bytes().to_vec();
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

pub fn try_store_deactivate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deactivated: bool,
) -> StdResult<HandleResponse> {
    let status = Success;
    let msg = None;

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

pub fn try_store_block<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
    block: bool,
) -> StdResult<HandleResponse> {
    let mut status = Success;
    let mut msg = None;

    let blocker = deps.api.canonical_address(&env.message.sender)?;
    match get_account_for_handle(&deps.storage, &handle) {
        Ok(blocked) => {
            store_account_block(&mut deps.storage, &blocker, &blocked, block)?;
        },
        _ => {
            status = Failure;
            msg = Some(String::from("Handle not in use."));
        },
    }

    if block {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Block { status, msg })?),
        })
    } else {
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&HandleAnswer::Unblock { status, msg })?),
        })
    }
}

pub fn try_follow<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    handle: String,
) -> StdResult<HandleResponse> {
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    // TODO: check if blocked
    store_following(&mut deps.storage, &message_sender, handle)?;

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
    remove_following(&mut deps.storage, &message_sender, handle)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Unfollow { 
            status: Success,
        })?),
    })
}

// carry a new fardel to the network
pub fn try_carry_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    public_message: String,
    tags: Vec<String>,
    contents_data: Vec<String>,
    cost: Uint128,
    countable: bool,
    approval_req: bool,
    img: Option<String>,
    seal_time: Option<i32>,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let mut fardel_id: Option<Uint128> = None;

    let config = ReadonlyConfig::from_storage(&deps.storage);
    let constants = config.constants()?;

    let mut tag_size_ok = true;
    for tag in tags.clone() {
        if tag.as_bytes().len() > constants.max_tag_len.into() {
            tag_size_ok = false;
            break;
        }
    }

    let mut img_size_ok = true;
    // if fardel img sent, check size
    if img.is_some() {
        let img_vec: Vec<u8> = img.clone().unwrap().as_bytes().to_vec();
        if img_vec.len() as u32 > constants.max_fardel_img_size {
            img_size_ok = false;
        } 
    }

    let contents_data_size = contents_data.iter().fold(0_usize, |acc, x| acc + x.as_bytes().len());

    if !tag_size_ok ||
       !img_size_ok ||
       (public_message.as_bytes().len() > constants.max_public_message_len.into()) ||
       (tags.len() > constants.max_number_of_tags.into()) || 
       (contents_data_size > constants.max_contents_data_len.into()) ||
       (cost.u128() > constants.max_cost) ||
       (contents_data.len() == 0) {
        status = Failure;
        msg = Some(String::from("Invalid fardel data"));
    } else if !countable && contents_data.len() != 1 {
        // non-countable fardels can only have one package
        status = Failure;
        msg = Some(String::from("Invalid fardel data: non-countable fardels can only have one package"));
    } else {
        let stored_seal_time = valid_seal_time(seal_time)?;

        let message_sender = deps.api.canonical_address(&env.message.sender)?;

        // generate fardel hash id using xx3h

        let hash_data_len = 8 + 16 + 16 + env.message.sender.len() + public_message.as_bytes().len();
        let mut hash_data = Vec::with_capacity(hash_data_len);
        hash_data.extend_from_slice(&env.block.height.to_be_bytes());
        hash_data.extend_from_slice(&cost.u128().to_be_bytes());
        hash_data.extend_from_slice(&get_total_fardel_count(&deps.storage).to_be_bytes());
        hash_data.extend_from_slice(&env.message.sender.0.as_bytes());
        hash_data.extend_from_slice(&public_message.as_bytes());
        let hash_id = hash128_with_seed(&hash_data, env.block.time);

        // make sure unique (probably overkill!)
        // TODO? 50% chance of collision with 19 quintillion fardels :)

        let fardel = Fardel {
            // global_id will be overwritten in store_fardel, just a placeholder
            global_id: Uint128(0),
            hash_id: Uint128(hash_id),
            public_message,
            tags,
            contents_data,
            cost: Coin {
                amount: cost,
                denom: DENOM.to_string(),
            },
            countable,
            approval_req,
            seal_time: stored_seal_time,
            timestamp: env.block.time,
        }.into_stored()?;
    
        let global_id = store_fardel(
            &mut deps.storage, fardel.hash_id, &message_sender, 
            fardel.public_message, fardel.tags, fardel.contents_data, 
            fardel.cost, fardel.countable, fardel.approval_req, 
            fardel.seal_time, fardel.timestamp,
        )?;
        // if fardel img sent, then store it as well
        if img.is_some() {
            store_fardel_img(&mut deps.storage, global_id, img.unwrap().as_bytes().to_vec())?;
        }
        fardel_id = Some(Uint128(fardel.hash_id));
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

    match get_fardel_by_hash(&deps.storage, fardel_id) {
        Ok(_) => {
            let global_id = get_global_id_by_hash(&deps.storage, fardel_id)?;
            let owner = deps.api.human_address(&get_fardel_owner(&deps.storage, global_id)?)?;
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

pub fn try_approve_pending_unpacks<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    number: Option<i32>,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let number = number.unwrap_or_else(|| 10_i32);

    let owner = deps.api.canonical_address(&env.message.sender.clone())?;
    let mut messages: Vec<CosmosMsg> = vec![];

    if number < 1 {
        status = Failure;
        msg = Some(String::from("invalid number of unpacks to approve"));
    } else {
        let pending_unpacks = get_pending_unpacks_from_start(&deps.storage, &owner, number as u32)?;
        let new_idx: u32 = get_pending_start(&deps.storage, &owner) + pending_unpacks.len() as u32;
        let pending_unpacks: Vec<PendingUnpack> = pending_unpacks.into_iter().filter(|pu| !pu.canceled).collect();

        let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
        let mut total_commission: u128 = 0;

        for pending_unpack in pending_unpacks {
            // complete the unpack
            store_unpack(
                &mut deps.storage, 
                &pending_unpack.unpacker, 
                pending_unpack.fardel_id, 
                pending_unpack.package_idx,
            )?;

            // handle the transaction
            let cost = pending_unpack.coin.amount.u128();

            // commission_amount = cost * commission_rate_nom / commission_rate_denom
            let cost_u256 = Some(U256::from(cost));
            let commission_rate_nom = Some(U256::from(constants.transaction_fee.commission_rate_nom));
            let commission_rate_denom = Some(U256::from(constants.transaction_fee.commission_rate_denom));
            let commission_amount = div(
                mul(cost_u256, commission_rate_nom),
                commission_rate_denom,
            ).ok_or_else(|| {
                StdError::generic_err(format!(
                    "Cannot calculate cost {} * commission_rate_nom {} / commission_rate_denom {}",
                    cost_u256.unwrap(),
                    commission_rate_nom.unwrap(),
                    commission_rate_denom.unwrap(),
                ))
            })?;
    
            let payment_amount = sub(cost_u256, Some(commission_amount)).ok_or_else(|| {
                StdError::generic_err(format!(
                    "Cannot calculate cost {} - commission_amount {}",
                    cost_u256.unwrap(),
                    commission_amount,
                ))
            })?;
    
            // push payment
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: env.message.sender.clone(),
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(payment_amount.low_u128()),
                }],
            }));
    
            // sum commission
            total_commission += commission_amount.low_u128();
        }
        
        // update the start idx for pending unpacks
        set_pending_start(&mut deps.storage, &owner, new_idx)?;

        if total_commission > 0 {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: deps.api.human_address(&constants.admin)?,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(total_commission),
                }],
            }));
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ApprovePendingUnpacks {
            status,
            msg,
        })?),
    })
}

pub fn try_unpack_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut pending: bool = false;
    let mut msg: Option<String> = None;
    let mut contents_data: Option<String> = None;
    let mut cost: u128 = 0;
    let mut canonical_owner: Option<CanonicalAddr> = None;
    let mut package_id = 0;

    // fardel id from hash
    let fardel_id = fardel_id.u128();
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let sent_coins = env.message.sent_funds.clone();
    if sent_coins[0].denom != DENOM {
        status = Failure;
        msg = Some(String::from("Wrong denomination."))
    } else {
        let fardel = get_fardel_by_hash(&deps.storage, fardel_id);
        match fardel {
            Ok(fardel) => {
                match fardel {
                    Some(f) => {
                        let global_id = f.global_id.u128();

                        // 1. Check if sender is blocked by fardel owner
                        let owner = get_fardel_owner(&deps.storage, global_id)?;
                        if is_blocked_by(&deps.storage, &owner, &message_sender) {
                            return Err(StdError::unauthorized());
                        }

                        cost = f.cost.amount.u128();
                        let sent_amount: u128 = sent_coins[0].amount.u128();
                        let next_package = get_fardel_next_package(&deps.storage, global_id).unwrap_or_else(|_| 0_u16);
                        let total = f.contents_data.len() as u16;
                        let num_packages_left = 0_u16.max(total - next_package);

                        // 2. check it has not already been unpacked by the user
                        if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, global_id).unpacked {
                            status = Failure;
                            msg = Some(String::from("You have already unpacked this fardel."));
                        // 3. check not pending
                        } else if get_pending_unpacked_status_by_fardel_id(&deps.storage, &message_sender, global_id).value {
                            status = Failure;
                            msg = Some(String::from("You have a currently pending unpack for this fardel."));
                        // 4. check it is not sealed
                        } else if get_sealed_status(&deps.storage, global_id) {
                            status = Failure;
                            msg = Some(String::from("Fardel has been sealed."));
                        // 5. check it has not expired, 0 seal_time means never expires
                        } else if f.seal_time > 0 && f.seal_time < env.block.time {
                            seal_fardel(&mut deps.storage, global_id)?;
                            status = Failure;
                            msg = Some(String::from("Fardel has been sealed."));
                        // 6. check that countable packages have not been all unpacked
                        } else if f.countable && num_packages_left == 0 {
                            seal_fardel(&mut deps.storage, global_id)?;
                            status = Failure;
                            msg = Some(String::from("Fardel has been sealed."));
                        // 7. check cost is correct
                        } else if sent_amount != cost {
                            status = Failure;
                            msg = Some(String::from("Didn't send correct amount of coins to unpack."));
                        } else {
                            package_id = next_package.clone();
                            if f.approval_req {
                                // do a pending unpack
                                store_pending_unpack(
                                    &mut deps.storage, 
                                    &owner, 
                                    &message_sender, 
                                    global_id, 
                                    next_package,
                                    env.message.sent_funds[0].clone(),
                                    env.block.time,
                                )?;
                                pending = true;
                                msg = Some(String::from("Fardel unpack is pending approval by owner."));
                            } else {
                                store_unpack(&mut deps.storage, &message_sender, global_id, next_package)?;
                                contents_data = Some(f.contents_data[next_package as usize].clone());
                            }

                            // both pending and completed unpacks use up a countable package
                            if f.countable {
                                store_fardel_next_package(&mut deps.storage, global_id, next_package + 1)?;
                            }
                            canonical_owner = Some(owner);
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

    if pending {
        // have contract hold on to the coin
    } else if status == Success {
        let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;

        // commission_amount = cost * commission_rate_nom / commission_rate_denom
        let cost_u256 = Some(U256::from(cost));
        let commission_rate_nom = Some(U256::from(constants.transaction_fee.commission_rate_nom));
        let commission_rate_denom = Some(U256::from(constants.transaction_fee.commission_rate_denom));
        let commission_amount = div(
            mul(cost_u256, commission_rate_nom),
            commission_rate_denom,
        ).ok_or_else(|| {
            StdError::generic_err(format!(
                "Cannot calculate cost {} * commission_rate_nom {} / commission_rate_denom {}",
                cost_u256.unwrap(),
                commission_rate_nom.unwrap(),
                commission_rate_denom.unwrap(),
            ))
        })?;

        let payment_amount = sub(cost_u256, Some(commission_amount)).ok_or_else(|| {
            StdError::generic_err(format!(
                "Cannot calculate cost {} - commission_amount {}",
                cost_u256.unwrap(),
                commission_amount,
            ))
        })?;

        // push payment
        let fardel_owner = deps.api.human_address(&canonical_owner.clone().unwrap())?;
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: fardel_owner,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: Uint128(payment_amount.low_u128()),
            }],
        }));

        // push commission
        let commission_amount = commission_amount.low_u128();
        if commission_amount > 0 {
            messages.push(CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: deps.api.human_address(&constants.admin)?,
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128(commission_amount),
                }],
            }));
        }

        append_sale_tx(&mut deps.storage, canonical_owner.clone().unwrap(), message_sender.clone(), fardel_id, package_id.into(), cost, commission_amount, env.block.time)?;
        append_purchase_tx(&mut deps.storage, canonical_owner.unwrap(), message_sender, fardel_id, package_id.into(), cost, commission_amount, env.block.time)?;
    } else { // return coins to sender if there was a Failure
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: env.message.sender,
            amount: env.message.sent_funds,
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UnpackFardel { 
            status,
            msg,
            contents_data,
        })?),
    })
}

pub fn try_cancel_pending<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
) -> StdResult<HandleResponse> {
    let mut status = Success;
    let mut msg: Option<String> = None;
    let mut refund = Uint128(0);
    let mut messages: Vec<CosmosMsg> = vec![];

    let fardel_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;
    let owner = get_fardel_owner(&deps.storage, fardel_id)?;
    match get_fardel_by_id(&deps.storage, fardel_id)? {
        Some(fardel) => {
            refund = fardel.cost.amount;
        },
        None => {
            status = Failure;
            msg = Some(String::from("No fardel with that id found."));
        },
    }

    let unpacker = deps.api.canonical_address(&env.message.sender)?;
    cancel_pending_unpack(&mut deps.storage, &owner, &unpacker, fardel_id)?;

    if status == Success {
        // return coins to sender
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address.clone(),
            to_address: env.message.sender,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: refund,
            }],
        }));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CancelPending { 
            status,
            msg,
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
    let fardel_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;

    // check that fardel has been unpacked by the user
    if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, fardel_id).unpacked {
        // check if user has already rated it
        if has_rated(&deps.storage, &message_sender, fardel_id) {
            status = Failure;
            msg = Some(String::from("Cannot rate a fardel more than once."));
        } else if rating {
            set_rated(&mut deps.storage, &message_sender, fardel_id, true)?;
            add_upvote_fardel(&mut deps.storage, fardel_id)?;
        } else {
            set_rated(&mut deps.storage, &message_sender, fardel_id, false)?;
            add_downvote_fardel(&mut deps.storage, fardel_id)?;
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

pub fn try_unrate_fardel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    let fardel_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;

    match get_rating(&deps.storage, &message_sender, fardel_id) {
        Ok(rating) => {
            remove_rated(&mut deps.storage, &message_sender, fardel_id);
            if rating {
                subtract_upvote_fardel(&mut deps.storage, fardel_id)?;
            } else {
                subtract_downvote_fardel(&mut deps.storage, fardel_id)?;
            }
        },
        _ => {
            status = Failure;
            msg = Some(String::from("Cannot unrate a fardel that you have not rated."));    
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UnrateFardel { 
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
    let fardel_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;

    if get_unpacked_status_by_fardel_id(&deps.storage, &message_sender, fardel_id).unpacked {
        // fardel has been unpacked by the user
        // add comment
        comment_on_fardel(&mut deps.storage, &message_sender, fardel_id, comment)?;

        // handle rating if it is here
        match rating {
            Some(r) => {
                if has_rated(&deps.storage, &message_sender, fardel_id) {
                    status = Failure;
                    msg = Some(String::from("Comment left but cannot rate a fardel more than once."));
                } else if r {
                    set_rated(&mut deps.storage, &message_sender, fardel_id, true)?;
                    add_upvote_fardel(&mut deps.storage, fardel_id)?;
                } else {
                    set_rated(&mut deps.storage, &message_sender, fardel_id, false)?;
                    add_downvote_fardel(&mut deps.storage, fardel_id)?;
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

pub fn try_delete_comment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    fardel_id: Uint128,
    comment_id: i32,
) -> StdResult<HandleResponse> {
    let mut status: ResponseStatus = Success;
    let mut msg: Option<String> = None;
    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    let fardel_id = get_global_id_by_hash(&deps.storage, fardel_id.u128())?;

    if comment_id < 0 {
        status = Failure;
        msg = Some(String::from("invalid comment_id"));
    } else {
        let comment = get_comment_by_id(&deps.storage, fardel_id, comment_id as u32)?;
        if comment.commenter == message_sender {
            delete_comment(&mut deps.storage, fardel_id, comment_id as u32)?;
        } else {
            status = Failure;
            msg = Some(String::from("cannot delete another user's comment"));
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::DeleteComment { 
            status, msg
        })?),
    })
}