{pkgs, ...}:
pkgs.runCommand "rustfmt-check" {
  nativeBuildInputs = [pkgs.rustfmt];
} ''
  cp -r ${../../src} ./src
  find ./src -name "*.rs" | xargs rustfmt --check --config tab_spaces=2 --edition 2021
  touch $out
''
