pub mod patch;
pub mod required_action;
pub mod serde;

pub trait Apply {
    /// Applies the function `f` with a mutable reference to `self`.
    fn with<X>(mut self, f: impl FnOnce(&mut Self) -> X) -> Self
    where
        Self: Sized,
    {
        f(&mut self);
        self
    }

    /// Applies the function `f` only if `value` is `Some(...)` and provides the
    /// contained value to `f`.
    ///
    /// #### Example
    /// ```rust
    /// # use academy_utils::Apply;
    /// fn add_option(a: i32, b: Option<i32>) -> i32 {
    ///     a.apply_map(b, |slf, arg| slf + arg)
    /// }
    /// assert_eq!(add_option(1, None), 1);
    /// assert_eq!(add_option(1, Some(2)), 3);
    /// ```
    fn apply_map<U>(self, value: Option<U>, f: impl FnOnce(Self, U) -> Self) -> Self
    where
        Self: Sized,
    {
        if let Some(value) = value {
            f(self, value)
        } else {
            self
        }
    }
}

impl<T> Apply for T {}

#[macro_export]
macro_rules! assert_matches {
    ($expr:expr, $pat:pat) => {
        match ($expr) {
            $pat => (),
            val => ::core::panic!(
                "Assertion failed: Value {val:?} did not match pattern {}",
                ::core::stringify!($pat)
            ),
        }
    };
    ($expr:expr, $pat:pat if $pred:expr) => {{
        let val = $expr;
        match (&val) {
            $pat if $pred => (),
            #[allow(unused_variables)]
            $pat => ::core::panic!(
                "Assertion failed: Value {val:?} does not match predicate {}",
                ::core::stringify!($pred)
            ),
            _ => ::core::panic!(
                "Assertion failed: Value {val:?} did not match pattern {}",
                ::core::stringify!($pat)
            ),
        }
    }};
}
