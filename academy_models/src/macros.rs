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

pub(crate) use id;
