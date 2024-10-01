macro_rules! id {
    ($ident:ident) => {
        #[::nutype::nutype(derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Deref,
            From,
            Serialize,
            Deserialize,
        ))]
        pub struct $ident(::uuid::Uuid);

        impl ::schemars::JsonSchema for $ident {
            fn schema_name() -> ::std::string::String {
                ::core::stringify!($ident).into()
            }

            fn json_schema(
                gen: &mut ::schemars::gen::SchemaGenerator,
            ) -> ::schemars::schema::Schema {
                <::uuid::Uuid as ::schemars::JsonSchema>::json_schema(gen)
            }
        }
    };
}

macro_rules! sha256hash {
    ($ident:ident) => {
        #[::nutype::nutype(derive(
            Debug,
            Display,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Deref,
            From,
            Serialize,
            Deserialize
        ))]
        pub struct $ident($crate::Sha256Hash);
    };
}

macro_rules! nutype_string {
    ($ident:ident) => {
        $crate::macros::nutype_string!($ident());
    };
    ($ident:ident(
        $(sanitize( $($sanitize:tt)* ) $(,)? )?
        validate( $($validate:tt)* ) $(,)?
        $(derive( $($derive:tt)* ) $(,)? )?
        $(default = $default:expr $(,)? )?
    )) => {
        #[nutype(
            $(sanitize($($sanitize)*),)?
            validate($($validate)*),
            derive(
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, TryFrom, Serialize, Deserialize, JsonSchema,
                $($($derive)*)?
            ),
            $(default = $default,)?
        )]
        pub struct $ident(String);
    };
    ($ident:ident(
        $(sanitize( $($sanitize:tt)* ) $(,)? )?
        $(derive( $($derive:tt)* ) $(,)? )?
        $(default = $default:expr $(,)? )?
    )) => {
        #[nutype(
            $(sanitize($($sanitize)*),)?
            derive(
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Serialize, Deserialize, JsonSchema,
                $($($derive)*)?
            ),
            $(default = $default,)?
        )]
        pub struct $ident(String);
    };
}

pub(crate) use id;
pub(crate) use nutype_string;
pub(crate) use sha256hash;
