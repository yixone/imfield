use crate::MimeType;

/// Defines a matcher closure for the MimeType signature
macro_rules! pattern_matcher {
    ($val: expr, $ex: expr, $idx: expr, ? $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex,
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr, $pattern: literal $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex && ($val[$idx] == $pattern),
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr) => {
        $val.len() >= ($idx) && $ex
    };
}

/// Defines a list of patterns for guessing the MIME type
macro_rules! patterns {
    (
        $(
            $id: ident, $p_id: ident => [ $( ( $offset: expr, [ $( $tokens:tt ),* ] ) ) | + ]
        );* $(;)?
    ) => {
        $(
            pub const $p_id: &MimePattern = &MimePattern {
                mime: $crate::MimeType::$id,
                matcher: |b| $( pattern_matcher!(b, !b.is_empty(), $offset, $($tokens),*) )||+,
            };
        )*

        pub const PATTERNS: &[&MimePattern] = &[
            $( $p_id ),*
        ];

        impl $crate::MimeType {
            pub fn pattern(&self) -> Option<&'static MimePattern> {
                match self {
                    $(
                        Self::$id => Some($p_id),
                    )*
                    Self::OctetStream => None
                }
            }
        }
    };
}

/// Builds a prefix tree to speed up MIME type detection
macro_rules! ptree {
    (
        $(
            $prefix: literal => [
                $( $mime_pattern: expr ),+
            ] as $bucket: ident;
        )+
    ) => {
        $(
            const $bucket: &[&MimePattern] = &[$( $mime_pattern ),+];
        )+

        static PREFIX_TREE: [&[&MimePattern]; 256] = {
            const EMPTY: &[&MimePattern] = &[];
            let mut arr = [EMPTY;256];
            $(
                arr[$prefix] = $bucket;
            )+
            arr
        };
    };
}

/// Pattern for guessing the [`MimeType`] based on the magic bytes signature
pub struct MimePattern {
    pub mime: MimeType,
    pub matcher: fn(&[u8]) -> bool,
}

patterns! {
    // -- Image patterns --
    Jpeg, JPEG_PATTERN => [
        (0, [0xFF, 0xD8, 0xFF, 0xDB]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01]) |
        (0, [0xFF, 0xD8, 0xFF, 0xEE]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE1, ?, ?, 0x45, 0x78, 0x69, 0x66, 0x00, 0x00]) |
        (0, [0xFF, 0xD8, 0xFF, 0xE0])
    ];
    Png, PNG_PATTERN => [
        (0, [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
    ];
    Bmp, BMP_PATTERN => [
        (0, [0x42, 0x4D])
    ];
    Webp, WEBP_PATTERN => [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x57, 0x45, 0x42, 0x50])
    ];
    Gif, GIF_PATTERN => [
        (0, [0x47, 0x49, 0x46, 0x38, 0x37, 0x61]) |
        (0, [0x47, 0x49, 0x46, 0x38, 0x39, 0x61])
    ];

    // -- Video patterns --
    Mp4, MP4_PATTERN => [
        (4, [0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D]) |
        (4, [0x66, 0x74, 0x79, 0x70, 0x4D, 0x53, 0x4E, 0x56])
    ];
    Avi, AVI_PATTERN => [
        (0, [0x52, 0x49, 0x46, 0x46, ?, ?, ?, ?, 0x41, 0x56, 0x49, 0x20])
    ];
    Webm, WEBM_PATTERN => [
        (0, [0x1A, 0x45, 0xDF, 0xA3])
    ];
}

ptree! {
    0x42 => [BMP_PATTERN] as _PB_42;
    0x47 => [GIF_PATTERN] as _PB_47;
    0x52 => [WEBM_PATTERN, AVI_PATTERN] as _PB_52;
    0x89 => [PNG_PATTERN] as _PB_89;
    0x1A => [WEBM_PATTERN] as _PB_1A;
    0xFF => [JPEG_PATTERN] as _PB_FF;
}

/// Attempts to identify the [`MimeType`] of the given byte slice
///
/// The [`MimeType`] is guessing from the file's content using known
/// magic-bytes signatures. If the content does not match any known signature,
/// [`MimeType::OctetStream`] is returned
///
/// Returns `None` if the input is empty
pub fn guess_mime<T>(slice: T) -> Option<MimeType>
where
    T: AsRef<[u8]>,
{
    let magic = slice.as_ref();

    if magic.is_empty() {
        return None;
    }

    // Tries to guess the mimetype using a prefix tree
    let prefix_byte = magic[0] as usize;
    let prefix = PREFIX_TREE[prefix_byte];
    if !prefix.is_empty() {
        for p in prefix {
            if (p.matcher)(magic) {
                return Some(p.mime);
            }
        }
    }

    // Fallback
    for p in PATTERNS {
        if (p.matcher)(magic) {
            return Some(p.mime);
        }
    }

    // If the guess fails returns an `OctetStream`
    Some(MimeType::OctetStream)
}

#[cfg(test)]
mod tests {
    use crate::MimeType;

    #[test]
    fn guess_mime() {
        const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        let mime = super::guess_mime(PNG_MAGIC).unwrap();
        assert_eq!(mime, MimeType::Png);
    }

    #[test]
    fn guess_mime_with_placeholder() {
        const WEBP_MAGIC: &[u8] = &[
            0x52, 0x49, 0x46, 0x46, 0x00, 0x42, 0xFF, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];

        let mime = super::guess_mime(WEBP_MAGIC).unwrap();
        assert_eq!(mime, MimeType::Webp);
    }

    #[test]
    fn return_octet_stream_on_unknown() {
        const MAGIC: &[u8] = &[0xFF];
        let mime = super::guess_mime(MAGIC).unwrap();
        assert_eq!(mime, MimeType::OctetStream);
    }

    #[test]
    fn return_none_on_empty() {
        assert!(super::guess_mime([]).is_none());
    }
}
