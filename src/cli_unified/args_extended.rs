//! Extended tool commands for all 56+ tools

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum VideoToolsExtended {
    /// Transcode video format
    Transcode { input: PathBuf, output: PathBuf },
    /// Extract audio from video
    ExtractAudio { input: PathBuf, output: PathBuf },
    /// Trim video
    Trim {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        start: f64,
        #[arg(short, long)]
        end: f64,
    },
    /// Scale/resize video
    Scale {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        width: Option<u32>,
        #[arg(short = 'H', long)]
        height: Option<u32>,
    },
    /// Create GIF from video
    ToGif {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value = "10")]
        fps: u32,
    },
    /// Extract thumbnail
    Thumbnail {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value = "0")]
        timestamp: f64,
    },
    /// Mute video
    Mute { input: PathBuf, output: PathBuf },
    /// Add watermark
    Watermark {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        text: Option<String>,
        #[arg(short = 'i', long)]
        image: Option<PathBuf>,
    },
    /// Adjust video speed
    Speed {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value = "1.0")]
        factor: f64,
    },
    /// Concatenate videos
    Concat {
        inputs: Vec<PathBuf>,
        output: PathBuf,
    },
    /// Burn subtitles
    Subtitles {
        video: PathBuf,
        subtitles: PathBuf,
        output: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum AudioToolsExtended {
    /// Convert audio format
    Convert { input: PathBuf, output: PathBuf },
    /// Trim audio
    Trim {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        start: f64,
        #[arg(short, long)]
        duration: f64,
    },
    /// Merge audio files
    Merge {
        inputs: Vec<PathBuf>,
        output: PathBuf,
    },
    /// Normalize audio
    Normalize { input: PathBuf, output: PathBuf },
    /// Remove silence
    RemoveSilence { input: PathBuf, output: PathBuf },
    /// Split audio by silence
    Split { input: PathBuf, output_dir: PathBuf },
    /// Add audio effects
    Effects {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        effect: String,
    },
    /// Generate spectrum visualization
    Spectrum { input: PathBuf, output: PathBuf },
    /// Read audio metadata
    Metadata { input: PathBuf },
}

#[derive(Subcommand, Debug)]
pub enum ImageToolsExtended {
    /// Convert image format
    Convert {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        quality: Option<u8>,
    },
    /// Resize image
    Resize {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        width: Option<u32>,
        #[arg(short = 'H', long)]
        height: Option<u32>,
    },
    /// Compress image
    Compress {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long, default_value = "80")]
        quality: u8,
    },
    /// Generate favicons
    Favicon { input: PathBuf, output_dir: PathBuf },
    /// Add watermark
    Watermark {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        text: Option<String>,
    },
    /// Apply filters
    Filter {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        filter: String,
    },
    /// Read EXIF data
    Exif { input: PathBuf },
    /// Generate QR code
    Qr { text: String, output: PathBuf },
    /// Extract color palette
    Palette {
        input: PathBuf,
        #[arg(short, long, default_value = "5")]
        colors: usize,
    },
    /// OCR text extraction
    Ocr { input: PathBuf },
}

#[derive(Subcommand, Debug)]
pub enum ArchiveToolsExtended {
    /// Create ZIP archive
    Zip {
        files: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Extract ZIP archive
    Unzip {
        input: PathBuf,
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Create TAR archive
    Tar {
        files: Vec<PathBuf>,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Extract TAR archive
    Untar {
        input: PathBuf,
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Compress with gzip
    Gzip { input: PathBuf, output: PathBuf },
    /// Decompress gzip
    Gunzip { input: PathBuf, output: PathBuf },
    /// List archive contents
    List { input: PathBuf },
}

#[derive(Subcommand, Debug)]
pub enum DocumentToolsExtended {
    /// Convert markdown to HTML
    MarkdownToHtml { input: PathBuf, output: PathBuf },
    /// Extract text from document
    ExtractText { input: PathBuf, output: PathBuf },
    /// Merge PDF files
    PdfMerge {
        inputs: Vec<PathBuf>,
        output: PathBuf,
    },
    /// Split PDF
    PdfSplit { input: PathBuf, output_dir: PathBuf },
    /// Compress PDF
    PdfCompress { input: PathBuf, output: PathBuf },
    /// Encrypt PDF
    PdfEncrypt {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        password: String,
    },
    /// Add PDF watermark
    PdfWatermark {
        input: PathBuf,
        output: PathBuf,
        #[arg(short, long)]
        text: String,
    },
    /// Convert PDF to images
    PdfToImage { input: PathBuf, output_dir: PathBuf },
    /// Convert HTML to PDF
    HtmlToPdf { input: PathBuf, output: PathBuf },
}

#[derive(Subcommand, Debug)]
pub enum UtilityToolsExtended {
    /// Calculate file hash
    Hash {
        input: PathBuf,
        #[arg(short, long, default_value = "sha256")]
        algorithm: String,
    },
    /// Base64 encode
    Base64Encode { input: PathBuf },
    /// Base64 decode
    Base64Decode { input: String, output: PathBuf },
    /// URL encode
    UrlEncode { text: String },
    /// URL decode
    UrlDecode { text: String },
    /// Generate UUID
    Uuid,
    /// Validate UUID
    ValidateUuid { uuid: String },
    /// Convert timestamp
    Timestamp {
        #[arg(short, long)]
        unix: Option<i64>,
    },
    /// Find duplicate files
    FindDuplicates { directory: PathBuf },
    /// Verify checksum
    VerifyChecksum { file: PathBuf, checksum: String },
    /// Convert JSON to YAML
    JsonToYaml { input: PathBuf, output: PathBuf },
    /// Convert YAML to JSON
    YamlToJson { input: PathBuf, output: PathBuf },
    /// Format JSON
    FormatJson { input: PathBuf, output: PathBuf },
    /// Convert CSV
    ConvertCsv {
        input: PathBuf,
        output: PathBuf,
        #[arg(long = "csv-format", default_value = "json")]
        csv_format: String,
    },
}
