# GBAND NSEC 2022 snapshot
![Logo](logos/gband-2-transparent.png)

## What is GBAND?
GBAND is a GBC emulator written for NorthSec CTF 2022. The emulator supports async link cable transfer, which was used for the final exploit.    
The contestants had to reverse engineer the emulator and a ROM to find a flag and a backdoor that could be used to leak another flag. At the very end, the contestants were required to exploit a buffer overflow using the link cable on a headless emulator instance and exfiltrate the flag  that was stored in the save data of that instance.  
**This snapshot of the repo was made for a cybersecurity competition and contains a backdoor and code specific to that competition. A cleaned and current version of the emulator can be found [here!](https://github.com/zer0x64/gband)**

## How to build and run.

### Vulnerable rom
Refer to `vuln-rom/README.md`. Note that a prebuilt version can be found in `gband-webclient/roms/super-myco-boi.gbc`.

### Emulator
You can use the `-s` and `-c` command line argument to connect via link cable as a server or client, respectively.
```
cd gband-wgpu
cargo run --release -- <path/to/rom>
```

### Headless server
To build and run the server, run the following commands. Note that the following command assumed the vulnerable rom is built and in the right directory:
```
cd gband-server
cargo run --release -- -a 0.0.0.0:8080 -i ./inputs.ron -r ./super-myco-boi.gbc
```

### Web Client
This componnent is optionnal to setup the challenge, and contains a few links hardcoded from the northsec infrastructure.
However it does contain a webassembly version of the emulator and can be fun to play with:
```
cd gband-webclient
trunk serve
```

## Writeups
Solutions for the challenges can be found in the `writeups` directory. If you want to link your own writeup here, simply open a Pull Request!

## License
Code is provided under the MIT or Apache license.
