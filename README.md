# RSRayTracingV2

## Testing locally

```
cargo run --release
```

On Linux you need to first run:

```
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev
```

## Compiling for the web

For running the `build_web.sh` script you also need to install `jq` and `binaryen` with your packet manager of choice.

``` sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```
