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
-- add_rules("mode.debug", "mode.release")
set_policy("check.auto_ignore_flags", false)  -- 强制启用所有编译标志
set_arch("riscv64")  -- 明确指定目标架构

-- 禁用 xmake 自动添加的架构相关标志
set_policy("build.across_targets_in_parallel", false)
add_cflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_asflags("-march=rv64i", "-mabi=lp64", "-static", "-nostdlib", "-nostartfiles", "-g", {force = true})
add_cflags("-O0", {force = true})
add_asflags("-O0", {force = true})
add_ldflags("-T../../runtime/linker.ld", {force = true})

add_files("../../runtime/start.S")
add_files("../../runtime/c_runtime.c")
add_includedirs("../../runtime")
add_rules("plugin.compile_commands.autoupdate")

-- 设置默认使用 riscv64 工具链
set_toolchains("riscv64")

-- 定义目标
target("hello")
    set_kind("binary")
    add_files("hello.c")
    set_targetdir("bin")

target("add")
    set_kind("binary")
    add_files("add.c")
    set_targetdir("bin")

target("loop")
    set_kind("binary")
    add_files("loop.c")
    set_targetdir("bin")

target("uart")
    set_kind("binary")
    add_files("uart.c")
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
        import("core.base.option")
        local targetname = option.get("target") or "hello"
        local binary = path.join("bin", targetname)
        if os.isfile(binary) then
            os.exec("riscv64-linux-gnu-objdump -d %s", binary)
        end
    end)
    set_menu {
        usage = "xmake disasm",
        description = "Disassemble binaries",
        options = {
            {nil, "target", "v", nil, "Target to run (hello/add/loop...)"}
        }
    }


-- 批量测试任务
task("test_all")
    on_run(function()
        import("core.project.task")
        local results = {}

        -- 动态收集 targets (logic is ok)
        -- ... (target collection logic remains the same)
        local targets = {}
        -- 尝试通过 project API 获取 (使用 xmake 的 try/catch 保护)
        local project = nil
        try {
            function()
                project = import("core.project.project")
            end,
            catch {
                function(err)
                    project = nil
                end
            }
        }
        if project and project.targets then
            local t = project.targets()
            if t then
                -- 有些 xmake 版本返回数组，有些返回 map (key=targetname)
                for k, v in pairs(t) do
                    local name = nil
                    if type(k) == "string" then
                        -- map 风格：key 是 target 名称
                        name = k
                    else
                        -- 数组风格：v 可能是字符串或对象
                        if type(v) == "string" then
                            name = v
                        elseif type(v) == "table" then
                            if type(v.name) == "string" then
                                name = v.name
                            elseif type(v.filename) == "string" then
                                name = v.filename
                            elseif type(v.name) == "function" then
                                try {
                                    function() name = v:name() end,
                                    catch { function(e) end }
                                }
                            end
                        end
                    end
                    -- 只接受字符串类型的 target 名称，防止误捕获局部变量名等
                    if type(name) == "string" and name ~= "" then
                        table.insert(targets, name)
                    end
                end
            end
        end

        -- 回退解析当前 xmake.lua 中的 target("name") 声明
        if #targets == 0 then
            local fname = path.join(os.curdir(), "xmake.lua")
            if os.isfile(fname) then
                local fh = io.open(fname, "r")
                if fh then
                    local content = fh:read("*a") or ""
                    fh:close()
                    for name in content:gmatch('target%(%s*"(.-)"') do
                        table.insert(targets, name)
                    end
                    for name in content:gmatch("target%(%s*'(.-)'") do
                        table.insert(targets, name)
                    end
                end
            end
        end

        -- 去重并保持顺序
        local seen = {}
        local uniq = {}
        for _, v in ipairs(targets) do
            if v and not seen[v] then seen[v] = true table.insert(uniq, v) end
        end
        targets = uniq

        if #targets == 0 then
            print("No targets found to test.")
            return
        end

        -- 1. Build all targets first
        print("Building all targets...")
        local build_all_ok = false
        try {
            function()
                task.run("build", {all = true})
                build_all_ok = true
                print("All targets built successfully.")
            end,
            catch {
                function(errors)
                    print("Build failed for one or more targets: " .. tostring(errors))
                    -- Even if some builds fail, we can attempt to run the ones that succeeded.
                    -- The individual test runs will then correctly report build failures.
                end
            }
        }


        -- 2. Run tests for each target
        for _, name in ipairs(targets) do
            print("Testing target:", name)
            local entry = {name = name}

            -- Check if binary exists as a proxy for successful build
            local binary = path.join("bin", name)
            if os.isfile(binary) then
                entry.build_ok = true
            else
                entry.build_ok = false
                entry.build_err = "Binary not found after build phase."
            end

            -- 运行阶段(如果构建成功)
            if entry.build_ok then
                try {
                    function()
                        -- We have already built it, so tell the 'test' task not to build it again
                        task.run("test", {target = name, no_build = true})
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

    on_run(function(opt)
        -- Ensure opt is not nil and has default values
        opt = opt or {}
        local target = opt.target or "hello"
        local gdb = opt.gdb
        local tracer = opt.tracer
        local no_build = opt.no_build

        import("core.project.task")
        -- os.setenv("CARGO_TERM_QUIET", "true")
        os.setenv("RUSTFLAGS", "-Awarnings")
        if not no_build then
            task.run("build", {target = target})
        end
        local binary = path.join("bin", target)
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

        -- If called with parameters (from test_all), use them
        -- Otherwise, get from command line options
        local target, gdb, tracer, no_build
        target = option.get("target")
        gdb = option.get("gdb")
        tracer = option.get("tracer")
        no_build = option.get("no_build")
        opt = {
            target = target,
            gdb = gdb,
            tracer = tracer,
            no_build = no_build
        }

        -- Pass all options to test_internal
        task.run("test_internal", {}, opt)
    end)

    set_menu {
        usage = "xmake test [--gdb] [--tracer] [--target <target>]",
        description = "Run binary in emulator",
        options = {
            {'g', "gdb", "k", nil, "Enable GDB support"},
            {'tr', "tracer", "k", nil, "Enable execution tracer"},
            {nil, "no_build", "k", nil, "Skip building the target"},
            {nil, "target", "v", nil, "Target to run (hello/add/loop...)"},
        }
    }
