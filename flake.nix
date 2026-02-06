{
  description = "A niri shell built with iced-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixgl.url = "github:nix-community/nixGL";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    nixgl,
    git-hooks,
    ...
  }: let
    winitRuntimeLibs = pkgs:
      with pkgs; [
        wayland
        libxkbcommon

        vulkan-loader
        vulkan-headers
        libGL
      ];
    mkPackage = pkgs: let
      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      runtimeLibs = winitRuntimeLibs pkgs;
    in
      pkgs.rustPlatform.buildRustPackage {
        pname = manifest.name;
        version = manifest.version;
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        # buildtime
        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
        ];

        # runtime
        buildInputs = runtimeLibs;
        postFixup = ''
          wrapProgram $out/bin/${manifest.name} \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeLibs}"
        '';
        meta = with pkgs.lib; {
          description = "A niri shell built with rust and iced-rs";
          license = licenses.mit;
          mainProgram = manifest.name;
        };
      };
  in
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.git-hooks.flakeModule
      ];
      systems = ["x86_64-linux"];
      perSystem = {
        config,
        system,
        lib,
        ...
      }: let
        # Manifest via Cargo.toml
        manifest = (lib.importTOML ./Cargo.toml).package;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            nixgl.overlay
          ];
        };
        runtimeLibs = winitRuntimeLibs pkgs;
      in {
        packages.default = mkPackage pkgs;
        formatter.default = pkgs.alejandra;

        pre-commit.settings.hooks.alejandra.enable = true;
        pre-commit.settings.hooks.clippy.enable = true;
        pre-commit.settings.hooks.rustfmt.enable = true;

        devShells.default = pkgs.mkShell {
          name = "${manifest.name}";

          nativeBuildInputs =
            (with pkgs; [
              cargo
              cargo-generate
              cargo-watch
              clippy
              rustc
              rustfmt

              openssl
            ])
            ++ [pkgs.nixgl.nixGLMesa]
            ++ runtimeLibs;

          WINIT_UNIX_BACKEND = "wayland";
          LD_LIBRARY_PATH = lib.makeLibraryPath (
            (with pkgs; [
              gcc
              libiconv
              llvmPackages.llvm
            ])
            ++ runtimeLibs
          );
          LIBCLANG_PATH = lib.makeLibraryPath [pkgs.libclang];
          NIX_LDFLAGS = "-L${pkgs.libiconv}/lib";
          NIXGL = "${pkgs.nixgl.nixGLMesa}/bin/nixGLMesa";

          RUST_BACKTRACE = "full";
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

          shellHook = ''
            ${config.pre-commit.shellHook}

            function menu () {
              echo
              echo -e "\033[1;34m>==> ️  '$name' shell\n\033[0m"
              just --list
              echo
              echo "(Run 'just --list' to display this menu again)"
              echo
            }

            menu
            just --list
          '';
        };
      };
      flake = {
        overlays.default = final: prev: let
          manifest = (prev.lib.importTOML ./Cargo.toml).package;
        in {
          ${manifest.name} = mkPackage final;
        };
      };
    };
}
