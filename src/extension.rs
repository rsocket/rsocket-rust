use std::fmt;

#[derive(PartialEq, Debug)]
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

impl From<u8> for WellKnownMIME {
    fn from(n: u8) -> WellKnownMIME {
        return match n {
            0x00 => WellKnownMIME::ApplicationAvro,
            0x01 => WellKnownMIME::ApplicationCbor,
            0x02 => WellKnownMIME::ApplicationGraphql,
            0x03 => WellKnownMIME::ApplicationGzip,
            0x04 => WellKnownMIME::ApplicationJavascript,
            0x05 => WellKnownMIME::ApplicationJson,
            0x06 => WellKnownMIME::ApplicationOctetStream,
            0x07 => WellKnownMIME::ApplicationPdf,
            0x08 => WellKnownMIME::ApplicationVndApacheThriftBinary,
            0x09 => WellKnownMIME::ApplicationVndGoogleProtobuf,
            0x0A => WellKnownMIME::ApplicationXml,
            0x0B => WellKnownMIME::ApplicationZip,
            0x0C => WellKnownMIME::AudioAac,
            0x0D => WellKnownMIME::AudioMp3,
            0x0E => WellKnownMIME::AudioMp4,
            0x0F => WellKnownMIME::AudioMpeg3,
            0x10 => WellKnownMIME::AudioMpeg,
            0x11 => WellKnownMIME::AudioOgg,
            0x12 => WellKnownMIME::AudioOpus,
            0x13 => WellKnownMIME::AudioVorbis,
            0x14 => WellKnownMIME::ImageBmp,
            0x15 => WellKnownMIME::ImageGif,
            0x16 => WellKnownMIME::ImageHeicSequence,
            0x17 => WellKnownMIME::ImageHeic,
            0x18 => WellKnownMIME::ImageHeifSequence,
            0x19 => WellKnownMIME::ImageHeif,
            0x1A => WellKnownMIME::ImageJpeg,
            0x1B => WellKnownMIME::ImagePng,
            0x1C => WellKnownMIME::ImageTiff,
            0x1D => WellKnownMIME::MultipartMixed,
            0x1E => WellKnownMIME::TextCss,
            0x1F => WellKnownMIME::TextCsv,
            0x20 => WellKnownMIME::TextHtml,
            0x21 => WellKnownMIME::TextPlain,
            0x22 => WellKnownMIME::TextXml,
            0x23 => WellKnownMIME::VideoH264,
            0x24 => WellKnownMIME::VideoH265,
            0x25 => WellKnownMIME::VideoVP8,
            0x26 => WellKnownMIME::ApplicationXHessian,
            0x27 => WellKnownMIME::ApplicationXJavaObject,
            0x28 => WellKnownMIME::ApplicationCloudeventsJson,
            0x7D => WellKnownMIME::MessageXRSocketTracingZipkinV0,
            0x7E => WellKnownMIME::MessageXRSocketRoutingV0,
            0x7F => WellKnownMIME::MessageXRsocketCompositeMetadataV0,
            _ => WellKnownMIME::Unknown,
        };
    }
}

impl fmt::Display for WellKnownMIME {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return match *self {
            WellKnownMIME::ApplicationAvro => write!(f, "{}", "application/avro"),
            WellKnownMIME::ApplicationCbor => write!(f, "{}", "application/cbor"),
            WellKnownMIME::ApplicationGraphql => write!(f, "{}", "application/graphql"),
            WellKnownMIME::ApplicationGzip => write!(f, "{}", "application/gzip"),
            WellKnownMIME::ApplicationJavascript => write!(f, "{}", "application/javascript"),
            WellKnownMIME::ApplicationJson => write!(f, "{}", "application/json"),
            WellKnownMIME::ApplicationOctetStream => write!(f, "{}", "application/octet-stream"),
            WellKnownMIME::ApplicationPdf => write!(f, "{}", "application/pdf"),
            WellKnownMIME::ApplicationVndApacheThriftBinary => {
                write!(f, "{}", "application/vnd.apache.thrift.binary")
            }
            WellKnownMIME::ApplicationVndGoogleProtobuf => {
                write!(f, "{}", "application/vnd.google.protobuf")
            }
            WellKnownMIME::ApplicationXml => write!(f, "{}", "application/xml"),
            WellKnownMIME::ApplicationZip => write!(f, "{}", "application/zip"),
            WellKnownMIME::AudioAac => write!(f, "{}", "audio/aac"),
            WellKnownMIME::AudioMp3 => write!(f, "{}", "audio/mp3"),
            WellKnownMIME::AudioMp4 => write!(f, "{}", "audio/mp4"),
            WellKnownMIME::AudioMpeg3 => write!(f, "{}", "audio/mpeg3"),
            WellKnownMIME::AudioMpeg => write!(f, "{}", "audio/mpeg"),
            WellKnownMIME::AudioOgg => write!(f, "{}", "audio/ogg"),
            WellKnownMIME::AudioOpus => write!(f, "{}", "audio/opus"),
            WellKnownMIME::AudioVorbis => write!(f, "{}", "audio/vorbis"),
            WellKnownMIME::ImageBmp => write!(f, "{}", "image/bmp"),
            WellKnownMIME::ImageGif => write!(f, "{}", "image/gif"),
            WellKnownMIME::ImageHeicSequence => write!(f, "{}", "image/heic-sequence"),
            WellKnownMIME::ImageHeic => write!(f, "{}", "image/heic"),
            WellKnownMIME::ImageHeifSequence => write!(f, "{}", "image/heif-sequence"),
            WellKnownMIME::ImageHeif => write!(f, "{}", "image/heif"),
            WellKnownMIME::ImageJpeg => write!(f, "{}", "image/jpeg"),
            WellKnownMIME::ImagePng => write!(f, "{}", "image/png"),
            WellKnownMIME::ImageTiff => write!(f, "{}", "image/tiff"),
            WellKnownMIME::MultipartMixed => write!(f, "{}", "multipart/mixed"),
            WellKnownMIME::TextCss => write!(f, "{}", "text/css"),
            WellKnownMIME::TextCsv => write!(f, "{}", "text/csv"),
            WellKnownMIME::TextHtml => write!(f, "{}", "text/html"),
            WellKnownMIME::TextPlain => write!(f, "{}", "text/plain"),
            WellKnownMIME::TextXml => write!(f, "{}", "text/xml"),
            WellKnownMIME::VideoH264 => write!(f, "{}", "video/H264"),
            WellKnownMIME::VideoH265 => write!(f, "{}", "video/H265"),
            WellKnownMIME::VideoVP8 => write!(f, "{}", "video/VP8"),
            WellKnownMIME::ApplicationXHessian => write!(f, "{}", "application/x-hessian"),
            WellKnownMIME::ApplicationXJavaObject => write!(f, "{}", "application/x-java-object"),
            WellKnownMIME::ApplicationCloudeventsJson => {
                write!(f, "{}", "application/cloudevents+json")
            }
            WellKnownMIME::MessageXRSocketTracingZipkinV0 => {
                write!(f, "{}", "message/x.rsocket.tracing-zipkin.v0")
            }
            WellKnownMIME::MessageXRSocketRoutingV0 => {
                write!(f, "{}", "message/x.rsocket.routing.v0")
            }
            WellKnownMIME::MessageXRsocketCompositeMetadataV0 => {
                write!(f, "{}", "message/x.rsocket.composite-metadata.v0")
            }
            _ => write!(f, "{}", "unknown"),
        };
    }
}

pub struct CompositeMetadata {}
