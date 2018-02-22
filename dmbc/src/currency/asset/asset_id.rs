use std::error::Error;
use std::fmt;

use exonum::crypto::PublicKey;
use exonum::encoding;
use exonum::encoding::{Field, Offset, CheckedOffset};
use exonum::encoding::serialize::WriteBufferWrapper;
use exonum::encoding::serialize::json::ExonumJson;
use exonum::storage::StorageKey;
use uuid;
use uuid::Uuid;
use serde_json;

pub const ASSET_ID_LEN: usize = 16;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AssetId(pub [u8; ASSET_ID_LEN]);

impl AssetId {
    pub fn zero() -> AssetId {
        AssetId([0; 16])
    }

    /// Creates unique `AssetId` from
    /// `&str` and `&PublicKey`
    /// # Example:
    /// ```
    /// # extern crate exonum;
    /// # extern crate dmbc;
    /// #
    /// # fn main () {
    /// #
    /// # use exonum::crypto::{PublicKey};
    /// # use exonum::encoding::serialize::FromHex;
    /// # use dmbc::currency::asset::AssetId;
    ///
    /// let data = "a8d5c97d-9978-4b0b-9947-7a95dcb31d0f";
    /// let public_key = PublicKey::from_hex("3115dbc2ff73f4819672d5e9e421692305a9de1a18e4389df041c0cf6c8918a8").unwrap();
    ///
    /// let assetid = AssetId::from_data(&data, &public_key).unwrap();
    /// assert_eq!(assetid.to_string(), "82c1f90bed24508e9ce74b536f97fa9c");
    /// # }
    /// ```
    pub fn from_data(data: &str, pub_key: &PublicKey) -> AssetId {
        let s = pub_key.to_hex();
        let ful_s = s + &data;

        let uuid = Uuid::new_v5(&uuid::NAMESPACE_DNS, &ful_s);
        AssetId::from_slice(uuid.as_bytes()).unwrap()
    }

    pub fn from_slice(b: &[u8]) -> Result<AssetId, ParseError> {
        let len = b.len();
        if len != ASSET_ID_LEN {
            return Err(ParseError::InvalidLength(len));
        }

        let mut assetid = AssetId::zero();
        assetid.0.copy_from_slice(b);
        Ok(assetid)
    }

    pub fn from_hex(hex: &str) -> Result<AssetId, ParseError> {
        let mut buffer: [u8; ASSET_ID_LEN] = [0; ASSET_ID_LEN];

        if hex.len() < ASSET_ID_LEN {
            return Err(ParseError::InvalidLength(hex.len()));
        }

        for i in 0..hex.len() {
            buffer[i] = hex.as_bytes()[i];
        }

        Ok(AssetId(buffer))
    }
}

impl<'a> Field<'a> for AssetId {
    fn field_size() -> Offset {
        ASSET_ID_LEN as Offset
    }

    unsafe fn read(buffer: &'a [u8], from: Offset, to: Offset) -> AssetId {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[from as usize..to as usize]);
        AssetId(bytes)
    }

    fn write(&self, buffer: &mut Vec<u8>, from: Offset, to: Offset) {
        buffer[from as usize..to as usize].copy_from_slice(&self.0);
    }

    fn check(
        _: &'a [u8],
        from: CheckedOffset,
        to: CheckedOffset,
        latest_segment: CheckedOffset,
    ) -> encoding::Result {
        if (to - from)?.unchecked_offset() != Self::field_size() {
            Err(encoding::Error::OffsetOverflow)
        } else {
            Ok(latest_segment)
        }
    }
}

/// Error details for string parsing failures.
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ParseError {
    InvalidLength(usize),
}

const SIMPLE_LENGTH: usize = 32;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::InvalidLength(found) => write!(
                f,
                "Invalid length; expecting {}, found {}",
                SIMPLE_LENGTH, found
            ),
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "AssetId parse error"
    }
}

impl ExonumJson for AssetId {
    fn serialize_field(&self) -> Result<serde_json::Value, Box<Error>> {
        serde_json::to_value(self).map_err(|e| e.into())
    }

    fn deserialize_field<B: WriteBufferWrapper>(
        value: &serde_json::Value,
        buffer: &mut B,
        from: Offset,
        to: Offset,
    ) -> Result<(), Box<Error>>
    {
        match serde_json::from_value::<AssetId>(value.clone()) {
            Ok(asset_id) => {
                buffer.write(from, to, asset_id);
                Ok(())
            }
            Err(err) => Err(Box::new(err)),
        }
    }
}

impl StorageKey for AssetId {
    fn size(&self) -> usize {
        ASSET_ID_LEN
    }

    fn read(buffer: &[u8]) -> Self {
        let mut bytes = [0; ASSET_ID_LEN];
        bytes.copy_from_slice(buffer);
        AssetId(bytes)
    }

    fn write(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.0);
    }
}

