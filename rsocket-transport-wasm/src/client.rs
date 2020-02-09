use bytes::BytesMut;
use js_sys::{ArrayBuffer, Uint8Array};
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{BoxResult, ClientTransport, Rx, SafeFuture, Tx};
use rsocket_rust::utils::Writeable;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, FileReader, MessageEvent, ProgressEvent, WebSocket};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub struct WebsocketClientTransport {
    url: String,
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(self, incoming: Tx<Frame>, mut sending: Rx<Frame>) -> SafeFuture<BoxResult<()>> {
        Box::pin(async move {
            let sending = RefCell::new(sending);
            // Connect to an echo server
            let ws = WebSocket::new(&self.url).unwrap();

            // on message
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                let data: JsValue = e.data();
                read_binary(data, incoming.clone());
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();

            // on error
            let on_error = Closure::wrap(Box::new(move |_e: ErrorEvent| {
                // TODO: handle error
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            on_error.forget();

            // on open
            let cloned_ws = ws.clone();
            let on_open = Closure::wrap(Box::new(move |_| {
                let mut sending = sending.borrow_mut();
                while let Ok(Some(f)) = sending.try_next() {
                    let mut bf = BytesMut::new();
                    f.write_to(&mut bf);
                    let mut raw = bf.to_vec();
                    cloned_ws.send_with_u8_array(&mut raw[..]).unwrap();
                }
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();

            console_log!("***** attch end *****");

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

#[inline]
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
            incoming.unbounded_send(msg).unwrap();
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
