#[macro_use]
extern crate log;

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use rsocket_rust::prelude::{Flux, Payload, RSocket};
use tokio_stream::wrappers::ReceiverStream;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use async_stream::stream;
    use futures::Future;
    use rsocket_rust::prelude::*;
    use rsocket_rust::utils::EchoRSocket;
    use rsocket_rust::Client;
    use rsocket_rust_transport_tcp::{
        TcpClientTransport, TcpServerTransport, UnixClientTransport, UnixServerTransport,
    };
    use rsocket_rust_transport_websocket::{WebsocketClientTransport, WebsocketServerTransport};
    use serial_test::serial;
    use tokio::runtime::Runtime;

    use crate::TestSocket;

    #[serial]
    #[test]
    fn request_stream_can_be_cancelled_by_client_uds() {
        init_logger();
        with_uds_test_socket_run(request_stream_can_be_cancelled_by_client);
    }

    #[serial]
    #[test]
    fn request_stream_can_be_cancelled_by_client_tcp() {
        init_logger();
        with_tcp_test_socket_run(request_stream_can_be_cancelled_by_client);
    }

    #[serial]
    #[test]
    fn request_stream_can_be_cancelled_by_client_ws() {
        init_logger();
        with_ws_test_socket_run(request_stream_can_be_cancelled_by_client);
    }

    ///
    /// Client requests a channel, consumes an item and drops the stream handle.
    ///
    /// Amount of active streams is verified before and after requesting and after dropping.
    ///
    /// Before request_stream: 0 subscribers
    /// When request_stream is called: 1 subscriber
    /// When request_stream handle is dropped: 0 subscribers
    async fn request_stream_can_be_cancelled_by_client(client: Client) {
        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("0")
        );

        let mut results = client.request_stream(Payload::from(""));
        let payload = results.next().await.expect("valid payload").unwrap();
        assert_eq!(payload.metadata_utf8(), Some("subscribers: 1"));
        assert_eq!(payload.data_utf8(), Some("0"));

        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("1")
        );

        debug!("when the Flux is dropped");
        drop(results);
        // Give the server enough time to receive the CANCEL frame
        tokio::time::sleep(Duration::from_millis(250)).await;

        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("0")
        );
    }

    #[serial]
    #[test]
    fn request_channel_can_be_cancelled_by_client_uds() {
        init_logger();
        with_uds_test_socket_run(request_channel_can_be_cancelled_by_client);
    }

    #[serial]
    #[test]
    fn request_channel_can_be_cancelled_by_client_tcp() {
        init_logger();
        with_tcp_test_socket_run(request_channel_can_be_cancelled_by_client);
    }

    #[serial]
    #[test]
    fn request_channel_can_be_cancelled_by_client_ws() {
        init_logger();
        with_ws_test_socket_run(request_channel_can_be_cancelled_by_client);
    }

    ///
    /// Client requests a stream, consumes an item and drops the stream handle.
    ///
    /// Amount of active streams is verified before and after requesting and after dropping.
    ///
    /// Before request_channel: 0 subscribers
    /// When request_channel is called: 1 subscriber
    /// When request_channel handle is dropped: 0 subscribers
    async fn request_channel_can_be_cancelled_by_client(client: Client) {
        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("0")
        );

        let mut results = client.request_channel(stream! { yield Ok(Payload::from("")) }.boxed());
        let payload = results.next().await.expect("valid payload").unwrap();
        assert_eq!(payload.metadata_utf8(), Some("subscribers: 1"));
        assert_eq!(payload.data_utf8(), Some("0"));

        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("1")
        );

        debug!("when the Flux is dropped");
        drop(results);
        // Give the server enough time to receive the CANCEL frame
        tokio::time::sleep(Duration::from_millis(250)).await;

        assert_eq!(
            client
                .request_response(Payload::from("subscribers"))
                .await
                .unwrap()
                .unwrap()
                .data_utf8(),
            Some("0")
        );
    }

    fn init_logger() {
        let _ = env_logger::builder()
            .format_timestamp_millis()
            .filter_level(log::LevelFilter::Debug)
            // .is_test(true)
            .try_init();
    }

    /// Executes the [run_test] scenario using a client which is connected over a UDS transport to
    /// a TestSocket
    fn with_uds_test_socket_run<F, Fut>(run_test: F)
    where
        F: (FnOnce(Client) -> Fut) + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        info!("=====> begin uds");
        let server_runtime = Runtime::new().unwrap();

        server_runtime.spawn(async move {
            RSocketFactory::receive()
                .transport(UnixServerTransport::from(
                    "/tmp/rsocket-uds.sock".to_owned(),
                ))
                .acceptor(Box::new(|_setup, _socket| Ok(Box::new(TestSocket::new()))))
                .serve()
                .await
        });

        std::thread::sleep(Duration::from_millis(500));

        let client_runtime = Runtime::new().unwrap();

        client_runtime.block_on(async {
            let client = RSocketFactory::connect()
                .acceptor(Box::new(|| Box::new(EchoRSocket)))
                .transport(UnixClientTransport::from(
                    "/tmp/rsocket-uds.sock".to_owned(),
                ))
                .setup(Payload::from("READY!"))
                .mime_type("text/plain", "text/plain")
                .start()
                .await
                .unwrap();
            run_test(client).await;
        });
        info!("<===== uds done!");
    }

    /// Executes the [run_test] scenario using a client which is connected over a UDS transport to
    /// a TestSocket
    fn with_ws_test_socket_run<F, Fut>(run_test: F)
    where
        F: (FnOnce(Client) -> Fut) + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        info!("=====> begin ws");
        let server_runtime = Runtime::new().unwrap();
        server_runtime.spawn(async move {
            RSocketFactory::receive()
                .transport(WebsocketServerTransport::from("127.0.0.1:8080".to_owned()))
                .acceptor(Box::new(|_setup, _socket| Ok(Box::new(TestSocket::new()))))
                .serve()
                .await
        });

        std::thread::sleep(Duration::from_millis(500));

        let client_runtime = Runtime::new().unwrap();

        client_runtime.block_on(async {
            let client = RSocketFactory::connect()
                .acceptor(Box::new(|| Box::new(EchoRSocket)))
                .transport(WebsocketClientTransport::from("127.0.0.1:8080"))
                .setup(Payload::from("READY!"))
                .mime_type("text/plain", "text/plain")
                .start()
                .await
                .unwrap();

            run_test(client).await;
        });
        info!("<===== ws done!");
    }

    /// Executes the [run_test] scenario using a client which is connected over a TCP transport to
    /// a TestSocket
    fn with_tcp_test_socket_run<F, Fut>(run_test: F)
    where
        F: (FnOnce(Client) -> Fut) + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        info!("=====> begin tcp");
        let server_runtime = Runtime::new().unwrap();
        server_runtime.spawn(async move {
            RSocketFactory::receive()
                .transport(TcpServerTransport::from("127.0.0.1:7878".to_owned()))
                .acceptor(Box::new(|_setup, _socket| Ok(Box::new(TestSocket::new()))))
                .serve()
                .await
        });

        std::thread::sleep(Duration::from_millis(500));

        let client_runtime = Runtime::new().unwrap();

        client_runtime.block_on(async {
            let client = RSocketFactory::connect()
                .acceptor(Box::new(|| Box::new(EchoRSocket)))
                .transport(TcpClientTransport::from("127.0.0.1:7878".to_owned()))
                .setup(Payload::from("READY!"))
                .mime_type("text/plain", "text/plain")
                .start()
                .await
                .unwrap();
            run_test(client).await;
        });
        info!("<===== tpc done!");
    }
}

