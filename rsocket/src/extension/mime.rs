use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;

use once_cell::sync::Lazy;

use crate::error::RSocketError;
use crate::Result;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum MimeType {
    Normal(String),
    WellKnown(u8),
}

static U8_TO_STR: Lazy<HashMap<u8, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for it in list_all().iter() {
        m.insert(it.0, it.1);
    }
    m
});

static STR_TO_U8: Lazy<HashMap<&'static str, u8>> = Lazy::new(|| {
    let mut m = HashMap::new();
    for it in list_all().iter() {
        m.insert(it.1, it.0);
    }
    m
});

impl MimeType {
    pub fn parse(value: u8) -> Option<MimeType> {
        U8_TO_STR.get(&value).map(|it| Self::WellKnown(value))
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Self::WellKnown(n) => Some(*n),
            Self::Normal(_) => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Normal(s) => Some(s.as_ref()),
            Self::WellKnown(n) => U8_TO_STR.get(n).copied(),
        }
    }
}

impl Into<String> for MimeType {
    fn into(self) -> String {
        match self {
            Self::Normal(s) => s,
            Self::WellKnown(n) => match U8_TO_STR.get(&n) {
                Some(v) => v.to_string(),
                None => "UNKNOWN".to_string(),
            },
        }
    }
}

impl From<&str> for MimeType {
    fn from(value: &str) -> MimeType {
        match STR_TO_U8.get(value) {
            Some(v) => Self::WellKnown(*v),
            None => Self::Normal(value.to_owned()),
        }
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Normal(s) => write!(f, "{}", s),
            Self::WellKnown(n) => match U8_TO_STR.get(n) {
                Some(v) => write!(f, "{}", v),
                None => Err(fmt::Error),
            },
        }
    }
}

macro_rules! mime {
    ($name:ident,$n:expr,$s:expr) => {
        const $name: (u8, &str) = ($n, $s);
        impl MimeType {
            pub const $name: Self = Self::WellKnown($n);
        }
    };
}

