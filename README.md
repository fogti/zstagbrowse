# zstagbrowse

A tagging browser, recursively scanning a source directory tree
and putting symlinks to matches in the target tree.

Current backend method: XATTRS (which don't work on NFS, tho).
