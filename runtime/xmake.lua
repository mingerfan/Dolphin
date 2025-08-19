-- runtime 子工程的 xmake 配置
-- 只把 C 实现打包成静态库，start.S 与 linker.ld 仍由上层 test 工程直接引用

-- 设置工具链
toolchain("riscv64")
    set_kind("standalone")
    set_toolset("cc", "riscv64-linux-gnu-gcc")
    set_toolset("as", "riscv64-linux-gnu-gcc")
    set_toolset("ld", "riscv64-linux-gnu-ld")
    set_toolset("ar", "riscv64-linux-gnu-ar")
    set_toolset("strip", "riscv64-linux-gnu-strip")
toolchain_end()

-- 全局编译选项
set_policy("check.auto_ignore_flags", false)
set_arch("riscv64")
set_policy("build.fence", false)
add_cflags("-march=rv64im", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_cflags("-O0", {force = true})

target("dolphin_runtime")
    set_kind("static")
    set_toolchains("riscv64")
    add_files("*.c")
    add_includedirs(".")
    set_targetdir("build/runtime")
