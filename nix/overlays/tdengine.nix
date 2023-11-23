self: super: {
  tdengine = let
    version = "3.0.4.2";
    #version = "3.2.1.0";
  in
    super.gcc9Stdenv.mkDerivation {
      name = "TDengine";
      version = version;

      src = super.fetchFromGitHub {
        owner = "taosdata";
        repo = "TDEngine";
        rev = "ver-${version}";
        hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE="; #v3.0.4.2
        #hash = "sha256-/g6LGpMmCmhO46C8mRvliOMdErKQcuiuOgo7d4WbWG8="; #v3.2.1.0
      };
      
      /*
      src = super.fetchgit  {
        url = "https://github.com/taosdata/TDengine";
        #owner = "taosdata";
        #repo = "TDEngine";
        rev = "ver-${version}";
        hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE=";
        fetchSubmodules = true;
      };
      */
      #dontUseCmakeConfigure = true;

      SSL_CERT_FILE = "${super.cacert}/etc/ssl/certs/ca-bundle.crt";
      outputHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      outputHashMode = "recursive";

      #cmakeFlags = ["-DBUILD_TEST=off" "-DBUILD_HTTP=on"];# "-DBUILD_TOOLS=off" "-DBUILD_CONTRIB=off" "-DBUILD_WITH_COS=off" "-DBUILD_WITH_GEOS=off"];

      nativeBuildInputs = with super; [
        cmake
        cacert
        git
      ];
      preConfig = ''
      '';

      installPhase = ''
        substituteInPlace ../../source/packaging/tools/make_install.sh --replace "/usr" "$out/usr"
        substituteInPlace ../../source/packaging/tools/make_install.sh --replace "/var" "$out/var"
        substituteInPlace ../../source/packaging/tools/make_install.sh --replace "/etc" "$out/etc"
        make install
        cp $out/usr/local/taos/driver/libtaos.so.${version} $out/usr/local/taos/driver/libtaos.so
        cp $out/usr/local/taos/driver/libtaos.so.${version} $out/usr/local/taos/driver/libtaos.so.1
      '';

      outputs = ["out"];
    };
}
