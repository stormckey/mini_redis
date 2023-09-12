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
    let addr: SocketAddr = "[::]:8087".parse().unwrap();
    let addr = volo::net::Address::from(addr);
    volo_gen::mini_redis::RedisServiceServer::new(S {
        map: Arc::new(Mutex::new(HashMap::<String, String>::new())),
        channels: Mutex::new(HashMap::<String, broadcast::Sender<String>>::new()),
    })
    // .layer_front(LogLayer)
    .layer_front(FilterLayer)
    .run(addr)
    .await
    .unwrap();
}
