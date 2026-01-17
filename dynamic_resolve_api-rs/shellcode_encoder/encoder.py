import sys
import argparse
from pwn import log


def djb2_hash(name: str) -> int:
    h = 5381
    for c in name:
        v = ord(c)
        if 0x61 <= v <= 0x7A:  # a-z
            v -= 0x20  # uppercase
        h = ((h << 5) + h + v) & 0xFFFFFFFF
    return h


def load_dictionary(file_path):
    try:
        with open(file_path, "r") as f:
            words = [line.strip() for line in f.readlines() if line.strip()]
        if len(words) != len(set(words)):
            log.warning(
                "Probably wont work. Because duplicated strings included in the shellcode."
            )
            seen = set()
            words = [x for x in words if not (x in seen or seen.add(x))]
        return words
    except FileNotFoundError:
        log.error(f"Dictionary file '{file_path}' not found!")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--input", help="Raw shellcode input file (.bin)")
    parser.add_argument(
        "-o",
        "--output",
        default="payload.out",
        help="Output file name (default: payload.out)",
    )
    parser.add_argument(
        "-d",
        "--dict",
        default="dictionary.txt",
        help="Dictionary file name (default: dictionary.txt)",
    )
    parser.add_argument(
        "--check-hash", help="Calculate DJB2 hash of a string (e.g., kernel32.dll)"
    )

    args = parser.parse_args()

    if args.check_hash:
        target = args.check_hash
        result = djb2_hash(target)
        log.info(f"DJB2 Hash for '{target}': {hex(result)}")
        sys.exit(0)

    p1 = log.progress("Dictionary")
    p1.status("Reading dictionary...")
    jowo_dictionary = load_dictionary(args.dict)

    if len(jowo_dictionary) < 256:
        log.warning(
            f"Dictionary only contains {len(jowo_dictionary)} words. Minimum 256 required!"
        )
        for i in range(len(jowo_dictionary), 256):
            jowo_dictionary.append(f"padding_{i}")

    p1.success(f"Ready ({len(jowo_dictionary[:256])} unique words)")

    p2 = log.progress("Shellcode")
    try:
        with open(args.input, "rb") as f:
            shellcode = f.read()
        p2.success(f"Loaded {len(shellcode)} bytes from {args.input}")
    except FileNotFoundError:
        p2.failure(f"File {args.input} not found!")
        sys.exit(1)

    p3 = log.progress("Encoding")
    p3.status("Mapping bytes to words...")

    encoded_words = [jowo_dictionary[byte] for byte in shellcode]

    p3.success("Done")

    with open(args.output, "w") as f:
        f.write(" ".join(encoded_words))

    log.info(f"Payload saved as: {args.output}")


if __name__ == "__main__":
    main()
