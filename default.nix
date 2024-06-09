{
  lib,
  darwin,
  stdenv,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  inherit ((builtins.fromTOML (builtins.readFile ./Cargo.toml)).package) name version;
  src = lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
  buildInputs = lib.optionals stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [ SystemConfiguration ]
  );
}
