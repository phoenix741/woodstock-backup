# Storage Library for Woodstock

## Annexes

### Chunk size

I made some test to get statististics on the repartition of files in directory. This give me the the following repartitions.

```bash
find . -type f -print0 | xargs -0 ls -l | awk '{ n=int(log($5)/log(2)); if (n<10) { n=10; } size[n]++ } END { for (i in size) printf("%d %d\n", 2^i, size[i]) }' | sort -n | awk 'function human(x) { x[1]/=1024; if (x[1]>=1024) { x[2]++; human(x) } } { a[1]=$1; a[2]=0; human(a); printf("%3d%s: %6d\n", a[1],substr("kMGTEPYZ",a[2]+1,1),$2) }'
```

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

The conclusion is that a chunk of 2Mo should be enough to split a file and detect part to transfert. For many file the whole file
would be transferted. For bigger file we can add or update by chunk of 2Mo.

For a chunk a 2Mo and hash made with a SHA256, and the repartition of file on my computer, the gain of space should be between 360M
and 450Mo to save disk space.

### Some command for btrfs

This is the command that i have bookmarked for writing the btrfs storage.

- `/bin/df --print-type /mnt/woodstock/`
- `btrfs filesystem du -s /mnt/woodstock/hosts/*`: Show, total, exclusif, and set shared, but very slow.
- `btrfs quota enable`: Can be used instead of du, but only for each subvolume
- `sudo btrfs qgroup show -reF /mnt/woodstock/hosts/pc-ulrich/` to get the qgroup
- `sudo compsize /mnt/woodstock/hosts/pc-ulrich/0` to calculate the compression size of a backup.
