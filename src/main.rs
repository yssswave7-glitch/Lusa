// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    env,
    ffi::{OsStr, OsString},
    io::{self, Read},
    path::PathBuf,
    process::ExitCode,
};

use lune::Runtime;
use mlua::{Function as LuaFunction, MultiValue as LuaMultiValue, Value as LuaValue};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ROBLOX_API_REGISTRY_CHUNKS: &[&str] = &[
    include_str!("roblox_api_registry_00.luau"),
    include_str!("roblox_api_registry_01.luau"),
    include_str!("roblox_api_registry_02.luau"),
    include_str!("roblox_api_registry_03.luau"),
    include_str!("roblox_api_registry_04.luau"),
    include_str!("roblox_api_registry_05.luau"),
    include_str!("roblox_api_registry_06.luau"),
    include_str!("roblox_api_registry_07.luau"),
    include_str!("roblox_api_registry_08.luau"),
    include_str!("roblox_api_registry_09.luau"),
    include_str!("roblox_api_registry_10.luau"),
    include_str!("roblox_api_registry_11.luau"),
];
const ROBLOX_BOOTSTRAP: &str = include_str!("roblox_bootstrap.luau");
const ROBLOX_METATABLE_LOCK: &str = "The metatable is locked";
const POTASSIUM_EXECUTOR_NAME: &str = "potassium";
const POTASSIUM_EXECUTOR_VERSION: &str = "v2.4.8";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeMode {
    Roblox,
    Raw,
}

struct Cli {
    mode: RuntimeMode,
    isolated: bool,
    jit: bool,
    potassium_identity: bool,
    script: OsString,
    script_args: Vec<OsString>,
}

fn usage() {
    eprintln!(
        "Lusa {VERSION}\n\
         Roblox-focused Luau compatibility runtime for tooling and offline analysis\n\n\
         Usage:\n\
           lusa <script.luau> [args...]\n\
           lusa run <script.luau> [args...]\n\
           lusa --raw <script.luau> [args...]\n\
           lusa --raw --isolated <script.luau> [args...]\n\
           lusa --executor-profile=potassium <script.luau> [args...]\n\
           lusa -O2 <script.luau> [args...]\n\
           lusa --capabilities\n\
           lusa --version\n\n\
         Tooling compatibility:\n\
           - accepts Luau CLI optimization/debug flags: -O0/-O1/-O2, -g0/-g1/-g2\n\
           - accepts '-' as stdin\n\
           - LUSA_RAW=1 skips the Roblox bootstrap without changing the command line\n\n\
           - Luau JIT is enabled by default; --no-jit or LUSA_LUAU_JIT=0 disables it\n\
           - --isolated denies filesystem/network/process requires and hides environment values\n\n\
           - Potassium identity compatibility is enabled by default\n\
           - --executor-profile=none disables executor compatibility\n\n\
         Lusa targets Roblox/Luau language and offline engine-surface compatibility.\n\
         Executor compatibility is offline-only and does not provide live hooks,\n\
         injection, network interception, or hidden stack frames."
    );
}

fn env_truthy(name: &str) -> bool {
    let Ok(value) = env::var(name) else {
        return false;
    };
    !matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off"
    )
}

fn env_enabled(name: &str) -> Option<bool> {
    let value = env::var(name).ok()?;
    Some(!matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off"
    ))
}

fn potassium_profile_from_env() -> bool {
    env::var("LUSA_EXECUTOR_PROFILE").map_or(true, |profile| {
        !matches!(
            profile.trim().to_ascii_lowercase().as_str(),
            "none" | "off" | "0" | "false"
        )
    })
}

fn is_luau_compat_flag(arg: &OsStr) -> bool {
    matches!(
        arg.to_str(),
        Some("-O0" | "-O1" | "-O2" | "-g0" | "-g1" | "-g2")
    )
}

fn print_capabilities() {
    println!(
        "{{\"name\":\"lusa\",\"version\":\"{VERSION}\",\
         \"lune_version\":\"0.10.5\",\
         \"direct_file\":true,\
         \"run_subcommand\":true,\
         \"stdin\":true,\
         \"raw_mode\":true,\
         \"isolated_mode\":true,\
         \"executor_identity_emulation\":true,\
         \"executor_identity_default\":\"potassium\",\
         \"executor_identity_profiles\":[\"potassium\"],\
         \"jit_default\":true,\
         \"roblox_bootstrap\":true,\
         \"api_registry\":true,\
         \"luau_cli_flags\":[\"-O0\",\"-O1\",\"-O2\",\"-g0\",\"-g1\",\"-g2\"],\
         \"host_io\":true,\
         \"host_io_modules\":[\"fs\",\"net\",\"process\",\"regex\"],\
         \"isolated_security\":\"defense_in_depth_only\",\
         \"sandbox\":\"external_required\"}}"
    );
}

