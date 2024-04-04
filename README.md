# `lsdisk`

Print a list of attached disk drives.  Designed as a simplifying replacement for `fdisk -l`.

# demo

`lsdisk` prints the device path, the size of the block device in bytes, the model, and `[fixed|removable]`, sorted by device name:

```console
$ lsdisk --help
Print a list of attached disks

Usage: lsdisk [OPTIONS]

Options:
  -n, --non-loop-only   Return only non-loop devices
  -r, --removable-only  Return only removable devices
  -e, --expect-one      Return error if matching devices not one
  -b, --brief           Only print the device path
  -h, --help            Print help
  -V, --version         Print version
```

To find the only removable storage device, or return error if 0 or more than one removable device present:

```console
$ ./target/release/lsdisk -bre
/dev/sda
```

# build and run

```console
$ cargo build
$ ./target/debug/lsdisk
```