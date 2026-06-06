# `flake-graph`

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

Visualize your Nix flake.lock!

This project provides facilities for parsing and analyzing `flake.lock` files.

## Usage

```sh
$ nix shell github:nikitawootten/flake-graph nixpkgs#graphviz \
    --command sh -c 'flake-graph flake.lock | dot -Tsvg > flake-lock.svg'
```

### Input sizes

Pass `--size` to annotate each input with the on-disk size of its source in the Nix store.

```sh
$ nix shell github:nikitawootten/flake-graph nixpkgs#graphviz \
    --command sh -c 'flake-graph --size flake.lock | dot -Tsvg > flake-lock.svg'
```

This invokes `nix` to resolve each input and may fetch inputs from the network if they are not already in the store.

## Sample

![image](https://gist.githubusercontent.com/nikitawootten/a0b5b3e0afdaaa8e02ace16b955da7ec/raw/flake-graph.svg)

Flake lock diagram of [`nikitawotten/infra`](https://github.com/nikitawootten/infra)
