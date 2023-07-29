use {
    keymacro::{
        defer,
        text,
    },
    std::{
        mem::size_of,
        ptr::null,
        slice::from_raw_parts as slice,
    },
    windows::{
        core::{
            wcslen,
            Result,
            PCWSTR,
            PWSTR,
        },
        Win32::{
            Foundation::{
                CloseHandle,
                WIN32_ERROR,
            },
            System::{
                Console::{
                    AttachConsole,
                    FreeConsole,
                    ATTACH_PARENT_PROCESS,
                },
                Environment::GetCommandLineW,
                Threading::{
                    CreateProcessW,
                    ExitProcess,
                    GetExitCodeProcess,
                    WaitForSingleObject,
                    PROCESS_CREATION_FLAGS,
                    PROCESS_INFORMATION,
                    STARTUPINFOW,
                },
                WindowsProgramming::INFINITE,
            },
        },
    },
};

fn main() {
    unsafe {
        ExitProcess(match run() {
            Result::Ok(code) => code,
            Result::Err(error) => {
                eprintln!("{error}");
                1
            },
        })
    };
}

fn run() -> Result<u32> {
    const NULL: u16 = b'\0' as u16;
    const TAB: u16 = b'\t' as u16;
    const SPACE: u16 = b' ' as u16;
    const QUOTE: u16 = b'"' as u16;

    // Solve problem of being attached to a new console when running as administrator.
    unsafe { FreeConsole() }.ok()?;
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.ok()?;

    let command_line = unsafe { GetCommandLineW() }.as_ptr();
    let command_line = unsafe { slice(command_line, wcslen(PCWSTR(command_line)) + 1) }; // Include the null character.

    let args = &command_line[match command_line[0] {
        QUOTE => 1 + command_line[1..].iter().position(|&c| c == QUOTE).unwrap() + 1,
        _ => command_line
            .iter()
            .position(|&c| c == SPACE || c == TAB || c == NULL)
            .unwrap(),
    }..];

    let command = &args[args.iter().position(|&c| c != SPACE && c != TAB).unwrap()..];

    // If there are no parameters, display information and usage.
    if command[0] == NULL {
        println!(
            text!(
                "{0} {1}"
                "{2}."
                "Usage: {0} [command]"
            ),
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_DESCRIPTION"),
        );

        return Result::Ok(0);
    }

    let mut command = command.to_vec();

    let info = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..STARTUPINFOW::default()
    };

    let mut target = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            PCWSTR::null(),
            PWSTR(command.as_mut_ptr()),
            null(),
            null(),
            false,
            PROCESS_CREATION_FLAGS::default(),
            null(),
            PCWSTR::null(),
            &info,
            &mut target,
        )
    }
    .ok()?;

    defer! {
        unsafe { CloseHandle(target.hProcess) };
    }

    unsafe { CloseHandle(target.hThread) };
    WIN32_ERROR(unsafe { WaitForSingleObject(target.hProcess, INFINITE) }).ok()?;
    let mut code = 0;
    unsafe { GetExitCodeProcess(target.hProcess, &mut code) }.ok()?;
    Result::Ok(code)
}
