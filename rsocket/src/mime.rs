use std::collections::HashMap;
use std::fmt;
use std::slice::Iter;

pub const APPLICATION_BINARY: &str = "application/binary";
pub const APPLICATION_AVRO: &str = "application/avro";
pub const APPLICATION_CBOR: &str = "application/cbor";
pub const APPLICATION_GRAPHQL: &str = "application/graphql";
pub const APPLICATION_GZIP: &str = "application/gzip";
pub const APPLICATION_JAVASCRIPT: &str = "application/javascript";
pub const APPLICATION_JSON: &str = "application/json";
pub const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";
pub const APPLICATION_PDF: &str = "application/pdf";
pub const APPLICATION_THRIFT: &str = "application/vnd.apache.thrift.binary";
pub const APPLICATION_PROTOBUF: &str = "application/vnd.google.protobuf";
pub const APPLICATION_XML: &str = "application/xml";
pub const APPLICATION_ZIP: &str = "application/zip";
pub const AUDIO_AAC: &str = "audio/aac";
pub const AUDIO_MP3: &str = "audio/mp3";
pub const AUDIO_MP4: &str = "audio/mp4";
pub const AUDIO_MPEG3: &str = "audio/mpeg3";
pub const AUDIO_MPEG: &str = "audio/mpeg";
pub const AUDIO_OGG: &str = "audio/ogg";
pub const AUDIO_OPUS: &str = "audio/opus";
pub const AUDIO_VORBIS: &str = "audio/vorbis";
pub const IMAGE_BMP: &str = "image/bmp";
pub const IMAGE_GIF: &str = "image/gif";
pub const IMAGE_HEIC_SEQUENCE: &str = "image/heic-sequence";
pub const IMAGE_HEIC: &str = "image/heic";
pub const IMAGE_HEIF_SEQUENCE: &str = "image/heif-sequence";
pub const IMAGE_HEIF: &str = "image/heif";
pub const IMAGE_JPEG: &str = "image/jpeg";
pub const IMAGE_PNG: &str = "image/png";
pub const IMAGE_TIFF: &str = "image/tiff";
pub const MULTIPART_MIXED: &str = "multipart/mixed";
pub const TEXT_CSS: &str = "text/css";
pub const TEXT_CSV: &str = "text/csv";
pub const TEXT_HTML: &str = "text/html";
pub const TEXT_PLAIN: &str = "text/plain";
pub const TEXT_XML: &str = "text/xml";
pub const VIDEO_H264: &str = "video/H264";
pub const VIDEO_H265: &str = "video/H265";
pub const VIDEO_VP8: &str = "video/VP8";
pub const APPLICATION_X_HESSIAN: &str = "application/x-hessian";
pub const APPLICATION_X_JAVA_OBJECT: &str = "application/x-java-object";
pub const APPLICATION_CLOUDEVENTS_JSON: &str = "application/cloudevents+json";
pub const MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0: &str = "message/x.rsocket.tracing-zipkin.v0";
pub const MESSAGE_X_RSOCKET_ROUTING_V0: &str = "message/x.rsocket.routing.v0";
pub const MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0: &str = "message/x.rsocket.composite-metadata.v0";

