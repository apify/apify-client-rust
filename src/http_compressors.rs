//! Request-body compression.
//!
//! The client compresses request bodies of at least 1024 bytes whose content type
//! is not already compressed and that carry no `Content-Encoding` of their own. The algorithm is an
//! [`HttpCompressor`]: the built-in [`BrotliHttpCompressor`] (the default) and
//! [`GzipHttpCompressor`], or a custom implementation passed to
//! [`ApifyClientBuilder::http_compressor`](crate::ApifyClientBuilder::http_compressor).

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Write;

use crate::error::{ApifyClientError, ApifyClientResult};

/// Request bodies at least this many bytes are compressed before sending. Smaller bodies are
/// left uncompressed because the CPU cost outweighs the transfer savings. Matches the reference
/// client's `MIN_COMPRESS_BYTES` threshold.
pub(crate) const MIN_COMPRESS_BYTES: usize = 1024;
/// `Content-Encoding` value used for brotli-compressed request bodies.
const CONTENT_ENCODING_BROTLI: &str = "br";
/// `Content-Encoding` value used for gzip-compressed request bodies.
const CONTENT_ENCODING_GZIP: &str = "gzip";
/// Lowest brotli quality, the fastest with the least compression.
const BROTLI_MIN_QUALITY: u32 = 0;
/// Highest brotli quality, the slowest with the best compression.
const BROTLI_MAX_QUALITY: u32 = 11;
/// Brotli quality used by [`BrotliHttpCompressor::default`]. Quality 6 balances the compression
/// ratio against CPU cost, matching the reference client.
const BROTLI_DEFAULT_QUALITY: u32 = 6;
/// Brotli sliding-window size (log2), 22 is the library default (a 4 MiB window).
const BROTLI_WINDOW_SIZE: u32 = 22;
/// Internal buffer size for the brotli encoder.
const BROTLI_BUFFER_SIZE: usize = 4096;
/// Lowest gzip quality, the fastest with the least compression.
const GZIP_MIN_QUALITY: u32 = 1;
/// Highest gzip quality, the slowest with the best compression.
const GZIP_MAX_QUALITY: u32 = 9;
/// Gzip quality used by [`GzipHttpCompressor::default`], matching the reference client.
const GZIP_DEFAULT_QUALITY: u32 = 9;

/// Media-type prefixes whose payloads already carry their own compression, so running them
/// through brotli/gzip burns CPU and memory for a result that is usually no smaller — and the
/// request also keeps the intended `Content-Type`, rather than becoming an unreadable
/// `Content-Encoding: br` blob the destination may not expect for these formats.
/// Matches the reference client's `ALREADY_COMPRESSED_MEDIA_TYPE_PREFIXES`.
const ALREADY_COMPRESSED_MEDIA_TYPE_PREFIXES: [&str; 3] = ["audio/", "image/", "video/"];

/// Exact media types (outside the prefixes above) whose payloads already carry their own
/// compression. Matches the reference client's `ALREADY_COMPRESSED_MEDIA_TYPES`.
const ALREADY_COMPRESSED_MEDIA_TYPES: [&str; 19] = [
    "application/epub+zip",
    "application/gzip",
    "application/java-archive",
    "application/vnd.android.package-archive",
    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.rar",
    "application/x-7z-compressed",
    "application/x-bzip",
    "application/x-bzip2",
    "application/x-gzip",
    "application/x-rar-compressed",
    "application/x-xz",
    "application/x-zip-compressed",
    "application/zip",
    "application/zstd",
    "font/woff",
    "font/woff2",
];

/// Uncompressed media types that sit under an already-compressed prefix above, so compressing
/// them still pays off. Matches the reference client's `COMPRESSIBLE_MEDIA_TYPES`.
const COMPRESSIBLE_MEDIA_TYPES: [&str; 16] = [
    "audio/aiff",
    "audio/basic",
    "audio/l16",
    "audio/l24",
    "audio/midi",
    "audio/vnd.wave",
    "audio/wav",
    "audio/wave",
    "audio/x-aiff",
    "audio/x-wav",
    "image/bmp",
    "image/tiff",
    "image/vnd.adobe.photoshop",
    "image/vnd.microsoft.icon",
    "image/x-icon",
    "image/x-ms-bmp",
];

/// Structured-syntax suffixes that mark a media type as text even under an already-compressed
/// prefix (e.g. `image/svg+xml`). Matches the reference client's
/// `COMPRESSIBLE_MEDIA_TYPE_SUFFIXES`.
const COMPRESSIBLE_MEDIA_TYPE_SUFFIXES: [&str; 2] = ["+json", "+xml"];

/// Strategy for compressing request bodies.
///
/// Implement this trait to plug in a custom compressor. [`content_encoding`](Self::content_encoding)
/// is sent in the `Content-Encoding` header and [`compress`](Self::compress) produces the body. The
/// client calls it only for bodies that are large enough to benefit from compression, whose content
/// type does not already carry its own compression, and which the caller did not send with a
/// `Content-Encoding` of their own.
pub trait HttpCompressor: Send + Sync + Debug {
    /// Value sent in the `Content-Encoding` header, for example `gzip` or `br`.
    fn content_encoding(&self) -> &str;

