# Chronos

## Overview

The chronos is a novel logical clock system designed for open networks with Byzantine participants, offering improved fault tolerance and performance. It introduces a customizable validator abstraction and has been successfully applied to develop decentralized applications with minimal overhead.

This repository is the developing prototype of Hetu chronos, a general-propose message-passing framework which consider causality as first class citizen.

## Layout & Toc

Chronos organization layout is as follows:

* `crates/` the common dependences and core functional crates.
* `demos/` some use cases of applied the Chronos and proposals.
* `docs/` design and applied documents for demonstrating thoughts.
* `src/` the source codebase of shared definitions.

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