lazy_static! {
    static ref MIME_MAP: HashMap<WellKnownMIME, (u8, &'static str)> = {
        let mut m = HashMap::new();
        m.insert(WellKnownMIME::ApplicationAvro, (0x00, APPLICATION_AVRO));
        m.insert(WellKnownMIME::ApplicationCbor, (0x01, APPLICATION_CBOR));
        m.insert(
            WellKnownMIME::ApplicationGraphql,
            (0x02, APPLICATION_GRAPHQL),
        );
        m.insert(WellKnownMIME::ApplicationGzip, (0x03, APPLICATION_GZIP));
        m.insert(
            WellKnownMIME::ApplicationJavascript,
            (0x04, APPLICATION_JAVASCRIPT),
        );
        m.insert(WellKnownMIME::ApplicationJson, (0x05, APPLICATION_JSON));
        m.insert(
            WellKnownMIME::ApplicationOctetStream,
            (0x06, APPLICATION_OCTET_STREAM),
        );
        m.insert(WellKnownMIME::ApplicationPdf, (0x07, APPLICATION_PDF));
        m.insert(
            WellKnownMIME::ApplicationVndApacheThriftBinary,
            (0x08, APPLICATION_THRIFT),
        );
        m.insert(
            WellKnownMIME::ApplicationVndGoogleProtobuf,
            (0x09, APPLICATION_PROTOBUF),
        );
        m.insert(WellKnownMIME::ApplicationXml, (0x0A, APPLICATION_XML));
        m.insert(WellKnownMIME::ApplicationZip, (0x0B, APPLICATION_ZIP));
        m.insert(WellKnownMIME::AudioAac, (0x0C, AUDIO_AAC));
        m.insert(WellKnownMIME::AudioMp3, (0x0D, AUDIO_MP3));
        m.insert(WellKnownMIME::AudioMp4, (0x0E, AUDIO_MP4));
        m.insert(WellKnownMIME::AudioMpeg3, (0x0F, AUDIO_MPEG3));
        m.insert(WellKnownMIME::AudioMpeg, (0x10, AUDIO_MPEG));
        m.insert(WellKnownMIME::AudioOgg, (0x11, AUDIO_OGG));
        m.insert(WellKnownMIME::AudioOpus, (0x12, AUDIO_OPUS));
        m.insert(WellKnownMIME::AudioVorbis, (0x13, AUDIO_VORBIS));
        m.insert(WellKnownMIME::ImageBmp, (0x14, IMAGE_BMP));
        m.insert(WellKnownMIME::ImageGif, (0x15, IMAGE_GIF));
        m.insert(
            WellKnownMIME::ImageHeicSequence,
            (0x16, IMAGE_HEIC_SEQUENCE),
        );
        m.insert(WellKnownMIME::ImageHeic, (0x17, IMAGE_HEIC));
        m.insert(
            WellKnownMIME::ImageHeifSequence,
            (0x18, IMAGE_HEIF_SEQUENCE),
        );
        m.insert(WellKnownMIME::ImageHeif, (0x19, IMAGE_HEIF));
        m.insert(WellKnownMIME::ImageJpeg, (0x1A, IMAGE_JPEG));
        m.insert(WellKnownMIME::ImagePng, (0x1B, IMAGE_PNG));
        m.insert(WellKnownMIME::ImageTiff, (0x1C, IMAGE_TIFF));
        m.insert(WellKnownMIME::MultipartMixed, (0x1D, MULTIPART_MIXED));
        m.insert(WellKnownMIME::TextCss, (0x1E, TEXT_CSS));
        m.insert(WellKnownMIME::TextCsv, (0x1F, TEXT_CSV));
        m.insert(WellKnownMIME::TextHtml, (0x20, TEXT_HTML));
        m.insert(WellKnownMIME::TextPlain, (0x21, TEXT_PLAIN));
        m.insert(WellKnownMIME::TextXml, (0x22, TEXT_XML));
        m.insert(WellKnownMIME::VideoH264, (0x23, VIDEO_H264));
        m.insert(WellKnownMIME::VideoH265, (0x24, VIDEO_H265));
        m.insert(WellKnownMIME::VideoVP8, (0x25, VIDEO_VP8));
        m.insert(
            WellKnownMIME::ApplicationXHessian,
            (0x26, APPLICATION_X_HESSIAN),
        );
        m.insert(
            WellKnownMIME::ApplicationXJavaObject,
            (0x27, APPLICATION_X_JAVA_OBJECT),
        );
        m.insert(
            WellKnownMIME::ApplicationCloudeventsJson,
            (0x28, APPLICATION_CLOUDEVENTS_JSON),
        );
        m.insert(
            WellKnownMIME::MessageXRSocketTracingZipkinV0,
            (0x7D, MESSAGE_X_RSOCKET_TRACING_ZIPKIN_V0),
        );
        m.insert(
            WellKnownMIME::MessageXRSocketRoutingV0,
            (0x7E, MESSAGE_X_RSOCKET_ROUTING_V0),
        );
        m.insert(
            WellKnownMIME::MessageXRsocketCompositeMetadataV0,
            (0x7F, MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0),
        );
        m
    };
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum WellKnownMIME {
    ApplicationAvro,
    ApplicationCbor,
    ApplicationGraphql,
    ApplicationGzip,
    ApplicationJavascript,
    ApplicationJson,
    ApplicationOctetStream,
    ApplicationPdf,
    ApplicationVndApacheThriftBinary,
    ApplicationVndGoogleProtobuf,
    ApplicationXml,
    ApplicationZip,
    AudioAac,
    AudioMp3,
    AudioMp4,
    AudioMpeg3,
    AudioMpeg,
    AudioOgg,
    AudioOpus,
    AudioVorbis,
    ImageBmp,
    ImageGif,
    ImageHeicSequence,
    ImageHeic,
    ImageHeifSequence,
    ImageHeif,
    ImageJpeg,
    ImagePng,
    ImageTiff,
    MultipartMixed,
    TextCss,
    TextCsv,
    TextHtml,
    TextPlain,
    TextXml,
    VideoH264,
    VideoH265,
    VideoVP8,
    ApplicationXHessian,
    ApplicationXJavaObject,
    ApplicationCloudeventsJson,
    MessageXRSocketTracingZipkinV0,
    MessageXRSocketRoutingV0,
    MessageXRsocketCompositeMetadataV0,
    Unknown,
}

impl WellKnownMIME {
    pub fn foreach(f: impl Fn(&WellKnownMIME)) {
        for k in MIME_MAP.keys() {
            f(k);
        }
    }

    pub fn str(&self) -> &'static str {
        let (_, s) = MIME_MAP.get(self).unwrap();
        s
    }

    pub fn raw(&self) -> u8 {
        let (v, _) = MIME_MAP.get(self).unwrap();
        *v
    }
}

impl From<String> for WellKnownMIME {
    fn from(s: String) -> WellKnownMIME {
        let mut result = WellKnownMIME::Unknown;
        for (k, v) in MIME_MAP.iter() {
            let (_, vv) = v;
            if *vv == s {
                result = k.clone();
                break;
            }
        }
        result
    }
}

impl From<&str> for WellKnownMIME {
    fn from(s: &str) -> WellKnownMIME {
        let mut result = WellKnownMIME::Unknown;
        for (k, v) in MIME_MAP.iter() {
            let (_, vv) = v;
            if *vv == s {
                result = k.clone();
                break;
            }
        }
        result
    }
}

impl From<u8> for WellKnownMIME {
    fn from(n: u8) -> WellKnownMIME {
        let mut result = WellKnownMIME::Unknown;
        for (k, v) in MIME_MAP.iter() {
            let (a, _) = v;
            if *a == n {
                result = k.clone();
                break;
            }
        }
        result
    }
}

impl fmt::Display for WellKnownMIME {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match MIME_MAP.get(self) {
            Some((_, v)) => write!(f, "{}", v),
            None => write!(f, "unknown"),
        }
    }
}