    /// Compresses a request body and returns the compressed bytes.
    fn compress(&self, data: &[u8]) -> ApifyClientResult<Vec<u8>>;
}

/// Brotli compressor (`Content-Encoding: br`), the client's default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrotliHttpCompressor {
    quality: u32,
}

impl BrotliHttpCompressor {
    /// Creates a brotli compressor with the given quality, from 0 (fastest, least compression) to
    /// 11 (slowest, best compression). Returns
    /// [`ApifyClientError::InvalidArgument`] when `quality` is out of range.
    pub fn new(quality: u32) -> ApifyClientResult<Self> {
        check_quality("brotli", quality, BROTLI_MIN_QUALITY, BROTLI_MAX_QUALITY)?;
        Ok(Self { quality })
    }
}

impl Default for BrotliHttpCompressor {
    /// Brotli at quality 6.
    fn default() -> Self {
        Self {
            quality: BROTLI_DEFAULT_QUALITY,
        }
    }
}

impl HttpCompressor for BrotliHttpCompressor {
    fn content_encoding(&self) -> &str {
        CONTENT_ENCODING_BROTLI
    }

    fn compress(&self, data: &[u8]) -> ApifyClientResult<Vec<u8>> {
        let mut writer = brotli::CompressorWriter::new(
            Vec::new(),
            BROTLI_BUFFER_SIZE,
            self.quality,
            BROTLI_WINDOW_SIZE,
        );
        write_all(&mut writer, data)?;
        Ok(writer.into_inner())
    }
}

/// Gzip compressor (`Content-Encoding: gzip`). Choose it for environments or intermediaries that
/// do not handle brotli.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GzipHttpCompressor {
    quality: u32,
}

impl GzipHttpCompressor {
    /// Creates a gzip compressor with the given quality, from 1 (fastest, least compression) to 9
    /// (slowest, best compression). Returns [`ApifyClientError::InvalidArgument`] when `quality`
    /// is out of range.
    pub fn new(quality: u32) -> ApifyClientResult<Self> {
        check_quality("gzip", quality, GZIP_MIN_QUALITY, GZIP_MAX_QUALITY)?;
        Ok(Self { quality })
    }
}

impl Default for GzipHttpCompressor {
    /// Gzip at quality 9.
    fn default() -> Self {
        Self {
            quality: GZIP_DEFAULT_QUALITY,
        }
    }
}

impl HttpCompressor for GzipHttpCompressor {
    fn content_encoding(&self) -> &str {
        CONTENT_ENCODING_GZIP
    }

    fn compress(&self, data: &[u8]) -> ApifyClientResult<Vec<u8>> {
        let mut encoder =
            flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(self.quality));
        write_all(&mut encoder, data)?;
        encoder.finish().map_err(compression_failed)
    }
}

/// Built-in compression algorithm selected by name, at its default quality. A shorthand for
/// [`BrotliHttpCompressor::default`] and [`GzipHttpCompressor::default`]; use the compressor
/// structs directly to tune the quality, or implement [`HttpCompressor`] for another algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RequestCompression {
    /// Brotli (`Content-Encoding: br`). The default: best compression ratio, and the encoding the
    /// reference client prefers.
    #[default]
    Brotli,
    /// Gzip (`Content-Encoding: gzip`). Choose this for environments or intermediaries that do not
    /// handle brotli.
    Gzip,
}

impl RequestCompression {
    /// The built-in compressor this algorithm name stands for.
    pub(crate) fn into_compressor(self) -> std::sync::Arc<dyn HttpCompressor> {
        match self {
            RequestCompression::Brotli => std::sync::Arc::new(BrotliHttpCompressor::default()),
            RequestCompression::Gzip => std::sync::Arc::new(GzipHttpCompressor::default()),
        }
    }
}

fn check_quality(algorithm: &str, quality: u32, min: u32, max: u32) -> ApifyClientResult<()> {
    if (min..=max).contains(&quality) {
        Ok(())
    } else {
        Err(ApifyClientError::InvalidArgument(format!(
            "{algorithm} quality must be between {min} and {max}, got {quality}"
        )))
    }
}

fn write_all(writer: &mut impl Write, data: &[u8]) -> ApifyClientResult<()> {
    writer.write_all(data).map_err(compression_failed)
}

fn compression_failed(err: std::io::Error) -> ApifyClientError {
    ApifyClientError::InvalidArgument(format!("compressing the request body failed: {err}"))
}

