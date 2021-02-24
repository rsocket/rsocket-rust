#[macro_export]
macro_rules! composite {
    ($($x:expr,$y:expr),+) => {
        {
            let mut b = $crate::extension::CompositeMetadata::builder();
            $(
                b = b.push($x.into(),$y);
            )*
            b.build()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite() {
        let c = composite!("application/json", "123", "text/plain", "ccc");
        let vv: Vec<_> = c
            .iter()
            .map(|it| {
                format!(
                    "{}={}",
                    it.get_mime_type(),
                    it.get_metadata_utf8().unwrap_or_default()
                )
            })
            .collect();
        assert_eq!(vv.join(","), "application/json=123,text/plain=ccc");
    }
}
