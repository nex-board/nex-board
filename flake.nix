{
  description = "Electrical-Bulletin-Board";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        toolchain = fenix.packages.${system}.default.toolchain;
        pkgs = nixpkgs.legacyPackages.${system};
        linux-deps = with pkgs; [
          alsa-lib
          udev
          wayland
          wayland-protocols
          libxkbcommon
          mesa
          vulkan-loader
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            toolchain
            rust-analyzer
            glfw
            lldb
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux linux-deps;
        };

        packages.default = pkgs.buildFHSEnv {
          name = "nex-board-fhs";
          targetPkgs = pkgs: [
            (self.packages.${system}.nex-board-unwrapped)
            pkgs.glfw
            pkgs.pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux linux-deps;
          runScript = "nex-board";
        };

        packages.nex-board-unwrapped =
          (pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
            rustfmt = toolchain;
          }).buildRustPackage
            {
              pname = "nex-board";
              version = "0.2.0";
              src = ./.;
              rpath = true;
              cargoLock.lockFile = ./Cargo.lock;
              nativeBuildInputs = [
                toolchain
                pkgs.libclang
                pkgs.pkg-config
              ];
              propagatedBuildInputs = with pkgs; [
                openssl
              ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux linux-deps;
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath ([
                pkgs.libclang
                pkgs.pkg-config
              ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux linux-deps);
              postFixup = (if pkgs.stdenv.isLinux then ''
                lib_path="${
                  pkgs.lib.makeLibraryPath [
                    pkgs.wayland
                    pkgs.wayland-protocols
                    pkgs.alsa-lib
                    pkgs.udev
                    pkgs.libxkbcommon
                    pkgs.glfw
                    pkgs.libxkbfile
                  ]};"
              '' else "");
              postInstall = ''
                          cp -r assets $out/bin/assets
              '';
          };
      }
    );
}
