use std::{collections::HashSet, io::Cursor, mem::size_of};

use action_definition::tree::ActionTree;
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
use image::{ExtendedColorType, ImageEncoder as _, ImageReader, Limits, codecs::webp::WebPEncoder};
use indexmap::IndexMap;
use rayon::prelude::*;
use rodio::Source as _;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use types::{Size, size};
use uuid::Uuid;

use crate::{
    Error,
    attachment::{Attachment, AttachmentKind},
    attachment_bytes::AttachmentBytes,
    checked_payload_size,
    compression::{
        JSON_COMPRESSION_THRESHOLD, MAX_ATTACHMENT_UNCOMPRESSED_SIZE, MAX_AUDIO_DECODED_SIZE,
        compress, decompress,
    },
    media_type::MediaType,
};

// Bounds untrusted image decoding so a crafted file cannot force an unbounded allocation.
const MAX_IMAGE_DIMENSION: u32 = 32_768;
const MAX_IMAGE_ALLOC: u64 = MAX_ATTACHMENT_UNCOMPRESSED_SIZE as u64;

#[derive(Deserialize, Serialize)]
pub(crate) struct FileWire {
    pub version: u16,
    pub tree: ActionTree,
    pub attachments: Vec<(Uuid, AttachmentWire)>,
    #[serde(default)]
    pub metadata: crate::FileMetadata,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct AttachmentWire {
    filename: Option<String>,
    kind: AttachmentKindWire,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AttachmentKindWire {
    Image {
        media_type: String,
        data: String,
        dimensions: Option<Size>,
    },
    Binary {
        media_type: Option<String>,
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        compression: Option<CompressionWire>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uncompressed_size: Option<u64>,
    },
    Audio {
        media_type: String,
        data: String,
    },
    Text {
        media_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        compression: Option<CompressionWire>,
        #[serde(skip_serializing_if = "Option::is_none")]
        uncompressed_size: Option<u64>,
    },
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum CompressionWire {
    Zstd,
}

impl FileWire {
    // Converts attachments in parallel because image, audio, and zstd transforms are CPU-bound.
    pub fn from_file(
        file: crate::File,
        version: u16,
        cancellation_token: &CancellationToken,
    ) -> Result<Self, Error> {
        let crate::File {
            actions: tree,
            attachments,
            metadata,
        } = file;
        let wire_attachments = attachments
            .par_iter()
            .map(|(id, attachment)| {
                if cancellation_token.is_cancelled() {
                    return Err(Error::Canceled);
                }
                attachment_to_wire(attachment, cancellation_token)
                    .map(|attachment| (*id, attachment))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            version,
            tree,
            attachments: wire_attachments,
            metadata,
        })
    }

    // Decodes independently stored attachment payloads in parallel while retaining their order.
    pub fn into_file(self, cancellation_token: &CancellationToken) -> Result<crate::File, Error> {
        validate_attachment_layout(&self.attachments)?;
        let attachments = self
            .attachments
            .into_par_iter()
            .map(|(id, attachment)| {
                if cancellation_token.is_cancelled() {
                    return Err(Error::Canceled);
                }
                attachment_from_wire(id, attachment)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(crate::File {
            actions: self.tree,
            attachments: attachments.into_iter().collect::<IndexMap<_, _>>(),
            metadata: self.metadata,
        })
    }
}

// Produces the JSON representation, applying the storage codec appropriate to each media kind.
fn attachment_to_wire(
    attachment: &Attachment,
    cancellation_token: &CancellationToken,
) -> Result<AttachmentWire, Error> {
    let kind = match &attachment.kind {
        AttachmentKind::Image { data, .. } => image_to_wire(data.as_slice())?,
        AttachmentKind::Binary { media_type, data } => binary_to_wire(media_type, data)?,
        AttachmentKind::Audio { media_type, data } => {
            audio_to_wire(media_type, data, cancellation_token)?
        }
        AttachmentKind::Text { media_type, text } => text_to_wire(media_type, text)?,
    };

    Ok(AttachmentWire {
        filename: attachment.filename.clone(),
        kind,
    })
}

// Restores the runtime representation and validates MIME types and compressed payload metadata.
fn attachment_from_wire(id: Uuid, attachment: AttachmentWire) -> Result<(Uuid, Attachment), Error> {
    let kind = match attachment.kind {
        AttachmentKindWire::Image {
            media_type,
            data,
            dimensions,
        } => AttachmentKind::Image {
            media_type: parse_media_type(media_type)?,
            data: AttachmentBytes::from(
                Base64
                    .decode(data)
                    .map_err(|error| Error::Attachment(error.to_string()))?,
            ),
            dimensions,
        },
        AttachmentKindWire::Binary {
            media_type,
            data,
            compression,
            uncompressed_size,
        } => AttachmentKind::Binary {
            media_type: media_type.map(parse_media_type).transpose()?,
            data: decode_bytes(data, compression, uncompressed_size)?.into(),
        },
        AttachmentKindWire::Audio { media_type, data } => AttachmentKind::Audio {
            media_type: parse_media_type(media_type)?,
            data: AttachmentBytes::from(
                Base64
                    .decode(data)
                    .map_err(|error| Error::Attachment(error.to_string()))?,
            ),
        },
        AttachmentKindWire::Text {
            media_type,
            text,
            data,
            compression,
            uncompressed_size,
        } => AttachmentKind::Text {
            media_type: media_type.map(parse_media_type).transpose()?,
            text: decode_text(text, data, compression, uncompressed_size)?,
        },
    };

    Ok((
        id,
        Attachment {
            filename: attachment.filename,
            kind,
        },
    ))
}

fn parse_media_type(value: String) -> Result<MediaType, Error> {
    value
        .parse::<mime::Mime>()
        .map(MediaType::from)
        .map_err(|error| Error::Attachment(error.to_string()))
}

// Decodes any supported source image and writes a lossless WebP payload for JSON storage.
fn image_to_wire(data: &[u8]) -> Result<AttachmentKindWire, Error> {
    checked_payload_size(
        "image attachment",
        data.len() as u64,
        MAX_ATTACHMENT_UNCOMPRESSED_SIZE,
    )?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC);

    let mut reader = ImageReader::new(Cursor::new(data)).with_guessed_format()?;
    reader.limits(limits);
    let image = reader.decode()?;

    let rgba_size = u64::from(image.width())
        .checked_mul(u64::from(image.height()))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| Error::Attachment("decoded image is too large".to_owned()))?;
    checked_payload_size("decoded image", rgba_size, MAX_ATTACHMENT_UNCOMPRESSED_SIZE)?;

    let rgba = image.to_rgba8();
    let dimensions = size(rgba.width(), rgba.height());

    let mut encoded = Vec::new();
    WebPEncoder::new_lossless(&mut encoded).write_image(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        ExtendedColorType::Rgba8,
    )?;

    Ok(AttachmentKindWire::Image {
        media_type: "image/webp".to_owned(),
        data: Base64.encode(encoded),
        dimensions: Some(dimensions),
    })
}

fn text_to_wire(media_type: &Option<MediaType>, text: &str) -> Result<AttachmentKindWire, Error> {
    let compressed = compressed_bytes(text.as_bytes())?;
    Ok(match compressed {
        Some(data) => AttachmentKindWire::Text {
            media_type: media_type.as_ref().map(ToString::to_string),
            text: None,
            data: Some(Base64.encode(data)),
            compression: Some(CompressionWire::Zstd),
            uncompressed_size: Some(text.len() as u64),
        },
        None => AttachmentKindWire::Text {
            media_type: media_type.as_ref().map(ToString::to_string),
            text: Some(text.to_owned()),
            data: None,
            compression: None,
            uncompressed_size: None,
        },
    })
}

// Keeps valid Ogg Opus data or normalizes other supported audio to Ogg Opus at 48 kHz.
fn audio_to_wire(
    media_type: &MediaType,
    data: &AttachmentBytes,
    cancellation_token: &CancellationToken,
) -> Result<AttachmentKindWire, Error> {
    checked_payload_size(
        "audio attachment",
        data.as_slice().len() as u64,
        MAX_ATTACHMENT_UNCOMPRESSED_SIZE,
    )?;
    if media_type.as_mime().essence_str() == "audio/ogg" && is_ogg_opus(data.as_slice()) {
        return Ok(AttachmentKindWire::Audio {
            media_type: "audio/ogg; codecs=opus".to_owned(),
            data: Base64.encode(data.as_slice()),
        });
    }

    let mut decoder = rodio::Decoder::try_from(Cursor::new(data.as_slice().to_vec()))
        .map_err(|error| Error::Attachment(error.to_string()))?;
    let channels = usize::from(decoder.channels().get());
    if !(1..=2).contains(&channels) {
        return Err(Error::Attachment(
            "only mono and stereo audio attachments can be encoded as Ogg Opus".to_owned(),
        ));
    }

    let sample_rate = decoder.sample_rate().get();
    let samples = collect_audio_samples(&mut decoder, cancellation_token)?;
    let samples = resample_to_48khz(samples, channels, sample_rate, cancellation_token)?;
    if cancellation_token.is_cancelled() {
        return Err(Error::Canceled);
    }

    let bitrate = if channels == 1 { 48_000 } else { 96_000 };
    let data = ruopus::encode_ogg_opus(&samples, channels, bitrate);

    Ok(AttachmentKindWire::Audio {
        media_type: "audio/ogg; codecs=opus".to_owned(),
        data: Base64.encode(data),
    })
}

fn is_ogg_opus(data: &[u8]) -> bool {
    ruopus::ogg::OggOpusReader::new(data).is_ok()
}

fn collect_audio_samples(
    decoder: &mut rodio::Decoder<Cursor<Vec<u8>>>,
    cancellation_token: &CancellationToken,
) -> Result<Vec<f32>, Error> {
    let max_samples = MAX_AUDIO_DECODED_SIZE / size_of::<f32>();
    let mut samples = Vec::new();
    for sample in decoder {
        if samples.len() == max_samples {
            return Err(Error::PayloadTooLarge {
                kind: "decoded audio",
                size: (max_samples + 1) as u64 * size_of::<f32>() as u64,
                limit: MAX_AUDIO_DECODED_SIZE,
            });
        }
        samples.push(sample);
        if samples.len() % 8_192 == 0 && cancellation_token.is_cancelled() {
            return Err(Error::Canceled);
        }
    }
    Ok(samples)
}

// Uses linear interpolation to meet the fixed sample-rate requirement of the Opus encoder.
fn resample_to_48khz(
    samples: Vec<f32>,
    channels: usize,
    sample_rate: u32,
    cancellation_token: &CancellationToken,
) -> Result<Vec<f32>, Error> {
    const TARGET_RATE: u32 = 48_000;
    if sample_rate == TARGET_RATE {
        return Ok(samples);
    }

    let frames = samples.len() / channels;
    if frames == 0 {
        return Ok(Vec::new());
    }
    let target_frames = (frames as u64)
        .checked_mul(TARGET_RATE as u64)
        .ok_or_else(|| Error::Attachment("resampled audio is too large".to_owned()))?
        .div_ceil(sample_rate as u64);
    let target_samples = target_frames
        .checked_mul(channels as u64)
        .ok_or_else(|| Error::Attachment("resampled audio is too large".to_owned()))?;
    let target_bytes = target_samples
        .checked_mul(size_of::<f32>() as u64)
        .ok_or_else(|| Error::Attachment("resampled audio is too large".to_owned()))?;
    checked_payload_size("resampled audio", target_bytes, MAX_AUDIO_DECODED_SIZE)?;
    let target_samples = usize::try_from(target_samples)
        .map_err(|_| Error::Attachment("resampled audio is too large".to_owned()))?;
    let mut output = Vec::with_capacity(target_samples);

    for target_frame in 0..target_frames as usize {
        if target_frame % 8_192 == 0 && cancellation_token.is_cancelled() {
            return Err(Error::Canceled);
        }
        let source = target_frame as f64 * sample_rate as f64 / TARGET_RATE as f64;
        let before = source.floor() as usize;
        let after = (before + 1).min(frames - 1);
        let fraction = (source - before as f64) as f32;
        for channel in 0..channels {
            let a = samples[before.min(frames - 1) * channels + channel];
            let b = samples[after * channels + channel];
            output.push(a + (b - a) * fraction);
        }
    }

    Ok(output)
}

fn binary_to_wire(
    media_type: &Option<MediaType>,
    data: &AttachmentBytes,
) -> Result<AttachmentKindWire, Error> {
    let compressed = compressed_bytes(data.as_slice())?;
    Ok(AttachmentKindWire::Binary {
        media_type: media_type.as_ref().map(ToString::to_string),
        data: Base64.encode(compressed.as_deref().unwrap_or(data.as_slice())),
        compression: compressed.as_ref().map(|_| CompressionWire::Zstd),
        uncompressed_size: compressed.as_ref().map(|_| data.as_slice().len() as u64),
    })
}

// Keeps zstd only when the final Base64 representation is smaller than the original one.
fn compressed_bytes(data: &[u8]) -> Result<Option<Vec<u8>>, Error> {
    if data.len() < JSON_COMPRESSION_THRESHOLD || data.len() > MAX_ATTACHMENT_UNCOMPRESSED_SIZE {
        return Ok(None);
    }

    // Base64 length is a pure function of byte length, so compare it without encoding either buffer.
    let compressed = compress(data)?;
    let compressed_len = base64::encoded_len(compressed.len(), true).unwrap_or(usize::MAX);
    let original_len = base64::encoded_len(data.len(), true).unwrap_or(usize::MAX);
    Ok((compressed_len < original_len).then_some(compressed))
}

// Rejects duplicate IDs before Rayon reconstructs the attachment map.
fn validate_attachment_layout(attachments: &[(Uuid, AttachmentWire)]) -> Result<(), Error> {
    let mut ids = HashSet::with_capacity(attachments.len());
    for (id, _) in attachments {
        if !ids.insert(*id) {
            return Err(Error::Attachment(format!("duplicate attachment ID {id}")));
        }
    }
    Ok(())
}

// Decodes Base64 first, then verifies the declared output size for compressed payloads.
fn decode_bytes(
    data: String,
    compression: Option<CompressionWire>,
    uncompressed_size: Option<u64>,
) -> Result<Vec<u8>, Error> {
    let data = Base64
        .decode(data)
        .map_err(|error| Error::Attachment(error.to_string()))?;

    match compression {
        None => Ok(data),
        Some(CompressionWire::Zstd) => {
            let size = uncompressed_size
                .ok_or_else(|| Error::Attachment("missing uncompressed size".to_owned()))?;
            let size = checked_payload_size("attachment", size, MAX_ATTACHMENT_UNCOMPRESSED_SIZE)?;

            Ok(decompress(&data, size, MAX_ATTACHMENT_UNCOMPRESSED_SIZE)?)
        }
    }
}

// Accepts either readable text or the compressed byte form, but never an ambiguous combination.
fn decode_text(
    text: Option<String>,
    data: Option<String>,
    compression: Option<CompressionWire>,
    uncompressed_size: Option<u64>,
) -> Result<String, Error> {
    match (text, data, compression) {
        (Some(text), None, None) => Ok(text),
        (None, Some(data), Some(compression)) => {
            let bytes = decode_bytes(data, Some(compression), uncompressed_size)?;
            String::from_utf8(bytes).map_err(|error| Error::Attachment(error.to_string()))
        }
        _ => Err(Error::Attachment(
            "invalid text attachment payload".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::is_ogg_opus;

    #[test]
    fn only_accepts_parseable_ogg_opus_streams() {
        let valid = ruopus::encode_ogg_opus(&[], 1, 48_000);
        assert!(is_ogg_opus(&valid));
        assert!(!is_ogg_opus(b"not an Ogg stream containing OpusHead"));
    }
}
