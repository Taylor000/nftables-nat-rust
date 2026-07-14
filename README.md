# nftables-nat-rust

自用的 nftables 端口转发工具，主要用于 Debian / Ubuntu 服务器。

默认读取本地配置文件 `/etc/nat.conf`，配置写法尽量简单：

```text
本地端口:远程IP或域名:远程端口
```

保存配置文件后，服务会自动检测变更并重新加载 nftables 规则。

## 功能

- 默认本地配置：`/etc/nat.conf`
- 简化转发格式：`本地端口:远程IP或域名:远程端口`
- 默认转发类型：`single`
- 默认协议：`all`
- 默认 IP 版本：`all`
- 支持 IPv4 / IPv6
- 支持域名解析
- 支持 systemd 开机自启
- 兼容旧格式和 TOML 格式

## 一键安装

默认安装简化配置模式：

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/setup.sh)
```

安装脚本会自动完成这些步骤：

- 安装基础依赖：`curl`、`nftables`
- 启用 `nftables`
- 尝试关闭旧的 `iptables` / `ip6tables` 服务
- 下载并安装 `/usr/local/bin/nat`
- 创建 `/etc/nat.conf`
- 创建并启动 `nat.service`

## 手动准备

一般不需要手动执行。排查环境问题时可以单独运行：

```bash
apt update
apt install -y curl nftables
systemctl enable --now nftables
systemctl disable --now iptables 2>/dev/null || true
systemctl disable --now ip6tables 2>/dev/null || true
```

安装后会创建：

- 程序：`/usr/local/bin/nat`
- 配置：`/etc/nat.conf`
- 示例：`/etc/nat_example.conf`
- 服务：`nat.service`

## 配置

编辑 `/etc/nat.conf`：

```bash
vim /etc/nat.conf
```

最简单的格式：

```text
# 本机 10000 端口转发到 1.2.3.4:443
10000:1.2.3.4:443

# 本机 10001 端口转发到 example.com:443
10001:example.com:443

# IPv6 地址建议加中括号
10002:[2001:db8::1]:443
```

这类配置会自动转换成程序内部支持的：

```toml
type = "single"
protocol = "all"
ip_version = "all"
```

修改并保存 `/etc/nat.conf` 后，服务会自动重新加载规则，不需要手动重启。

## 兼容旧格式

如果需要端口段、指定协议、本地重定向或 Drop 规则，仍然可以在 `/etc/nat.conf` 里使用旧格式：

```text
# 单端口转发
SINGLE,10000,443,example.com,tcp,all

# 端口段转发
RANGE,20000,20100,example.com,tcp,all

# 本地重定向
REDIRECT,8080,3128,all,all

# Drop 规则
DROP,input,src_ip=1.2.3.4,dst_port=22,tcp
```

TOML 模式仍然保留：

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/setup.sh) toml
```

## 服务管理

```bash
# 查看状态
systemctl status nat

# 启动
systemctl start nat

# 停止
systemctl stop nat

# 重启
systemctl restart nat

# 开机自启
systemctl enable nat

# 实时日志
journalctl -fu nat
```

## 查看 nftables 规则

```bash
nft list ruleset
nft list table ip self-nat
nft list table ip6 self-nat
```

`ss -lntp` 看不到这些转发端口是正常的。这个工具创建的是 nftables DNAT 规则，不会启动进程监听本地端口。

如果日志里只看到类似下面的内容，说明配置行被注释了，不会生成转发规则：

```text
# 33351:node.example.com:33344
```

有效规则不要带 `#`：

```text
33351:node.example.com:33344
```

如果用 `vi` / `vim` 编辑时，回车后下一行自动出现 `#`，这是编辑器的自动续注释功能。可以在 vim 里执行：

```vim
:set formatoptions-=r formatoptions-=o
```

也可以直接删掉行首的 `#`，确保真正的转发行长这样：

```text
33351:node.example.com:33344
```

## 注意

- `REDIRECT` 工作在 `PREROUTING` 链，只对外部访问有效，本机访问不会触发。
- Docker v28 可能把 `FORWARD` 默认策略设为 `DROP`，程序会自动调整为 `ACCEPT`。
- 从旧版 iptables 工具迁移时，建议先清理旧规则，避免规则互相影响。

## License

[MIT](LICENSE)
