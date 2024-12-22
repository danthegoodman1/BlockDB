# BlockDB

A database using disk blocks directly.

### Erasure Coding

For redundancies across latent sector errors (partial disk failures), an `erasure_coding` crate is included.

XOR is an n+1 method. Reed-solomon is another (not yet implemented).

This is not required if you have multiple disks and/or machines and can replicate across.

However if you can get to n+1 disks, then you can XOR across disks.
