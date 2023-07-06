Brute forces 32-bit CRC collisions

Note this is probably not an optimal program, I just needed some CRC collisions.

``` bash
$ make
$ ./crcbrute hello_world_ 0
hello_world_M\x14\xb4\x87
```

Can also limit search to nice ascii characters:

``` bash
$ make
$ ./crcbrute hello_world_ 0 --ascii
hello_world_jLmpQiPH
```
