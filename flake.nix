{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    fenix.url = "github:nix-community/fenix";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  # https://devenv.sh/reference/options/
                  languages.rust = {
                    enable = true;
                    channel = "nightly";
                  };

                  packages = with pkgs; [
                    gnuplot_qt
                    openssl
                  ];

                  pre-commit = {
                    # settings.clippy = {
                    #   allFeatures = true;
                    #   offline = false;
                    # };
                    hooks = {
                      rustfmt.enable = true;
                      # clippy.enable = true;
                      # cargo test
                      # "cargo-test" = {
                      #   enable = true;
                      #   name = "cargo test";
                      #   description = "Run cargo test";
                      #   entry = "${self.devShells.${system}.default.config.languages.rust.toolchain.cargo}/bin/cargo test";
                      #   fail_fast = true;
                      #   pass_filenames = false;
                      #   stages = [ "manual" ];
                      # };
                    };
                  };
                }
              ];
            };
          });
    };
}