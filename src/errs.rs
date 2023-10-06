use core::fmt::{Debug, Display};

/// Error type of `TryFrom<[(K, V); CAP]>`
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct DuplicateKeysError<K>(
    /// The first duplicate key found
    pub K,
);

impl<K: Debug> Display for DuplicateKeysError<K> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "duplicate key: {:#?}", self.0)
    }
}
