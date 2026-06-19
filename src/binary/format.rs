//! Binary format detection using magic bytes.
//!
//! Fast, zero-allocation format detection for media files.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Media format categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryFormat {
    // Image formats
    /// PNG image format - lossless compression, supports transparency.
    Png,
    /// JPEG image format - lossy compression, best for photographs.
    Jpeg,
    /// GIF image format - supports animation and transparency, limited to 256 colors.
    Gif,
    /// WebP image format - modern format with both lossy and lossless compression.
    WebP,
    /// BMP image format - uncompressed bitmap, large file sizes.
    Bmp,
    /// TIFF image format - flexible format supporting multiple compression methods.
    Tiff,
    /// ICO image format - Windows icon format, supports multiple sizes.
    Ico,
    /// AVIF image format - modern format based on AV1 video codec.
    Avif,
    /// HEIC image format - High Efficiency Image Container, used by Apple devices.
    Heic,
    /// SVG image format - vector graphics using XML markup.
    Svg,

    // Video formats
    /// MP4 video format - widely supported container for H.264/H.265 video.
    Mp4,
    /// WebM video format - open format optimized for web streaming.
    Webm,
    /// MKV video format - Matroska container, supports multiple tracks.
    Mkv,
    /// AVI video format - legacy Microsoft container format.
    Avi,
    /// MOV video format - Apple QuickTime container.
    Mov,
    /// FLV video format - Flash Video, legacy streaming format.
    Flv,
    /// WMV video format - Windows Media Video container.
    Wmv,

    // Audio formats
    /// MP3 audio format - lossy compression, widely supported.
    Mp3,
    /// WAV audio format - uncompressed PCM audio, high quality.
    Wav,
    /// FLAC audio format - lossless compression, audiophile quality.
    Flac,
    /// OGG audio format - open container, typically with Vorbis codec.
    Ogg,
    /// AAC audio format - Advanced Audio Coding, better than MP3 at same bitrate.
    Aac,
    /// M4A audio format - MPEG-4 audio container, often with AAC codec.
    M4a,
    /// WMA audio format - Windows Media Audio, Microsoft proprietary.
    Wma,

    // Document formats
    /// PDF document format - Portable Document Format, preserves layout.
    Pdf,
    /// DOCX document format - Microsoft Word Open XML format.
    Docx,
    /// XLSX document format - Microsoft Excel Open XML spreadsheet.
    Xlsx,
    /// PPTX document format - Microsoft PowerPoint Open XML presentation.
    Pptx,
    /// ODT document format - OpenDocument Text, open standard.
    Odt,
    /// RTF document format - Rich Text Format, cross-platform text.
    Rtf,

    // Archive formats
    /// ZIP archive format - widely supported compression format.
    Zip,
    /// GZIP archive format - GNU zip, single-file compression.
    Gzip,
    /// BZIP2 archive format - high compression ratio, slower than gzip.
    Bzip2,
    /// XZ archive format - LZMA2 compression, excellent ratio.
    Xz,
    /// TAR archive format - tape archive, bundles files without compression.
    Tar,
    /// RAR archive format - proprietary compression with good ratio.
    Rar,
    /// 7Z archive format - 7-Zip format with LZMA compression.
    SevenZip,
    /// ZSTD archive format - Zstandard, fast compression with good ratio.
    Zstd,

    // Other
    /// WASM binary format - WebAssembly portable bytecode.
    Wasm,
    /// Unknown format - could not be detected from magic bytes.
    Unknown,
}

