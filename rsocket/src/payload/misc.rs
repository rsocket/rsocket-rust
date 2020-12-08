use bytes::Bytes;

#[inline]
pub(crate) fn bytes_to_utf8(input: &Option<Bytes>) -> Option<&str> {
    match input {
        Some(b) => std::str::from_utf8(b).ok(),
        None => None,
    }
}
