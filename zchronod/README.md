# zchronod

## Overview

The zchronod or chronos is a implement of vlc(verifiable logical clock).

It use the [zeb](https://github.com/hetu-project/zeb) p2p relay as network module. And as a backend project node for supporting verifiable logical clock and causality ordering. This system is currently in poc stage.

## Dependences

### PostgreDB

The zchronod depends on postgre db for data persistence, so please install postgre and setup a pg instance. 

### Zeb p2p relayer

The zchronod play a role of inner logic and state layer in vlc overview. One zchronod process matches a zeb p2p node, and them use inner net socket for communication.

For now, zchronod and zeb use the same node identity for two processes. So first generate a key pair identity, then address it to `node_id` in [config-tempelete.yaml](../zchronod/config-tempelete.yaml) of zchronod.

### Net messaging

The zchronod and zeb using protobuf proto3 as serialization compression algorithm and communication protocol. More messages body details, please see [crates/protos](../crates/protos/) for check it.

## Compile

### Build from source

```bash
git clone https://github.com/NagaraTech/chronos.git

cd chronos

cargo build -p zchronod
```

## Run a node

```bash
# 1.for help info
./target/debug/zchronod -h

# 2.init db & dna business pg tables
./target/debug/zchronod --init_pg postgres://postgres:hetu@0.0.0.0:5432/vlc_inner_db

# 3.setup the node & please update the config file to your dev environment
./target/debug/zchronod --config ./zchronod/config-tempelete.yaml
```

## How to test

```shell
cargo run --package zchronod --bin client_write
cargo run --package zchronod --bin client_read
```
