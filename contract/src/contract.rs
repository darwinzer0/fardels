use crate::exec::{
    try_approve_pending_unpacks, try_cancel_pending, try_carry_fardel, try_change_admin,
    try_comment_on_fardel, try_delete_comment, try_follow,
    try_generate_viewing_key, try_hide_fardel, try_rate_fardel, try_register, try_seal_fardel,
    try_set_constants, try_set_description, try_set_handle, try_set_private_settings,
    try_set_profile_img, try_set_view_settings, try_set_viewing_key, try_store_ban,
    try_store_block, try_store_deactivate, try_store_frozen_contract, try_unfollow,
    try_unhide_fardel, try_unpack_fardel, try_unrate_fardel, try_remove_fardel,
};
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::query::{
    query_get_comments, query_get_fardel_by_id, query_get_fardels, query_get_fardels_batch,
    query_get_followers, query_get_following, query_get_handle, query_get_pending_approvals,
    query_get_profile, query_get_purchase_transactions, query_get_rating,
    query_get_registered_addresses, query_get_sale_transactions, query_get_unpacked,
    query_is_following, query_is_handle_available, query_is_pending_unpack, query_get_profile_by_index,
};
use crate::state::{is_frozen, Config, Constants, ReadonlyConfig};
use crate::user_state::{is_banned, read_viewing_key};
use crate::utils::space_pad;
use crate::validation::{
    valid_max_contents_data_len, valid_max_description_len, valid_max_handle_len,
    valid_max_number_of_tags, valid_max_private_settings_len, valid_max_public_message_len,
    valid_max_query_page_size, valid_max_tag_len, valid_max_thumbnail_img_size,
    valid_max_view_settings_len, valid_transaction_fee,
};
use crate::viewing_key::VIEWING_KEY_SIZE;
use cosmwasm_std::{
    Api, Env, Extern, HandleResponse, InitResponse, Querier, QueryResult, StdError, StdResult,
    Storage,
};
use secret_toolkit::crypto::sha_256;

/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

// maximum cost of a fardel in uscrt
pub const DEFAULT_MAX_COST: u128 = 5000000_u128;

pub const DENOM: &str = "uscrt";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let admin = deps
        .api
        .canonical_address(&(msg.admin.unwrap_or_else(|| env.message.sender)))?;

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);

    let max_cost = match msg.max_cost {
        Some(cost) => cost.u128(),
        None => DEFAULT_MAX_COST,
    };

    // admin settings
    let transaction_fee = valid_transaction_fee(msg.transaction_fee)?.into_stored()?;
    let max_query_page_size = valid_max_query_page_size(msg.max_query_page_size)?;

    // fardel settings
    let max_public_message_len = valid_max_public_message_len(msg.max_public_message_len)?;
    let max_tag_len = valid_max_tag_len(msg.max_tag_len)?;
    let max_number_of_tags = valid_max_number_of_tags(msg.max_number_of_tags)?;
    let max_fardel_img_size = valid_max_thumbnail_img_size(msg.max_fardel_img_size)?;
    let max_contents_data_len = valid_max_contents_data_len(msg.max_contents_data_len)?;

    // user settings
    let max_handle_len = valid_max_handle_len(msg.max_handle_len)?;
    let max_description_len = valid_max_description_len(msg.max_description_len)?;
    let max_view_settings_len = valid_max_view_settings_len(msg.max_view_settings_len)?;
    let max_private_settings_len = valid_max_private_settings_len(msg.max_private_settings_len)?;
    let max_profile_img_size = valid_max_thumbnail_img_size(msg.max_profile_img_size)?;

    let mut config = Config::from_storage(&mut deps.storage);
    config.set_constants(&Constants {
        admin,
        transaction_fee,
        max_query_page_size,
        max_cost,
        max_public_message_len,
        max_tag_len,
        max_number_of_tags,
        max_fardel_img_size,
        max_contents_data_len,
        max_handle_len,
        max_description_len,
        max_view_settings_len,
        max_private_settings_len,
        max_profile_img_size,
        prng_seed: prng_seed_hashed.to_vec(),
    })?;

    Ok(InitResponse::default())
}

