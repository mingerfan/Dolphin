-- 设置工具链
toolchain("riscv64")
    set_kind("standalone")
    set_toolset("cc", "riscv64-linux-gnu-gcc")
    set_toolset("cxx", "riscv64-linux-gnu-g++")
    set_toolset("as", "riscv64-linux-gnu-gcc")
    set_toolset("ld", "riscv64-linux-gnu-ld")
    set_toolset("ar", "riscv64-linux-gnu-ar")
    set_toolset("strip", "riscv64-linux-gnu-strip")
toolchain_end()

-- 全局编译选项
-- add_rules("mode.debug", "mode.release")
set_policy("check.auto_ignore_flags", false)  -- 强制启用所有编译标志
set_arch("riscv64")  -- 明确指定目标架构

-- 禁用 xmake 自动添加的架构相关标志
set_policy("build.fence", false)
add_cflags("-march=rv64im", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_cxxflags("-march=rv64im", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_asflags("-march=rv64im", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_cflags("-O0", {force = true})
add_cxxflags("-O0", {force = true})
add_asflags("-O0", {force = true})
add_ldflags("-T../../runtime/linker.ld", {force = true})

add_files("../../runtime/start.S")
includes("../../runtime")
add_includedirs("../../runtime")
add_rules("plugin.compile_commands.autoupdate")

-- 设置默认使用 riscv64 工具链
set_toolchains("riscv64")

target("microbench")
    set_kind("binary")
    add_files("src/**.c", "src/**.cc")
    add_includedirs("include")
    add_deps("dolphin_runtime")
    set_targetdir("bin")

task("test")
    local emu_dir = "../../emulator"
    local cur_dir = os.curdir()
    on_run(function ()
        import("core.base.option")
        import("core.project.target")
        local gdb, tracer, difftest
        gdb = option.get("gdb")
        tracer = option.get("tracer")
        difftest = option.get("difftest")

        local binary = path.join(cur_dir, "bin", "microbench")
        if os.isfile(binary) then
            os.cd(emu_dir)
            -- 构建features参数
            local features = {}
            if gdb then
                table.insert(features, "gdb")
            end
            if tracer then
                table.insert(features, "tracer")
            end
            if difftest then
                table.insert(features, "difftest")
            end
            local cmd = "cargo run --release"
            if #features > 0 then
                cmd = cmd .. " --features " .. table.concat(features, ",")
            end
            cmd = cmd .. " -- -e " .. binary
            print(string.format("Running command: %s", cmd))
            os.exec(cmd)
        else
            raise("Binary not found: " .. binary)
        end
    end)

    set_menu {
        usage = "xmake test [--gdb] [--tracer] [--target <target>]",
        description = "Run binary in emulator",
        options = {
            {'g', "gdb", "k", nil, "Enable GDB support"},
            {'tr', "tracer", "k", nil, "Enable execution tracer"},
            {'d', "difftest", "kv", true, "Enable diff test"},
        }
    }
