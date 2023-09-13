#![feature(impl_trait_in_assoc_type)]

use mini_redis::FilterLayer;
use mini_redis::S;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast;

#[volo::main]
async fn main() {
    let port = std::env::args().nth(1).unwrap_or(String::from("8081"));
    println!("{}",port);
    let addr: SocketAddr = ("[::]:".to_owned() +  &port).parse().unwrap();
    let addr = volo::net::Address::from(addr);
    volo_gen::mini_redis::RedisServiceServer::new(S {
        port: port.parse().unwrap(),
        map: Arc::new(Mutex::new(HashMap::<String, String>::new())),
        channels: Mutex::new(HashMap::<String, broadcast::Sender<String>>::new()),
    })
    // .layer_front(LogLayer)
    .layer_front(FilterLayer)
    .run(addr)
    .await
    .unwrap();
}
