use file_format_types::{FileFormat, Kind};
use zstd::bulk::compress;

const COMPRESSION_SAMPLE_SIZE: usize = 256 * 1024; // 256 KiB
const MINIMUM_COMPRESSION_SAVINGS_PERCENT: usize = 1; // 1%

pub struct Format {
    pub mimetype: String,
    pub should_compress: bool,
}

fn should_compress(format: FileFormat, contents: &[u8]) -> bool {
    if matches!(format.kind(), Kind::Compressed) {
        // Compressed formats are not compressed again
        return false;
    }

    match format {
        // Compressed formats are not compressed again
        FileFormat::Ace
        | FileFormat::AdobeIntegratedRuntime
        | FileFormat::AgeEncryption
        | FileFormat::Alz
        | FileFormat::AndroidAppBundle
        | FileFormat::AndroidPackage
        | FileFormat::AnimatedPortableNetworkGraphics
        | FileFormat::Appimage
        | FileFormat::AppleItunesAudio
        | FileFormat::AppleItunesAudiobook
        | FileFormat::AppleItunesProtectedAudio
        | FileFormat::AppleItunesVideo
        | FileFormat::Av1ImageFileFormat
        | FileFormat::Av1ImageFileFormatSequence
        | FileFormat::ElectronicPublication
        | FileFormat::FlashMp4Audio
        | FileFormat::FlashMp4Audiobook
        | FileFormat::FlashMp4ProtectedVideo
        | FileFormat::FlashMp4Video
        | FileFormat::FreeLosslessAudioCodec
        | FileFormat::FreeLosslessImageFormat
        | FileFormat::GraphicsInterchangeFormat
        | FileFormat::HighEfficiencyImageCoding
        | FileFormat::HighEfficiencyImageCodingSequence
        | FileFormat::HighEfficiencyImageFileFormat
        | FileFormat::HighEfficiencyImageFileFormatSequence
        | FileFormat::JavaArchive
        | FileFormat::JointPhotographicExpertsGroup
        | FileFormat::Jpeg2000Codestream
        | FileFormat::Jpeg2000Part1
        | FileFormat::Jpeg2000Part2
        | FileFormat::Jpeg2000Part3
        | FileFormat::Jpeg2000Part6
        | FileFormat::JpegExtendedRange
        | FileFormat::JpegLs
        | FileFormat::JpegNetworkGraphics
        | FileFormat::JpegXl
        | FileFormat::MatroskaAudio
        | FileFormat::MatroskaVideo
        | FileFormat::MonkeysAudio
        | FileFormat::Mpeg12AudioLayer3
        | FileFormat::Mpeg4Part14Audio
        | FileFormat::Mpeg4Part14Video
        | FileFormat::OfficeOpenXmlDocument
        | FileFormat::OfficeOpenXmlDrawing
        | FileFormat::OfficeOpenXmlPresentation
        | FileFormat::OfficeOpenXmlSpreadsheet
        | FileFormat::OggFlac
        | FileFormat::OggMedia
        | FileFormat::OggOpus
        | FileFormat::OggSpeex
        | FileFormat::OggTheora
        | FileFormat::OggVorbis
        | FileFormat::OpendocumentDatabase
        | FileFormat::OpendocumentFormula
        | FileFormat::OpendocumentFormulaTemplate
        | FileFormat::OpendocumentGraphics
        | FileFormat::OpendocumentGraphicsTemplate
        | FileFormat::OpendocumentPresentation
        | FileFormat::OpendocumentPresentationTemplate
        | FileFormat::OpendocumentSpreadsheet
        | FileFormat::OpendocumentSpreadsheetTemplate
        | FileFormat::OpendocumentText
        | FileFormat::OpendocumentTextMaster
        | FileFormat::OpendocumentTextMasterTemplate
        | FileFormat::OpendocumentTextTemplate
        | FileFormat::PortableDocumentFormat
        | FileFormat::Webm
        | FileFormat::Webp
        | FileFormat::Wavpack => return false,

        _ => {}
    }

    let Some(start) = contents.split_first_chunk::<COMPRESSION_SAMPLE_SIZE>() else {
        // If the contents are not at least COMPRESSION_SAMPLE_SIZE bytes long, don't compress
        return false;
    };

    let Ok(compressed) = compress(start.0, 0) else {
        // Don't compress if the sample compression failed
        return false;
    };

    if !has_minimum_compression_savings(start.0.len(), compressed.len()) {
        // Don't compress if the sample does not save enough space
        return false;
    }

    true
}

const fn has_minimum_compression_savings(uncompressed_size: usize, compressed_size: usize) -> bool {
    compressed_size <= uncompressed_size * (100 - MINIMUM_COMPRESSION_SAVINGS_PERCENT) / 100
}

pub fn guess_format(contents: &[u8]) -> Format {
    let contents_format = FileFormat::from_bytes(contents);

    Format {
        mimetype: contents_format.media_type().to_owned(),
        should_compress: should_compress(contents_format, contents),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_format_from_contents() {
        let format = guess_format(b"\x89PNG\r\n\x1a\n");

        assert_eq!(format.mimetype, "image/png");
        assert!(!format.should_compress);
    }

    #[test]
    fn unknown_short_contents_use_the_default_media_type() {
        let format = guess_format(b"\0\0\0\0");

        assert_eq!(format.mimetype, "application/octet-stream");
        assert!(!format.should_compress);
    }

    #[test]
    fn known_compressed_formats_are_not_compressed_again() {
        let format = guess_format(b"\x1f\x8b");

        assert_eq!(format.mimetype, "application/gzip");
        assert!(!format.should_compress);
    }

    #[test]
    fn compression_trial_requires_compressible_contents() {
        let compressible = vec![0; COMPRESSION_SAMPLE_SIZE];
        assert!(guess_format(&compressible).should_compress);

        let mut state = 0x9e37_79b9_u32;
        let incompressible = (0..COMPRESSION_SAMPLE_SIZE)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                u8::try_from(state).unwrap()
            })
            .collect::<Vec<_>>();
        assert!(!guess_format(&incompressible).should_compress);
    }

    #[test]
    fn compression_trial_requires_at_least_one_percent_savings() {
        let threshold = COMPRESSION_SAMPLE_SIZE * (100 - MINIMUM_COMPRESSION_SAVINGS_PERCENT) / 100;

        assert!(has_minimum_compression_savings(
            COMPRESSION_SAMPLE_SIZE,
            threshold
        ));
        assert!(!has_minimum_compression_savings(
            COMPRESSION_SAMPLE_SIZE,
            threshold + 1
        ));
    }
}