/// Stateful socket for tests, can be used to count active subscribers.
struct TestSocket {
    subscribers: Arc<Mutex<u32>>,
}

impl TestSocket {
    fn new() -> Self {
        TestSocket {
            subscribers: Arc::new(Mutex::new(0)),
        }
    }

    fn inc_subscriber_count(subscribers: &Arc<Mutex<u32>>) {
        let mut guard = subscribers.lock().unwrap();
        *guard = *guard + 1;
        info!(target: "TestSocket", "subscribers:({})", guard);
    }

    fn dec_subscriber_count(subscribers: &Arc<Mutex<u32>>) {
        let mut guard = subscribers.lock().unwrap();
        *guard = *guard - 1;
        info!(target: "TestSocket", "subscribers:({})", guard);
    }
}

#[async_trait]
impl RSocket for TestSocket {
    async fn metadata_push(&self, _req: Payload) -> Result<()> {
        unimplemented!();
    }

    async fn fire_and_forget(&self, _req: Payload) -> Result<()> {
        unimplemented!();
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        let subscribers = *self.subscribers.lock().unwrap();
        let response = match req.data_utf8() {
            Some("subscribers") => format!("{}", subscribers),
            _ => "Request payload did not contain a known key!".to_owned(),
        };
        Ok(Some(Payload::builder().set_data_utf8(&response).build()))
    }

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let subscribers = self.subscribers.clone();
        tokio::spawn(async move {
            TestSocket::inc_subscriber_count(&subscribers);

            for i in 0 as u32..100 {
                if tx.is_closed() {
                    debug!(target: "TestSocket", "tx is closed, break!");
                    break;
                }
                let payload = Payload::builder()
                    .set_data_utf8(format!("{}", i).as_str())
                    .set_metadata_utf8(
                        format!("subscribers: {}", *subscribers.lock().unwrap()).as_str(),
                    )
                    .build();
                tx.send(Ok(payload)).await.unwrap();
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            TestSocket::dec_subscriber_count(&subscribers);
        });

        ReceiverStream::new(rx).boxed()
    }

    fn request_channel(&self, _reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        self.request_stream(Payload::from(""))
    }
}