fn parse_cli() -> Result<Option<Cli>, ExitCode> {
    let mut args = env::args_os();
    let _program = args.next();

    let mut mode = if env_truthy("LUSA_RAW") {
        RuntimeMode::Raw
    } else {
        RuntimeMode::Roblox
    };
    let mut isolated = env_truthy("LUSA_ISOLATED");
    let mut jit = env_enabled("LUSA_LUAU_JIT")
        .or_else(|| env_enabled("LUNE_LUAU_JIT"))
        .unwrap_or(true);
    let mut potassium_identity = potassium_profile_from_env();
    let mut script: Option<OsString> = None;
    let mut script_args = Vec::new();
    let mut parse_options = true;
    let mut accepted_run_subcommand = false;

    for arg in args {
        if script.is_some() {
            script_args.push(arg);
            continue;
        }

        if parse_options && arg == "--" {
            parse_options = false;
            continue;
        }

        if parse_options && arg == "run" && !accepted_run_subcommand {
            accepted_run_subcommand = true;
            continue;
        }

        if parse_options && (arg == "--version" || arg == "-V") {
            println!("lusa {VERSION} (Lune 0.10.5 compatibility runtime)");
            return Ok(None);
        }

        if parse_options && (arg == "--help" || arg == "-h") {
            usage();
            return Ok(None);
        }

        if parse_options && arg == "--capabilities" {
            print_capabilities();
            return Ok(None);
        }

        if parse_options && arg == "--raw" {
            mode = RuntimeMode::Raw;
            continue;
        }

        if parse_options && arg == "--roblox" {
            mode = RuntimeMode::Roblox;
            continue;
        }

        if parse_options && arg == "--isolated" {
            isolated = true;
            continue;
        }

        if parse_options && arg == "--no-isolated" {
            isolated = false;
            continue;
        }

        if parse_options && arg == "--jit" {
            jit = true;
            continue;
        }

        if parse_options && arg == "--no-jit" {
            jit = false;
            continue;
        }

        if parse_options && (arg == "--potassium" || arg == "--executor-profile=potassium") {
            potassium_identity = true;
            continue;
        }

        if parse_options && arg == "--executor-profile=none" {
            potassium_identity = false;
            continue;
        }

        // Several Luau-based tools invoke their runtime as:
        //     luau -O2 file.luau
        // or pass a debug-info level. Lusa uses Lune's bundled Luau compiler,
        // so these flags are accepted for CLI compatibility. They do not change
        // Lusa's own release build optimization level.
        if parse_options && is_luau_compat_flag(&arg) {
            continue;
        }

        if parse_options
            && arg != "-"
            && let Some(text) = arg.to_str()
            && text.starts_with('-')
        {
            eprintln!("lusa: unknown option: {text}");
            eprintln!("use -- before a script path that begins with '-'");
            return Err(ExitCode::from(2));
        }

        script = Some(arg);
    }

    let Some(script) = script else {
        usage();
        return Err(ExitCode::from(2));
    };

    Ok(Some(Cli {
        mode,
        isolated,
        jit,
        potassium_identity,
        script,
        script_args,
    }))
}

fn main() -> ExitCode {
    let cli = match parse_cli() {
        Ok(Some(cli)) => cli,
        Ok(None) => return ExitCode::SUCCESS,
        Err(code) => return code,
    };

    async_io::block_on(run(cli))
}

