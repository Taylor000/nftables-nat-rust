#![deny(warnings)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
mod config;
mod ip;
mod prepare;

use clap::Parser;
use log::{error, info};
use nat_common::{Args, logger};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

const NFTABLES_ETC: &str = "/etc/nftables-nat";
const FILE_NAME_SCRIPT: &str = "/etc/nftables-nat/nat-diy.nft";
const IP_FORWARD: &str = "/proc/sys/net/ipv4/ip_forward";
const IPV6_FORWARD: &str = "/proc/sys/net/ipv6/conf/all/forwarding";
const CARGO_CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const DEFAULT_CONFIG_FILE: &str = "/etc/nat.conf";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init(CARGO_CRATE_NAME);
    // 使用 clap 解析命令行参数
    let args = Args::parse();

    // 启动时解析一次配置文件，并且快速失败
    if let Err(e) = parse_conf(&args).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)) {
        info!("解析配置文件失败: {e:?}");
        return Err(e.into());
    }
    global_prepare()?;
    Ok(handle_loop(&args)?)
}

fn parse_conf(
    args: &Args,
) -> Result<Vec<config::RuntimeCell>, Box<dyn std::error::Error + Send + Sync>> {
    let nat_cells = if let Some(toml) = &args.toml {
        config::read_toml_config(toml).map_err(|e| {
            info!("读取配置文件失败: {e:?}");
            if let Err(e) = config::toml_example(toml) {
                info!("{e:?}");
            }
            e
        })?
    } else {
        let config_file = config_path(args);
        config::read_config(&config_file).map_err(|e| {
            info!("读取配置文件失败: {e:?}");
            config::example(&config_file);
            e
        })?
    };
    Ok(nat_cells)
}

fn config_path(args: &Args) -> String {
    if let Some(toml) = &args.toml {
        toml.clone()
    } else {
        args.compatible_config_file
            .clone()
            .unwrap_or_else(|| DEFAULT_CONFIG_FILE.to_string())
    }
}

