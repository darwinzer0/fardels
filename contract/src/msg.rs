use crate::state::StoredFee;
use crate::tx_state::{PurchaseTx, SaleTx};
use crate::viewing_key::ViewingKey;
use cosmwasm_std::{Binary, HumanAddr, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub max_view_settings_len: Option<i32>,
    pub max_private_settings_len: Option<i32>,

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
    // Disables the ability for non-admin users to execute handle functions, essentially making it read-only
    FreezeContract {
        padding: Option<String>,
    },
    // Unfreezes the contract
    UnfreezeContract {
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
    // admin function to remove a fardel
    RemoveFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    UnremoveFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },

    // Account
    Register {
        // can also be used to change full profile for already registered accounts
        handle: String,
        description: Option<String>,
        view_settings: Option<String>,
        private_settings: Option<String>,
        img: Option<String>,
        // optionally generate viewing key, if entropy is sent
        entropy: Option<String>,
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
    SetViewSettings {
        view_settings: String,
        padding: Option<String>,
    },
    SetPrivateSettings {
        private_settings: String,
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

        /// contents_data is private data (UI expects JSON string) only accessible upon unpacking
        contents_data: String,

        /// cost in uscrt
        cost: Uint128,

        /// countable determines maximum number of times a fardel can be unpacked
        ///   None: No limit on number of sales.
        countable: Option<i32>,

        /// approval_req means each unpacking requires approval before the transaction is completed
        approval_req: bool,

        /// img is a public thumbnail image that goes with the public_message
        img: Option<String>,

        /// seal_time sets an automatic timestamp for when the fardel will seal
        seal_time: Option<i32>,
        padding: Option<String>,
    },
    /// Seals a fardel so no one can unpack it anymore
    ///   Once a fardel has been sealed it cannot be unsealed.
    ///   If the owner of the fardel wants to make the contents
    ///   available again, then they need to carry a new fardel.
    SealFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    /// Hides a fardel so that it will not be returned by GetFardels or GetFardelsById
    ///   Also, it cannot be unpacked, but it is still available to people who have already
    ///   unpacked it.
    HideFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    /// unhides a previously hidden fardel
    UnhideFardel {
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
    },
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
        msg: String,
    },
    FreezeContract {
        status: ResponseStatus,
    },
    UnfreezeContract {
        status: ResponseStatus,
    },
    Ban {
        status: ResponseStatus,
        msg: Option<String>,
    },
    Unban {
        status: ResponseStatus,
        msg: Option<String>,
    },
    RemoveFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    UnremoveFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },

    // Account
    Register {
        status: ResponseStatus,
        key: Option<ViewingKey>,
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
    SetViewSettings {
        status: ResponseStatus,
        msg: Option<String>,
    },
    SetPrivateSettings {
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
    HideFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    UnhideFardel {
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
    // Get the public profile for a given handle (description, profile img)
    GetProfile {
        handle: String,
    },
    // Get a profile by number
    GetProfileByIndex {
        idx: i32,
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
    // Get the handle and private settings for the currently logged in user
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
    // Returns bool saying whether the given fardel is a pending unpack for the logged in user
    IsPendingUnpack {
        address: HumanAddr,
        key: String,
        fardel_id: Uint128,
    },
    // Get paginated list of fardels that logged in user has unpacked
    GetUnpacked {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    // Get information about pending unpacks needing approval by the currently logged in user
    GetPendingApprovals {
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
    // Gets whether the logged in user has rated (upvoted or downvoted) the given fardel
    // returns the rating (true: upvote, false: downvote, None: no vote)
    GetRating {
        address: HumanAddr,
        key: String,
        fardel_id: Uint128,
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
    GetRegisteredAddresses {
        // must match admin
        address: HumanAddr,
        key: String,
        start: Option<i32>,
        count: Option<i32>,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetSaleTransactions { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::GetPurchaseTransactions { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::GetHandle { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::GetFollowing { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::IsFollowing { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFollowers { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFardelByIdAuth { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::GetFardelsAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::IsPendingUnpack { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetUnpacked { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetPendingApprovals { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
            Self::GetCommentsAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetRating { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            // Admin functions
            Self::GetFardelsBatch { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetRegisteredAddresses { address, key, .. } => {
                (vec![address], ViewingKey(key.clone()))
            }
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
    pub carrier: String,
    pub public_message: String,
    pub cost: Uint128,
    pub pending_unpack: bool,
    pub unpacked: bool,
    pub sealed: bool,
    pub tags: Vec<String>,
    // total number of comments
    pub number_of_comments: i32,
    pub upvotes: i32,
    pub downvotes: i32,
    pub timestamp: i32,
    pub img: String,
    pub seal_time: Option<i32>,
    // if countable returns the number left for sale
    pub remaining: Option<i32>,
    // unpacked parts
    pub contents_data: Option<String>,
    // user's current rating of this fardel if there is one
    pub rating: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingApprovalResponse {
    pub handle: String,
    pub fardel_id: Uint128,
    pub canceled: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct FardelBatchResponse {
    pub global_id: Uint128,
    pub hash_id: Uint128,
    pub owner: HumanAddr,
    pub handle: String,
    pub public_message: String,
    pub cost: Uint128,
    pub sealed: bool,
    pub tags: Vec<String>,
    // total number of comments
    pub number_of_comments: i32,
    pub upvotes: i32,
    pub downvotes: i32,
    pub timestamp: i32,
    pub img: String,
    pub seal_time: Option<i32>,
    pub remaining: Option<i32>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct RegisteredAccountsResponse {
    pub address: HumanAddr,
    pub handle: Option<String>,
    pub banned: bool,
    pub deactivated: bool,
    pub description: Option<String>,
    pub view_settings: Option<String>,
    pub img: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetProfile {
        status: ResponseStatus,
        handle: Option<String>,
        description: Option<String>,
        view_settings: Option<String>,
        img: Option<String>,
        follower_count: i32,
    },
    GetProfileByIndex {
        status: ResponseStatus,
        handle: Option<String>,
        description: Option<String>,
        view_settings: Option<String>,
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
    GetRating {
        rating: Option<bool>,
    },

    GetSaleTransactions {
        txs: Vec<SaleTx>,
        total_count: i32,
    },
    GetPurchaseTransactions {
        txs: Vec<PurchaseTx>,
        total_count: i32,
    },
    GetHandle {
        status: ResponseStatus,
        handle: Option<String>,
        private_settings: Option<String>,
    },
    GetFollowing {
        following: Vec<String>,
        // for pagination
        total_count: i32,
    },
    IsFollowing {
        response: bool,
    },
    GetFollowers {
        followers: Vec<String>,
        // for pagination
        total_count: i32,
    },
    IsPendingUnpack {
        response: bool,
    },
    GetUnpacked {
        fardels: Vec<FardelResponse>,
        total_count: i32,
    },
    GetPendingApprovals {
        pending: Vec<PendingApprovalResponse>,
    },
    GetFardelsBatch {
        fardels: Vec<FardelBatchResponse>,
    },
    GetRegisteredAccounts {
        accounts: Vec<RegisteredAccountsResponse>,
        total_registered: i32,
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
