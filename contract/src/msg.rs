use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, HumanAddr, Uint128, StdResult};
use crate::viewing_key::ViewingKey;
use crate::state::{StoredFee, SaleTx, PurchaseTx};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub transaction_fee: Option<Fee>,

    // query settings
    pub max_query_page_size: Option<i32>,

    // fardel (public) settings
    pub max_cost: Option<Uint128>,
    pub max_public_message_len: Option<i32>,
    pub max_tag_len: Option<i32>,
    pub max_number_of_tags: Option<i32>,
    pub max_fardel_img_size: Option<i32>,

    // fardel (private) settings
    pub max_contents_data_len: Option<i32>,
 
    // user data settings
    pub max_handle_len: Option<i32>,
    pub max_profile_img_size: Option<i32>,
    pub max_description_len: Option<i32>,

    pub prng_seed: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Admin-only functions
    SetConstants {
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
        padding: Option<String>,
    },
    ChangeAdmin {
        admin: HumanAddr,
        padding: Option<String>,
    },
    Ban {
        handle: Option<String>,
        address: Option<HumanAddr>,
        padding: Option<String>,
    },
    Unban {
        handle: Option<String>,
        address: Option<HumanAddr>,
        padding: Option<String>,
    },
    DrawCommission {
        address: Option<HumanAddr>,
        amount: Option<Uint128>,
        padding: Option<String>,
    },

    // Account
    Register { 
        handle: String,
        description: Option<String>,
        img: Option<String>,
        padding: Option<String>,
    },
    SetHandle {
        handle: String,
        padding: Option<String>,
    },
    SetDescription {
        description: String,
        padding: Option<String>,
    },
    SetProfileImg {
        img: String,
        padding: Option<String>,
    },
    GenerateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    Deactivate { 
        padding: Option<String>, 
    },
    Reactivate {
        padding: Option<String>,
    },

    // Other accounts
    Block {
        handle: String,
        padding: Option<String>,
    },
    Unblock {
        handle: String,
        padding: Option<String>,
    },
    Follow {
        handle: String,
        padding: Option<String>,
    },
    Unfollow {
        handle: String,
        padding: Option<String>,
    },

    // My Fardels
    CarryFardel { 
        /// public_message is the message that is visible prior to unpacking
        public_message: String,

        /// tags is a vector of labels used to facilitate search
        tags: Vec<String>,

        /// contents_data is a vector of private data (JSON strings) only accessible upon unpacking
        contents_data: Vec<String>,

        /// cost in uscrt
        cost: Uint128,

        /// countable determines how data will be unpacked
        ///   false: No limit on number of sales. If false, then contents_data length must be 1
        ///   true:  Each element of contents_data vec can only be unpacked by one account.
        ///          Once all the contents are unpacked the fardel will be sealed.
        countable: bool,

        /// approval_req means each unpacking requires approval before the transaction is completed
        approval_req: bool,

        /// img is a public thumbnail image that goes with the public_message
        img: Option<String>,

        /// seal_time sets an automatic timestamp for when the fardel will seal
        seal_time: Option<i32>,
        padding: Option<String>,
    },
    /// seals a fardel so no one can unpack it anymore
    ///   Once a fardel has been sealed it cannot be unsealed. 
    ///   If the owner of the fardel wants to make the contents 
    ///   available again, then they need to carry a new fardel.
    SealFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    /// approves the unpacking of a set number of pending fardels, 
    ///   and processes transactions.
    ApprovePendingUnpacks {
        number: Option<i32>,
        padding: Option<String>,
    },

    // Other fardels

    // If the fardel requires approval it will be pending, 
    //   otherwise it will unpack and process transaction immediately.
    UnpackFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    // Cancels a pending unpacking and returns scrt to sender
    CancelPending {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    // Rates a fardel, rating values are defined as follows:
    //   false - downvote
    //   true - upvote
    // The ratings are public, but only accounts that have unpacked the fardel can rate it. 
    // An account can only upvote or downvote a fardel once. 
    RateFardel {
        fardel_id: Uint128,
        rating: bool,
        padding: Option<String>,
    },
    // Removes a rating on a fardel
    UnrateFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    // creates a new comment on a fardel with option to send rating at same time
    CommentOnFardel {
        fardel_id: Uint128,
        comment: String,
        rating: Option<bool>,
        padding: Option<String>,
    },
    // deletes a comment
    DeleteComment {
        fardel_id: Uint128,
        comment_id: i32,
        padding: Option<String>,
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // Admin
    SetConstants {
        status: ResponseStatus,
    },
    ChangeAdmin {
        status: ResponseStatus,
    },
    DrawCommission {
        status: ResponseStatus,
        address: HumanAddr,
        amount: Uint128,
        msg: Option<String>,
    },
    Ban {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Unban {
        status: ResponseStatus,
        msg: Option<String>,
    },

    // Account
    Register {
        status: ResponseStatus,
        msg: Option<String>,
    },
    SetHandle {
        status: ResponseStatus,
        msg: Option<String>,
    },
    SetDescription {
        status: ResponseStatus,
        msg: Option<String>,
    },
    SetProfileImg {
        status: ResponseStatus,
        msg: Option<String>,
    },
    GenerateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Deactivate {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Reactivate {
        status: ResponseStatus,
        msg: Option<String>,
    },

    // Other accounts
    Block {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Unblock {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Follow {
        status: ResponseStatus,
    },
    Unfollow {
        status: ResponseStatus,
    },

    // My Fardels
    CarryFardel {
        status: ResponseStatus,
        msg: Option<String>,
        fardel_id: Option<Uint128>,
    },
    SealFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    ApprovePendingUnpacks {
        status: ResponseStatus,
        msg: Option<String>,
    },

    // Other Fardels
    UnpackFardel {
        status: ResponseStatus,
        msg: Option<String>,
        contents_data: Option<String>,
    },
    CancelPending {
        status: ResponseStatus,
        msg: Option<String>,
    },
    RateFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    UnrateFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    CommentOnFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    DeleteComment {
        status: ResponseStatus,
        msg: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    //
    // Queries without authentication
    //

    // User queries
    // Get the profile for a given handle (description, profile img)
    GetProfile {
        handle: String,
    },
    // Check if the given handle is available
    IsHandleAvailable {
        handle: String,
    },
    // Get a fardel by hash id, not logged in
    GetFardelById {
        fardel_id: Uint128,
    },
    // Get fardels for a given handle, not logged in
    GetFardels {
        handle: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get paginated list of comments for the given fardel
    GetComments {
        fardel_id: Uint128,
        page: Option<i32>,
        page_size: Option<i32>,
    },

    //
    // Queries requiring authentication (viewing key)
    //

    // Get historical transaction data
    GetSaleTransactions {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    GetPurchaseTransactions {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get the handle for the currently logged in user
    GetHandle {
        address: HumanAddr,
        key: String,
    },
    // Get logged in user's list of handles they are currently following
    GetFollowing {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get whether the logged in user is following a given handle
    IsFollowing {
        address: HumanAddr,
        key: String,
        handle: String,
    },
    // Get logged in user's list of followers
    GetFollowers {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get a fardel by hash id, as a logged in user (with unpacked private data)
    GetFardelByIdAuth {
        address: HumanAddr,
        key: String,
        fardel_id: Uint128,
    },
    // Get fardels for a given handle, as a logged in user (with unpacked private data)
    GetFardelsAuth {
        address: HumanAddr,
        key: String,
        handle: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get paginated list of fardels that logged in user has unpacked
    GetUnpacked {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get information about pending unpacks for the currently logged in user's fardels
    GetPendingUnpacks {
        address: HumanAddr,
        key: String,
        number: Option<i32>,
    },
    // Get paginated list of comments for the given fardel, as a logged in user
    GetCommentsAuth {
        address: HumanAddr,
        key: String,
        fardel_id: Uint128,
        page: Option<i32>,
        page_size: Option<i32>,
    },

    //
    // Queries requiring authentication, admin user only
    //

    // Admin-only batch get fardels by global id
    GetFardelsBatch {
        // must match admin
        address: HumanAddr,
        key: String,
        start: Option<Uint128>,
        count: Option<Uint128>,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetSaleTransactions { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetPurchaseTransactions { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetHandle { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::GetFollowing { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::IsFollowing { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFollowers { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFardelByIdAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFardelsAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetUnpacked { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetPendingUnpacks { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetCommentsAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFardelsBatch { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CommentResponse {
    pub text: String,
    pub handle: String,
    // comment id and fardel id is only set if the authenticated user 
    //   is the commenter (enables deletion)
    pub fardel_id: Option<Uint128>,
    pub comment_id: Option<i32>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct FardelResponse {
    pub id: Uint128,
    pub public_message: String,
    pub cost: Uint128,
    pub unpacked: bool,
    pub sealed: bool,
    pub tags: Vec<String>,
    // returns only latest comments, use GetComments for older comments
    pub comments: Vec<CommentResponse>,
    // total number of comments
    pub number_of_comments: i32,
    pub upvotes: i32,
    pub downvotes: i32,
    pub timestamp: i32,
    pub img: String,
    pub seal_time: Option<i32>,
    // unpacked parts
    pub contents_data: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingUnpackResponse {
    pub handle: String,
    pub fardel_id: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetProfile {
        status: ResponseStatus,
        handle: Option<String>,
        description: Option<String>,
        img: Option<String>,
        follower_count: i32,
    },
    IsHandleAvailable {
        response: bool,
    },
    GetFardelById {
        fardel: FardelResponse,
    },
    GetFardels {
        fardels: Vec<FardelResponse>,
        total_count: i32,
    },
    GetComments {
        comments: Vec<CommentResponse>,
    },

    GetSaleTransactions {
        txs: Vec<SaleTx>,
    },
    GetPurchaseTransactions {
        txs: Vec<PurchaseTx>,
    },
    GetHandle {
        status: ResponseStatus,
        handle: Option<String>,
    },
    GetFollowing {
        following: Vec<String>,
        total_count: i32,
    },
    IsFollowing {
        response: bool,
    },
    GetFollowers {
        followers: Vec<String>,
        total_count: i32,
    },
    GetUnpacked {
        fardels: Vec<FardelResponse>,
        total_count: i32,
    },
    GetPendingUnpacks {
        pending: Vec<PendingUnpackResponse>,
    },
    GetFardelsBatch {
        fardels: Vec<FardelResponse>,
    },

    // Authentication error
    ViewingKeyError {
        msg: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub commission_rate_nom: Uint128,
    pub commission_rate_denom: Uint128,
}

impl Fee {
    pub fn into_stored(self) -> StdResult<StoredFee> {
        let fee = StoredFee {
            commission_rate_nom: self.commission_rate_nom.u128(),
            commission_rate_denom: self.commission_rate_denom.u128(),
        };
        Ok(fee)
    }
}