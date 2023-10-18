{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk.url = "github:nix-community/naersk";
  };
/*
  outputs = { self, nixpkgs }:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; overlays = [ (import ./overlays) ]; };
        inherit system;
      });
    in
    {
      devShells = forAllSystems ({ pkgs, system }: {
        default = pkgs.mkShell {
          packages = (with pkgs; [
            flatbuffers
            rustup
            cmake
            ninja
            zlib
            zstd
            rdkafka
            tdengine
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
          */
  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [
            (import ./nix/overlays/hdf5.nix)
            (import ./nix/overlays/tdengine.nix)
          ];
        };

        toolchain = fenix.packages.${system}.toolchainOf {
          channel = "1.72";
          date = "2023-09-19";
          sha256 = "dxE7lmCFWlq0nl/wKcmYvpP9zqQbBitAQgZ1zx9Ooik=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain.rust;
          rustc = toolchain.rust;
        };

        ws_cargo_toml = builtins.readFile ./Cargo.toml;
        ws_cargo = builtins.fromTOML ws_cargo_toml;

        version = ws_cargo.workspace.package.version;
        git_revision = self.shortRev or self.dirtyShortRev;

        hdf5-joined = pkgs.symlinkJoin {
          name = "hdf5";
          paths = with pkgs; [hdf5 hdf5.dev];
        };
        nativeBuildInputs = with pkgs; [cmake flatbuffers hdf5-joined perl tcl pkg-config];
        buildInputs = with pkgs; [openssl cyrus_sasl hdf5-joined];
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = nativeBuildInputs ++ [toolchain.toolchain];
          packages = with pkgs; [nix skopeo alejandra treefmt];
        };

        packages =
          {
            clippy = naersk'.buildPackage {
              src = ./.;
              nativeBuildInputs = nativeBuildInputs;
              buildInputs = buildInputs;
              HDF5_DIR = "${hdf5-joined}";
              mode = "clippy";
            };

            test = naersk'.buildPackage {
              src = ./.;
              nativeBuildInputs = nativeBuildInputs;
              buildInputs = buildInputs;
              HDF5_DIR = "${hdf5-joined}";
              mode = "test";
              # Ensure detailed test output appears in nix build log
              cargoTestOptions = x: x ++ ["1>&2"];
            };
          }
          // import ./events-to-histogram {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./kafka-daq-report {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./simulator {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;}
          // import ./stream-to-file {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs hdf5-joined;}
          // import ./trace-archiver {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs hdf5-joined;}
          // import ./trace-to-events {inherit pkgs naersk' version git_revision nativeBuildInputs buildInputs;};
      }
    );
}