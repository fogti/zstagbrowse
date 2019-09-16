# zstagbrowse

A tagging browser, recursively scanning a source directory tree
and putting symlinks to matches in the target tree.

## Backends

- xattr: uses extended attributes on files (which don't work on NFS, tho; xattr name = "user.zstags")
- persy: uses a persy database (index name = "zstags")