fn pad_response(response: StdResult<HandleResponse>) -> StdResult<HandleResponse> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    // permission check to make sure not banned (admin cannot be banned accidentally)
    let constants = Config::from_storage(&mut deps.storage).constants()?;
    let sender = deps.api.canonical_address(&env.message.sender)?;
    if (sender != constants.admin)
        && (is_banned(&deps.storage, &sender) || is_frozen(&deps.storage))
    {
        return Err(StdError::unauthorized());
    }

    let response = match msg {
        // Admin
        HandleMsg::SetConstants {
            transaction_fee,
            max_query_page_size,
            max_cost,
            max_public_message_len,
            max_tag_len,
            max_number_of_tags,
            max_fardel_img_size,
            max_contents_data_len,
            max_handle_len,
            max_profile_img_size,
            max_description_len,
            ..
        } => try_set_constants(
            deps,
            env,
            transaction_fee,
            max_query_page_size,
            max_cost,
            max_public_message_len,
            max_tag_len,
            max_number_of_tags,
            max_fardel_img_size,
            max_contents_data_len,
            max_handle_len,
            max_profile_img_size,
            max_description_len,
        ),
        HandleMsg::ChangeAdmin { admin, .. } => try_change_admin(deps, env, admin),
        HandleMsg::FreezeContract { .. } => try_store_frozen_contract(deps, env, true),
        HandleMsg::UnfreezeContract { .. } => try_store_frozen_contract(deps, env, false),
        HandleMsg::Ban {
            handle, address, ..
        } => try_store_ban(deps, env, handle, address, true),
        HandleMsg::Unban {
            handle, address, ..
        } => try_store_ban(deps, env, handle, address, false),
        HandleMsg::RemoveFardel {
            fardel_id, ..
        } => try_remove_fardel(deps, env, fardel_id, true),
        HandleMsg::UnremoveFardel {
            fardel_id, ..
        } => try_remove_fardel(deps, env, fardel_id, false),

        // Account
        HandleMsg::Register {
            handle,
            description,
            view_settings,
            private_settings,
            img,
            entropy,
            ..
        } => try_register(
            deps,
            env,
            handle,
            description,
            view_settings,
            private_settings,
            img,
            entropy,
        ),
        HandleMsg::SetHandle { handle, .. } => try_set_handle(deps, env, handle),
        HandleMsg::SetDescription { description, .. } => {
            try_set_description(deps, env, description)
        }
        HandleMsg::SetViewSettings { view_settings, .. } => {
            try_set_view_settings(deps, env, view_settings)
        }
        HandleMsg::SetPrivateSettings {
            private_settings, ..
        } => try_set_private_settings(deps, env, private_settings),
        HandleMsg::SetProfileImg { img, .. } => try_set_profile_img(deps, env, img),
        HandleMsg::GenerateViewingKey { entropy, .. } => {
            try_generate_viewing_key(deps, env, entropy)
        }
        HandleMsg::SetViewingKey { key, .. } => try_set_viewing_key(deps, env, key),
        HandleMsg::Deactivate { .. } => try_store_deactivate(deps, env, true),
        HandleMsg::Reactivate { .. } => try_store_deactivate(deps, env, false),

        // Other accounts
        HandleMsg::Block { handle, .. } => try_store_block(deps, env, handle, true),
        HandleMsg::Unblock { handle, .. } => try_store_block(deps, env, handle, false),
        HandleMsg::Follow { handle, .. } => try_follow(deps, env, handle),
        HandleMsg::Unfollow { handle, .. } => try_unfollow(deps, env, handle),

        // My fardels
        HandleMsg::CarryFardel {
            public_message,
            tags,
            contents_data,
            cost,
            countable,
            approval_req,
            img,
            seal_time,
            ..
        } => try_carry_fardel(
            deps,
            env,
            public_message,
            tags,
            contents_data,
            cost,
            countable,
            approval_req,
            img,
            seal_time,
        ),
        HandleMsg::SealFardel { fardel_id, .. } => try_seal_fardel(deps, env, fardel_id),
        HandleMsg::HideFardel { fardel_id, .. } => try_hide_fardel(deps, env, fardel_id),
        HandleMsg::UnhideFardel { fardel_id, .. } => try_unhide_fardel(deps, env, fardel_id),
        HandleMsg::ApprovePendingUnpacks { number, .. } => {
            try_approve_pending_unpacks(deps, env, number)
        }

        // Other fardels
        HandleMsg::UnpackFardel { fardel_id, .. } => try_unpack_fardel(deps, env, fardel_id),
        HandleMsg::CancelPending { fardel_id, .. } => try_cancel_pending(deps, env, fardel_id),
        HandleMsg::RateFardel {
            fardel_id, rating, ..
        } => try_rate_fardel(deps, env, fardel_id, rating),
        HandleMsg::UnrateFardel { fardel_id, .. } => try_unrate_fardel(deps, env, fardel_id),
        HandleMsg::CommentOnFardel {
            fardel_id,
            comment,
            rating,
            ..
        } => try_comment_on_fardel(deps, env, fardel_id, comment, rating),
        HandleMsg::DeleteComment {
            fardel_id,
            comment_id,
            ..
        } => try_delete_comment(deps, env, fardel_id, comment_id),
    };

    pad_response(response)
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetProfile { handle } => query_get_profile(deps, handle),
        QueryMsg::GetProfileByIndex { idx } => query_get_profile_by_index(deps, idx),
        QueryMsg::IsHandleAvailable { handle } => query_is_handle_available(deps, handle),
        QueryMsg::GetFardelById { fardel_id } => query_get_fardel_by_id(deps, &None, fardel_id),
        QueryMsg::GetFardels {
            handle,
            page,
            page_size,
        } => query_get_fardels(deps, &None, handle, page, page_size),
        QueryMsg::GetComments {
            fardel_id,
            page,
            page_size,
        } => query_get_comments(deps, &None, fardel_id, page, page_size),
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
            // permission check to make sure not banned (admin cannot be banned accidentally)
            let constants = ReadonlyConfig::from_storage(&deps.storage).constants()?;
            if (canonical_addr != constants.admin) && (is_banned(&deps.storage, &canonical_addr)) {
                return Err(StdError::unauthorized());
            }

            return match msg {
                // Base
                QueryMsg::GetSaleTransactions {
                    address,
                    page,
                    page_size,
                    ..
                } => query_get_sale_transactions(&deps, &address, page, page_size),
                QueryMsg::GetPurchaseTransactions {
                    address,
                    page,
                    page_size,
                    ..
                } => query_get_purchase_transactions(&deps, &address, page, page_size),
                QueryMsg::GetHandle { address, .. } => query_get_handle(&deps, &address),
                QueryMsg::GetFollowing {
                    address,
                    page,
                    page_size,
                    ..
                } => query_get_following(&deps, &address, page, page_size),
                QueryMsg::IsFollowing {
                    address, handle, ..
                } => query_is_following(&deps, &address, handle),
                QueryMsg::GetFollowers {
                    address,
                    page,
                    page_size,
                    ..
                } => query_get_followers(&deps, &address, page, page_size),
                QueryMsg::GetFardelByIdAuth {
                    address, fardel_id, ..
                } => query_get_fardel_by_id(&deps, &Some(address), fardel_id),
                QueryMsg::GetFardelsAuth {
                    address,
                    handle,
                    page,
                    page_size,
                    ..
                } => query_get_fardels(&deps, &Some(address), handle, page, page_size),
                QueryMsg::IsPendingUnpack {
                    address, fardel_id, ..
                } => query_is_pending_unpack(&deps, &address, fardel_id),
                QueryMsg::GetUnpacked {
                    address,
                    page,
                    page_size,
                    ..
                } => query_get_unpacked(&deps, &address, page, page_size),
                QueryMsg::GetPendingApprovals {
                    address, number, ..
                } => query_get_pending_approvals(&deps, &address, number),
                QueryMsg::GetCommentsAuth {
                    address,
                    fardel_id,
                    page,
                    page_size,
                    ..
                } => query_get_comments(&deps, &Some(address), fardel_id, page, page_size),
                QueryMsg::GetRating {
                    address, fardel_id, ..
                } => query_get_rating(&deps, &address, fardel_id),
                QueryMsg::GetFardelsBatch {
                    address,
                    start,
                    count,
                    ..
                } => query_get_fardels_batch(&deps, &address, start, count),
                QueryMsg::GetRegisteredAddresses {
                    address,
                    start,
                    count,
                    ..
                } => query_get_registered_addresses(&deps, &address, start, count),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Err(StdError::unauthorized())
}
