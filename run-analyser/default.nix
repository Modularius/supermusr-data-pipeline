{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  run-analyser = naersk'.buildPackage {
    name = "events-analyser";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "run-analyser"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  run-analyser-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-run-analyser";
    tag = "latest";
    created = "now";

    copyToRoot = pkgs.buildEnv {
      name = "image-root";
      paths = with pkgs; [bashInteractive coreutils];
      pathsToLink = ["/bin"];
    };

    config = {
      ExposedPorts = {
        "9090/tcp" = {};
      };
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${run-analyser}/bin/run-analyser"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
