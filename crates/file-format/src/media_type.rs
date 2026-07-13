use std::fmt;

use mime::Mime;
use serde::{Deserializer, Serializer};

#[derive(Clone)]
pub struct MediaType(Mime);

impl MediaType {
    pub fn as_mime(&self) -> &Mime {
        &self.0
    }
}

impl From<Mime> for MediaType {
    fn from(value: Mime) -> Self {
        Self(value)
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl serde::Serialize for MediaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

impl<'de> serde::Deserialize<'de> for MediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use mime::Mime;
    use rstest::rstest;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use super::MediaType;

    #[derive(Deserialize, Serialize)]
    struct RequiredMime(MediaType);

    #[derive(Deserialize, Serialize)]
    struct OptionalMime(Option<MediaType>);

    #[rstest]
    #[case("image/png")]
    #[case("application/octet-stream")]
    fn required_mime_roundtrips_in_json_and_postcard(#[case] value: &str) {
        let value = RequiredMime(MediaType::from(
            value.parse::<Mime>().expect("valid MIME type"),
        ));

        let json = serde_json::to_value(&value).expect("serialize JSON");
        assert_eq!(json, json!(value.0.to_string()));
        let decoded: RequiredMime = serde_json::from_value(json).expect("deserialize JSON");
        assert_eq!(decoded.0.to_string(), value.0.to_string());

        let postcard = postcard::to_allocvec(&value).expect("serialize postcard");
        let decoded: RequiredMime = postcard::from_bytes(&postcard).expect("deserialize postcard");
        assert_eq!(decoded.0.to_string(), value.0.to_string());
    }

    #[rstest]
    #[case(Some("text/plain"))]
    #[case(None)]
    fn optional_mime_roundtrips_in_json_and_postcard(#[case] value: Option<&str>) {
        let value = OptionalMime(
            value.map(|value| MediaType::from(value.parse::<Mime>().expect("valid MIME type"))),
        );

        let json = serde_json::to_value(&value).expect("serialize JSON");
        let decoded: OptionalMime = serde_json::from_value(json).expect("deserialize JSON");
        assert_eq!(
            decoded.0.as_ref().map(ToString::to_string),
            value.0.as_ref().map(ToString::to_string),
        );

        let postcard = postcard::to_allocvec(&value).expect("serialize postcard");
        let decoded: OptionalMime = postcard::from_bytes(&postcard).expect("deserialize postcard");
        assert_eq!(
            decoded.0.as_ref().map(ToString::to_string),
            value.0.as_ref().map(ToString::to_string),
        );
    }

    #[rstest]
    #[case("not a MIME type")]
    #[case("image")]
    fn rejects_invalid_required_mime(#[case] value: &str) {
        let result = serde_json::from_value::<RequiredMime>(json!(value));

        assert!(result.is_err());
    }
}