mime!(APPLICATION_AVRO, 0x00, "application/avro");
mime!(APPLICATION_CBOR, 0x01, "application/avro");
mime!(APPLICATION_GRAPHQL, 0x02, "application/graphql");
mime!(APPLICATION_GZIP, 0x03, "application/gzip");
mime!(APPLICATION_JAVASCRIPT, 0x04, "application/javascript");
mime!(APPLICATION_JSON, 0x05, "application/json");
mime!(APPLICATION_OCTET_STREAM, 0x06, "application/octet-stream");
mime!(APPLICATION_PDF, 0x07, "application/pdf");
mime!(
    APPLICATION_VND_APACHE_THRIFT_BINARY,
    0x08,
    "application/vnd.apache.thrift.binary"
);
mime!(
    APPLICATION_VND_GOOGLE_PROTOBUF,
    0x09,
    "application/vnd.google.protobuf"
);
mime!(APPLICATION_XML, 0x0A, "application/xml");
mime!(APPLICATION_ZIP, 0x0B, "application/zip");
mime!(AUDIO_AAC, 0x0C, "audio/aac");
mime!(AUDIO_MP3, 0x0D, "audio/mp3");
mime!(AUDIO_MP4, 0x0E, "audio/mp4");
mime!(AUDIO_MPEG3, 0x0F, "audio/mpeg3");
mime!(AUDIO_MPEG, 0x10, "audio/mpeg");
mime!(AUDIO_OGG, 0x11, "audio/ogg");
mime!(AUDIO_OPUS, 0x12, "audio/opus");
mime!(AUDIO_VORBIS, 0x13, "audio/vorbis");
mime!(IMAGE_BMP, 0x14, "image/bmp");
mime!(IMAGE_GIF, 0x15, "image/gif");
mime!(IMAGE_HEIC_SEQUENCE, 0x16, "image/heic-sequence");
mime!(IMAGE_HEIC, 0x17, "image/heic");
mime!(IMAGE_HEIF_SEQUENCE, 0x18, "image/heif-sequence");
mime!(IMAGE_HEIF, 0x19, "image/heif");
mime!(IMAGE_JPEG, 0x1A, "image/jpeg");
mime!(IMAGE_PNG, 0x1B, "image/png");
mime!(IMAGE_TIFF, 0x1C, "image/tiff");
mime!(MULTIPART_MIXED, 0x1D, "multipart/mixed");
mime!(TEXT_CSS, 0x1E, "text/css");
mime!(TEXT_CSV, 0x1F, "text/csv");
mime!(TEXT_HTML, 0x20, "text/html");
mime!(TEXT_PLAIN, 0x21, "text/plain");
mime!(TEXT_XML, 0x22, "text/xml");
mime!(VIDEO_H264, 0x23, "video/H264");
mime!(VIDEO_H265, 0x24, "video/H265");
mime!(VIDEO_VP8, 0x25, "video/VP8");
mime!(APPLICATION_X_HESSIAN, 0x26, "application/x-hessian");
mime!(APPLICATION_X_JAVA_OBJECT, 0x27, "application/x-java-object");
mime!(
    APPLICATION_CLOUDEVENTS_JSON,
    0x28,
    "application/cloudevents+json"
);
mime!(
    MESSAGE_X_RSOCKET_MIME_TYPE_V0,
    0x7A,
    "message/x.rsocket.mime-type.v0"
);
mime!(
    MESSAGE_X_RSOCKET_ACCEPT_TIME_TYPES_V0,
    0x7B,
    "message/x.rsocket.accept-mime-types.v0"
);
mime!(
    MESSAGE_X_RSOCKET_AUTHENTICATION_V0,
    0x7C,
    "message/x.rsocket.authentication.v0"
);
mime!(
    MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0,
    0x7D,
    "message/x.rsocket.tracing-zipkin.v0"
);
mime!(
    MESSAGE_X_RSOCKET_ROUTING_V0,
    0x7E,
    "message/x.rsocket.routing.v0"
);
mime!(
    MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0,
    0x7F,
    "message/x.rsocket.composite-metadata.v0"
);

fn list_all() -> Vec<(u8, &'static str)> {
    vec![
        APPLICATION_AVRO,
        APPLICATION_CBOR,
        APPLICATION_GRAPHQL,
        APPLICATION_GZIP,
        APPLICATION_JAVASCRIPT,
        APPLICATION_JSON,
        APPLICATION_OCTET_STREAM,
        APPLICATION_PDF,
        APPLICATION_VND_APACHE_THRIFT_BINARY,
        APPLICATION_VND_GOOGLE_PROTOBUF,
        APPLICATION_XML,
        APPLICATION_ZIP,
        AUDIO_AAC,
        AUDIO_MP3,
        AUDIO_MP4,
        AUDIO_MPEG3,
        AUDIO_MPEG,
        AUDIO_OGG,
        AUDIO_OPUS,
        AUDIO_VORBIS,
        IMAGE_BMP,
        IMAGE_GIF,
        IMAGE_HEIC_SEQUENCE,
        IMAGE_HEIC,
        IMAGE_HEIF_SEQUENCE,
        IMAGE_HEIF,
        IMAGE_JPEG,
        IMAGE_PNG,
        IMAGE_TIFF,
        MULTIPART_MIXED,
        TEXT_CSS,
        TEXT_CSV,
        TEXT_HTML,
        TEXT_PLAIN,
        TEXT_XML,
        VIDEO_H264,
        VIDEO_H265,
        VIDEO_VP8,
        APPLICATION_X_HESSIAN,
        APPLICATION_X_JAVA_OBJECT,
        APPLICATION_CLOUDEVENTS_JSON,
        MESSAGE_X_RSOCKET_MIME_TYPE_V0,
        MESSAGE_X_RSOCKET_ACCEPT_TIME_TYPES_V0,
        MESSAGE_X_RSOCKET_AUTHENTICATION_V0,
        MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0,
        MESSAGE_X_RSOCKET_ROUTING_V0,
        MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0,
    ]
}
