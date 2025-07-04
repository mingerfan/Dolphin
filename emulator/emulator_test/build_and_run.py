#!/usr/bin/env python3
import argparse
import os
import subprocess
import sys
import shutil

def setup_paths(source_name):
    """Setup and return required paths"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    source_path = os.path.join(script_dir, "test_c", f"{source_name}.c")
    build_dir = os.path.join(script_dir, "test_c", "build")
    target_dir = os.path.join(script_dir, "target/")
    binary_name = source_name
    source_binary = os.path.join(build_dir, "bin", binary_name)
    target_binary = os.path.join(target_dir, f"{binary_name}.bin")

    return {
        "script_dir": script_dir,
        "source_path": source_path,
        "build_dir": build_dir,
        "target_dir": target_dir,
        "source_binary": source_binary,
        "target_binary": target_binary
    }

def clean_build(build_dir):
    """Clean build directory if it exists"""
    if os.path.exists(build_dir):
        print("Cleaning build directory...")
        shutil.rmtree(build_dir)

def build_project(paths, source_name):
    """Build the project using CMake"""
    # Create directories
    os.makedirs(paths["build_dir"], exist_ok=True)
    os.makedirs(paths["target_dir"], exist_ok=True)

    # Configure CMake
    print("Configuring CMake...")
    cmake_configure = [
        "cmake",
        "-S", os.path.join(paths["script_dir"], "test_c"),
        "-B", paths["build_dir"]
    ]

    try:
        subprocess.run(cmake_configure, check=True)
    except subprocess.CalledProcessError as e:
        print(f"CMake configuration failed: {e}")
        sys.exit(1)

    # Build
    print(f"Building {source_name}...")
    cmake_build = [
        "cmake",
        "--build", paths["build_dir"],
        "--target", source_name
    ]

    try:
        subprocess.run(cmake_build, check=True)
    except subprocess.CalledProcessError as e:
        print(f"Build failed: {e}")
        sys.exit(1)

    # Copy binary
    if not os.path.exists(paths["source_binary"]):
        print(f"Error: Built binary not found at {paths['source_binary']}")
        sys.exit(1)

    shutil.copy2(paths["source_binary"], paths["target_binary"])
    print(f"Binary copied to {paths['target_binary']}")

def run_simulator(target_binary, debug_mode, script_dir):
    """Run the simulator with the compiled binary"""
    print(f"Running {os.path.basename(target_binary)} in simulator...")
    sim_cmd = ["cargo", "run", "--", "-e", target_binary]
    if debug_mode:
        sim_cmd.append("-d")

    try:
        subprocess.run(sim_cmd, cwd=os.path.dirname(script_dir))
    except subprocess.CalledProcessError as e:
        print(f"Simulator execution failed: {e}")
        sys.exit(1)

def disassemble_binary(binary_path, text_only=False):
    """Disassemble the binary using objdump"""
    print(f"Disassembling {os.path.basename(binary_path)}...")

    objdump_cmd = ["riscv64-linux-gnu-objdump"]

    if text_only:
        objdump_cmd.extend(["-d", binary_path])  # disassemble code sections
    else:
        objdump_cmd.extend(["-D", binary_path])  # disassemble all sections

    try:
        result = subprocess.run(objdump_cmd, check=True, capture_output=True, text=True)
        print(result.stdout)
    except subprocess.CalledProcessError as e:
        print(f"Disassembly failed: {e}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description='Compile and run C programs in RISC-V simulator')
    parser.add_argument("source", help="Name of the C source file (without .c)")
    parser.add_argument("--debug", "-d", action="store_true", help="Enable debug mode")
    parser.add_argument("--clean", "-c", action="store_true", help="Clean build directory before building")
    parser.add_argument("--disassemble", "-dis", action="store_true", help="Show disassembly of the binary")
    parser.add_argument("--text-only", "-t", action="store_true", help="Show only code section disassembly")
    args = parser.parse_args()

    # Setup paths
    paths = setup_paths(args.source)

    # Check if source exists
    if not os.path.exists(paths["source_path"]):
        print(f"Error: Source file {paths['source_path']} not found")
        sys.exit(1)

    # Clean if requested
    if args.clean:
        clean_build(paths["build_dir"])

    # Build the project
    build_project(paths, args.source)

    # Show disassembly if requested
    if args.disassemble or args.text_only:
        disassemble_binary(paths["target_binary"], args.text_only)
    else:
        # Run the simulator
        run_simulator(paths["target_binary"], args.debug, paths["script_dir"])

if __name__ == "__main__":
    main()
