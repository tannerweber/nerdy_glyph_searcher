{
  lib,
  rustPlatform,
  installShellFiles,
}:

let
  name = "nerdy_glyph_searcher";
in
rustPlatform.buildRustPackage {
  name = name;
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-iISx+bAhvABMrkG1+HwKW77p1UkFIMktpLY/lYubl6s=";

  nativeBuildInputs = [ installShellFiles ];
  postInstall = ''
    installShellCompletion --cmd ${name} \
      --bash <($out/bin/${name} --shell-completion bash) \
      --fish <($out/bin/${name} --shell-completion fish) \
      --zsh <($out/bin/${name} --shell-completion zsh)
  '';

  meta = {
    description = "Search through font glyphs in the terminal.";
    repository = "https://github.com/tannerweber/nerdy_glyph_searcher";
    license = lib.licenses.agpl3Plus;
  };
}
