{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  name = "nerdy_glyph_searcher";
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-ItzS/qDw+MJAjm5u+pm/bRjNZrddC0BnoGU0aIoWnFk=";

  meta = {
    description = "Search through font glyphs in the terminal.";
    repository = "https://github.com/tannerweber/nerdy_glyph_searcher";
    license = lib.licenses.agpl3Plus;
  };
}