/// Compresses `body` with `compressor` and adds the matching `Content-Encoding` header, when the
/// body is large enough, no `Content-Encoding` is already set, and its content type is not
/// already compressed. Returns the (possibly unchanged) body.
///
/// The size threshold and the "compress once, before retries" behaviour mirror the reference
/// client.
pub(crate) fn maybe_compress(
    compressor: &dyn HttpCompressor,
    headers: &mut HashMap<String, String>,
    body: Vec<u8>,
) -> ApifyClientResult<Vec<u8>> {
    if body.len() < MIN_COMPRESS_BYTES {
        return Ok(body);
    }
    // Respect a caller-provided `Content-Encoding` (case-insensitive): the body is then assumed to
    // already be encoded, so re-compressing it would corrupt it.
    let already_encoded = headers
        .keys()
        .any(|k| k.eq_ignore_ascii_case("Content-Encoding"));
    if already_encoded {
        return Ok(body);
    }
    // Skip media types that already carry their own compression (images, audio, video,
    // archives): compressing them again burns CPU for little to no size reduction. This is
    // mainly relevant to a raw-bytes Actor input (e.g. `ActorClient::start_raw` with
    // `content_type: Some("application/zip")`); JSON request bodies always pass this check.
    let content_type = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Content-Type"))
        .map(|(_, v)| v.as_str());
    if !is_compressible_content_type(content_type) {
        return Ok(body);
    }

    let compressed = compressor.compress(&body)?;
    headers.insert(
        "Content-Encoding".to_string(),
        compressor.content_encoding().to_string(),
    );
    Ok(compressed)
}

/// Decides whether a request body with the given content type is worth compressing.
///
/// Images, audio, video and archives already carry their own compression: running them through
/// brotli or gzip burns CPU, holds a second full copy of the body in memory, and usually produces
/// output no smaller than the input (sometimes larger). Formats that are raw despite such a media
/// type, e.g. `image/bmp` or `audio/wav`, are still compressed. A body with no content type is
/// assumed to be compressible. Matches the reference client's `isCompressibleContentType`.
fn is_compressible_content_type(content_type: Option<&str>) -> bool {
    let Some(content_type) = content_type else {
        return true;
    };
    // `Content-Type` is case-insensitive and may carry parameters, e.g. `text/plain; charset=utf-8`.
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();

    if COMPRESSIBLE_MEDIA_TYPES.contains(&media_type.as_str()) {
        return true;
    }
    if COMPRESSIBLE_MEDIA_TYPE_SUFFIXES
        .iter()
        .any(|suffix| media_type.ends_with(suffix))
    {
        return true;
    }
    if ALREADY_COMPRESSED_MEDIA_TYPES.contains(&media_type.as_str()) {
        return false;
    }
    !ALREADY_COMPRESSED_MEDIA_TYPE_PREFIXES
        .iter()
        .any(|prefix| media_type.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn quality_outside_the_algorithm_range_is_rejected() {
        assert!(BrotliHttpCompressor::new(11).is_ok());
        assert!(BrotliHttpCompressor::new(12).is_err());
        assert!(GzipHttpCompressor::new(1).is_ok());
        assert!(GzipHttpCompressor::new(0).is_err());
        assert!(GzipHttpCompressor::new(10).is_err());
    }

    #[test]
    fn built_in_compressors_report_their_content_encoding() {
        assert_eq!(BrotliHttpCompressor::default().content_encoding(), "br");
        assert_eq!(GzipHttpCompressor::default().content_encoding(), "gzip");
    }

    #[test]
    fn small_bodies_are_left_uncompressed() {
        let mut headers = headers(&[("Content-Type", "application/json")]);
        let body = maybe_compress(
            &GzipHttpCompressor::default(),
            &mut headers,
            vec![b'a'; MIN_COMPRESS_BYTES - 1],
        )
        .unwrap();
        assert_eq!(body.len(), MIN_COMPRESS_BYTES - 1);
        assert!(!headers.contains_key("Content-Encoding"));
    }

    #[test]
    fn large_json_bodies_are_compressed_with_the_configured_compressor() {
        let mut headers = headers(&[("Content-Type", "application/json")]);
        let body = maybe_compress(
            &GzipHttpCompressor::default(),
            &mut headers,
            vec![b'a'; MIN_COMPRESS_BYTES * 4],
        )
        .unwrap();
        assert!(body.len() < MIN_COMPRESS_BYTES);
        assert_eq!(headers["Content-Encoding"], "gzip");
    }

    #[test]
    fn caller_set_content_encoding_is_respected_case_insensitively() {
        let mut headers = headers(&[("content-encoding", "gzip")]);
        let original = vec![b'a'; MIN_COMPRESS_BYTES * 2];
        let body = maybe_compress(
            &BrotliHttpCompressor::default(),
            &mut headers,
            original.clone(),
        )
        .unwrap();
        assert_eq!(body, original);
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn already_compressed_content_types_are_skipped() {
        for (content_type, compressible) in [
            ("image/png", false),
            ("application/zip", false),
            ("image/bmp", true),
            ("image/svg+xml", true),
            ("application/octet-stream", true),
            ("Application/JSON; charset=utf-8", true),
        ] {
            assert_eq!(
                is_compressible_content_type(Some(content_type)),
                compressible,
                "{content_type}"
            );
        }
        assert!(is_compressible_content_type(None));
    }
}
