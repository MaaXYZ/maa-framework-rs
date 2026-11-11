#!/usr/bin/env python3
"""Run device-free Linux checks through the static Rust binding path."""

import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile


def main() -> None:
    if sys.platform != "linux":
        raise SystemExit("This fixture runner requires Linux and a C compiler.")

    fixtures = Path(__file__).resolve().parent
    crate = fixtures.parents[1]
    with tempfile.TemporaryDirectory(prefix="maa-ios-static-") as temporary:
        project = Path(temporary)
        sdk = project / "sdk"
        libraries = sdk / "lib"
        libraries.mkdir(parents=True)
        # build.rs recognizes the SDK's include/lib layout; the fixture itself
        # needs no SDK headers because it declares the small ABI under test.
        (sdk / "include").mkdir()
        subprocess.run(
            [
                *shlex.split(os.environ.get("CC", "cc")),
                "-std=c11",
                "-D_POSIX_C_SOURCE=200809L",
                "-shared",
                "-fPIC",
                "-pthread",
                str(fixtures / "ios_contract.c"),
                "-o",
                str(libraries / "libMaaFramework.so"),
            ],
            check=True,
        )
        for name in ("MaaToolkit", "MaaAgentClient", "MaaAgentServer"):
            (libraries / f"lib{name}.so").symlink_to("libMaaFramework.so")

        # JSON strings are valid TOML basic strings for these Linux paths.
        manifest = project / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "maa-ios-static-contract"\n'
            'version = "0.0.0"\nedition = "2024"\n\n'
            '[dependencies]\nmaa-framework = { path = '
            + json.dumps(str(crate), ensure_ascii=False)
            + ', features = ["static"] }\n\n'
            '[[bin]]\nname = "ios-static-smoke"\npath = '
            + json.dumps(str(fixtures / "ios_static_smoke.rs"), ensure_ascii=False)
            + "\n",
            encoding="utf-8",
        )
        environment = os.environ.copy()
        environment["MAA_SDK_PATH"] = str(sdk)
        environment["LD_LIBRARY_PATH"] = str(libraries) + (
            os.pathsep + environment["LD_LIBRARY_PATH"]
            if environment.get("LD_LIBRARY_PATH")
            else ""
        )
        subprocess.run(
            ["cargo", "run", "--manifest-path", str(manifest)],
            env=environment,
            check=True,
        )


if __name__ == "__main__":
    main()
