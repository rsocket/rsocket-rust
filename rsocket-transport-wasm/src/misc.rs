use super::client::WebsocketClientTransport;
use super::runtime::WASMSpawner;
use bytes::Bytes;
use js_sys::{Promise, Uint8Array};
use rsocket_rust::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub struct JsPayload {
    d: Option<Uint8Array>,
    m: Option<Uint8Array>,
}

#[wasm_bindgen]
pub struct JsClient {
    inner: Client<WASMSpawner>,
}

impl From<Payload> for JsPayload {
    fn from(input: Payload) -> JsPayload {
        let d = match input.data() {
            Some(raw) => Some(Uint8Array::from(raw.as_ref())),
            None => None,
        };
        let m = match input.metadata() {
            Some(raw) => Some(Uint8Array::from(raw.as_ref())),
            None => None,
        };
        JsPayload { d, m }
    }
}

impl Into<Payload> for JsPayload {
    fn into(self) -> Payload {
        let mut bu = Payload::builder();
        if let Some(raw) = self.d {
            bu = bu.set_data(Bytes::from(raw.to_vec()));
        }
        if let Some(raw) = self.m {
            bu = bu.set_metadata(Bytes::from(raw.to_vec()));
        }
        bu.build()
    }
}

#[wasm_bindgen]
impl JsPayload {
    #[wasm_bindgen(constructor)]
    pub fn new(data: JsValue, metadata: JsValue) -> JsPayload {
        JsPayload {
            d: Self::to_uint8array(data),
            m: Self::to_uint8array(metadata),
        }
    }

    pub fn data(&self) -> JsValue {
        match self.d {
            Some(ref data) => data.clone().into(),
            None => JsValue::null(),
        }
    }

    pub fn metadata(&self) -> JsValue {
        match self.m {
            Some(ref raw) => raw.clone().into(),
            None => JsValue::null(),
        }
    }

    #[inline]
    fn to_uint8array(input: JsValue) -> Option<Uint8Array> {
        if input.is_null() || input.is_undefined() {
            None
        } else {
            Some(Uint8Array::from(input))
        }
    }
}

#[wasm_bindgen]
impl JsClient {
    pub async fn connect(url: String) -> Result<JsClient, JsValue> {
        let inner = RSocketFactory::connect()
            .transport(WebsocketClientTransport::from(url))
            .start_with_runtime(WASMSpawner)
            .await
            .unwrap();
        Ok(JsClient { inner })
    }

    pub fn bingo(&self) -> Promise {
        future_to_promise(async { Ok(JsValue::from("bingo")) })
    }

    pub async fn request_response(self, request: JsPayload) -> Result<JsPayload, JsValue> {
        match self.inner.request_response(request.into()).await {
            Ok(response) => Ok(JsPayload::from(response)),
            Err(e) => Err(JsValue::from(&format!("{:?}", e))),
        }
    }
}
