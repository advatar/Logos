# Performance plan

The prize target is post proof generation under 10 seconds on a standard laptop.

Before UI polish, benchmark:

```bash
RISC0_DEV_MODE=0 cargo run -p zk-membership-host --release -- benchmark
```

Record:

- laptop CPU and RAM;
- RISC Zero version;
- guest cycle count;
- wall-clock proving time;
- verification time;
- LEZ CU cost for registration;
- LEZ CU cost for slash submission with K certificates.

Development simulator numbers are not security or prize metrics.
