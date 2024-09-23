# Test VLC Net

This module is mainly convenient for testing vlc network. It combines vlc and gossip for constructing the multi-node network.   

And some features as follow:
- Configuration: It uses the variety commands to define detail of the vlc network.
- Demonstrate: The test_vlc_net will collect useful data metric in work period.
- Streamlined workflow: This program keep concise and core workflows for better testing.

## Compile server node

```bash
# if enable the tokio console function, need export env variate. Then build
export RUSTFLAGS="--cfg tokio_unstable"

cargo build -p test_vlc_net
cd target/debug
```

## Generate config files

## DHT feature

1. If use dht as its discovery protocol, please generate `bootnode` config.

```bash
./test_vlc_net  --server server0.json generate
cat server0.json
```

2. generate other business node config.

```bash
for I in {1..4}
do
    PORT=$((9600 + I))
    TOKIO_CONSOLE_PORT=$((6669 + I))
    ./test_vlc_net  --server server"$I".json generate --host /ip4/127.0.0.1/tcp/ --port "$PORT" --topic vlc --trigger-us 1000 --bootnodes "/ip4/127.0.0.1/tcp/9601" --enable-tx-send --tokio-console-port "$TOKIO_CONSOLE_PORT"
done
```

Replace the `--bootnodes` value by `multi_addr` field in bootnode config file  `server0.json`.
eg: `/ip4/127.0.0.1/tcp/9600/p2p/12D3KooWQt4eiRVEZGFcThutsLvUbJS2rgLJ9QnE1ptp9ZmjRMay` 

## Run multi-node network

If DHT enable, first setup bootnode, `nohup ./test_vlc_net  --server server0.json run >>server0.log 2>&1 &` 

```bash
for I in {1..4}
do
    nohup ./test_vlc_net  --server server"$I".json run >>server.log 2>&1 &
done

# if enable the tokio console function for watching debug metric
cargo install tokio-console
tokio-console
```
## Watch the metric

```bash
tail -f server.log

# or filter node connected num
tail -F server.log | grep --line-buffered "Connected node num"
```

## Exit all node

```bash
ps aux | grep -i 'server ' | awk '{print $2}' | xargs kill
```