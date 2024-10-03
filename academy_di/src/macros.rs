#[macro_export]
macro_rules! provider {
    ($(#[doc=$doc:literal])* $vis:vis $ident:ident {
        $( $field:ident: $ty:ty, )*
        $( .. $bfield:ident: $base:ty { $($ity:ty,)* $(,)? } )*
    }) => {
        $(#[doc=$doc])*
        $vis struct $ident {
            _cache: $crate::TypeMap,
            $( $field: $ty, )*
            $( $bfield: $base, )*
        }

        impl $crate::Provider for $ident {
            fn get<T: 'static + Clone>(&self) -> Option<T> {
                self._cache.get().cloned()
            }
            fn insert<T: 'static>(&mut self, value: T) {
                self._cache.insert(value)
            }
        }

        $(
            impl $crate::Build<$ident> for $ty {
                fn build(provider: &mut $ident) -> Self {
                    ::core::clone::Clone::clone(&provider.$field)
                }
            }
        )*

        $($(
            impl $crate::Build<$ident> for $ity {
                fn build(provider: &mut $ident) -> Self {
                    $crate::Provide::provide(&mut provider.$bfield)
                }
            }
        )*)*
    };
}
