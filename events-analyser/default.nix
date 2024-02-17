{
  pkgs,
  naersk',
  version,
  git_revision,
  nativeBuildInputs,
  buildInputs,
}: rec {
  events-analyser = naersk'.buildPackage {
    name = "events-analyser";
    version = version;

    src = ./..;
    cargoBuildOptions = x: x ++ ["--package" "events-analyser"];

    nativeBuildInputs = nativeBuildInputs;
    buildInputs = buildInputs;

    overrideMain = p: {
      GIT_REVISION = git_revision;
    };
  };

  events-analyser-container-image = pkgs.dockerTools.buildImage {
    name = "supermusr-events-analyser";
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
      Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${events-analyser}/bin/events-analyser"];
      Env = [
        "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
        "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
      ];
    };
  };
}
