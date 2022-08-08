# QUIC test

This is is a minimal code example with the `quinn` Rust crate.

# How to run

Run server:

```bash
cargo r --release -- server 127.0.0.1:10410
```

Run client in a separate terminal:

```bash
cargo r --release -- client 127.0.0.1:10410
```

# Performance

HW: AMD Ryzen 5 5600X 6-Core Processor, 32Gb Ram

## Localhost

```
sending 0.0001 Mb, requests 100, throughtput 1.9274675 Mbs
sending 0.0001 Mb, requests 1000, throughtput 1.7238587 Mbs
sending 0.0001 Mb, requests 10000, throughtput 1.8769759 Mbs
sending 0.001 Mb, requests 100, throughtput 0.18611357 Mbs
sending 0.001 Mb, requests 1000, throughtput 1.7051907 Mbs
sending 0.001 Mb, requests 10000, throughtput 9.232116 Mbs
sending 0.01 Mb, requests 100, throughtput 0.91459125 Mbs
sending 0.01 Mb, requests 1000, throughtput 8.36981 Mbs
sending 0.01 Mb, requests 10000, throughtput 44.57813 Mbs
sending 0.1 Mb, requests 100, throughtput 4.3509245 Mbs
sending 0.1 Mb, requests 1000, throughtput 35.13409 Mbs
sending 0.1 Mb, requests 10000, throughtput 120.23455 Mbs
sending 1 Mb, requests 100, throughtput 11.397376 Mbs
sending 1 Mb, requests 1000, throughtput 75.62348 Mbs
sending 1 Mb, requests 10000, throughtput 175.62933 Mbs
sending 10 Mb, requests 100, throughtput 16.092371 Mbs
sending 10 Mb, requests 1000, throughtput 87.176285 Mbs
sending 10 Mb, requests 10000, throughtput 153.5462 Mbs
```
