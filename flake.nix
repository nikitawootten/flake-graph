{
  description = "Visualize your flake.lock";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }: let 
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      # forEachSystem [ "x86_64-linux" ] (_: { example = true; }) -> { x86_64-linux.example = true }
      forEachSystem = nixpkgs.lib.genAttrs systems;
  in {
    packages = forEachSystem (system: {
      default = nixpkgs.legacyPackages.${system}.callPackage ./. { };
    });
    devShells = forEachSystem (system: {
      default = nixpkgs.legacyPackages.${system}.callPackage ./shell.nix { };
    });
  };
}
