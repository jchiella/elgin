{ pkgs ? import <nixos> {} }:
  pkgs.mkShell {
    buildInputs = [
      pkgs.cargo
      pkgs.llvm_10
      pkgs.libxml2
      pkgs.valgrind
      pkgs.gdb
      pkgs.clang
    ];
}
