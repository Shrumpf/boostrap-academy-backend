use std::fmt::Debug;

use academy_di::Build;
use academy_shared_contracts::id::IdService;
use academy_utils::trace_instrument;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Build)]
pub struct IdServiceImpl;

impl IdService for IdServiceImpl {
    #[trace_instrument(skip(self))]
    fn generate<I: From<Uuid> + Debug>(&self) -> I {
        Uuid::new_v4().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate() {
        // Arrange
        let sut = IdServiceImpl;

        // Act
        let id1 = sut.generate::<Uuid>();
        let id2 = sut.generate::<Uuid>();

        // Assert
        assert_ne!(id1, id2);
    }
}
