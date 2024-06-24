# Chronos

## Overview

This repository is the developing prototype of Hetu chronos, a general-propose message-passing framework which consider causality as first class citizen.

The chronos is a novel logical clock system designed for open networks with Byzantine participants, offering improved fault tolerance and performance. It introduces a customizable validator abstraction and has been successfully applied to develop decentralized applications with minimal overhead.

## Layout & Toc

Chronos organization layout is as follows:

* [`crates/`](./crates/) the common dependences and core functional crates folder.
* [`demos/`](./demos/) some use cases of applied the Chronos and proposals.
* [`docs/`](./docs/) design and applied documents for demonstrating thoughts.
* [`src/`](./src/) the source codebase of shared definitions and some common codes.

## Features

The Chronos is a novel verifiable logical clock system that can target many problems that can't be handled by the regular logical clock. 

Here are some core new features are provided by the Chronos:

* [x] Programmable and verifiable vector logic clock
* [x] Provides networks events causality partially order graph capability 
* [x] High-performance replication state machine with logical clock Byzantine fault tolerance
* [x] Three verifiable warranties are provided: clock update proof, monotonicity proof, application-specific proof
* [x] Three validation backends are supported: quorum certificate, trusted hardware (TEE), verifiable computation (ZKP)

Please refer to [hetu key research](https://github.com/hetu-project#hetu-key-research) for more details

## Applied scenarios

Regular logical clock have been applied in many scenarios. As follows:

- Weakly consistent storage systems
    - [Cops: Causal Consistency Data Storage](https://www.cs.cmu.edu/~dga/papers/cops-sosp2011.pdf)
- Broadcast events with causally ordered 
    - [Reference: verifiable clock & p2p combined or optimizing](https://github.com/hetu-project/docs/blob/main/Zeb/vlc.md) 
- Deadlock detection
    - Mutual exclusion of shared resources in a distributed system
    - Bakery algorithm
- Distributed snapshots
- Distributed system debugging.

For sure, the verifiable logical clock is an enhanced version of a regular logical clock that can do everything a regular clock can do.


## Building

```sh
git clone https://github.com/hetu-project/chronos.git

cd chronos

cargo build
```

## Contributing

We welcome all contributions! There are many ways to contribute to the project, including but not limited to:

- Cloning code repo and opening a [PR](https://github.com/hetu-project/chronos/pulls).
- Submitting feature requests or [bugs](https://github.com/hetu-project/chronos/issues).
- Improving our product or contribution [documentation](./docs/).
- Contributing [use cases](./demos/) to a feature request.

## Contact

- [Open an Issue](https://github.com/hetu-project/chronos/issues)
- [Hetu Protocol](https://github.com/hetu-project#hetu-key-research)
- [Follow us on Twitter](https://x.com/hetu_protocol)