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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeMode {
    Roblox,
    Raw,
}

struct Cli {
    mode: RuntimeMode,
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
           lusa -O2 <script.luau> [args...]\n\
           lusa --capabilities\n\
           lusa --version\n\n\
         Tooling compatibility:\n\
           - accepts Luau CLI optimization/debug flags: -O0/-O1/-O2, -g0/-g1/-g2\n\
           - accepts '-' as stdin\n\
           - LUSA_RAW=1 skips the Roblox bootstrap without changing the command line\n\n\
         Lusa targets Roblox/Luau language and offline engine-surface compatibility.\n\
         It does not fabricate exploit-executor identity, hook state, native provenance,\n\
         or hidden stack frames."
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

fn is_luau_compat_flag(arg: &OsStr) -> bool {
    matches!(
        arg.to_str(),
        Some("-O0")
            | Some("-O1")
            | Some("-O2")
            | Some("-g0")
            | Some("-g1")
            | Some("-g2")
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
         \"roblox_bootstrap\":true,\
         \"api_registry\":true,\
         \"luau_cli_flags\":[\"-O0\",\"-O1\",\"-O2\",\"-g0\",\"-g1\",\"-g2\"],\
         \"host_io\":{}}}",
        cfg!(feature = "host-io")
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
    let mut script: Option<OsString> = None;
    let mut script_args = Vec::new();
    let mut parse_options = true;
    let mut accepted_run_subcommand = false;

    while let Some(arg) = args.next() {
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

        // Several Luau-based tools invoke their runtime as:
        //     luau -O2 file.luau
        // or pass a debug-info level. Lusa uses Lune's bundled Luau compiler,
        // so these flags are accepted for CLI compatibility. They do not change
        // Lusa's own release build optimization level.
        if parse_options && is_luau_compat_flag(&arg) {
            continue;
        }

        if parse_options && arg != "-" {
            if let Some(text) = arg.to_str()
                && text.starts_with('-')
            {
                eprintln!("lusa: unknown option: {text}");
                eprintln!("use -- before a script path that begins with '-'");
                return Err(ExitCode::from(2));
            }
        }

        script = Some(arg);
    }

    let Some(script) = script else {
        usage();
        return Err(ExitCode::from(2));
    };

    Ok(Some(Cli {
        mode,
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
        Ok(runtime) => runtime.with_args(cli.script_args),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    if cli.mode == RuntimeMode::Roblox {
        for (index, chunk) in ROBLOX_API_REGISTRY_CHUNKS.iter().enumerate() {
            let chunk_name = format!("lusa/roblox_api_registry_{index:02}");
            match runtime.run_custom(chunk_name, chunk).await {
                Ok(result) if result.success() => {}
                Ok(result) => return ExitCode::from(result.status()),
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::FAILURE;
                }
            }
        }

        match runtime
            .run_custom("lusa/roblox_bootstrap", ROBLOX_BOOTSTRAP)
            .await
        {
            Ok(result) if result.success() => {}
            Ok(result) => return ExitCode::from(result.status()),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }

        let version_source = format!("getfenv(0)._VERSION = \"Lusa {VERSION}\"");
        match runtime.run_custom("lusa/version", version_source).await {
            Ok(result) if result.success() => {}
            Ok(result) => return ExitCode::from(result.status()),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }
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
