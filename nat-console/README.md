# nat-console

## 编译

```bash
cargo build --release --package nat-console
```

## 运行

```bash
./target/release/nat-console \
  --host 127.0.0.1 \
  --port 8080 \
  --username admin \
  --password YOUR_PASSWORD \
  --compatible-config /etc/nat.conf
```

HTTPS：

```bash
./target/release/nat-console \
  --host 0.0.0.0 \
  --port 8443 \
  --username admin \
  --password YOUR_PASSWORD \
  --compatible-config /etc/nat.conf \
  --cert /path/to/cert.pem \
  --key /path/to/key.pem
```
