{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      allSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit system; };
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
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);

          nativeBuildInputs = [
            #(import ./tdengine/default.nix { stdenv = pkgs.stdenv; }).default
          ];

          shellHooks = ''
          '';
        };
      });
    };
}
