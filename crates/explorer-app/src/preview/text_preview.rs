use std::io::{Read, Seek, SeekFrom};

use encoding_rs::{GBK, UTF_16BE, UTF_16LE, WINDOWS_1252};
use explorer_core::FileEntry;

const DETECT_PREFIX: usize = 64 * 1024;
/// Cap displayed characters per logical line so the UI stays responsive.
const MAX_LINE_CHARS: usize = 4096;
/// Max bytes read per logical line (UTF-8 worst case / UTF-16 surrogates).
const MAX_LINE_BYTES: usize = MAX_LINE_CHARS * 4;

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

    fn is_utf16(self) -> bool {
        matches!(self, Self::Utf16Le | Self::Utf16Be)
    }
}

/// Whether switching encodings requires rebuilding the line-offset index.
pub fn needs_reindex(from: TextEncoding, to: TextEncoding) -> bool {
    from.is_utf16() || to.is_utf16()
}

#[derive(Clone)]
pub struct TextPreview {
    pub size: u64,
    /// Encoding detected from a file-prefix sample (Auto resolves to this).
    pub resolved_encoding: TextEncoding,
    file: FileEntry,
}

impl std::fmt::Debug for TextPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextPreview")
            .field("size", &self.size)
            .field("resolved_encoding", &self.resolved_encoding)
            .field("name", &self.file.name)
            .finish_non_exhaustive()
    }
}

pub fn is_extension(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "md"
            | "markdown"
            | "json"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "rs"
            | "toml"
            | "yaml"
            | "yml"
            | "ftl"
            | "log"
            | "ini"
            | "cfg"
            | "conf"
            | "csv"
            | "sql"
            | "sh"
            | "bat"
            | "ps1"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "go"
            | "java"
            | "kt"
            | "py"
            | "rb"
            | "php"
            | "swift"
            | "zig"
            | "lua"
            | "env"
    )
}

pub fn load(file: &FileEntry) -> Result<TextPreview, String> {
    let resolved_encoding = detect_from_file(file)?;
    Ok(TextPreview {
        size: file.size,
        resolved_encoding,
        file: file.clone(),
    })
}

impl TextPreview {
    pub fn resolve_encoding(&self, selected: TextEncoding) -> TextEncoding {
        if selected == TextEncoding::Auto {
            self.resolved_encoding
        } else {
            selected
        }
    }

    /// Scan the file and return the byte offset of each line start.
    pub fn build_line_index(&self, selected: TextEncoding) -> Result<Vec<u64>, String> {
        let encoding = self.resolve_encoding(selected);
        let mut reader = self.file.open()?;
        match encoding {
            TextEncoding::Utf16Le => scan_utf16_lines(&mut *reader, self.size, true),
            TextEncoding::Utf16Be => scan_utf16_lines(&mut *reader, self.size, false),
            _ => scan_byte_nl_lines(&mut *reader, self.size),
        }
    }

    /// Read and decode lines in `[start_line, end_line)`.
    ///
    /// Each logical line is read independently and capped at [`MAX_LINE_BYTES`],
    /// so a huge line does not pull the remainder of the line from storage.
    pub fn read_lines(
        &self,
        offsets: &[u64],
        start_line: usize,
        end_line: usize,
        selected: TextEncoding,
    ) -> Result<Vec<String>, String> {
        if offsets.is_empty() || start_line >= end_line || start_line >= offsets.len() {
            return Ok(Vec::new());
        }
        let encoding = self.resolve_encoding(selected);
        let end_line = end_line.min(offsets.len());
        let mut reader = self.file.open()?;
        let mut lines = Vec::with_capacity(end_line - start_line);

        for index in start_line..end_line {
            let start = offsets[index];
            if start >= self.size {
                lines.push(String::new());
                continue;
            }
            let logical_end = if index + 1 < offsets.len() {
                offsets[index + 1]
            } else {
                self.size
            };
            let capped_end = start.saturating_add(MAX_LINE_BYTES as u64).min(logical_end);
            let capped = capped_end < logical_end;
            let len = (capped_end - start) as usize;

            reader
                .seek(SeekFrom::Start(start))
                .map_err(|err| err.to_string())?;
            let mut bytes = read_exact_up_to(&mut *reader, len)?;
            clamp_encoded_bytes(&mut bytes, encoding);

            let strip_bom = index == 0 && start == 0;
            let decoded = decode_window(&bytes, encoding, strip_bom)?;
            let trimmed = decoded.trim_end_matches(['\r', '\n']);
            let mut line = truncate_line(trimmed);
            if capped && !line.ends_with('…') {
                line.push('…');
            }
            lines.push(line);
        }

        Ok(lines)
    }
}