impl BinaryFormat {
    /// Get the typical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Heic => "heic",
            Self::Svg => "svg",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Mkv => "mkv",
            Self::Avi => "avi",
            Self::Mov => "mov",
            Self::Flv => "flv",
            Self::Wmv => "wmv",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::Aac => "aac",
            Self::M4a => "m4a",
            Self::Wma => "wma",
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Odt => "odt",
            Self::Rtf => "rtf",
            Self::Zip => "zip",
            Self::Gzip => "gz",
            Self::Bzip2 => "bz2",
            Self::Xz => "xz",
            Self::Tar => "tar",
            Self::Rar => "rar",
            Self::SevenZip => "7z",
            Self::Zstd => "zst",
            Self::Wasm => "wasm",
            Self::Unknown => "",
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Ico => "image/x-icon",
            Self::Avif => "image/avif",
            Self::Heic => "image/heic",
            Self::Svg => "image/svg+xml",
            Self::Mp4 => "video/mp4",
            Self::Webm => "video/webm",
            Self::Mkv => "video/x-matroska",
            Self::Avi => "video/x-msvideo",
            Self::Mov => "video/quicktime",
            Self::Flv => "video/x-flv",
            Self::Wmv => "video/x-ms-wmv",
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Flac => "audio/flac",
            Self::Ogg => "audio/ogg",
            Self::Aac => "audio/aac",
            Self::M4a => "audio/mp4",
            Self::Wma => "audio/x-ms-wma",
            Self::Pdf => "application/pdf",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Odt => "application/vnd.oasis.opendocument.text",
            Self::Rtf => "application/rtf",
            Self::Zip => "application/zip",
            Self::Gzip => "application/gzip",
            Self::Bzip2 => "application/x-bzip2",
            Self::Xz => "application/x-xz",
            Self::Tar => "application/x-tar",
            Self::Rar => "application/vnd.rar",
            Self::SevenZip => "application/x-7z-compressed",
            Self::Zstd => "application/zstd",
            Self::Wasm => "application/wasm",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// Check if this is an image format.
    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::Png
                | Self::Jpeg
                | Self::Gif
                | Self::WebP
                | Self::Bmp
                | Self::Tiff
                | Self::Ico
                | Self::Avif
                | Self::Heic
                | Self::Svg
        )
    }

    /// Check if this is a video format.
    pub fn is_video(&self) -> bool {
        matches!(
            self,
            Self::Mp4 | Self::Webm | Self::Mkv | Self::Avi | Self::Mov | Self::Flv | Self::Wmv
        )
    }

    /// Check if this is an audio format.
    pub fn is_audio(&self) -> bool {
        matches!(
            self,
            Self::Mp3 | Self::Wav | Self::Flac | Self::Ogg | Self::Aac | Self::M4a | Self::Wma
        )
    }

    /// Check if this is a document format.
    pub fn is_document(&self) -> bool {
        matches!(
            self,
            Self::Pdf | Self::Docx | Self::Xlsx | Self::Pptx | Self::Odt | Self::Rtf
        )
    }

    /// Check if this is an archive format.
    pub fn is_archive(&self) -> bool {
        matches!(
            self,
            Self::Zip
                | Self::Gzip
                | Self::Bzip2
                | Self::Xz
                | Self::Tar
                | Self::Rar
                | Self::SevenZip
                | Self::Zstd
        )
    }
}

/// Magic byte signature for format detection.
#[derive(Debug, Clone)]
pub struct MediaSignature {
    /// Byte pattern to match.
    pub pattern: &'static [u8],
    /// Offset from start of file.
    pub offset: usize,
    /// Format this signature indicates.
    pub format: BinaryFormat,
}

/// Fast format detector using magic bytes.
pub struct FormatDetector {
    /// Registered signatures.
    signatures: Vec<MediaSignature>,
}

