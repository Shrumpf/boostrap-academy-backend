#[macro_export]
macro_rules! const_schema {
    ($($vis:vis $ident:ident($expr:expr));* $(;)*) => { $(
        $vis struct $ident;
        impl $ident {
            pub fn value() -> ::serde_json::Value { ($expr).into() }
        }
        impl ::schemars::JsonSchema for $ident {
            fn schema_name() -> ::std::string::String { ::core::stringify!($ident).into() }
            fn is_referenceable() -> ::core::primitive::bool { false }
            fn json_schema(_gen: &mut ::schemars::gen::SchemaGenerator) -> ::schemars::schema::Schema {
                ::schemars::schema::SchemaObject {
                    const_value: ::core::option::Option::Some(Self::value()),
                    ..::core::default::Default::default()
                }.into()
            }
        }
        impl ::serde::Serialize for $ident {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where S: ::serde::Serializer,
            { ::serde::Serialize::serialize(&Self::value(), serializer) }
        }
    )* };
}
