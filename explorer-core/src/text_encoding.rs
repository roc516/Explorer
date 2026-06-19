use encoding_rs::{GBK, UTF_16BE, UTF_16LE, WINDOWS_1252};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Auto,
    Utf8,
    Gbk,
    Utf16Le,
    Utf16Be,
    Latin1,
}

impl TextEncoding {
    pub const SELECTABLE: &'static [Self] = &[
        Self::Auto,
        Self::Utf8,
        Self::Gbk,
        Self::Utf16Le,
        Self::Utf16Be,
        Self::Latin1,
    ];

    pub fn message_id(self) -> &'static str {
        match self {
            Self::Auto => crate::i18n::ids::PREVIEW_ENCODING_AUTO,
            Self::Utf8 => crate::i18n::ids::PREVIEW_ENCODING_UTF8,
            Self::Gbk => crate::i18n::ids::PREVIEW_ENCODING_GBK,
            Self::Utf16Le => crate::i18n::ids::PREVIEW_ENCODING_UTF16LE,
            Self::Utf16Be => crate::i18n::ids::PREVIEW_ENCODING_UTF16BE,
            Self::Latin1 => crate::i18n::ids::PREVIEW_ENCODING_LATIN1,
        }
    }

    pub fn decode(self, bytes: &[u8]) -> Result<(String, Self), String> {
        let resolved = if self == Self::Auto {
            detect_encoding(bytes)
        } else {
            self
        };

        let content = decode_with_resolved(bytes, resolved)?;
        Ok((content, resolved))
    }
}

#[derive(Debug, Clone)]
pub struct TextPreview {
    pub raw: Vec<u8>,
    pub content: String,
    pub resolved_encoding: TextEncoding,
}

impl TextPreview {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        let (content, resolved_encoding) = TextEncoding::Auto.decode(&bytes)?;
        Ok(Self {
            raw: bytes,
            content,
            resolved_encoding,
        })
    }

    pub fn redecode(&mut self, encoding: TextEncoding) -> Result<(), String> {
        let (content, resolved_encoding) = encoding.decode(&self.raw)?;
        self.content = content;
        self.resolved_encoding = resolved_encoding;
        Ok(())
    }
}

pub fn detect_encoding(bytes: &[u8]) -> TextEncoding {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return TextEncoding::Utf8;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return TextEncoding::Utf16Le;
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return TextEncoding::Utf16Be;
    }

    if let Some(endian) = guess_utf16_endianness(bytes) {
        return endian;
    }

    if std::str::from_utf8(bytes).is_ok() {
        return TextEncoding::Utf8;
    }

    TextEncoding::Gbk
}

fn guess_utf16_endianness(bytes: &[u8]) -> Option<TextEncoding> {
    if bytes.len() < 4 || bytes.len() % 2 != 0 {
        return None;
    }

    let pairs = bytes.len() / 2;
    let mut le_ascii = 0usize;
    let mut be_ascii = 0usize;

    for chunk in bytes.chunks_exact(2) {
        if chunk[0] != 0 && chunk[1] == 0 {
            le_ascii += 1;
        }
        if chunk[1] != 0 && chunk[0] == 0 {
            be_ascii += 1;
        }
    }

    if le_ascii * 3 >= pairs * 2 {
        return Some(TextEncoding::Utf16Le);
    }
    if be_ascii * 3 >= pairs * 2 {
        return Some(TextEncoding::Utf16Be);
    }

    None
}

fn decode_with_resolved(bytes: &[u8], encoding: TextEncoding) -> Result<String, String> {
    match encoding {
        TextEncoding::Utf8 => decode_utf8(bytes),
        TextEncoding::Gbk => decode_gbk(bytes),
        TextEncoding::Utf16Le => decode_utf16(bytes, true),
        TextEncoding::Utf16Be => decode_utf16(bytes, false),
        TextEncoding::Latin1 => Ok(decode_latin1(bytes)),
        TextEncoding::Auto => unreachable!("resolved encoding cannot be Auto"),
    }
}

fn decode_utf8(bytes: &[u8]) -> Result<String, String> {
    let bytes = strip_prefix(bytes, &[0xEF, 0xBB, 0xBF]);
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| "preview-decode-failed".to_string())
}

fn decode_gbk(bytes: &[u8]) -> Result<String, String> {
    let (decoded, _, had_errors) = GBK.decode(bytes);
    if had_errors {
        return Err("preview-decode-failed".to_string());
    }
    Ok(decoded.into_owned())
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> Result<String, String> {
    let bytes = if little_endian {
        strip_prefix(bytes, &[0xFF, 0xFE])
    } else {
        strip_prefix(bytes, &[0xFE, 0xFF])
    };

    if bytes.len() % 2 != 0 {
        return Err("preview-decode-failed".to_string());
    }

    let (decoded, _, had_errors) = if little_endian {
        UTF_16LE.decode(bytes)
    } else {
        UTF_16BE.decode(bytes)
    };

    if had_errors {
        return Err("preview-decode-failed".to_string());
    }
    Ok(decoded.into_owned())
}

fn decode_latin1(bytes: &[u8]) -> String {
    let (decoded, _, _) = WINDOWS_1252.decode(bytes);
    decoded.into_owned()
}

fn strip_prefix<'a>(bytes: &'a [u8], prefix: &[u8]) -> &'a [u8] {
    if bytes.starts_with(prefix) {
        &bytes[prefix.len()..]
    } else {
        bytes
    }
}
