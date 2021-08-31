[![Linux Build](https://github.com/Niall-/yaml-to-fstab/actions/workflows/rust.yml/badge.svg)](https://github.com/Niall-/yaml-to-fstab/actions/workflows/rust.yml)

# yaml-to-fstab
yaml-to-fstab is a tool that will take a yaml document
(['example.yml'](example.yml)) and
generate and append appropriate entries to /etc/fstab, e.g., as part of an
automated install.

TODO:
- [ ] Accept input via stdin
- [ ] Specify where to write the output (e.g., the tool is being ran on a
  remote system)
- [ ] Add a flag to set a default value, if any, for fs_mntops
- [ ] Additional flags for --smart-fsck to optionally disable fsck on /boot,
  swap, or remote filesystems

LIMITATIONS:
- Currently fs_mntops is always prepended with `defaults` even if the yaml
  file provides option values
- Currently there is no support for per-entry dump or fsck options in the
  yaml parsing, --smart-fsck will attempt to provide some defaults with root
  being 1 and everything else being 2, but there's also no support for changing
  this for filesystems that you may not want to fsck

## Compiling and running
Rust and Cargo are required, please see [Install
Rust](https://www.rust-lang.org/tools/install)

```
$ git clone https://github.com/Niall-/yaml-to-fstab && cd yaml-to-fstab
$ cargo build --release
$ ./target/release/yaml-to-fstab --help
```

```
yaml-to-fstab 0.5.0

USAGE:
    yaml-to-fstab [FLAGS] [OPTIONS] --input <input>

FLAGS:
    -d, --dry-run       Performs a dry run
    -h, --help          Prints help information
        --smart-fsck    Sets the root partition to 1, all other partitions to 2
                        If enabled, ignores --dump and --fsck options
    -V, --version       Prints version information

OPTIONS:
        --dump <dump>      Global value for fs_freq/dump
                           Should be either 0 or 1 [default: 0]
        --fsck <fsck>      Global value for fs_passno/fsck
                           Should be either 0, 1, or 2 [default: 0]
    -i, --input <input>    Input yaml to parse
```

### Typical use
`# ./target/release/yaml-to-fstab --input=./example.yml --dry-run`


### Explanation of flags
- --input \<input\>:  The input yaml file to parse from
- --dry-run:        Performs a dry run - will not write to /etc/fstab
- --smart-fsck:     If enabled, will set fs_passno/fsck to 1 for root partitions
  and 2 for non-root partitions
- --dump \<dump\>:    Will set fs_freq/dump to \<dump\> for **all** entries, if this
  is undesirable do not use
- --fsck \<fsck\>:    Will set fs_passno/fsck to \<fsck\> for **all** entries,
  if this is undesirable do not use
