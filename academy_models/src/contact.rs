use email_address::EmailAddress;
use nutype::nutype;

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

#[nutype(
    validate(len_char_max = 256),
    derive(Debug, Clone, PartialEq, Eq, TryFrom, Deref, Serialize, Deserialize)
)]
pub struct ContactMessageAuthorName(String);

#[nutype(
    validate(len_char_max = 256),
    derive(Debug, Clone, PartialEq, Eq, TryFrom, Deref, Serialize, Deserialize)
)]
pub struct ContactMessageSubject(String);

#[nutype(
    validate(len_char_max = 4096),
    derive(Debug, Clone, PartialEq, Eq, TryFrom, Deref, Serialize, Deserialize)
)]
pub struct ContactMessageContent(String);
