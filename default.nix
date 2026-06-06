{
  pkgs ? import ./pkgs.nix,
  lib ? pkgs.lib
}:
let
  manifest = (lib.importTOML ./Cargo.toml).package;
in
pkgs.rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  doCheck = true;
  checkPhase = ''
    runHook preCheck
    cargoCheckHook
    runHook postCheck
  '';

  meta = {
    description = "Visualize your Nix flake.lock!";
    homepage = "https://github.com/nikitawootten/flake-graph";
    license = lib.licenses.mit;
    maintainers = [ ];
    mainProgram = manifest.name;
  };
}
