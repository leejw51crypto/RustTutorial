use futures::future::lazy;
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;

use futures::sync::oneshot;

use futures::sync::mpsc::Receiver;
use futures::sync::mpsc::Sender;
use futures::try_ready;
use futures::{Async, Poll};
use jsonrpc_client_transports::transports::ws::connect;
use jsonrpc_client_transports::RawClient;
use jsonrpc_client_transports::RpcChannel;
use jsonrpc_client_transports::SubscriptionStream;
use jsonrpc_core::types::params::Params;
use serde_json::json;
use serde_json::map::Map;
use serde_json::Value;
use std::fmt;
use std::io::Read;
use std::thread;
use std::time;
use tokio::io;
use tokio::net::{tcp::ConnectFuture, TcpStream};
use tokio::runtime::Runtime;
use websocket::result::WebSocketError;
use websocket::OwnedMessage;
const CONNECTION: &'static str = "ws://localhost:26657/websocket";
#[derive(Clone)]
struct MyClient(RawClient);

impl From<RpcChannel> for MyClient {
    fn from(channel: RpcChannel) -> Self {
        MyClient(channel.into())
    }
}

pub struct ProcessFuture<T> {
    stream: T,
}
impl<T> ProcessFuture<T> {
    fn new(stream: T) -> ProcessFuture<T> {
        ProcessFuture { stream }
    }
}
impl<T> Future for ProcessFuture<T>
where
    T: Stream,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        println!("polling");
        loop {
            match try_ready!(self.stream.poll()) {
                Some(_value) => println!("got value"),

                None => {}
            };
        }
        Ok(Async::Ready(()))
    }
}

fn main() {
    let (channel_tx, channel_rx): (Sender<Value>, Receiver<Value>) = mpsc::channel(0);
    let mut rt = Runtime::new().unwrap();
    let a = connect::<MyClient>(CONNECTION);
    let b: MyClient = rt.block_on(a.unwrap()).unwrap();
    println!("connected..");

    let mut map = Map::new();
    map.insert("query".to_string(), "tm.event='NewBlock'".into());
    let fut = b.0.subscribe("subscribe", Params::Map(map), "NewBlock", "");
    let stream: SubscriptionStream = rt.block_on(fut).unwrap();

    println!("subscribed ok!");
    let m = ProcessFuture::new(stream).map_err(|e| {});
    tokio::run(m);
    /* stream.filter_map(|a| {
        None
    });*/
    println!("wait");
    loop {
        thread::sleep(time::Duration::from_millis(1000));
    }
    println!("done");
}
