
use std::net::SocketAddr;

use futures::{Future, future};
use log::{info, error};
use pretty_env_logger;
use rmp_rpc::{Client, serve, Service, Value};
use tokio::net::TcpStream;

#[derive(Clone)]
pub struct Master;

impl Service for Master {
    type RequestFuture = Box<Future<Item = Value, Error = Value> + 'static + Send>;

    fn handle_request(&mut self, method: &str, params: &[Value]) -> Self::RequestFuture {
        match method {
            "register" => {
                info!("REG: {}", params[0].as_str().unwrap_or(""));
                Box::new(future::ok("".into()))
            },
            method => {
                error!("Unknown method: {}", method);
                Box::new(future::err("Unknown method".into()))
            },
        }
    }

    fn handle_notification(&mut self, method: &str, params: &[Value]) {
        match method {
            "heartbeat" => info!("HB {}", params[0]),
            method => error!("Unknown notification: {} from {}", method, params[0]),
        }
    }
}

fn main() {
    pretty_env_logger::init();
    let addr: SocketAddr = "127.0.0.1:54321".parse().unwrap();

    // Create a future that connects to the server, and send a notification and a request.
    let client = TcpStream::connect(&addr)
        .or_else(|e| {
            error!("I/O error in the client: {}", e);
            Err(())
        })
        .and_then(move |stream| {
            let client = Client::new(stream);

            // Use the client to send a notification.
            // The future returned by client.notify() finishes when the notification
            // has been sent, in case we care about that. We can also just drop it.
            client.notify("subscribe", &["audio".into(), "system_status".into(), "observation_data".into()]);

            // client.notifu("unsubscribe", &["audio".into()]);

            // Use the client to send a request with the method "dostuff", and two parameters:
            // the string "foo" and the integer "42".
            // The future returned by client.request() finishes when the response
            // is received.
            client
                .request("dostuff", &["foo".into(), 42.into()])
                .and_then(|response| {
                    info!("Response: {:?}", response);
                    Ok(())
                })
        });

    tokio::run(client);
}
