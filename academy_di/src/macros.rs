#[macro_export]
macro_rules! provider {
    ($vis:vis $ident:ident {
        $( $field:ident: $ty:ty, )*
        $( .. $bfield:ident: $base:ty { $($ity:ty,)* $(,)? } )*
    }) => {
        $vis struct $ident {
            _state: $crate::ProviderState,
            $( $field: $ty, )*
            $( $bfield: $base, )*
        }

        impl $crate::Provider for $ident {
            fn state(&mut self) -> &mut $crate::ProviderState {
                &mut self._state
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
                    $crate::Provides::provide(&mut provider.$bfield)
                }
            }
        )*)*
    };
}
