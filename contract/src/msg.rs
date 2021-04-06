use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use crate::viewing_key::ViewingKey;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,

    //fardel
    pub max_public_message_len: Option<i32>,
    pub max_thumbnail_img_size: Option<i32>,
    pub max_contents_text_len: Option<i32>,
    pub max_cost: Option<Uint128>,
    // new
    pub max_contents_data_len: Option<i32>,
    // del
    pub max_ipfs_cid_len: Option<i32>,
    pub max_contents_passphrase_len: Option<i32>,
 
    // user
    pub max_handle_len: Option<i32>,
    pub max_profile_img_size: Option<i32>,
    pub max_description_len: Option<i32>,


    pub prng_seed: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Account
    Register { 
        handle: String,
        description: String,
        profile_ing: Option<String>,
        padding: Option<String>,
    },
    SetProfileThumbnailImg {
        img: Binary,
    },
    //RegisterAndGenerateViewingKey {
    //    handle: String,
    //    entropy: String,
    //    padding: Option<String>,
    //},
    //RegisterAndSetViewingKey {
    //    handle: String,
    //    key: String,
    //    padding: Option<String>,
    //},

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

    // My Fardels
    CarryFardel { 
        public_message: String,
        contents_text: String,
        ipfs_cid: String,
        passphrase: String,
        cost: Uint128,
        padding: Option<String>,
    },
    SealFardel {
        fardel_id: Uint128,
    },

    // Other fardels
    Follow {
        handle: String,
        padding: Option<String>,
    },
    Unfollow {
        handle: String,
        padding: Option<String>,
    },
    UnpackFardel {
        fardel_id: Uint128,
        padding: Option<String>,
    },
    RateFardel {
        fardel_id: Uint128,
        rating: bool,
        padding: Option<String>,
    },
    CommentOnFardel {
        fardel_id: Uint128,
        comment: String,
        rating: Option<bool>,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // Account
    Register {
        status: ResponseStatus,
        msg: Option<String>,
    },
    SetProfileThumbnailImg {
        status: ResponseStatus,
        msg: Option<String>,
    },
    //RegisterAndGenerateViewingKey {
    //    status: ResponseStatus,
    //    key: Option<ViewingKey>,
    //},
    //RegisterAndSetViewingKey {
    //    status: ResponseStatus,
    //},
    GenerateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
    },
    Deactivate {
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

    // Other Fardels
    Follow {
        status: ResponseStatus,
    },
    Unfollow {
        status: ResponseStatus,
    },
    UnpackFardel {
        status: ResponseStatus,
        msg: Option<String>,
        contents_text: Option<String>,
        ipfs_cid: Option<String>,
        passphrase: Option<String>,
    },
    RateFardel {
        status: ResponseStatus,
        msg: Option<String>,
    },
    CommentOnFardel {
        status: ResponseStatus,
        msg: Option<String>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProfile {
        handle: String,
    },
    GetHandle {
        address: HumanAddr,
        key: String,
    },
    GetFollowing {
        address: HumanAddr,
        key: String,
    },
    IsHandleAvailable {
        handle: String,
    },
    GetFardelById {
        fardel_id: Uint128,
    },
    GetFardelByIdAuth {
        address: HumanAddr,
        key: String,
        fardel_id: Uint128,
    },
    // Get fardels for a given handle
    GetFardels {
        handle: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    GetFardelsAuth {
        address: HumanAddr,
        key: String,
        handle: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
    GetUnpacked {
        address: HumanAddr,
        key: String,
        page: Option<i32>,
        page_size: Option<i32>,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetHandle { address, key } => (vec![address], ViewingKey(key.clone())),
            Self::GetFollowing { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetFardelsAuth { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            Self::GetUnpacked { address, key, .. } => (vec![address], ViewingKey(key.clone())),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct FardelResponse {
    pub id: Uint128,
    pub public_message: String,
    pub cost: Uint128,
    pub packed: bool,
    pub has_ipfs_cid: bool,
    pub comments: Vec<String>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub timestamp: i32,
    // unpacked parts
    pub contents_text: Option<String>,
    pub ipfs_cid: Option<String>,
    pub passphrase: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetProfile {
        status: ResponseStatus,
        handle: Option<String>,
        description: Option<String>,
        img: Option<Binary>,
    },
    GetHandle {
        status: ResponseStatus,
        handle: Option<String>,
    },
    GetFollowing {
        following: Vec<String>,
    },
    IsHandleAvailable {
        response: bool,
    },
    GetFardelById {
        fardel: FardelResponse,
    },
    GetFardels {
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