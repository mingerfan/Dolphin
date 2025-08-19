-- runtime 子工程的 xmake 配置
-- 只把 C 实现打包成静态库，start.S 与 linker.ld 仍由上层 test 工程直接引用

target("dolphin_runtime")
    set_kind("static")
    -- 把 runtime 的 C 代码打包（包含 c_runtime.c 和 uart.c）
    add_files("*.c")
    add_includedirs(".")
    set_targetdir("build/runtime")
