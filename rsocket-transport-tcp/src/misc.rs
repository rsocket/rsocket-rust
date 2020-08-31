pub(crate) fn parse_uds_addr(addr: impl AsRef<str>) -> String {
    let addr = addr.as_ref();
    if addr.starts_with("unix://") || addr.starts_with("UNIX://") {
        addr.chars().skip(7).collect::<String>()
    } else {
        addr.to_owned()
    }
}

pub(crate) fn parse_tcp_addr(addr: impl AsRef<str>) -> String {
    let addr = addr.as_ref();
    if addr.starts_with("tcp://") || addr.starts_with("TCP://") {
        addr.chars().skip(6).collect::<String>()
    } else {
        addr.to_owned()
    }
}
