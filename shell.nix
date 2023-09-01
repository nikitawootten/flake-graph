{ pkgs ? import ./pkgs.nix }:
pkgs.mkShell {
  inputsFrom = [ (pkgs.callPackage ./default.nix { }) ];
  buildInputs = with pkgs; [
    rust-analyzer
    rustfmt
    clippy
    graphviz
  ];
  shellHook = ''
    export RUST_BACKTRACE="1"
  '';
}