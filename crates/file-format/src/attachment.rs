use serde::{Deserialize, Serialize};
use strum::EnumIs;
use types::Size;

use crate::{attachment_bytes::AttachmentBytes, media_type::MediaType};

#[derive(Clone, Deserialize, Serialize)]
pub struct Attachment {
    pub filename: Option<String>,
    pub kind: AttachmentKind,
}

#[derive(Clone, Deserialize, EnumIs, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentKind {
    Image {
        media_type: MediaType,
        data: AttachmentBytes,
        dimensions: Option<Size>,
    },
    Binary {
        media_type: Option<MediaType>,
        data: AttachmentBytes,
    },
    Audio {
        media_type: MediaType,
        data: AttachmentBytes,
    },
    Text {
        media_type: Option<MediaType>,
        text: String,
    },
}
