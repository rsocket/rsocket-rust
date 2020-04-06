use bytes::BytesMut;
use futures_channel::oneshot;
use futures_util::StreamExt;
use js_sys::{ArrayBuffer, Uint8Array};
use rsocket_rust::error::RSocketError;
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{ClientTransport, Rx, Tx, TxOnce};
use rsocket_rust::utils::Writeable;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ErrorEvent, Event, FileReader, MessageEvent, ProgressEvent, WebSocket};

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

impl WebsocketClientTransport {
    #[inline]
    fn wait_for_open(ws: &WebSocket) -> impl Future<Output = ()> {
        let (sender, receiver) = oneshot::channel();
        // The Closure is only called once, so we can use Closure::once
        let on_open = Closure::once(move |_e: Event| {
            // We don't need to send a value, so we just send ()
            sender.send(()).unwrap();
        });
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        async move {
            // Wait for it to open
            receiver.await.unwrap();
            // Clean up the Closure so we don't leak any memory
            drop(on_open);
        }
    }
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        mut sending: Rx<Frame>,
        connected: Option<TxOnce<Result<(), RSocketError>>>,
    ) {
        spawn_local(async move {
            match WebSocket::new(&self.url) {
                Ok(ws) => {
                    // on message
                    let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                        let data: JsValue = e.data();
                        read_binary(data, incoming.clone());
                    })
                        as Box<dyn FnMut(MessageEvent)>);
                    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                    on_message.forget();

                    // on error
                    let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
                        console_log!("websocket error: {}", e.message());
                    })
                        as Box<dyn FnMut(ErrorEvent)>);
                    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
                    on_error.forget();

                    // on_close
                    let on_close = Closure::once(Box::new(move |_e: Event| {
                        console_log!("websocket closed");
                    }) as Box<dyn FnMut(Event)>);
                    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
                    on_close.forget();

                    Self::wait_for_open(&ws).await;

                    if let Some(sender) = connected {
                        sender.send(Ok(())).unwrap();
                    }

                    while let Some(v) = sending.next().await {
                        let mut bf = BytesMut::new();
                        v.write_to(&mut bf);
                        let raw = bf.to_vec();
                        ws.send_with_u8_array(&raw[..])
                            .expect("write data into websocket failed.");
                    }
                    console_log!("***** attch end *****");
                }
                Err(e) => {
                    if let Some(sender) = connected {
                        sender
                            .send(Err(RSocketError::from(e.as_string().unwrap())))
                            .unwrap();
                    }
                }
            }
        });
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
