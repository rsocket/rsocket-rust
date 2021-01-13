use js_sys::{Promise, Uint8Array};
use rsocket_rust::prelude::*;
use rsocket_rust::Client;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};
use wasm_bindgen_futures::future_to_promise;

use super::client::WebsocketClientTransport;

#[derive(Serialize, Deserialize)]
pub struct JsPayload {
    data: Option<Vec<u8>>,
    metadata: Option<Vec<u8>>,
}

#[wasm_bindgen]
pub struct JsClient {
    inner: Client,
}

impl Into<JsValue> for &JsPayload {
    fn into(self) -> JsValue {
        JsValue::from_serde(self).unwrap()
    }
}

impl Into<Payload> for JsPayload {
    fn into(self) -> Payload {
        let mut bu = Payload::builder();
        if let Some(v) = self.data {
            bu = bu.set_data(v);
        }
        if let Some(v) = self.metadata {
            bu = bu.set_metadata(v);
        }
        bu.build()
    }
}

impl From<Payload> for JsPayload {
    fn from(input: Payload) -> JsPayload {
        let (d, m) = input.split();
        JsPayload {
            data: d.map(|v| v.to_vec()),
            metadata: m.map(|v| v.to_vec()),
        }
    }
}

#[wasm_bindgen]
pub fn new_payload(data: JsValue, metadata: JsValue) -> JsValue {
    let jp = JsPayload {
        data: to_vec(data),
        metadata: to_vec(metadata),
    };
    (&jp).into()
}

#[wasm_bindgen]
pub async fn connect(url: String) -> Result<JsClient, JsValue> {
    match RSocketFactory::connect()
        .transport(WebsocketClientTransport::from(url))
        .start()
        .await
    {
        Ok(inner) => Ok(JsClient { inner }),
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}

#[wasm_bindgen]
impl JsClient {
    pub fn request_response(&self, request: &JsValue) -> Promise {
        let inner = self.inner.clone();
        let request: JsPayload = request.into_serde().unwrap();
        future_to_promise(async move {
            match inner.request_response(request.into()).await {
                Ok(Some(v)) => {
                    let jp = JsPayload::from(v);
                    Ok((&jp).into())
                }
                Ok(None) => Ok(JsValue::UNDEFINED),
                Err(e) => Err(JsValue::from(&format!("{:?}", e))),
            }
        })
    }

    pub fn fire_and_forget(&self, request: &JsValue) -> Promise {
        let inner = self.inner.clone();
        let request: JsPayload = request.into_serde().unwrap();

        future_to_promise(async move {
            let _ = inner.fire_and_forget(request.into()).await;
            Ok(JsValue::NULL)
        })
    }
}

#[inline]
fn to_vec(input: JsValue) -> Option<Vec<u8>> {
    if input.is_null() || input.is_undefined() {
        None
    } else if input.is_string() {
        input.as_string().map(|s| s.into_bytes())
    } else {
        Some(Uint8Array::from(input).to_vec())
    }
}
