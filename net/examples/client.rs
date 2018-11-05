
use std::net::SocketAddr;

use futures::Future;
use log::{info, error};
use pretty_env_logger;
use rmp_rpc::Client;
use tokio::net::TcpStream;

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
            client.notify("ping", &["Rocinante".into()]);

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
