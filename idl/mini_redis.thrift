namespace rs mini_redis

enum RequestType {
    Get,
    Set,
    Del,
    Ping,
    Subscribe,
    publish,
}

struct RedisRequest {
    1: optional string key,
    2: optional string value,
    3: optional i32 expire_time,
    4: optional list<string> channels,
    5: optional bool block,
    6: required RequestType request_type,
}

enum ResponseType {
    Print,
    Trap,
}

struct RedisResponse {
    1: required ResponseType response_type,
    2: optional string value,
}

service RedisService {
    RedisResponse RedisCommand(1: RedisRequest req),
}