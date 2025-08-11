-- 设置工具链
toolchain("riscv64")
    set_toolset("cc", "riscv64-linux-gnu-gcc")
    set_toolset("as", "riscv64-linux-gnu-gcc")
    set_toolset("ld", "riscv64-linux-gnu-ld")
toolchain_end()

-- 全局编译选项
-- add_rules("mode.debug", "mode.release")
set_policy("check.auto_ignore_flags", false)  -- 强制启用所有编译标志
set_arch("riscv64")  -- 明确指定目标架构

-- 禁用 xmake 自动添加的架构相关标志
set_policy("build.across_targets_in_parallel", false)
add_cflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_asflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_cflags("-O0", {force = true})
add_asflags("-O0", {force = true})
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

-- 自定义任务: cleanriscv64-linux-gnu-objdump
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


-- 批量测试任务
task("test_all")
    on_run(function()
        import("core.project.task")
        local results = {}
        local targets = {"hello", "add", "loop"}

        for _, name in ipairs(targets) do
            print("Testing target:", name)
            local entry = {name = name}

            -- 构建阶段
            try {
                function()
                    task.run("build", {target = name})
                    entry.build_ok = true
                end,
                catch {
                    function(errors)
                        entry.build_ok = false
                        entry.build_err = tostring(errors)
                    end
                }
            }

            -- 运行阶段(如果构建成功)
            if entry.build_ok then
                try {
                    function()
                        task.run("test", {target = name})
                        entry.run_ok = true
                    end,
                    catch {
                        function(errors)
                            entry.run_ok = false
                            entry.run_err = tostring(errors)
                        end
                    }
                }
            end

            table.insert(results, entry)
        end

        -- 输出测试报告
        print("\nTest Results:")
        for _, r in ipairs(results) do
            print(string.format("%-8s: build %s, run %s",
                r.name,
                r.build_ok and "OK" or "FAIL",
                r.run_ok and "OK" or r.build_ok and "FAIL" or "SKIP"
            ))
            if not r.build_ok then
                print("  Build Error:", r.build_err)
            elseif not r.run_ok then
                print("  Run Error:", r.run_err)
            end
        end
    end)
    set_menu {
        usage = "xmake test_all",
        description = "Run all tests with error handling"
    }

task("test_internal")
    local emu_dir = "../../emulator"
    local cur_dir = os.curdir()

    on_run(function(target, gdb, tracer)
        import("core.project.task")
        import("core.base.option")
        -- os.setenv("CARGO_TERM_QUIET", "true")
        os.setenv("RUSTFLAGS", "-Awarnings")
        task.run("build", {target = target})
        local binary = path.join("bin", target or "hello")
        print("Running binary:", binary)
        binary = path.join(cur_dir, binary)
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

task("test")
    on_run(function()
        import("core.base.option")
        import("core.base.task")
        local target = option.get("target") or "hello"
        local gdb = option.get("gdb")
        local tracer = option.get("tracer")
        print("Running test for target:", target, "GDB:", gdb, "Tracer:", tracer)
        task.run("test_internal", {}, target, gdb, tracer)
    end)

    set_menu {
        usage = "xmake test [--gdb] [--tracer] [--target <target>]",
        description = "Run binary in emulator",
        options = {
            {'g', "gdb", "k", nil, "Enable GDB support"},
            {'tr', "tracer", "k", nil, "Enable execution tracer"},
            {nil, "target", "v", nil, "Target to run (hello/add/loop)"}
        }
    }
