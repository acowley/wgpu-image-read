{
  description = "Efficient manipulation of panoramic depth images";
  
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system: 
      let overlays = [(import rust-overlay)];
          pkgs = import nixpkgs { inherit system overlays; };
          rust-analyzer = pkgs.rust-bin.nightly.latest.rust-analyzer-preview;
          rustc = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src"];
          };
          rust = builtins.attrValues { 
            inherit (pkgs.rust-bin.stable.latest) rustfmt clippy;
            inherit rustc;
          };
          vulkan = builtins.attrValues {
            inherit (pkgs) pkgconfig libglvnd mesa glslang 
              vulkan-loader vulkan-headers vulkan-tools vulkan-validation-layers;
          };
      in {
      devShell = pkgs.mkShell {
        buildInputs = (pkgs.lib.optional pkgs.stdenv.isLinux pkgs.llvmPackages_latest.lld)
          ++ [ rust-analyzer pkgs.feh ]
          ++ rust
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (
            let frameworks = {inherit (pkgs.darwin.apple_sdk.frameworks) CoreGraphics CoreVideo AppKit;};
            in [pkgs.darwin.objc4 pkgs.libiconv] ++ builtins.attrValues frameworks)
          ++ pkgs.lib.optionals pkgs.stdenv.isLinux [pkgs.vulkan-loader pkgs.vulkan-validation-layers];
        shellHook = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
          export LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib''${LD_LIBRARY_PATH:+:''${LD_LIBRARY_PATH}}"
        '';
      };
      defaultPackage.${system} = self.devShell;
    });
}
