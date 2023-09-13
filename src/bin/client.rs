// use lazy_static::lazy_static;
use mini_redis::FilterLayer;
use std::net::SocketAddr;
use std::sync::Arc;
use structopt::StructOpt;
use volo::FastStr;
use volo_gen::mini_redis::{RedisRequest, RequestType};
use mini_redis::arg::{Args,Opt};

// lazy_static! {
//     static ref CLIENT: volo_gen::mini_redis::RedisServiceClient = {
//         let addr: SocketAddr = "127.0.0.1:8087".parse().unwrap();
//         volo_gen::mini_redis::RedisServiceClientBuilder::new("redis")
//             // .layer_outer(LogLayer)
//             .layer_outer(FilterLayer)
//             .address(addr)
//             .build()
//     };
// }
#[volo::main]
async fn main() {
    let args = Args::from_args();
    tracing_subscriber::fmt::init();
    // let mut args: Vec<String> = std::env::args().collect();
    let client: volo_gen::mini_redis::RedisServiceClient = {
        let addr: SocketAddr = ("127.0.0.1:".to_owned() + &args.port).parse().unwrap();
        volo_gen::mini_redis::RedisServiceClientBuilder::new("redis")
            // .layer_outer(LogLayer)
            .layer_outer(FilterLayer)
            .address(addr)
            .build()
    };
    let base_request = RedisRequest {
        key: None,
        value: None,
        request_type: RequestType::Ping,
        expire_time: None,
        channels: None,
        block: None,
    };
    let req = match args.cmd {
        Opt::Set { key, value , ex} => RedisRequest {
            key: Some(FastStr::from(Arc::new(key))),
            value: Some(FastStr::from(Arc::new(value))),
            request_type: RequestType::Set,
            expire_time: ex,
            ..base_request
        },
        Opt::Get { key } => RedisRequest {
            key: Some(FastStr::from(Arc::new(key))),
            request_type: RequestType::Get,
            ..base_request
        },
        Opt::Del { key } => RedisRequest {
            key: Some(FastStr::from(Arc::new(key))),
            request_type: RequestType::Del,
            ..base_request
        },
        Opt::Ping { value } => RedisRequest {
            request_type: RequestType::Ping,
            value: match value {
                Some(v) => Some(FastStr::from(Arc::new(v))),
                None => None,
            },
            ..base_request
        },
        Opt::Subscribe { channel, and } => {
            let mut channels = vec![channel];
            if let Some(and) = and {
                channels.extend(and);
            }
            RedisRequest {
                request_type: RequestType::Subscribe,
                channels: Some(
                    channels
                        .drain(..)
                        .map(|x| FastStr::from(Arc::new(x)))
                        .collect(),
                ),
                block: Some(false),
                ..base_request
            }
        },
        Opt::Publish { channel, value } => RedisRequest {
            value: Some(FastStr::from(Arc::new(value))),
            request_type: RequestType::Publish,
            channels: Some(vec![FastStr::from(Arc::new(channel))]),
            ..base_request
        },
    };
    let resp = client.redis_command(req.clone()).await;
    match resp {
        Ok(resp) => match resp.response_type {
            volo_gen::mini_redis::ResponseType::Print => {
                println!("{}", resp.value.unwrap())
            }
            volo_gen::mini_redis::ResponseType::Trap => {
                println!(
                    "subscribe {} channels",
                    req.channels.as_ref().unwrap().len()
                );
                loop {
                    let req = RedisRequest {
                        block: Some(true),
                        ..req.clone()
                    };
                    let resp = client.redis_command(req).await;
                    match resp {
                        Ok(info) => {
                            println!("{}", info.value.unwrap());
                        }
                        _ => {
                            println!("error");
                        }
                    }
                }
            }
        },
        Err(e) => match e {
            volo_thrift::ResponseError::Application(err) => {
                println!("{}", err.message)
            }
            _ => {
                tracing::error!("{:?}", e);
            }
        },
    }
}