fn config_modified(path: &str) -> Option<SystemTime> {
    Path::new(path)
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn global_prepare() -> Result<(), io::Error> {
    if let Err(e) = Command::new("/usr/sbin/nft").arg("-v").output() {
        if e.kind() == io::ErrorKind::NotFound {
            let err = "未检测到 nftables，请先安装 nftables (Debian/Ubuntu: apt install nftables, CentOS/RHEL: yum install nftables)";
            error!("{}", err);
            return Err(io::Error::new(io::ErrorKind::NotFound, err));
        }
        return Err(e);
    }

    std::fs::create_dir_all(NFTABLES_ETC)?;
    // 修改内核参数，开启IPv4端口转发
    match std::fs::write(IP_FORWARD, "1") {
        Ok(_s) => {
            info!("kernel ip_forward config enabled!\n")
        }
        Err(e) => {
            info!(
                "enable ip_forward FAILED! cause: {e:?}\nPlease excute `echo 1 > /proc/sys/net/ipv4/ip_forward` manually\n"
            );
            return Err(e);
        }
    };

    // 修改内核参数，开启IPv6端口转发
    match std::fs::write(IPV6_FORWARD, "1") {
        Ok(_s) => {
            info!("kernel ipv6_forward config enabled!\n")
        }
        Err(e) => {
            info!(
                "enable ipv6_forward FAILED! cause: {e:?}\nPlease excute `echo 1 > /proc/sys/net/ipv6/conf/all/forwarding` manually\n"
            );
            // IPv6转发失败不作为致命错误，因为可能系统不支持IPv6
            info!("IPv6 forwarding setup failed, continuing with IPv4 only...");
        }
    };
    Ok(())
}

fn handle_loop(args: &Args) -> Result<(), io::Error> {
    let mut latest_script = String::new();
    let conf_path = config_path(args);
    let mut latest_config_mtime = config_modified(&conf_path);
    loop {
        let current_config_mtime = config_modified(&conf_path);
        let config_changed = current_config_mtime != latest_config_mtime;
        if config_changed {
            info!("检测到配置文件变化，重新加载服务配置: {conf_path}");
            latest_config_mtime = current_config_mtime;
        }

        let nat_cells = match parse_conf(args) {
            Ok(cells) => cells,
            Err(e) => {
                error!("解析配置文件失败: {e:?}");
                wait_for_next_reload(&conf_path, latest_config_mtime);
                continue;
            }
        };
        let rule_count = nat_cells
            .iter()
            .filter(|cell| matches!(cell, config::RuntimeCell::Rule(_)))
            .count();
        if rule_count == 0 {
            error!(
                "配置文件未解析到任何有效规则，请检查 {conf_path} 中的转发行是否被 # 注释"
            );
        }
        let script = build_new_script(&nat_cells)?;
        prepare::check_and_prepare()?;
        if script != latest_script {
            info!("当前配置: ");
            for ele in &nat_cells {
                info!("{ele:?}");
            }
            info!("nftables脚本如下：\n{script}");
            latest_script.clone_from(&script);
            let f = File::create(FILE_NAME_SCRIPT);
            if let Ok(mut file) = f {
                file.write_all(script.as_bytes())?;
            }

            let output = Command::new("/usr/sbin/nft")
                .arg("-f")
                .arg(FILE_NAME_SCRIPT)
                .output()?;
            info!(
                "执行/usr/sbin/nft -f {FILE_NAME_SCRIPT} 执行结果: {}",
                output.status
            );
            log::info!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            log::error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            info!("WAIT:等待配置或目标IP发生改变....\n");
        }

        wait_for_next_reload(&conf_path, latest_config_mtime);
    }
}

fn wait_for_next_reload(conf_path: &str, latest_config_mtime: Option<SystemTime>) {
    let max_wait = if cfg!(debug_assertions) {
        Duration::from_secs(5)
    } else {
        Duration::from_secs(60)
    };
    let mut waited = Duration::ZERO;

    while waited < max_wait {
        sleep(Duration::from_secs(1));
        if config_modified(conf_path) != latest_config_mtime {
            return;
        }
        waited += Duration::from_secs(1);
    }
}

fn build_new_script(nat_cells: &[config::RuntimeCell]) -> Result<String, io::Error> {
    //脚本的前缀 - 创建IPv4和IPv6表
    let mut script = String::from(
        "#!/usr/sbin/nft -f\n\
        \n\
        # IPv4 NAT table\n\
        add table ip self-nat\n\
        delete table ip self-nat\n\
        add table ip self-nat\n\
        add chain ip self-nat PREROUTING { type nat hook prerouting priority -110 ; }\n\
        add chain ip self-nat POSTROUTING { type nat hook postrouting priority 110 ; }\n\
        \n\
        # IPv6 NAT table\n\
        add table ip6 self-nat\n\
        delete table ip6 self-nat\n\
        add table ip6 self-nat\n\
        add chain ip6 self-nat PREROUTING { type nat hook prerouting priority -110 ; }\n\
        add chain ip6 self-nat POSTROUTING { type nat hook postrouting priority 110 ; }\n\
        \n\
        # IPv4 Drop table\n\
        add table ip self-filter\n\
        delete table ip self-filter\n\
        add table ip self-filter\n\
        add chain ip self-filter INPUT { type filter hook input priority filter - 1 ; }\n\
        add chain ip self-filter FORWARD { type filter hook forward priority filter - 1 ; }\n\
        \n\
        # IPv6 Drop table\n\
        add table ip6 self-filter\n\
        delete table ip6 self-filter\n\
        add table ip6 self-filter\n\
        add chain ip6 self-filter INPUT { type filter hook input priority filter - 1 ; }\n\
        add chain ip6 self-filter FORWARD { type filter hook forward priority filter - 1 ; }\n\
        ",
    );

    for x in nat_cells.iter() {
        match x.build() {
            Ok(rule) => script += &rule,
            Err(e) => {
                log::error!("Failed to build rule for {x:?}: {e}");
            }
        }
    }
    Ok(script)
}