fn detect_from_file(file: &FileEntry) -> Result<TextEncoding, String> {
    if file.size == 0 {
        return Ok(TextEncoding::Utf8);
    }
    let mut reader = file.open()?;
    let n = (file.size as usize).min(DETECT_PREFIX);
    let bytes = read_exact_up_to(&mut *reader, n)?;
    Ok(detect_encoding(&bytes))
}

fn scan_byte_nl_lines(reader: &mut dyn Read, size: u64) -> Result<Vec<u64>, String> {
    let mut offsets = vec![0u64];
    if size == 0 {
        return Ok(offsets);
    }

    let mut buf = [0u8; 64 * 1024];
    let mut pos = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|err| err.to_string())?;
        if n == 0 {
            break;
        }
        for (i, byte) in buf[..n].iter().enumerate() {
            if *byte == b'\n' {
                let next = pos + i as u64 + 1;
                if next <= size {
                    offsets.push(next);
                }
            }
        }
        pos += n as u64;
        if pos >= size {
            break;
        }
    }
    Ok(offsets)
}

fn scan_utf16_lines(
    reader: &mut dyn Read,
    size: u64,
    little_endian: bool,
) -> Result<Vec<u64>, String> {
    let mut offsets = vec![0u64];
    if size < 2 {
        return Ok(offsets);
    }

    let mut leftover = Vec::new();
    let mut abs_offset = 0u64;
    let mut read_buf = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut read_buf).map_err(|err| err.to_string())?;
        if n == 0 && leftover.is_empty() {
            break;
        }
        leftover.extend_from_slice(&read_buf[..n]);

        let mut i = 0usize;
        while i + 1 < leftover.len() {
            let pair = [leftover[i], leftover[i + 1]];
            if is_utf16_newline(&pair, little_endian) {
                let next = abs_offset + 2;
                if next <= size {
                    offsets.push(next);
                }
            }
            i += 2;
            abs_offset += 2;
        }
        leftover.drain(..i);

        if n == 0 {
            break;
        }
        if abs_offset >= size {
            break;
        }
    }

    Ok(offsets)
}

fn is_utf16_newline(pair: &[u8; 2], little_endian: bool) -> bool {
    if little_endian {
        pair[0] == b'\n' && pair[1] == 0
    } else {
        pair[0] == 0 && pair[1] == b'\n'
    }
}

fn clamp_encoded_bytes(bytes: &mut Vec<u8>, encoding: TextEncoding) {
    match encoding {
        TextEncoding::Utf16Le | TextEncoding::Utf16Be => {
            if bytes.len() % 2 == 1 {
                bytes.pop();
            }
        }
        TextEncoding::Utf8 => {
            while !bytes.is_empty() && std::str::from_utf8(bytes).is_err() {
                bytes.pop();
            }
        }
        TextEncoding::Gbk => {
            if bytes.is_empty() {
                return;
            }
            let (_, _, had_errors) = GBK.decode(bytes);
            if had_errors {
                bytes.pop();
            }
        }
        TextEncoding::Latin1 | TextEncoding::Auto => {}
    }
}

fn truncate_line(line: &str) -> String {
    let mut end = line.len();
    let mut truncated = false;
    for (count, (i, _)) in line.char_indices().enumerate() {
        if count == MAX_LINE_CHARS {
            end = i;
            truncated = true;
            break;
        }
    }
    if !truncated {
        return line.to_string();
    }
    let mut out = line[..end].to_string();
    out.push('…');
    out
}

fn decode_window(
    bytes: &[u8],
    encoding: TextEncoding,
    strip_bom: bool,
) -> Result<String, String> {
    let bytes = if strip_bom {
        match encoding {
            TextEncoding::Utf8 => strip_prefix(bytes, &[0xEF, 0xBB, 0xBF]),
            TextEncoding::Utf16Le => strip_prefix(bytes, &[0xFF, 0xFE]),
            TextEncoding::Utf16Be => strip_prefix(bytes, &[0xFE, 0xFF]),
            _ => bytes,
        }
    } else {
        bytes
    };

    match encoding {
        TextEncoding::Utf8 => decode_utf8(bytes),
        TextEncoding::Gbk => decode_gbk(bytes),
        TextEncoding::Utf16Le => decode_utf16(bytes, true),
        TextEncoding::Utf16Be => decode_utf16(bytes, false),
        TextEncoding::Latin1 => Ok(decode_latin1(bytes)),
        TextEncoding::Auto => unreachable!("resolved encoding cannot be Auto"),
    }
}

fn detect_encoding(bytes: &[u8]) -> TextEncoding {
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

fn decode_utf8(bytes: &[u8]) -> Result<String, String> {
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

fn read_exact_up_to(reader: &mut dyn Read, len: usize) -> Result<Vec<u8>, String> {
    let mut buf = vec![0u8; len];
    let mut filled = 0usize;
    while filled < len {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(err) => return Err(err.to_string()),
        }
    }
    buf.truncate(filled);
    Ok(buf)
}