async fn run(cli: Cli) -> ExitCode {
    let mut runtime = match Runtime::new() {
        Ok(runtime) => {
            // Bootstrap code is cold initialization work and does not benefit
            // from native compilation. Enable the requested JIT state after
            // Roblox setup so short scripts avoid paying that startup cost.
            let initial_jit = cli.mode == RuntimeMode::Raw && cli.jit;
            let runtime = runtime.with_args(cli.script_args).with_jit(initial_jit);
            if cli.isolated {
                runtime.with_env(Vec::<(OsString, OsString)>::new())
            } else {
                runtime
            }
        }
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    if cli.mode == RuntimeMode::Roblox {
        runtime = match install_roblox_fidelity_shims(runtime) {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        };

        // The registry stays split in source control for maintainability. At
        // runtime, registry, bootstrap, and version setup are one cold chunk so
        // standard-library injection and scheduler startup happen only once.
        let registry_len = ROBLOX_API_REGISTRY_CHUNKS
            .iter()
            .map(|chunk| chunk.len())
            .sum::<usize>();
        let mut bootstrap_source =
            String::with_capacity(registry_len + ROBLOX_BOOTSTRAP.len() + VERSION.len() + 40);
        for chunk in ROBLOX_API_REGISTRY_CHUNKS {
            bootstrap_source.push_str(chunk);
        }
        bootstrap_source.push_str(ROBLOX_BOOTSTRAP);
        bootstrap_source.push_str("\ngetfenv(0)._VERSION = \"Lusa ");
        bootstrap_source.push_str(VERSION);
        bootstrap_source.push_str("\"\n");

        match runtime
            .run_custom("lusa/roblox_bootstrap", bootstrap_source)
            .await
        {
            Ok(result) if result.success() => {}
            Ok(result) => return ExitCode::from(result.status()),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }

        runtime = runtime.with_jit(cli.jit);
    }

    if cli.isolated {
        runtime = match install_isolation_guards(runtime) {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        };
    }

    if cli.potassium_identity {
        runtime = match install_potassium_identity(runtime) {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        };
    }

    if cli.script == "-" {
        let mut source = Vec::new();
        if let Err(error) = io::stdin().read_to_end(&mut source) {
            eprintln!("lusa: failed to read stdin: {error}");
            return ExitCode::FAILURE;
        }

        return match runtime.run_custom("stdin", source).await {
            Ok(result) => ExitCode::from(result.status()),
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }

    match runtime.run_file(PathBuf::from(cli.script)).await {
        Ok(result) => ExitCode::from(result.status()),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn install_potassium_identity(runtime: Runtime) -> lune::RuntimeResult<Runtime> {
    runtime.with_lib("@lusa/potassium-identity", |lua| {
        let globals = lua.globals();
        let identify_executor =
            lua.create_function(|_, ()| Ok((POTASSIUM_EXECUTOR_NAME, POTASSIUM_EXECUTOR_VERSION)))?;
        let executor_env = globals.clone();
        let getgenv = lua.create_function(move |_, ()| Ok(executor_env.clone()))?;
        let roblox_env = lua.create_table()?;
        let roblox_env_meta = lua.create_table()?;
        roblox_env_meta.set("__index", globals.clone())?;
        roblox_env.set_metatable(Some(roblox_env_meta))?;
        let getrenv = lua.create_function(move |_, ()| Ok(roblox_env.clone()))?;
        let is_c_closure =
            lua.create_function(|_, function: LuaFunction| Ok(function.info().what == "C"))?;
        let is_lua_closure =
            lua.create_function(|_, function: LuaFunction| Ok(function.info().what != "C"))?;
        let is_executor_closure =
            lua.create_function(|_, function: LuaFunction| Ok(function.info().what != "C"))?;
        let newcclosure = lua.create_function(|lua, function: LuaFunction| {
            lua.create_function(move |_, args: LuaMultiValue| function.call::<LuaMultiValue>(args))
        })?;

        globals.set("identifyexecutor", identify_executor.clone())?;
        globals.set("getexecutorname", identify_executor)?;
        globals.set("getgenv", getgenv)?;
        globals.set("getrenv", getrenv)?;
        globals.set("iscclosure", is_c_closure)?;
        globals.set("islclosure", is_lua_closure.clone())?;
        globals.set("isluaclosure", is_lua_closure)?;
        globals.set("isexecutorclosure", is_executor_closure.clone())?;
        globals.set("isourclosure", is_executor_closure)?;
        globals.set("checkcaller", lua.create_function(|_, ()| Ok(true))?)?;
        globals.set("isourthread", lua.create_function(|_, ()| Ok(true))?)?;
        globals.set("newcclosure", newcclosure)?;
        globals.set("_LUSA_EXECUTOR_PROFILE", "potassium-emulated")?;

        Ok(LuaValue::Table(lua.create_table()?))
    })
}

fn install_isolation_guards(runtime: Runtime) -> lune::RuntimeResult<Runtime> {
    runtime.with_lib("@lusa/isolation", |lua| {
        let globals = lua.globals();
        let original_require = globals.get::<LuaFunction>("require")?;

        let guarded_require = lua.create_function(move |_, module: String| {
            let allowed = matches!(
                module.as_str(),
                "@lune/datetime" | "@lune/regex" | "@lune/roblox" | "@lune/serde" | "@lune/task"
            );

            if !allowed {
                return Err(mlua::Error::RuntimeError(format!(
                    "Lusa isolated mode denied require({module:?})"
                )));
            }

            original_require.call::<LuaValue>(module)
        })?;

        globals.set("require", guarded_require)?;
        Ok(LuaValue::Table(lua.create_table()?))
    })
}

fn install_roblox_fidelity_shims(runtime: Runtime) -> lune::RuntimeResult<Runtime> {
    runtime.with_lib("@lusa/roblox-fidelity", |lua| {
        let globals = lua.globals();
        let original_getmetatable = globals.get::<LuaFunction>("getmetatable")?;
        let typeof_function = globals.get::<LuaFunction>("typeof")?;

        // mlua protects Rust userdata metatables with boolean false. Roblox
        // protects Instance metatables with the public sentinel string below.
        // Keep this as a native Rust callback so debug.info(..., "s") remains
        // "[C]", while leaving non-Instance userdata/table behavior unchanged.
        let roblox_getmetatable = lua.create_function(move |lua, value: LuaValue| {
            let result = original_getmetatable.call::<LuaValue>(value.clone())?;
            if matches!(result, LuaValue::Boolean(false)) {
                let value_type = typeof_function.call::<String>(value)?;
                if value_type == "Instance" {
                    return Ok(LuaValue::String(lua.create_string(ROBLOX_METATABLE_LOCK)?));
                }
            }
            Ok(result)
        })?;

        globals.set("getmetatable", roblox_getmetatable)?;
        Ok(LuaValue::Table(lua.create_table()?))
    })
}
