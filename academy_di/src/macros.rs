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
            fn cache(&mut self) -> &mut $crate::TypeMap {
                &mut self._cache
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
