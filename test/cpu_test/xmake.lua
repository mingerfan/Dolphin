-- 设置工具链
toolchain("riscv64")
    set_toolset("cc", "riscv64-linux-gnu-gcc")
    set_toolset("as", "riscv64-linux-gnu-gcc")
    set_toolset("ld", "riscv64-linux-gnu-ld")
toolchain_end()

-- 全局编译选项
add_rules("mode.debug", "mode.release")
set_languages("c99")
set_policy("check.auto_ignore_flags", false)  -- 强制启用所有编译标志
set_arch("riscv64")  -- 明确指定目标架构

-- 禁用 xmake 自动添加的架构相关标志
set_policy("build.across_targets_in_parallel", false)
add_cflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_asflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_ldflags("-Tlinker.ld", {force = true})

-- 设置默认使用 riscv64 工具链
set_toolchains("riscv64")

-- 定义目标
target("hello")
    set_kind("binary")
    add_files("start.S", "hello.c")
    set_targetdir("bin")

target("add")
    set_kind("binary")
    add_files("start.S", "add.c")
    set_targetdir("bin")

target("loop")
    set_kind("binary")
    add_files("start.S", "loop.c")
    set_targetdir("bin")

-- 自定义任务: clean
task("clean")
    on_run(function()
        os.rm("build")
        os.rm("bin")
        os.rm("target")
    end)
    set_menu {
        usage = "xmake clean",
        description = "Remove build directories"
    }

-- 自定义任务: disasm
task("disasm")
    on_run(function()
        for _, targetname in ipairs({"hello", "add", "loop"}) do
            local binary = path.join("bin", targetname)
            if os.isfile(binary) then
                os.exec("riscv64-linux-gnu-objdump -d %s", binary)
            end
        end
    end)
    set_menu {
        usage = "xmake disasm",
        description = "Disassemble binaries"
    }

-- 自定义任务: run

task("run")
    local emu_dir = "../../emulator"
    local cur_dir = os.curdir()
    
    on_run(function(target)
        import("core.base.option")
        -- os.setenv("CARGO_TERM_QUIET", "true")
        os.setenv("RUSTFLAGS", "-Awarnings")
        local binary = path.join("bin", target or "hello")
        binary = path.join(cur_dir, binary)
        if os.isfile(binary) then
            os.cd(emu_dir)
            -- 构建features参数
            local features = {}
            if option.get("gdb") then
                table.insert(features, "gdb")
            end
            if option.get("gdb") then
                table.insert(features, "tracer")
            end
            
            local cmd = "cargo run --release"
            if #features > 0 then
                cmd = cmd .. " --features " .. table.concat(features, ",")
            end
            cmd = cmd .. " -- -e " .. binary
            
            os.exec(cmd)
        else
            raise("Binary not found: " .. binary)
        end
    end)
    set_menu {
        usage = "xmake [--gdb] [--tracer] run [target]",
        description = "Run binary in emulator",
        options = {
            {'g', "gdb", "k", nil, "Enable GDB support"},
            {'tr', "tracer", "k", nil, "Enable execution tracer"},
            {'t', "target", "kv", nil, "Target to run (hello/add/loop)"}
        }
    }
