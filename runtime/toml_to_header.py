#!/usr/bin/env python3
# python
"""
toml_to_header.py
Usage:
    python3 toml_to_header.py /abs/path/to/device.toml [/abs/path/to/output.h]

Behavior:
- Parse the TOML file and generate a C header with device/memory macros.
- Default output: runtime/device_config.h (same directory as script if output not provided).
- Uses tomllib (Python 3.11+) if available, otherwise falls back to tomli.
"""

from __future__ import annotations
import sys
import os
import argparse
import re
from typing import Any, Dict, List

# toml loader: prefer stdlib tomllib, otherwise tomli
try:
    import tomllib as toml
except Exception:
    try:
        import tomli as toml  # type: ignore
    except Exception:
        sys.exit("ERROR: need Python 3.11 (tomllib) or install tomli (pip3 install --user tomli)")

def norm_name(s: str) -> str:
    return re.sub(r'[^A-Za-z0-9]', '_', s).upper()

def int_to_hex_ul(v: int) -> str:
    return f"0x{v:X}UL"

def int_to_dec_ul(v: int) -> str:
    return f"{v}UL"

def write_header(out_path: str, data: Dict[str, Any]) -> None:
    lines: List[str] = []
    lines.append("#ifndef DOLPHIN_DEVICE_CONFIG_H")
    lines.append("#define DOLPHIN_DEVICE_CONFIG_H")
    lines.append("")

    # memory
    mem = data.get("memory", {})
    if mem:
        lines.append("/* memory */")
        if "memory_base" in mem:
            try:
                mb = int(mem["memory_base"])
                lines.append(f"#define MEMORY_BASE {int_to_hex_ul(mb)}")
            except Exception:
                lines.append(f"/* skipping invalid memory_base */")
        if "memory_size" in mem:
            try:
                ms = int(mem["memory_size"])
                lines.append(f"#define MEMORY_SIZE {int_to_dec_ul(ms)}")
            except Exception:
                lines.append(f"/* skipping invalid memory_size */")
        lines.append("")

    # devices
    devices = data.get("devices", [])
    if isinstance(devices, list) and devices:
        for dev in devices:
            name = dev.get("name")
            if not name:
                continue
            uname = norm_name(name)
            lines.append(f"/* device: {name} */")
            if "base" in dev:
                try:
                    b = int(dev["base"])
                    lines.append(f"#define DEVICE_{uname}_BASE {int_to_hex_ul(b)}")
                except Exception:
                    lines.append(f"/* skipping invalid base for {name} */")
            if "size" in dev:
                try:
                    s = int(dev["size"])
                    lines.append(f"#define DEVICE_{uname}_SIZE {int_to_hex_ul(s)}")
                except Exception:
                    lines.append(f"/* skipping invalid size for {name} */")
            if "enabled" in dev:
                enabled = 1 if dev.get("enabled") else 0
                lines.append(f"#define DEVICE_{uname}_ENABLED {enabled}")
            if "type" in dev:
                t = dev.get("type")
                # ensure no embedded quote characters break the macro
                t_esc = str(t).replace('"', '\\"')
                lines.append(f"#define DEVICE_{uname}_TYPE \"{t_esc}\"")
            lines.append("")

    lines.append("#endif // DOLPHIN_DEVICE_CONFIG_H")
    content = "\n".join(lines) + "\n"

    # ensure directory exists
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "w", newline="\n") as f:
        f.write(content)

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Convert TOML device/profile to C header")
    p.add_argument("input", help="Absolute path to input TOML file")
    p.add_argument("output", nargs="?", help="Output header path (optional). Default: runtime/device_config.h next to script")
    return p.parse_args()

def main() -> None:
    args = parse_args()
    inp = args.input
    if not os.path.isabs(inp):
        print("Warning: input path is not absolute. Proceeding anyway.")
    if not os.path.exists(inp):
        print(f"ERROR: input file not found: {inp}", file=sys.stderr)
        sys.exit(2)

    # default output: runtime/device_config.h (script directory)
    if args.output:
        out = args.output
    else:
        # choose runtime/device_config.h next to this script
        script_dir = os.path.dirname(os.path.realpath(__file__))
        out = os.path.join(script_dir, "device_config.h")

    # read and parse toml
    with open(inp, "rb") as f:
        try:
            data = toml.load(f)
        except Exception as e:
            print(f"ERROR: failed to parse TOML: {e}", file=sys.stderr)
            sys.exit(3)

    write_header(out, data)
    print(f"Wrote {out}")

if __name__ == "__main__":
    main()
