use super::runtime::WASMSpawner;
use bytes::BytesMut;
use js_sys::{ArrayBuffer, Uint8Array};
use rsocket_rust::frame::Frame;
use rsocket_rust::prelude::*;
use rsocket_rust::transport::{BoxResult, ClientTransport, Rx, SafeFuture, Tx};
use rsocket_rust::utils::Writeable;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, FileReader, MessageEvent, ProgressEvent, WebSocket};

pub struct WebsocketClientTransport {
    url: String,
}

impl WebsocketClientTransport {
    fn read_binary(value: JsValue, incoming: Tx<Frame>) {
        let reader = FileReader::new().unwrap_throw();

        let state = Rc::new(RefCell::new(None));

        let onload = {
            let state = state.clone();
            let reader = reader.clone();

            Closure::once(move |_: ProgressEvent| {
                *state.borrow_mut() = None;
                let data: ArrayBuffer = reader.result().unwrap_throw().unchecked_into();
                let raw: Vec<u8> = Uint8Array::new(&data).to_vec();
                // Use data...
                let mut bf = BytesMut::from(&raw[..]);
                let msg = Frame::decode(&mut bf).unwrap();
                incoming.send(msg).unwrap();
            })
        };

        let onerror = {
            let state = state.clone();
            Closure::once(move |_: ErrorEvent| {
                *state.borrow_mut() = None;
                // let err = e.error();
                // let error = reader.error().unwrap_throw();
                // TODO: Handle error...
            })
        };

        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        *state.borrow_mut() = Some((onload, onerror));

        reader
            .read_as_array_buffer(value.as_ref().unchecked_ref())
            .unwrap_throw();
    }
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(self, incoming: Tx<Frame>, mut sending: Rx<Frame>) -> SafeFuture<BoxResult<()>> {
        Box::pin(async move {
            let (sender, receiver) = channel::<Frame>();
            WASMSpawner.spawn(async move {
                while let Some(msg) = sending.recv().await {
                    sender.send(msg).unwrap()
                }
            });

            // Connect to an echo server
            let ws = WebSocket::new(&self.url).unwrap();

            // create callback
            let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
                let data: JsValue = e.data();
                Self::read_binary(data, incoming.clone());
            }) as Box<dyn FnMut(MessageEvent)>);

            // set message event handler on WebSocket
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            // forget the callback to keep it alive
            onmessage_callback.forget();

            let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
                // TODO: handle error
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            let cloned_ws = ws.clone();
            let onopen_callback = Closure::wrap(Box::new(move |_| {
                while let Ok(f) = receiver.recv() {
                    let mut bf = BytesMut::new();
                    f.write_to(&mut bf);
                    let mut raw = bf.to_vec();
                    cloned_ws.send_with_u8_array(&mut raw[..]).unwrap();
                }
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
            Ok(())
        })
    }
}

impl<I> From<I> for WebsocketClientTransport
where
    I: Into<String>,
{
    fn from(url: I) -> WebsocketClientTransport {
        WebsocketClientTransport { url: url.into() }
    }
}

// #[wasm_bindgen]
// pub fn start_websocket(url: &str) -> Result<(), JsValue> {
//     let s = url.to_owned();
//     spawn_local(async move {
//         let tp = WebsocketClientTransport::from(&s);
//         let cli = RSocketFactory::connect()
//             .transport(tp)
//             .setup(Payload::from("Hello WASM!"))
//             .start_with_runtime(WASMSpawner)
//             .await
//             .unwrap();
//         let res = cli
//             .request_response(Payload::from("Nice to meet you!"))
//             .await
//             .unwrap();
//         log("!!!!!!!!---&&&&&&");
//         log(&format!("Yes => {:?}", res));
//     });

//     Ok(())
// }
