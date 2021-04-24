use std::convert::TryFrom;
use cosmwasm_std::{
    StdError, StdResult,
};

pub const DEFAULT_MAX_QUERY_PAGE_SIZE: u16 = 10_u16;
pub const DEFAULT_MAX_PUBLIC_MESSAGE_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_THUMBNAIL_IMG_SIZE: u32 = 65536_u32;
pub const DEFAULT_MAX_CONTENTS_TEXT_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_IPFS_CID_LEN: u16 = 128_u16;
pub const DEFAULT_MAX_CONTENTS_PASSPHRASE_LEN: u16 = 64_u16;
pub const DEFAULT_MAX_HANDLE_LEN: u16 = 64_u16;
pub const DEFAULT_MAX_DESCRIPTION_LEN: u16 = 280_u16;

pub fn valid_max_query_page_size(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_query_page_size"))
            } else {
                u16::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_query_page_size")))
            }
        },
        None => Ok(DEFAULT_MAX_QUERY_PAGE_SIZE)
    }
}

// limit the max public message size to values in 1..65535, default 280 bytes
pub fn valid_max_public_message_len(val: Option<i32>) -> StdResult<u16> {
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
pub fn valid_max_thumbnail_img_size(val: Option<i32>) -> StdResult<u32> {
    match val {
        Some(v) => {
            u32::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_thumbnail_img_size")))
        },
        None => Ok(DEFAULT_MAX_THUMBNAIL_IMG_SIZE)
    }
}

// limit the max contents text to values in 1..65535, default 280 bytes
pub fn valid_max_contents_text_len(val: Option<i32>) -> StdResult<u16> {
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
pub fn valid_max_ipfs_cid_len(val: Option<i32>) -> StdResult<u16> {
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
pub fn valid_max_contents_passphrase_len(val: Option<i32>) -> StdResult<u16> {
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
pub fn valid_max_handle_len(val: Option<i32>) -> StdResult<u16> {
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
pub fn valid_max_description_len(val: Option<i32>) -> StdResult<u16> {
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