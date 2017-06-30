Toy Proxy
=================================
Playing around with [Rust] and IO performance.

## Start server

```bash
cargo run --bin single-thread
```

Or:

```bash
cargo run --bin multi-threads
```

### Some tests:

A simple way to test the app is [start a server with python](https://docs.python.org/3/library/http.server.html): `python3 -m http.server 9000` , then start the app and generate some requests:

```bash
curl -i 'http://localhost:3000/'

curl -i 'http://localhost:3000/path/to/files'
```

### Very small benchmark:

**System information:**

```
cristian-s:~$ uname -srmpio
Linux 3.13.0-121-generic x86_64 x86_64 x86_64 GNU/Linux
```
```
cristian-s:~$ lscpu
Architecture:          x86_64
CPU op-mode(s):        32-bit, 64-bit
Byte Order:            Little Endian
CPU(s):                4
On-line CPU(s) list:   0-3
Thread(s) per core:    2
Core(s) per socket:    2
Socket(s):             1
NUMA node(s):          1
Vendor ID:             GenuineIntel
CPU family:            6
Model:                 58
Stepping:              9
CPU MHz:               774.000
BogoMIPS:              3591.77
Virtualization:        VT-x
L1d cache:             32K
L1i cache:             32K
L2 cache:              256K
L3 cache:              3072K
NUMA node0 CPU(s):     0-3
```

**Test:**
```bash
ab -c 50 -n 50000 'http://localhost:3000/'
```

**Result with `single-thread`:**

```
Server Software:        SimpleHTTP/0.6
Server Hostname:        localhost
Server Port:            3000

Document Path:          /
Document Length:        658 bytes

Concurrency Level:      50
Time taken for tests:   80.346 seconds
Complete requests:      50000
Failed requests:        7
   (Connect: 0, Receive: 0, Length: 7, Exceptions: 0)
Non-2xx responses:      7
Total transferred:      40594960 bytes
HTML transferred:       32895394 bytes
Requests per second:    622.31 [#/sec] (mean)
Time per request:       80.346 [ms] (mean)
Time per request:       1.607 [ms] (mean, across all concurrent requests)
Transfer rate:          493.41 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       6
Processing:     5   79 338.6     15    7447
Waiting:        5   78 338.6     14    7433
Total:          5   79 338.6     15    7447

Percentage of the requests served within a certain time (ms)
  50%     15
  66%     17
  75%     19
  80%     20
  90%     25
  95%    224
  98%   1018
  99%   1034
 100%   7447 (longest request)
```

**Result with `multi-threads`:**

```
Server Software:        SimpleHTTP/0.6
Server Hostname:        localhost
Server Port:            3000

Document Path:          /
Document Length:        658 bytes

Concurrency Level:      50
Time taken for tests:   77.017 seconds
Complete requests:      50000
Failed requests:        3
   (Connect: 0, Receive: 0, Length: 3, Exceptions: 0)
Non-2xx responses:      3
Total transferred:      40597840 bytes
HTML transferred:       32898026 bytes
Requests per second:    649.21 [#/sec] (mean)
Time per request:       77.017 [ms] (mean)
Time per request:       1.540 [ms] (mean, across all concurrent requests)
Transfer rate:          514.77 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    0   0.1      0       4
Processing:     2   71 398.4      9   15044
Waiting:        2   71 398.4      9   15043
Total:          2   71 398.5     10   15044

Percentage of the requests served within a certain time (ms)
  50%     10
  66%     11
  75%     13
  80%     14
  90%     17
  95%    207
  98%   1010
  99%   1017
 100%  15044 (longest request)
```

[Rust]:https://www.rust-lang.org/en-US/index.html
