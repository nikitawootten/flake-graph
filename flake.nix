{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }: let 
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      # forEachSystem [ "x86_64-linux" ] { example = true; } -> { x86_64-linux.example = true }
      forEachSystem = nixpkgs.lib.genAttrs systems;
  in {
    devShells = forEachSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustfmt
          clippy

          graphviz
        ];
      };
    });
  };
}
