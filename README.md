# `flake-graph`

[![built with nix](https://builtwithnix.org/badge.svg)](https://builtwithnix.org)

Visualize your Nix flake.lock!

This project provides facilities for parsing and analyzing `flake.lock` files.

## Usage

```
$ flake-graph flake.lock | dot -Tsvg > flake-graph.svg
```

## Sample

![image](https://gist.githubusercontent.com/nikitawootten/a0b5b3e0afdaaa8e02ace16b955da7ec/raw/flake-graph.svg)

Flake lock diagram of [`nikitawotten/infra`](https://github.com/nikitawootten/infra)