impl FormatDetector {
    /// Create a new format detector with standard signatures.
    pub fn new() -> Self {
        Self {
            signatures: vec![
                // Images
                MediaSignature {
                    pattern: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
                    offset: 0,
                    format: BinaryFormat::Png,
                },
                MediaSignature {
                    pattern: &[0xFF, 0xD8, 0xFF],
                    offset: 0,
                    format: BinaryFormat::Jpeg,
                },
                MediaSignature {
                    pattern: b"GIF87a",
                    offset: 0,
                    format: BinaryFormat::Gif,
                },
                MediaSignature {
                    pattern: b"GIF89a",
                    offset: 0,
                    format: BinaryFormat::Gif,
                },
                MediaSignature {
                    pattern: b"RIFF",
                    offset: 0,
                    format: BinaryFormat::WebP,
                }, // + WEBP at offset 8
                MediaSignature {
                    pattern: b"BM",
                    offset: 0,
                    format: BinaryFormat::Bmp,
                },
                MediaSignature {
                    pattern: &[0x49, 0x49, 0x2A, 0x00],
                    offset: 0,
                    format: BinaryFormat::Tiff,
                }, // Little-endian
                MediaSignature {
                    pattern: &[0x4D, 0x4D, 0x00, 0x2A],
                    offset: 0,
                    format: BinaryFormat::Tiff,
                }, // Big-endian
                MediaSignature {
                    pattern: &[0x00, 0x00, 0x01, 0x00],
                    offset: 0,
                    format: BinaryFormat::Ico,
                },
                // Video
                MediaSignature {
                    pattern: b"ftyp",
                    offset: 4,
                    format: BinaryFormat::Mp4,
                },
                MediaSignature {
                    pattern: &[0x1A, 0x45, 0xDF, 0xA3],
                    offset: 0,
                    format: BinaryFormat::Webm,
                }, // Also MKV
                MediaSignature {
                    pattern: b"RIFF",
                    offset: 0,
                    format: BinaryFormat::Avi,
                }, // + AVI at offset 8
                MediaSignature {
                    pattern: b"FLV",
                    offset: 0,
                    format: BinaryFormat::Flv,
                },
                // Audio
                MediaSignature {
                    pattern: &[0xFF, 0xFB],
                    offset: 0,
                    format: BinaryFormat::Mp3,
                },
                MediaSignature {
                    pattern: &[0xFF, 0xFA],
                    offset: 0,
                    format: BinaryFormat::Mp3,
                },
                MediaSignature {
                    pattern: &[0xFF, 0xF3],
                    offset: 0,
                    format: BinaryFormat::Mp3,
                },
                MediaSignature {
                    pattern: &[0xFF, 0xF2],
                    offset: 0,
                    format: BinaryFormat::Mp3,
                },
                MediaSignature {
                    pattern: b"ID3",
                    offset: 0,
                    format: BinaryFormat::Mp3,
                },
                MediaSignature {
                    pattern: b"RIFF",
                    offset: 0,
                    format: BinaryFormat::Wav,
                }, // + WAVE at offset 8
                MediaSignature {
                    pattern: b"fLaC",
                    offset: 0,
                    format: BinaryFormat::Flac,
                },
                MediaSignature {
                    pattern: b"OggS",
                    offset: 0,
                    format: BinaryFormat::Ogg,
                },
                // Documents
                MediaSignature {
                    pattern: b"%PDF",
                    offset: 0,
                    format: BinaryFormat::Pdf,
                },
                MediaSignature {
                    pattern: &[0x50, 0x4B, 0x03, 0x04],
                    offset: 0,
                    format: BinaryFormat::Zip,
                }, // Also DOCX/XLSX/PPTX
                MediaSignature {
                    pattern: b"{\\rtf",
                    offset: 0,
                    format: BinaryFormat::Rtf,
                },
                // Archives
                MediaSignature {
                    pattern: &[0x1F, 0x8B],
                    offset: 0,
                    format: BinaryFormat::Gzip,
                },
                MediaSignature {
                    pattern: &[0x42, 0x5A, 0x68],
                    offset: 0,
                    format: BinaryFormat::Bzip2,
                },
                MediaSignature {
                    pattern: &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
                    offset: 0,
                    format: BinaryFormat::Xz,
                },
                MediaSignature {
                    pattern: b"ustar",
                    offset: 257,
                    format: BinaryFormat::Tar,
                },
                MediaSignature {
                    pattern: b"Rar!",
                    offset: 0,
                    format: BinaryFormat::Rar,
                },
                MediaSignature {
                    pattern: &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
                    offset: 0,
                    format: BinaryFormat::SevenZip,
                },
                MediaSignature {
                    pattern: &[0x28, 0xB5, 0x2F, 0xFD],
                    offset: 0,
                    format: BinaryFormat::Zstd,
                },
                // Other
                MediaSignature {
                    pattern: &[0x00, 0x61, 0x73, 0x6D],
                    offset: 0,
                    format: BinaryFormat::Wasm,
                },
            ],
        }
    }

    /// Detect format from byte slice.
    pub fn detect(&self, data: &[u8]) -> BinaryFormat {
        for sig in &self.signatures {
            if data.len() >= sig.offset + sig.pattern.len() {
                let slice = &data[sig.offset..sig.offset + sig.pattern.len()];
                if slice == sig.pattern {
                    // Special cases for RIFF container
                    if sig.pattern == b"RIFF" && data.len() >= 12 {
                        if &data[8..12] == b"WEBP" {
                            return BinaryFormat::WebP;
                        } else if &data[8..12] == b"WAVE" {
                            return BinaryFormat::Wav;
                        } else if &data[8..12] == b"AVI " {
                            return BinaryFormat::Avi;
                        }
                    }
                    return sig.format;
                }
            }
        }
        BinaryFormat::Unknown
    }

    /// Detect format from a file.
    pub fn detect_file(&self, path: impl AsRef<Path>) -> std::io::Result<BinaryFormat> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 512];
        let n = file.read(&mut buffer)?;
        Ok(self.detect(&buffer[..n]))
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        let detector = FormatDetector::new();
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(detector.detect(&png_header), BinaryFormat::Png);
    }

    #[test]
    fn test_detect_jpeg() {
        let detector = FormatDetector::new();
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0, 0x00];
        assert_eq!(detector.detect(&jpeg_header), BinaryFormat::Jpeg);
    }

    #[test]
    fn test_detect_pdf() {
        let detector = FormatDetector::new();
        let pdf_header = b"%PDF-1.4";
        assert_eq!(detector.detect(pdf_header), BinaryFormat::Pdf);
    }

    #[test]
    fn test_format_properties() {
        assert!(BinaryFormat::Png.is_image());
        assert!(BinaryFormat::Mp4.is_video());
        assert!(BinaryFormat::Mp3.is_audio());
        assert!(BinaryFormat::Pdf.is_document());
        assert!(BinaryFormat::Zip.is_archive());

        assert_eq!(BinaryFormat::Png.extension(), "png");
        assert_eq!(BinaryFormat::Png.mime_type(), "image/png");
    }
}
