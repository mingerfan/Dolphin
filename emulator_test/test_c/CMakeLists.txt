cmake_minimum_required(VERSION 3.10)
project(emulator_tests C ASM)

# Set RISC-V toolchain
set(CMAKE_C_COMPILER riscv64-linux-gnu-gcc)
set(CMAKE_ASM_COMPILER riscv64-linux-gnu-gcc)

# Common compiler flags
set(COMMON_FLAGS "-march=rv64ima -mabi=lp64 -static -nostdlib -nostartfiles -g")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${COMMON_FLAGS}")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} ${COMMON_FLAGS}")

# Linker flags
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -T${CMAKE_SOURCE_DIR}/linker.ld")

# Create binary directory
file(MAKE_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Function to add test executable
function(add_test_executable name)
    add_executable(${name} start.S ${name}.c)
    set_target_properties(${name}
        PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    )
endfunction()

# Add test executables
add_test_executable(hello)
add_test_executable(add)
add_test_executable(loop)
