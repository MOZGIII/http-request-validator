export type Mode = {
  name: string;
  cargoCommand: string;
  cargoArgs: string;
  cargoCacheKey: string;
  platformIndependent?: true;
};

export type Modes = Record<string, Mode>;

export const code = {
  clippy: {
    name: "clippy",
    cargoCommand: "clippy",
    cargoArgs: "--locked --workspace --all-targets -- -D warnings",
    cargoCacheKey: "clippy",
  },
  test: {
    name: "test",
    cargoCommand: "test",
    cargoArgs: "--locked --workspace",
    cargoCacheKey: "test",
  },
  build: {
    name: "build",
    cargoCommand: "build",
    cargoArgs: "--locked --workspace",
    cargoCacheKey: "build",
  },
  fmt: {
    name: "fmt",
    cargoCommand: "fmt",
    cargoArgs: "-- --check",
    platformIndependent: true,
    cargoCacheKey: "code",
  },
  docs: {
    name: "doc",
    cargoCommand: "doc",
    cargoArgs: "--locked --workspace --document-private-items",
    platformIndependent: true,
    cargoCacheKey: "doc",
  },
} satisfies Modes;
