with import <nixpkgs> {};
  mkShell {
    packages = [
      sqlx-cli
    ];

    DATABASE_URL = "sqlite:meow.db";

    shellHook = ''
      touch meow.db
    '';
  }
