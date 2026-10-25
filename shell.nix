{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    cmake
    pkg-config
    llvmPackages.libclang
    llvmPackages.libllvm
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
