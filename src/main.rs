// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

use lune::Runtime;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ROBLOX_BOOTSTRAP: &str = include_str!("roblox_bootstrap.luau");

fn usage() {
    eprintln!(
        "Lusa {VERSION}\n\
         Roblox-focused Luau compatibility runner built on Lune\n\n\
         Usage:\n  lusa <script.luau> [args...]\n  lusa run <script.luau> [args...]\n  lusa --version\n\n\
         Lusa targets Roblox/Luau language and engine-surface compatibility.\n\
         It does not fabricate exploit-executor identity or anti-analysis fingerprints."
    );
}

fn main() -> ExitCode {
    async_io::block_on(main_async())
}

async fn main_async() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();

    let Some(mut first) = args.next() else {
        usage();
        return ExitCode::from(2);
    };

    if first == "--version" || first == "-V" {
        println!("lusa {VERSION} (Lune 0.10.5 compatibility runtime)");
        return ExitCode::SUCCESS;
    }

    if first == "--help" || first == "-h" {
        usage();
        return ExitCode::SUCCESS;
    }

    if first == "run" {
        let Some(script) = args.next() else {
            usage();
            return ExitCode::from(2);
        };
        first = script;
    }

    let script = PathBuf::from(first);
    let script_args: Vec<OsString> = args.collect();

    let mut runtime = match Runtime::new() {
        Ok(runtime) => runtime.with_args(script_args),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    // Install Roblox-like globals before compiling/executing the user's file.
    // The user file remains the actual run_file entrypoint so Luau diagnostics
    // preserve its source path and line numbers rather than pointing at a wrapper.
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

    match runtime.run_file(script).await {
        Ok(result) => ExitCode::from(result.status()),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
