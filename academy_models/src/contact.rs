use email_address::EmailAddress;
use nutype::nutype;

use crate::macros::nutype_string;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactMessage {
    pub author: ContactMessageAuthor,
    pub subject: ContactMessageSubject,
    pub content: ContactMessageContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactMessageAuthor {
    pub name: ContactMessageAuthorName,
    pub email: EmailAddress,
}

nutype_string!(ContactMessageAuthorName(validate(
    len_char_min = 1,
    len_char_max = 256
)));

nutype_string!(ContactMessageSubject(validate(
    len_char_min = 1,
    len_char_max = 256
)));

nutype_string!(ContactMessageContent(validate(
    len_char_min = 1,
    len_char_max = 4096
)));
