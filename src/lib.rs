#![feature(impl_trait_in_assoc_type)]
use futures::future::select_all;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tracing::info;
use volo::FastStr;
pub struct S {
    pub map: Mutex<HashMap<String, String>>,
    pub channels: Mutex<HashMap<String, broadcast::Sender<String>>>,
}

use volo_gen::mini_redis::{RedisResponse, RequestType, ResponseType};

#[volo::async_trait]
impl volo_gen::mini_redis::RedisService for S {
    async fn redis_command(
        &self,
        _req: volo_gen::mini_redis::RedisRequest,
    ) -> ::core::result::Result<volo_gen::mini_redis::RedisResponse, ::volo_thrift::AnyhowError>
    {
        info!("enter");
        match _req.request_type {
            RequestType::Set => {
                self.map.lock().unwrap().insert(
                    _req.key.unwrap().into_string(),
                    _req.value.unwrap().into_string(),
                );
                Ok(RedisResponse {
                    value: Some("Ok".into()),
                    response_type: ResponseType::Print,
                })
            }
            RequestType::Get => {
                match self
                    .map
                    .lock()
                    .unwrap()
                    .get(&_req.key.unwrap().into_string())
                {
                    Some(v) => Ok(RedisResponse {
                        value: Some(FastStr::from(v.clone())),
                        response_type: ResponseType::Print,
                    }),
                    None => Ok(RedisResponse {
                        value: Some("nil".into()),
                        response_type: ResponseType::Print,
                    }),
                }
            }
            RequestType::Del => {
                match self
                    .map
                    .lock()
                    .unwrap()
                    .remove(&_req.key.unwrap().into_string())
                {
                    Some(_) => Ok(RedisResponse {
                        value: Some("Ok".into()),
                        response_type: ResponseType::Print,
                    }),
                    None => Ok(RedisResponse {
                        value: Some("nil".into()),
                        response_type: ResponseType::Print,
                    }),
                }
            }
            RequestType::Ping => Ok(RedisResponse {
                value: Some("PONG".into()),
                response_type: ResponseType::Print,
            }),
            RequestType::Subscribe => match _req.block.unwrap() {
                true => {
                    let mut vec = self
                        .channels
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|(k, _v)| {
                            _req.channels
                                .as_ref()
                                .unwrap()
                                .contains(&FastStr::from((*k).clone()))
                        })
                        .map(|(k, v)| (v.subscribe(), k.clone()))
                        .collect::<Vec<_>>();
                    let (res, index, _) =
                        select_all(vec.iter_mut().map(|(rx, _name)| Box::pin(rx.recv()))).await;
                    match res {
                        Ok(info) => Ok(RedisResponse {
                            value: Some(
                                (String::from("from ") + &vec[index].1 + ": " + &info).into(),
                            ),
                            response_type: ResponseType::Trap,
                        }),
                        Err(_) => Ok(RedisResponse {
                            value: None,
                            response_type: ResponseType::Trap,
                        }),
                    }
                }
                false => {
                    for channel in _req.channels.unwrap() {
                        if !self
                            .channels
                            .lock()
                            .unwrap()
                            .contains_key(&channel.clone().into_string())
                        {
                            let (tx, _) = broadcast::channel(10);
                            self.channels
                                .lock()
                                .unwrap()
                                .insert(channel.clone().into_string(), tx);
                        }
                    }
                    Ok(RedisResponse {
                        value: Some("Ok".into()),
                        response_type: ResponseType::Trap,
                    })
                }
            },
            RequestType::Publish => {
                let channel = _req.channels.unwrap()[0].clone().into_string();
                let _ = self
                    .channels
                    .lock()
                    .unwrap()
                    .get(&channel)
                    .unwrap()
                    .send(_req.value.unwrap().into_string())
                    .unwrap();
                info!("send over");
                Ok(RedisResponse {
                    value: Some("Ok".into()),
                    response_type: ResponseType::Print,
                })
            }
        }
    }
}

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let now = std::time::Instant::now();
        tracing::debug!("Received request {:?}", &req);
        let resp = self.0.call(cx, req).await;
        tracing::debug!("Sent response {:?}", &resp);
        tracing::info!("Request took {}ms", now.elapsed().as_millis());
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}

#[derive(Clone)]
pub struct FilterService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for FilterService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    anyhow::Error: Into<S::Error>,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let info = format!("{:?}", req);
        if info.contains("Genshin") {
            Err(anyhow::anyhow!("Genshin is forbidden. OP show show way!").into())
        } else {
            self.0.call(cx, req).await
        }
    }
}

pub struct FilterLayer;

impl<S> volo::Layer<S> for FilterLayer {
    type Service = FilterService<S>;

    fn layer(self, inner: S) -> Self::Service {
        FilterService(inner)
    }
}
