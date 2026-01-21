# reflect

packing executable with 3 Layered Encryption, Indirect PEB calls, Dynamic Resolve API via Custom Hash, Manual Mapping (Reflective PE Loader), & RC4 encryption (upx-like).

## Installation

make the chisel encrypted with [keyed-rs](./keyed-rs).

```console
$ keyed --key sh4rp1cx1337h4x0r0x133777 ./mimikatz.exe
```

after that, rename the `chisel.bin` into `data.bin`. put it in the source directory. then, build the source.

```console
$ cargo.exe build --release --verbose
```

after building this into an executable, we would be given the binary at `target/release/unknown.exe`.

```
$ ./target/release/unknown.exe

  .#####.   mimikatz 2.2.0 (x64) #19041 Sep 19 2022 17:44:08
 .## ^ ##.  "A La Vie, A L'Amour" - (oe.eo)
 ## / \ ##  /*** Benjamin DELPY `gentilkiwi` ( benjamin@gentilkiwi.com )
 ## \ / ##       > https://blog.gentilkiwi.com/mimikatz
 '## v ##'       Vincent LE TOUX             ( vincent.letoux@gmail.com )
  '#####'        > https://pingcastle.com / https://mysmartlogon.com ***/

mimikatz # coffee

    ( (
     ) )
  .______.
  |      |]
  \      /
   `----'

mimikatz #
```
