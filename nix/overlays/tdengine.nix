self: super: {
  tdengine = let
    #version = "3.0.4.2";
    version = "3.2.1.0";
  in
    super.gcc11Stdenv.mkDerivation {
      name = "TDengine";
      version = version;

      src = super.fetchFromGitHub {
        owner = "taosdata";
        repo = "TDEngine";
        rev = "ver-${version}";
        #hash = "sha256-CMpfaVhq3LOngugxp9POvXIQMjtpgwqP1VoCj2KkfYE="; #v3.0.4.2
        hash = "sha256-/g6LGpMmCmhO46C8mRvliOMdErKQcuiuOgo7d4WbWG8="; #v3.2.1.0
      };
      nativeBuildInputs = with super; [
        cmake
        cacert
      ];
/*
      SSL_CERT_FILE = "${super.cacert}/etc/ssl/certs/ca-bundle.crt";
      outputHashAlgo = "sha256";
      outputHash = "sha256-Kgxib68y0Vsfn+wGUVKM84JeSUDQMdBJHbljX+BEpRM=";
      outputHashMode = "recursive";
*/
      preConfigure =
      let 
        cjson-version = "1.7.15";
        lz4-version = "1.9.3";
        zlib-version = "1.2.11";
        libuv-version = "1.44.2";
        geos-version = "3.12.0";
        cos-version = "5.0.16";
      in let
       packages = {
          cos-c-sdk-v5 = {
            owner = "tencentyun";
            repo = "cos-c-sdk-v5";
            rev = "v${cos-version}";
            sha256 = "sha256-SDZcy72Y3/f+3iw5lhLBt4kTBU0DdoA5EMOeGY7V1UQ=";
          };
          geos = {
            owner = "libgeos";
            repo = "geos";
            rev = "${geos-version}";
            sha256 = "sha256-y5Pb+b3WeMTqC2nwz2Jzi0WwPb89zUs8Nb1VNauuG6A=";
          };
          cJson = {
            owner = "DaveGamble";
            repo = "cJSON";
            rev = "v${cjson-version}";
            sha256 = "sha256-PpUVsLklcs5hCCsQcsXw0oEVIWecKnQO16Hy0Ba8ov8=";
          };
          lz4 = {
            owner = "taosdata-contrib";
            repo = "lz4";
            rev = "v${lz4-version}";
            sha256 = "sha256-f7GZgOzUrkAfw1mqwlIKQQqDvkvIahGlHvq6AL+aAvA=";
          };
          zlib = {
            owner = "taosdata-contrib";
            repo = "zlib";
            rev = "v${zlib-version}";
            sha256 = "sha256-j5b6aki1ztrzfCqu8y729sPar8GpyQWIrajdzpJC+ww=";
          };
          libuv = {
            owner = "libuv";
            repo = "libuv";
            rev = "v${libuv-version}";
            sha256 = "sha256-K6v+00basjI32ON27ZjC5spQi/zWCcslDwQwyosq2iY=";
          };
        };
      in super.lib.concatStringsSep "\n" (
            super.lib.mapAttrsToList (name: params:
              "cp -r ${super.fetchFromGitHub params} contrib/${name} \n chmod -R +w contrib/${name}"
            ) packages
        );
      dontUseCmakeConfigure = true;
      configurePhase = ''
        runHook preConfigure
        mkdir debug
        cd debug
        cmake .. -DBUILD_HTTP=on -DBUILD_WITH_COS=off -DBUILD_WITH_GEOS=off -DBUILD_WITH_S3=off -DCMAKE_BUILD_TYPE=Release
        runHook postConfigure
      '';
      buildPhase = ''
        runHook preBuild
        make taos -j18
        runHook postBuild
      '';
      installPhase = ''
        runHook preInstall
        cp -R . $out
        runHook postInstall
      '';
    };
}
