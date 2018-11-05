use futures;
use log;
use rmp_rpc;
use tokio;


use futures::{future, Future, Stream};
use log::{info, error};
use rmp_rpc::{serve, ServiceWithClient, Client, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server;

impl ServiceWithClient for Server {
    type RequestFuture = Box<Future<Item = Value, Error = Value> + 'static + Send>;

    fn handle_request(
        &mut self,
        client: &mut Client,
        method: &str,
        params: &[Value],
    ) -> Self::RequestFuture {
        match method {
            // Upon receiving a "ping", send a "pong" back. Only after we get a response back from
            // "pong", we return the empty string.
            "ping" => {
                let id = params[0].as_i64().unwrap();
                info!("received ping({}), sending pong", id);
                let request = client
                    .request("pong", &[id.into()])
                    // After we get the "pong" back, send back an empty string.
                    .and_then(|_| Ok("".into()))
                    .map_err(|_| "".into());

                Box::new(request)
            }
            // Upon receiving a "pong" increment our pong counter and send the empty string back
            // immediately.
            "pong" => {
                let id = params[0].as_i64().unwrap();
                info!("received pong({}), incrementing pong counter", id);
                //*self.value.lock().unwrap() += 1;
                Box::new(future::ok("".into()))
            }
            method => {
                let err = format!("Invalid method {}", method).into();
                Box::new(future::err(err))
            }
        }
    }

    fn handle_notification(&mut self, _: &mut Client, method: &str, params: &[Value]) {
        match method {
            "ping" => info!("PING {}", params[0]),
            method => error!("Unknown method: {}", method),
        }
    }
}
