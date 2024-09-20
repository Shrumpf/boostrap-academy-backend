use nutype::nutype;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaginationSlice {
    pub limit: PaginationLimit,
    pub offset: u64,
}

#[nutype(
    validate(less_or_equal = PaginationLimit::MAX),
    derive(Debug, Clone, Copy, PartialEq, Eq, Deref, TryFrom, Serialize, Deserialize, JsonSchema)
)]
pub struct PaginationLimit(u64);

impl PaginationLimit {
    pub const MAX: u64 = 100;

    pub fn max() -> Self {
        Self::try_new(Self::MAX).unwrap()
    }
}

impl Default for PaginationLimit {
    fn default() -> Self {
        Self::max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_limit_default() {
        PaginationLimit::default();
    }
}
