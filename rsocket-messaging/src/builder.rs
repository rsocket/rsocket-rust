use super::requester::Requester;
use rsocket_rust::extension::MimeType;

pub struct RequesterBuilder {
    data_mime_type: MimeType,
    route: Option<String>,
    data: Option<Vec<u8>>,
}

impl Default for RequesterBuilder {
    fn default() -> Self {
        Self {
            data_mime_type: MimeType::APPLICATION_JSON,
            route: None,
            data: None,
        }
    }
}

impl RequesterBuilder {
    pub fn data_mime_type<I>(mut self, mime_type: I) -> Self
    where
        I: Into<MimeType>,
    {
        self.data_mime_type = mime_type.into();
        self
    }

    pub fn setup_route<I>(mut self, route: I) -> Self
    where
        I: Into<String>,
    {
        self.route = Some(route.into());
        self
    }

    pub fn setup_data<D>(mut self, data: D) -> Self
    where
        D: Into<Vec<u8>>,
    {
        self.data = Some(data.into());
        self
    }

    pub fn build(self) -> Requester {
        todo!("build requester")
    }
}
