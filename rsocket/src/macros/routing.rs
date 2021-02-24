#[macro_export]
macro_rules! tags {
    ($($v:expr),+) => {
        {
            let mut b = $crate::extension::RoutingMetadata::builder();
            $(
                b = b.push_str($v);
            )*
            b.build()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing() {
        let t = tags!("a", "b", "c");
        let tags = t.get_tags();
        println!("{:?}", tags);
        assert_eq!("a,b,c", t.get_tags().join(","))
    }
}
