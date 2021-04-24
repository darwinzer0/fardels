use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse,
    InitResponse, Querier, 
    StdResult, Storage, QueryResult, 
};
use secret_toolkit::crypto::sha_256;
use crate::exec::{
    try_register, try_set_profile_img, try_generate_viewing_key,
    try_set_viewing_key, try_deactivate, try_carry_fardel, try_seal_fardel,
    try_follow, try_unfollow, try_rate_fardel, try_comment_on_fardel,
    try_unpack_fardel,
};
use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, QueryAnswer,
};
use crate::query::{
    query_get_fardel_by_id, query_get_fardel_by_id_auth, query_get_fardels,
    query_get_fardels_auth, query_get_following, query_get_handle,
    query_get_profile, query_is_handle_available,
};
use crate::state::{
    Config, Constants, read_viewing_key,
};
use crate::validation::{
    valid_max_public_message_len, valid_max_thumbnail_img_size, valid_max_contents_data_len, 
    valid_max_handle_len, valid_max_tag_len, valid_max_number_of_tags,
    valid_max_description_len, valid_max_query_page_size,
};
use crate::viewing_key::{VIEWING_KEY_SIZE};

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
    let admin = msg.admin.unwrap_or_else(|| env.message.sender);

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);

    let max_cost = match msg.max_cost{
        Some(cost) => cost.u128(),
        None => DEFAULT_MAX_COST
    };

    // admin settings
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
    let max_profile_img_size = valid_max_thumbnail_img_size(msg.max_profile_img_size)?;

    let mut config = Config::from_storage(&mut deps.storage);
    config.set_constants(&Constants {
        admin,
        max_cost,
        max_public_message_len,
        max_tag_len,
        max_number_of_tags,
        max_fardel_img_size,
        max_contents_data_len,
        max_handle_len,
        max_description_len,
        max_profile_img_size,
        prng_seed: prng_seed_hashed.to_vec(),
    })?;

    Ok(InitResponse::default())
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
        HandleMsg::SetProfileImg { img, .. } =>
          try_set_profile_img(deps, env, img),
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
        QueryMsg::GetComments { fardel_id, page, page_size } =>
            query_get_comments(deps, fardel_id, page, page_size),
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
                QueryMsg::GetTransactions { address, page, page_size, .. } => 
                    query_get_transactions(&deps, &address, page, page_size),
                QueryMsg::GetHandle { address, .. } => 
                    query_get_handle(&deps, &address),
                QueryMsg::GetFollowing { address, .. } => 
                    query_get_following(&deps, &address),
                QueryMsg::GetFardelByIdAuth { address, fardel_id, .. } => 
                    query_get_fardel_by_id_auth(&deps, &address, fardel_id),
                QueryMsg::GetFardelsAuth { address, handle, page, page_size, .. } =>
                    query_get_fardels_auth(&deps, &address, handle, page, page_size),
                QueryMsg::GetUnpacked { address, page, page_size, .. } =>
                    query_get_unpacked(&deps, &address, page, page_size),
                QueryMsg::GetCommentsAuth { address, fardel_id, page, page_size, .. } =>
                    query_get_comments_auth(&deps, &address, fardel_id, page, page_size),
                QueryMsg::GetFardelsBatch { address, page, page_size, .. } =>
                    query_get_fardels_batch(&deps, &address, page, page_size),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Ok(to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })?)
}



