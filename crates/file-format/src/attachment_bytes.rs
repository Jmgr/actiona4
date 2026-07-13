use std::{fmt, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
use serde::{
    Deserializer, Serializer,
    de::{Error as _, Visitor},
};

// Payloads are shared via `Arc` so cloning a `File` before a blocking task is cheap.
#[derive(Clone)]
pub struct AttachmentBytes(Arc<Vec<u8>>);

impl AttachmentBytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for AttachmentBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl From<Vec<u8>> for AttachmentBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(Arc::new(value))
    }
}

impl serde::Serialize for AttachmentBytes {
    // Keeps binary files compact while making JSON attachments portable text.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&Base64.encode(self.as_slice()))
        } else {
            serializer.serialize_bytes(self.as_slice())
        }
    }
}

impl<'de> serde::Deserialize<'de> for AttachmentBytes {
    // Accepts the matching Base64 or raw-byte representation selected by the deserializer.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let value = String::deserialize(deserializer)?;
            Base64
                .decode(value)
                .map(|bytes| Self(Arc::new(bytes)))
                .map_err(D::Error::custom)
        } else {
            deserializer
                .deserialize_byte_buf(BytesVisitor)
                .map(|bytes| Self(Arc::new(bytes)))
        }
    }
}

struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a byte buffer")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Vec<u8>, E>
    where
        E: serde::de::Error,
    {
        Ok(value.to_vec())
    }

    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Vec<u8>, E>
    where
        E: serde::de::Error,
    {
        Ok(value.to_vec())
    }

    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Vec<u8>, E>
    where
        E: serde::de::Error,
    {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::AttachmentBytes;

    #[derive(Deserialize, Serialize)]
    struct Bytes(AttachmentBytes);

    #[rstest]
    #[case(&[], "")]
    #[case(&[0, 1, 2, 255], "AAEC/w==")]
    fn bytes_roundtrip_as_base64_in_json(#[case] data: &[u8], #[case] expected_base64: &str) {
        let value = Bytes(data.to_vec().into());

        let json = serde_json::to_value(&value).expect("serialize JSON");
        assert_eq!(json, json!(expected_base64));
        let decoded: Bytes = serde_json::from_value(json).expect("deserialize JSON");
        assert_eq!(decoded.0.as_slice(), data);
    }

    #[rstest]
    #[case(&[])]
    #[case(&[0, 1, 2, 255])]
    fn bytes_roundtrip_as_raw_data_in_postcard(#[case] data: &[u8]) {
        let value = Bytes(data.to_vec().into());

        let postcard = postcard::to_allocvec(&value).expect("serialize postcard");
        assert_eq!(postcard[0] as usize, data.len());
        assert_eq!(&postcard[1..], data);

        let decoded: Bytes = postcard::from_bytes(&postcard).expect("deserialize postcard");
        assert_eq!(decoded.0.as_slice(), data);
    }

    #[rstest]
    #[case("not base64")]
    #[case("A")]
    fn rejects_invalid_base64(#[case] value: &str) {
        let result = serde_json::from_value::<Bytes>(json!(value));

        assert!(result.is_err());
    }
}
