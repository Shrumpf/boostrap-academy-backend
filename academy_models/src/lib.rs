use std::sync::LazyLock;

use macros::nutype_string;
use nutype::nutype;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub mod auth;
pub mod contact;
mod macros;
pub mod mfa;
pub mod oauth2;
pub mod pagination;
pub mod session;
pub mod user;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sha256Hash(#[serde(with = "academy_utils::serde::hex")] pub [u8; 32]);

nutype_string!(SearchTerm(validate(len_char_max = 256)));

nutype_string!(VerificationCode(
    sanitize(uppercase),
    validate(regex = VERIFICATION_CODE_REGEX),
));

pub static VERIFICATION_CODE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    hyphenated_code_regex(VerificationCode::CHUNK_COUNT, VerificationCode::CHUNK_SIZE)
});

impl VerificationCode {
    pub const CHUNK_COUNT: usize = 4;
    pub const CHUNK_SIZE: usize = 4;
}

fn hyphenated_code_regex(chunk_count: usize, chunk_size: usize) -> Regex {
    Regex::new(&format!(
        "^([A-Z0-9]{{{0}}}-){{{1}}}[A-Z0-9]{{{0}}}$",
        chunk_size,
        chunk_count - 1
    ))
    .unwrap()
}

nutype_string!(RecaptchaResponse(validate(len_char_max = 256)));
