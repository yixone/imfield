#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(pub(crate) String);

impl Url {
    pub fn new(buff: String) -> Self {
        Self(buff)
    }

    // TODO!
}
