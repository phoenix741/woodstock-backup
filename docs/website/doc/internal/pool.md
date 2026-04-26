# Internal

The backup is stored on the server. The backups are stored in two directories:

- `hosts`: Contains the manifests of the backups
- `pool`: Contains the pool of chunks used for deduplication

## Pool of Chunks

The pool is used to store the content of files for deduplication. The goal of deduplication is to reduce the amount of space used
between multiple backups of the same host, files that have been moved, and the same file across different hosts.

To limit the space used by files that can change across multiple backups, we split the file into multiple chunks. If a part of the file
changes, only the chunk of this part will be re-uploaded.

### Size of chunk

The size of the chunk should be chosen depending on the distribution of file sizes across all files.

If the size of the chunk is too small, we will have too many chunks and the space taken by the pool will be greater than the
real size.

If the size of the chunk is too big, we need to retransfer the whole chunk if a part of the file changes.

Files that can change over time and should be deduplicated include:

- disk images of virtual machines (VirtualBox, VMware, WSL2, ...),
- log files (that will be appended automatically).

We should also take into account network latency.

What is the right size of chunks? I scanned a sample of all the files from a directory to see the distribution of sizes
on different hosts by number of files. This is the command I launched to view the distribution of the files:

```bash
find . -type f -print0 | \
xargs -0 ls -l | \
awk '{ n=int(log($5)/log(2)); if (n<10) { n=10; } size[n]++ } END { for (i in size) printf("%d %d\n", 2^i, size[i]) }' | \
sort -n | \
awk 'function human(x) { x[1]/=1024; if (x[1]>=1024) { x[2]++; human(x) } } { a[1]=$1; a[2]=0; human(a); printf("%3d%s: %6d\n", a[1],substr("kMGTEPYZ",a[2]+1,1),$2) }'
```

And this is the result:

| File size | Number     | Repartition |
| --------- | ---------- | ----------- |
| 1k        | 29,126,558 | 37.36%      |
| 2k        |  8,649,088 | 48.45%      |
| 4k        |  7,915,884 | 58.60%      |
| 8k        |  6,394,302 | 66.81%      |
| 16k       |  4,839,627 | 73.01%      |
| 32k       |  3,606,949 | 77.64%      |
| 64k       |  3,477,900 | 82.10%      |
| 128k      |  5,158,625 | 88.72%      |
| 256k      |  3,601,985 | 93.34%      |
| 512k      |    971,108 | 94.58%      |
| 1M        |    875,574 | 95.71%      |
| 2M        |  1,698,194 | 97.88%      |
| 4M        |  1,046,430 | 99.23%      |
| 8M        |    309,027 | 99.62%      |
| 16M       |    105,271 | 99.76%      |
| 32M       |     65,211 | 99.84%      |
| 64M       |     50,832 | 99.91%      |
| 128M      |     33,947 | 99.95%      |
| 256M      |     21,338 | 99.98%      |
| 512M      |      8,066 | 99.99%      |
| > 1G      |     10,068 | 100.00%     |

With a chunk size of 4MB, 99% of files wouldn't be split into chunks. These include text files, pictures, and all small
files. Most of these files will be unique.

In the latest version of Woodstock, the chunk size is set to 16MB. This choice was made to limit the number of chunks
and optimize the calculation of the hash. Blake3 is used to calculate the hash of the chunk (using parallelism).

In the first version of Woodstock, the chunk size cannot be modified, but in a future version, if it is a community
request, we could add an option to modify the chunk size.

### Hash table

All chunks should be easily accessible with a key without having to reread the content of the file. We use a hash table to store
these chunks. As the file system already manages files as a hash table (where the filename is the key), we use the underlying file system.

The key used to deduplicate the content is a **Blake3** hash of the content. Using a Blake3 key allows us to easily check
if the chunk already exists in the pool. Originally, we used SHA-256, but it was too slow. Blake3 is faster and supports
parallelism on multi-core machines.

> **Note on file naming**: Chunk files use a `-sha256.zz` suffix (e.g., `000003ef...-sha256.zz`) inherited from the
> original SHA-256 implementation. Despite this filename, the hash stored inside is Blake3 for all new pools.
> The algorithm actually used is stored in the pool's algorithm configuration file.

The hash algorithm can be configured via the `CHUNK_ALGORITHM` environment variable:

| Value | Algorithm |
|-------|-----------|
| `blake3` | Blake3 (default) |
| `sha2_256` | SHA-2 256 |
| `sha3_256` | SHA-3 256 |

During pool integrity checks, we can easily verify the coherence between the hash and the content of the file.

The risk of using a Blake3/SHA-256 key is collision. If two chunks have the same hash, the backup will fail silently (the
problem will emerge during restoration of the files). Based on cryptographic research, SHA-256 collisions should not be
a significant problem as the risk is extremely small.

In the first version, we only keep the Blake3 key. In a future version, we could use a sequence ID appended to it to
identify potential collisions.
However, this would require reading the file, which would slow down the backup process.

In a sample of 3,500,000 files, we didn't encounter any collision using MD5.

### Structure of the pool

The pool uses the file system as a hash table. The pool must be stored on a filesystem where the number of files has no limit. A SHA-256 or Blake3 hash can have approximately `1.15 x 10^77` different possible values.

