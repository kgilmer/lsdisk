# `lsdisk`

Print a list of attached disk drives.  Designed as a simplifying replacement for `fdisk -l`.

# demo

`lsdisk` prints the device path, the size of the block device in bytes, the model, and `[Fixed|Removable]`:

```console
$ lsdisk 
/dev/nvme0n1  0.9 TiB    CT1000P5PSSD8  Fixed     
/dev/nvme1n1  1.8 TiB    CT2000P5PSSD8  Fixed     
/dev/sda      58.9 GiB   STORAGE DEVIC… Removable 
```

Loop devices are filtered from the list.

# build and run

```console
$ cargo build
$ ./target/debug/lsdisk
```