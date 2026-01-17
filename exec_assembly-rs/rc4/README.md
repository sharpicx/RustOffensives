# rc4

encrypting .NET assemblies into RC4 raw binary.

## Installation

```console
via@ARCI$ cargo build --release
```

## Usage

```console
via@ARCI$ ./target/release/rc4.exe --key Ropcaster1337motherfucker $(wslpath -w /mnt/e/Tools/winpeas.exe)
[+] Success: E:\Tools\winpeas.exe -> E:\Tools\winpeas.bin (Key: 'Ropcaster1337motherfucker')
```

or

```console
via@ARCI$ ./target/release/rc4 --key Ropcaster1337motherfucker /opt/tools/winpeas.exe
[+] Success: /opt/tools/winpeas.exe -> /opt/tools/winpeas.bin (Key: 'Ropcaster1337motherfucker')
```
