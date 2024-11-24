# Internal

The backup is stored on the server. The backups are stored in two directories:

- `hosts`: Contains the manifest of the backups
- `pool`: Contains the pool of chunks used for deduplication

## Pool of Chunk

The pool is used to store the content of the file for deduplication. The goal of deduplication is to reduce the amount of space used
between multiple backups of the same host, files that have been moved, and the same file accross different hosts.

To limit used space of file that can change accross multiple backups, we split the file into multiple chunks. If a part of the file
changed, only the chunk of this part will be re-uploaded.

### Size of chunk

The size of the chunk should be chosen depending on the repartition of the size accross all files.

If the size of the chunk is too small, we will have too many chunks and the space taken by the the pool will be greater than the
real size.

If the size of the chunk is too big, we need to retransfer the whole chunk if a part of the file changed.

Files that can be changed over time and should be deduplicated will be:

- image disk of virtual machine,
- file log (that will be appended automatically).

We should take care of the network latency too.

What is the right size of chunks ? I scan a sample of all the file from the directory to see the repartition of the size
on my different host by number of files. This is the command I launch to view the repartition of the files.

```bash
find . -type f -print0 | \
xargs -0 ls -l | \
awk '{ n=int(log($5)/log(2)); if (n<10) { n=10; } size[n]++ } END { for (i in size) printf("%d %d\n", 2^i, size[i]) }' | \
sort -n | \
awk 'function human(x) { x[1]/=1024; if (x[1]>=1024) { x[2]++; human(x) } } { a[1]=$1; a[2]=0; human(a); printf("%3d%s: %6d\n", a[1],substr("kMGTEPYZ",a[2]+1,1),$2) }'
```

And this is the result.

| File size | Number   | Repartition |
| --------- | -------- | ----------- |
| 1k        | 29126558 | 37,36 %     |
| 2k        | 8649088  | 48,45 %     |
| 4k        | 7915884  | 58,60 %     |
| 8k        | 6394302  | 66,81 %     |
| 16k       | 4839627  | 73,01 %     |
| 32k       | 3606949  | 77,64 %     |
| 64k       | 3477900  | 82,10 %     |
| 128k      | 5158625  | 88,72 %     |
| 256k      | 3601985  | 93,34 %     |
| 512k      | 971108   | 94,58 %     |
| 1M        | 875574   | 95,71 %     |
| 2M        | 1698194  | 97,88 %     |
| 4M        | 1046430  | 99,23 %     |
| 8M        | 309027   | 99,62 %     |
| 16M       | 105271   | 99,76 %     |
| 32M       | 65211    | 99,84 %     |
| 64M       | 50832    | 99,91 %     |
| 128M      | 33947    | 99,95 %     |
| 256M      | 21338    | 99,98 %     |
| 512M      | 8066     | 99,99 %     |
| > 1G      | 10068    | 100,00 %    |

With a chunk of 4Mo, we have 99% of a file that wouldn't be split into chunks. We can found in this file: text, picture, and all small
files. All of these files will be unique most of the time.

In the first version, the size couldn't be changed, but in a future version the size of the chunk should be customizable (for different
possibilities).

### Hash table

All small pieces of chunk should be easily accessible with a key without rereading the content of the file. We use a hash table to store
this chunk of file. As the file system already manages this file as a hash table (where the filename is the key), we use the underlying file system.

The key used to deduplicated the content is a SHA-256 of the content. Using a SHA-256 key allows to easily check if the chunk of file already
exists on the pool.

In case of pool check, we can easily verify the coherence between the SHA-256 and the content of the file.

The risk of using a SHA-256 key is collision. If two chunks have the same SHA-256, the backup will fail silently (the problem will be the
restauration of the files). After reading some documentation SHA-256 key shouldn't be a big problem. The risk of collision is small.

In the first version, we keep only the SHA-256 key. In a future version, we can use a sequence id appended to it to identify multiple collisions.
In this case, we need to read the file and the backup will be slower.

In a sample of 3 500 000 of files, I don't have collision on an existing MD5.

### Structure of the pool

The pool will use the file system as a hashtable. The pool must be stored on a filesytem where the number of files has no limit. A SHA-256 can
have approximately `1.15 x 10^77` different possible chunks.

Even if the pool is split in different directories, the number of files will still be too high.

Old filesystems like FAT32, EXT2 can't be used (even if it depends on file really backedup).

Filesystems that should work are: EXT4, Btrfs, XFS, NTFS, ...

The pool will be split in a directory structure of three levels. The first three levels are composed of the 3 first bytes of the SHA-256.
This directory structure will be used to limit the number of locks on the chunk and refcnt file.

```bash
 pool
   ├── aa
   │    ├── aa
   │    │    ├── aa
   │    │    │    └── aaaaaacdefghih-sha256.zlib
   │    │    │    └── aaaaaacdefghih-sha256.info
   ├── unused
   ├── statistics.yml
   ├── REFCNT
   ├── history.yml
   ├── disk_history.yml
```

The `unused` file is used to list all the chunks that are not used in any backup. This file is used to purge the pool.
The `REFCNT` file is used to count the number of times the chunk is used in a backup. A lock file is used to lock the
pool when the `REFCNT` or `unused` file is modified. If the count of a sha256 in the `REFCNT` file is changed to 0, the
chunk is moved to the `unused` file.

The content of the chunk is stored directly in a file (that can be compressed).

Currently, only zlib compression is supported.

## Backup Manifest

The pool is used to store the content of the file. But it doesn't describe how to restore the backup.

The goal of the backup manifest is to describe files in the backup.

We will have one file by backup, each manifest file will be stored in the `hosts` directory (different for each host backuped).

The manifest file will contain, for each backup, the list of files saved, and for each file:

- metadata associated with the file (owner, size, acl, ...),
- list of hash of chunks of the file,
- a sha256 of the file.

For performance reasons, the file will be stored in a binary format (readable by the software). Like JSON, and XML for the text, in Woodstock
we will use a standard format for binary storage. This standard is [protocol-buffers](https://developers.google.com/protocol-buffers).

Using a protocol-buffer simplifies the writing of different programs in different languages to write and read the protocol buffer.

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

The manifest file will be a stream of `FileManifest` prefixed by the size of the file manifest:

```
int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest
int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest int32 FileManifest
(etc)
```
