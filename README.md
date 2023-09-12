# mini-redis

A simple mini-redis with six actions: set, get, del, ping, subscribe, publish

WARNING: This mini_redis is ran over a filter, any action that contains "Genshin" will be blocked. Try it out!

## Usage

first build , set the cli and start the server.

```bash
cargo build
source set_cli.sh
server
```

Then try out the six actions in another terminal by using the client

```bash
client set/get/del/ping/subscribe/publish [args]
```

mini-redis also support set expire time when setting:
```
client set zju 114514 ex 10
-----------wait 10 secs----------
client get zju
```

Or just run the test case in `test.sh`
```
source test.sh
```
![](images/README/2023-09-12-21-10-58.png#pic)

we can also try out the subscribe and publish. Here is a simple demo:
![](images/README/2023-09-12-21-14-18.png#pic)

WARNING: You'd better terminate the server after all the subscribers are terminated. It will take the server a bit long to terminate after get Ctrl-C.

