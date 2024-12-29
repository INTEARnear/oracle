use serde::Serialize;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Copy, Serialize)]
pub struct H256(pub [u8; 32]);

impl From<[u8; 32]> for H256 {
    fn from(bytes: [u8; 32]) -> Self {
        H256(bytes)
    }
}

impl TryFrom<Vec<u8>> for H256 {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(H256(value.try_into().map_err(|_| "Invalid hex length")?))
    }
}
