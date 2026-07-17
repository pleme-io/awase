{
  description = "Awase (合わせ) — global hotkey abstraction: key types, parser, and platform-agnostic manager trait";

  inputs.substrate.url = "github:pleme-io/substrate";

  outputs =
    { substrate, ... }:
    substrate.rust.library {
      src = ./.;
    };
}
