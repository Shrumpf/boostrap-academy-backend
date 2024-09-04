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
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, TryFrom, Serialize, Deserialize,
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
                Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, From, Serialize, Deserialize,
                $($($derive)*)?
            ),
            $(default = $default,)?
        )]
        pub struct $ident(String);
    };
}

pub(crate) use id;
pub(crate) use nutype_string;
