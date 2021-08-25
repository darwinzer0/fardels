use crate::msg::Fee;
use cosmwasm_std::Uint128;
use cosmwasm_std::{StdError, StdResult};
use std::convert::TryFrom;

pub const DEFAULT_TRANSACTION_FEE: Fee = Fee {
    commission_rate_nom: Uint128(3),
    commission_rate_denom: Uint128(1000),
};
pub const DEFAULT_MAX_QUERY_PAGE_SIZE: u16 = 10_u16;
pub const DEFAULT_MAX_PUBLIC_MESSAGE_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_TAG_LEN: u8 = 255_u8;
pub const DEFAULT_MAX_NUMBER_OF_TAGS: u8 = 20_u8;
pub const DEFAULT_MAX_THUMBNAIL_IMG_SIZE: u32 = 65536_u32;
pub const DEFAULT_MAX_CONTENTS_DATA_LEN: u16 = 1024_u16;
pub const DEFAULT_MAX_HANDLE_LEN: u16 = 64_u16;
pub const DEFAULT_MAX_DESCRIPTION_LEN: u16 = 280_u16;
pub const DEFAULT_MAX_VIEW_SETTINGS_LEN: u16 = 4096_u16;
pub const DEFAULT_MAX_PRIVATE_SETTINGS_LEN: u16 = 4096_u16;

pub fn valid_transaction_fee(val: Option<Fee>) -> StdResult<Fee> {
    match val {
        Some(v) => {
            if v.commission_rate_nom > v.commission_rate_denom {
                Err(StdError::generic_err("invalid fee, > 100%"))
            } else {
                Ok(v)
            }
        }
        None => Ok(DEFAULT_TRANSACTION_FEE),
    }
}

pub fn valid_max_query_page_size(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_query_page_size"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_query_page_size")))
            }
        }
        None => Ok(DEFAULT_MAX_QUERY_PAGE_SIZE),
    }
}

// limit the max public message size to values in 1..65535, default 280 bytes
pub fn valid_max_public_message_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_public_message_len"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_public_message_len")))
            }
        }
        None => Ok(DEFAULT_MAX_PUBLIC_MESSAGE_LEN),
    }
}

// limit the max tag len to values in 1..255, default 64 bytes
pub fn valid_max_tag_len(val: Option<i32>) -> StdResult<u8> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_tag_len"))
            } else {
                u8::try_from(v).or_else(|_| Err(StdError::generic_err("invalid max_tag_len")))
            }
        }
        None => Ok(DEFAULT_MAX_TAG_LEN),
    }
}

// limit the max number of tags per fardel to values in 1..255, default 10
pub fn valid_max_number_of_tags(val: Option<i32>) -> StdResult<u8> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid max_number_of_tags"))
            } else {
                u8::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_number_of_tags")))
            }
        }
        None => Ok(DEFAULT_MAX_NUMBER_OF_TAGS),
    }
}

// limit the max thumbnail img size in bytes to u32, default 64K
pub fn valid_max_thumbnail_img_size(val: Option<i32>) -> StdResult<u32> {
    match val {
        Some(v) => u32::try_from(v)
            .or_else(|_| Err(StdError::generic_err("invalid max_thumbnail_img_size"))),
        None => Ok(DEFAULT_MAX_THUMBNAIL_IMG_SIZE),
    }
}

// limit the max contents data to values in 1..65535, default 1024 bytes
pub fn valid_max_contents_data_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_contents_data_len"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_contents_data_len")))
            }
        }
        None => Ok(DEFAULT_MAX_CONTENTS_DATA_LEN),
    }
}

// limit the max handle length (in bytes) to values in 8..65535, default 64 bytes
pub fn valid_max_handle_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 8 {
                Err(StdError::generic_err("invalid_max_handle_length"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_handle_length")))
            }
        }
        None => Ok(DEFAULT_MAX_HANDLE_LEN),
    }
}

// limit the max description length (in bytes) to values in 1..65535, default 280 bytes
pub fn valid_max_description_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_description_length"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_description_length")))
            }
        }
        None => Ok(DEFAULT_MAX_DESCRIPTION_LEN),
    }
}

// limit the max view settings length (in bytes) to values in 1..65535, default 4096 bytes
pub fn valid_max_view_settings_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_view_settings_length"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_view_settings_length")))
            }
        }
        None => Ok(DEFAULT_MAX_VIEW_SETTINGS_LEN),
    }
}

// limit the max private settings length (in bytes) to values in 1..65535, default 4096 bytes
pub fn valid_max_private_settings_len(val: Option<i32>) -> StdResult<u16> {
    match val {
        Some(v) => {
            if v < 1 {
                Err(StdError::generic_err("invalid_max_private_settings_length"))
            } else {
                u16::try_from(v)
                    .or_else(|_| Err(StdError::generic_err("invalid max_private_settings_length")))
            }
        }
        None => Ok(DEFAULT_MAX_PRIVATE_SETTINGS_LEN),
    }
}

// check valid seal time for a fardel
pub fn valid_seal_time(val: Option<i32>) -> StdResult<u64> {
    match val {
        Some(v) => u64::try_from(v).or_else(|_| Err(StdError::generic_err("invalid seal_time"))),
        None => Ok(0_u64),
    }
}

pub fn has_whitespace(s: &String) -> bool {
    let mut string_copy = s.clone();
    string_copy.retain(|c| !c.is_whitespace());
    return string_copy.len() != s.len();
}