Even with the pool split into different directories, the potential number of files would still be very high.

Older filesystems like FAT32 and EXT2 cannot be used efficiently (even if it depends on the actual files being backed up).

Recommended filesystems include: EXT4, Btrfs, XFS, NTFS, and other modern filesystems.

The pool is organized in a directory structure with three levels. The first three levels use the first 3 bytes of the hash.
This directory structure helps limit the number of locks on chunk and reference count files.

```
pool/
├── algorithm               # Active hash algorithm (blake3, sha2_256, sha3_256)
├── REFCNT                  # Binary reference count database
├── REFCNT.dirty            # Consistency marker (present while a refcount operation is in progress)
├── refcnt/                 # Pending refcount operations
│   └── pending/
├── unused                  # List of chunks with zero references (candidates for deletion)
├── statistics.yml          # Pool statistics
├── history.yml             # Historical pool stats
├── _new/                   # Temporary directory for chunk creation
└── <xx>/                   # First byte of hash (hex)
     └── <xx>/              # Second byte of hash (hex)
          └── <xx>/         # Third byte of hash (hex)
               ├── <hash>-sha256.zz    # Compressed chunk data
               └── <hash>-sha256.info  # Chunk metadata (Protobuf)
```

> **Note on file naming**: Despite the `-sha256.zz` suffix, the hash algorithm used is determined by the `algorithm` file,
> not by the filename. New pools may use Blake3 while still storing chunks with the `-sha256.zz` naming convention
> inherited from the original implementation.

The `unused` file lists all the chunks that are not used in any backup. This file is used to purge the pool.
The `REFCNT` file counts the number of times each chunk is used across all backups. A lock mechanism is used when the
`REFCNT` or `unused` file is modified. When a chunk's reference count drops to 0, it is moved to the `unused` file.

The chunks themselves are stored in compressed files with a `.zz` extension, with additional metadata in `.info` files.
New chunks are temporarily placed in the `_new` directory during creation.

Two compression formats are supported, identified by a magic header:

| Header | Format | Description |
|--------|--------|--------------|
| `WSC01` | Zlib | Legacy format (older backups) |
| `WSC02` | Zstd | Default format for new chunks |

## Chunk Handling

When a new chunk is created:

1. A temporary file is created in the `_new` directory
2. The data is compressed with **Zstd** compression (or Zlib for legacy compatibility)
3. The Blake3 hash of the uncompressed data is calculated
4. The chunk is moved to its final location based on the hash
5. A reference count is updated in the `REFCNT` file

## Reference Counting

The reference counting system (`Refcnt`) tracks how many backups use each chunk. This allows:

- Identifying unused chunks for removal
- Generating statistics about deduplication efficiency
- Ensuring the integrity of the pool

During backup operations, the reference count for each used chunk is incremented. When backups are deleted, the
reference counts are decremented.
Chunks with a reference count of zero are marked as unused.

## Backup Manifest

The pool stores the content of files, but doesn't describe how to restore the backup.

The backup manifest describes all files in the backup.

We have one manifest file per share and per backup, each stored in the `hosts` directory (organized by host).

The manifest file contains, for each backup, the list of files saved, and for each file:

- metadata associated with the file (owner, group, permissions, size, timestamps, acl, etc.),
- list of hashes of chunks comprising the file,
- a Blake3 hash of the entire file for verification.

For performance reasons, the file is stored in a binary format. Like JSON and XML for text formats, Woodstock
uses a standard format for binary storage called [Protocol Buffers](https://developers.google.com/protocol-buffers).

Protocol Buffers simplify the development of programs in different languages that need to read and write the same data format.

```protobuf
enum FileManifestType {
    RegularFile = 0;
    Symlink = 1;
    Directory = 2;
    BlockDevice = 3;
    CharacterDevice = 4;
    Fifo = 5;
    Socket = 6;
    Unknown = 99;
}

message FileManifestStat {
  uint32 ownerId = 1;
  uint32 groupId = 2;
  uint64 size = 3;
  uint64 compressedSize = 4;
  int64 lastRead = 5;
  int64 lastModified = 6;
  int64 created = 7;
  uint32 mode = 8;
  FileManifestType type = 9;

  uint64 dev = 10;
  uint64 rdev = 11;

  uint64 ino = 12;
  uint64 nlink = 13;
}

enum FileManifestAclQualifier {
    UNDEFINED = 0;
    USER_OBJ = 1;
    GROUP_OBJ = 2;
    OTHER = 3;
    USER_ID = 4;
    GROUP_ID = 5;
    MASK = 6;
}

message FileManifestAcl {
  FileManifestAclQualifier qualifier = 1;
  uint32 id = 2;
  uint32 perm = 3;
}

message FileManifestXAttr {
  bytes key = 1;
  bytes value = 2;
}

// File manifest used to store the content of a file
message FileManifest {
  bytes path = 1;
  FileManifestStat stats = 2;
  bytes symlink = 3;

  repeated FileManifestXAttr xattr = 4;
  repeated FileManifestAcl acl = 5;

  repeated bytes chunks = 6;
  bytes hash = 8;

  map<string, bytes> metadata = 7;
}
```

The manifest file is a stream of `FileManifest` objects, each prefixed by its size:

```
int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest
int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest
(etc)
```
