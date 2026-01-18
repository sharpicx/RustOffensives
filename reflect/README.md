# reflect

packing executable with Dynamic Resolve API via Hash, Manual Mapping (Reflective PE Loader), & RC4 encryption (upx-like)

## Installation

make the chisel encrypted with [RC4](https://github.com/Ropcaster/rc4).

```console
$ rc4 --key Ropcaster1337motherfucker ./chisel.exe
[+] Success: ./chisel.exe -> ./chisel.bin (Key: 'Ropcaster1337motherfucker')
$ rc4 --key Ropcaster1337motherfucker ./mimikatz.exe
[+] Success: ./mimikatz.exe -> ./mimikatz.bin (Key: 'Ropcaster1337motherfucker')
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

## References

* <https://github.com/Ropcaster/rc4>
