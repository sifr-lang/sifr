"""The ordinary workspace CLI build shared by preparation consumers."""


def ordinary_cli_build_command() -> list[str]:
    return ["cargo", "build", "--locked", "--offline", "-p", "sifr"]
