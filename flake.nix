{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Read the file relative to the flake's root
        overrides = (builtins.fromTOML (builtins.readFile (self + "/rust-toolchain.toml")));
        libPath = with pkgs; lib.makeLibraryPath [
          # load external libraries that you need in your rust project here
          # X11 and OpenGL libraries for egui
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          libxkbcommon
          openssl
          mesa     # OpenGL implementation
          mesa.drivers # GPU drivers
          libGL    # OpenGL shared library
          libGLU   # OpenGL Utility library
        ];
      in
      {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [ 
            pkg-config 
            xorg.xorgserver # For Xvfb
          ];
          buildInputs = with pkgs; [
            clang
            llvmPackages.bintools
            rustup
            rust-analyzer
            wayland
            
            # Required dependencies for egui with glow backend
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libxcb        # Includes libxcb-render0-dev, libxcb-shape0-dev, libxcb-xfixes0-dev
            libxkbcommon       # libxkbcommon-dev
            openssl            # libssl-dev
            
            # OpenGL libraries
            mesa               # OpenGL implementation
            libGL              # OpenGL shared library
            libGLU             # OpenGL Utility library
            freeglut           # OpenGL utility toolkit
            glxinfo            # GLX information utility
            virtualgl          # X server GL provider
          ];

          RUSTC_VERSION = overrides.toolchain.channel;
          
          # https://github.com/rust-lang/rust-bindgen#environment-variables
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
          
          # OpenGL environment variables for better compatibility
          LIBGL_ALWAYS_SOFTWARE = "1";     # Force software rendering
          LIBGL_DEBUG = "verbose";         # Verbose logging for OpenGL
          
          shellHook = ''
            echo "Ensuring Rust toolchain '$RUSTC_VERSION' and component 'rust-analyzer' are installed..."
            rustup toolchain install $RUSTC_VERSION
            rustup component add rust-analyzer --toolchain $RUSTC_VERSION
            echo "Rust setup complete."
            echo "Required dependencies for egui with glow backend have been loaded."
            echo "OpenGL software rendering is enabled."
            
            # Simple OpenGL detection
            if command -v glxinfo >/dev/null 2>&1; then
              echo "OpenGL information:"
              glxinfo | grep "OpenGL vendor\|OpenGL renderer\|OpenGL version" || echo "Unable to detect OpenGL info"
            fi
          '';

          # Add precompiled library to rustc search path
          RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
            # add libraries here (e.g. pkgs.libvmi)
          ]);
          
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);

          
          # Add glibc, clang, glib, and other headers to bindgen search path
          BINDGEN_EXTRA_CLANG_ARGS =
            let
              includeFlgs = builtins.map (p: ''-I"${p}/include"'') [
                pkgs.glibc.dev
                pkgs.llvmPackages_latest.libclang.lib # for bindgen
                pkgs.glib.dev # glib
              ];
            in
            builtins.concatStringsSep " " includeFlgs;
        };
      }
    );
} 