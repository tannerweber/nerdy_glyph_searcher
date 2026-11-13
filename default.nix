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

  cargoHash = "sha256-swujy+XczCK0Mhr79J9t9OPqr5QyP9o3v4LHrQutsGk=";

  nativeBuildInputs = [ installShellFiles ];
  postInstall = ''
    installShellCompletion --cmd ${name} \
      --bash <($out/bin/${name} --shell-completion bash) \
      --fish <($out/bin/${name} --shell-completion fish) \
      --nushell <($out/bin/${name} --shell-completion nushell) \
      --zsh <($out/bin/${name} --shell-completion zsh)
    installManPage $releaseDir/build/${name}-*/out/${name}.1
  '';

  meta = {
    description = "Search through font glyphs in the terminal.";
    repository = "https://github.com/tannerweber/nerdy_glyph_searcher";
    license = lib.licenses.agpl3Plus;
  };
}
