#!/bin/bash
set -euo pipefail

#=============================================================
# Htkis_shareweb 一键部署脚本
# 适用于全新 Debian 13 系统
# 用法: sudo bash deploy.sh
#=============================================================

APP_DIR="/home/debian/shareweb"
APP_USER="debian"
APP_PORT=8888
NGINX_PORT=88
SHARE_DIR="/mnt/share"
SAMBA_USER="debian"
SAMBA_PASS="9090"
ADMIN_PASS="Admin@2026"
REPO_URL="https://github.com/wujupeng/shareweb.git"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        error "请使用 sudo 运行此脚本"
    fi
}

check_debian13() {
    if ! grep -q 'VERSION_ID="13"' /etc/os-release 2>/dev/null; then
        warn "此脚本针对 Debian 13 设计，当前系统可能不兼容"
        read -p "是否继续? [y/N] " ans
        [ "$ans" = "y" ] || exit 0
    fi
}

setup_env() {
    export DEBIAN_FRONTEND=noninteractive
    export RUSTUP_DIST_SERVER="https://mirrors.tuna.tsinghua.edu.cn/rustup"
    export RUSTUP_UPDATE_ROOT="https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"
}

#=============================================================
# 阶段1: 系统基础依赖
#=============================================================
phase1_system_deps() {
    info "===== 阶段1: 安装系统基础依赖 ====="

    export DEBIAN_FRONTEND=noninteractive

    # 修复 apt 源: 注释掉 cdrom 源，确保网络源（优先使用国内镜像）
    if grep -q '^deb cdrom:' /etc/apt/sources.list 2>/dev/null; then
        info "检测到 cdrom 源，切换为清华镜像源..."
        cat > /etc/apt/sources.list << 'APTEOF'
deb https://mirrors.tuna.tsinghua.edu.cn/debian trixie main contrib non-free-firmware
deb https://mirrors.tuna.tsinghua.edu.cn/debian trixie-updates main contrib non-free-firmware
deb https://mirrors.tuna.tsinghua.edu.cn/debian-security trixie-security main contrib non-free-firmware
APTEOF
    elif ! grep -q '^deb http' /etc/apt/sources.list 2>/dev/null && ! grep -q '^deb https' /etc/apt/sources.list 2>/dev/null; then
        info "未检测到网络源，添加清华镜像源..."
        cat >> /etc/apt/sources.list << 'APTEOF'
deb https://mirrors.tuna.tsinghua.edu.cn/debian trixie main contrib non-free-firmware
deb https://mirrors.tuna.tsinghua.edu.cn/debian trixie-updates main contrib non-free-firmware
deb https://mirrors.tuna.tsinghua.edu.cn/debian-security trixie-security main contrib non-free-firmware
APTEOF
    fi

    apt-get update -qq

    # DNS (确保能解析外网)
    if ! ping -c1 -W3 crates.io &>/dev/null; then
        info "配置DNS..."
        echo "nameserver 8.8.8.8" > /etc/resolv.conf
        echo "nameserver 223.5.5.5" >> /etc/resolv.conf
    fi

    # 基础工具
    apt-get install -y -qq \
        curl wget git build-essential pkg-config libssl-dev \
        nginx-full ufw sqlite3 \
        samba cifs-utils

    info "阶段1完成"
}

#=============================================================
# 阶段2: 安装 Rust
#=============================================================
phase2_rust() {
    info "===== 阶段2: 安装 Rust ====="

    if su - "$APP_USER" -c "source \$HOME/.cargo/env 2>/dev/null && rustc --version" 2>/dev/null | grep -q "1.9"; then
        info "Rust 已安装: $(su - $APP_USER -c 'source $HOME/.cargo/env && rustc --version')"
        return
    fi

    export RUSTUP_DIST_SERVER="https://mirrors.tuna.tsinghua.edu.cn/rustup"
    export RUSTUP_UPDATE_ROOT="https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup"

    su - "$APP_USER" -c "export RUSTUP_DIST_SERVER=https://mirrors.tuna.tsinghua.edu.cn/rustup && export RUSTUP_UPDATE_ROOT=https://mirrors.tuna.tsinghua.edu.cn/rustup/rustup && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source \$HOME/.cargo/env && rustc --version"

    info "阶段2完成"
}

#=============================================================
# 阶段3: 安装 Node.js
#=============================================================
phase3_nodejs() {
    info "===== 阶段3: 安装 Node.js ====="

    if command -v node &>/dev/null && [ "$(node -v | cut -d. -f1)" = "v20" ]; then
        info "Node.js 已安装: $(node -v)"
        return
    fi

    export DEBIAN_FRONTEND=noninteractive
    apt-get install -y -qq nodejs npm || {
        warn "系统Node.js版本可能过低，尝试NodeSource..."
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        apt-get install -y -qq nodejs
    }

    info "Node.js: $(node -v), npm: $(npm -v)"
    info "阶段3完成"
}

#=============================================================
# 阶段4: 配置 SMB 共享
#=============================================================
phase4_samba() {
    info "===== 阶段4: 配置 SMB 共享 ====="

    mkdir -p "$SHARE_DIR"
    chown "$APP_USER:$APP_USER" "$SHARE_DIR"
    chmod 775 "$SHARE_DIR"

    if ! id "$APP_USER" &>/dev/null; then
        useradd -m -s /bin/bash "$APP_USER"
        echo "$APP_USER:$SAMBA_PASS" | chpasswd
    fi

    if ! pdbedit -u "$SAMBA_USER" &>/dev/null; then
        echo -e "$SAMBA_PASS\n$SAMBA_PASS" | smbpasswd -a "$SAMBA_USER" -s
    fi

    if ! grep -q '\[share\]' /etc/samba/smb.conf; then
        cat >> /etc/samba/smb.conf << SMBCONF

[share]
   path = $SHARE_DIR
   browsable = yes
   writable = yes
   read only = no
   guest ok = no
   valid users = $SAMBA_USER
   force user = $SAMBA_USER
   create mask = 0664
   directory mask = 0775
SMBCONF
    fi

    systemctl enable smbd nmbd
    systemctl restart smbd nmbd

    info "SMB共享已配置: \\\\$SAMBA_USER@$SHARE_DIR"
    info "阶段4完成"
}

#=============================================================
# 阶段5: 获取源码并编译
#=============================================================
phase5_build() {
    info "===== 阶段5: 获取源码并编译 ====="

    if [ ! -d "$APP_DIR/.git" ]; then
        su - "$APP_USER" -c "git clone $REPO_URL $APP_DIR"
    else
        su - "$APP_USER" -c "cd $APP_DIR && git pull"
    fi

    mkdir -p "$APP_DIR/data"
    chown -R "$APP_USER:$APP_USER" "$APP_DIR/data"

    # 配置 Cargo 镜像 (加速下载，使用清华镜像)
    su - "$APP_USER" -c "mkdir -p ~/.cargo && cat > ~/.cargo/config.toml << CARGOEOF
[source.crates-io]
replace-with = \"tuna\"

[source.tuna]
registry = \"sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/\"

[net]
retry = 10
CARGOEOF"

    # 配置 npm 镜像
    npm config set registry https://registry.npmmirror.com

    # 编译后端
    info "编译后端 (可能需要5-10分钟)..."
    su - "$APP_USER" -c "source \$HOME/.cargo/env && cd $APP_DIR && cargo build --release 2>&1 | tail -3"

    # 构建前端
    info "构建前端..."
    if [ -d "$APP_DIR/frontend/dist" ] && [ -f "$APP_DIR/frontend/dist/index.html" ]; then
        info "前端dist已存在，跳过构建"
    else
        su - "$APP_USER" -c "cd $APP_DIR/frontend && npm install && npx vite build"
    fi

    info "阶段5完成"
}

#=============================================================
# 阶段6: 配置应用
#=============================================================
phase6_config() {
    info "===== 阶段6: 配置应用 ====="

    # 始终生成配置（覆盖 git clone 带来的默认配置）
    cat > "$APP_DIR/config.toml" << CONFEOF
[server]
host = \"0.0.0.0\"
port = $APP_PORT
workers = 4

[storage]
base_dir = \"$SHARE_DIR\"
max_file_size = 10737418240
chunk_size = 5242880
tmp_dir = \"/tmp/upload\"

[database]
path = \"$APP_DIR/data/web-file-manager.db\"

[auth]
jwt_secret = \"shareweb-jwt-secret-$(openssl rand -hex 16)\"
jwt_expiration_hours = 24
max_login_attempts = 5
lock_duration_minutes = 30
bcrypt_cost = 12

[log]
level = \"info\"
file_path = \"$APP_DIR/data/app.log\"
CONFEOF
    chown "$APP_USER:$APP_USER" "$APP_DIR/config.toml"

    info "阶段6完成"
}

#=============================================================
# 阶段7: 配置 systemd 服务
#=============================================================
phase7_systemd() {
    info "===== 阶段7: 配置 systemd 服务 ====="

    cat > /etc/systemd/system/shareweb.service << SVCEOF
[Unit]
Description=Web File Manager (Htkis_shareweb)
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
ExecStartPre=/bin/mkdir -p $APP_DIR/data /tmp/upload
ExecStart=$APP_DIR/target/release/web-file-manager $APP_DIR/config.toml
Restart=always
RestartSec=5
StartLimitIntervalSec=60
StartLimitBurst=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SVCEOF

    systemctl daemon-reload
    systemctl enable shareweb
    systemctl restart shareweb

    sleep 3
    if systemctl is-active --quiet shareweb; then
        info "shareweb 服务启动成功"
    else
        error "shareweb 服务启动失败，请检查: journalctl -u shareweb"
    fi

    info "阶段7完成"
}

#=============================================================
# 阶段8: 配置 Nginx
#=============================================================
phase8_nginx() {
    info "===== 阶段8: 配置 Nginx ====="

    cat > /etc/nginx/sites-available/shareweb << NGXEOF
server {
    listen $NGINX_PORT;
    server_name _;
    client_max_body_size 10G;

    root $APP_DIR/frontend/dist;
    index index.html;

    location /api/ {
        proxy_pass http://127.0.0.1:$APP_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
    }

    location / {
        try_files \$uri \$uri/ /index.html;
    }
}
NGXEOF

    ln -sf /etc/nginx/sites-available/shareweb /etc/nginx/sites-enabled/shareweb

    # 删除 Nginx 默认站点（避免与 shareweb 冲突）
    rm -f /etc/nginx/sites-enabled/default

    # 确保 Nginx (www-data) 能读取前端文件
    chmod 755 /home/debian
    chmod -R o+rX "$APP_DIR/frontend/dist"

    nginx -t || error "Nginx 配置错误"
    systemctl enable nginx
    systemctl restart nginx

    info "阶段8完成"
}

#=============================================================
# 阶段9: 配置防火墙
#=============================================================
phase9_firewall() {
    info "===== 阶段9: 配置防火墙 ====="

    if command -v ufw &>/dev/null; then
        ufw allow 22/tcp    comment "SSH"
        ufw allow "$NGINX_PORT/tcp" comment "ShareWeb"
        ufw allow 80/tcp    comment "HTTP"
        ufw allow 443/tcp   comment "HTTPS"
        ufw allow Samba     comment "SMB"

        if ufw status | grep -q "inactive"; then
            warn "UFW 当前未启用，如需启用请手动执行: ufw enable"
        fi
    else
        warn "UFW 未安装，跳过防火墙配置"
    fi

    info "阶段9完成"
}

#=============================================================
# 阶段10: IP变化检测
#=============================================================
phase10_ip_monitor() {
    info "===== 阶段10: 配置IP变化检测 ====="

    cat > "$APP_DIR/check_ip.sh" << IPEOF
#!/bin/bash
CONF_DIR="$APP_DIR"
IP_FILE="\$CONF_DIR/.current_ip"
NEW_IP=\$(hostname -I | awk '{print \$1}')

if [ -f "\$IP_FILE" ]; then
    OLD_IP=\$(cat "\$IP_FILE")
    if [ "\$OLD_IP" != "\$NEW_IP" ]; then
        echo "[\$(date)] IP changed: \$OLD_IP -> \$NEW_IP"
        echo "\$NEW_IP" > "\$IP_FILE"
        systemctl restart shareweb
        echo "[\$(date)] Services restarted for new IP: \$NEW_IP"
    fi
else
    echo "\$NEW_IP" > "\$IP_FILE"
    echo "[\$(date)] Initial IP recorded: \$NEW_IP"
fi
IPEOF

    chmod +x "$APP_DIR/check_ip.sh"
    su - "$APP_USER" -c "$APP_DIR/check_ip.sh"

    (crontab -u "$APP_USER" -l 2>/dev/null | grep -v check_ip; echo "*/5 * * * * $APP_DIR/check_ip.sh >> $APP_DIR/data/ip_check.log 2>&1") | crontab -u "$APP_USER" -

    info "阶段10完成"
}

#=============================================================
# 验证
#=============================================================
verify() {
    info "===== 部署验证 ====="

    echo ""
    echo "----------------------------------------------"
    echo "  Htkis_shareweb 部署状态"
    echo "----------------------------------------------"

    # 后端
    if curl -s --max-time 10 "http://127.0.0.1:$APP_PORT/api/health" | grep -q "healthy"; then
        echo "  后端服务:     运行正常 (端口 $APP_PORT)"
    else
        echo "  后端服务:     异常"
    fi

    # Nginx
    if curl -s --max-time 10 "http://127.0.0.1:$NGINX_PORT/" | grep -q "html"; then
        echo "  Nginx代理:    运行正常 (端口 $NGINX_PORT)"
    else
        echo "  Nginx代理:    异常"
    fi

    # SMB
    if systemctl is-active --quiet smbd; then
        echo "  SMB共享:      运行正常"
    else
        echo "  SMB共享:      异常"
    fi

    # 数据库
    if [ -f "$APP_DIR/data/web-file-manager.db" ]; then
        echo "  SQLite数据库: 正常"
    else
        echo "  SQLite数据库: 异常"
    fi

    # 获取IP
    LOCAL_IP=$(hostname -I | awk '{print $1}')

    echo ""
    echo "  访问地址:     http://$LOCAL_IP:$NGINX_PORT"
    echo "  管理员账号:   admin / $ADMIN_PASS"
    echo "  SMB路径:      \\\\$LOCAL_IP\\share"
    echo "  部署目录:     $APP_DIR"
    echo "  数据库:       $APP_DIR/data/web-file-manager.db"
    echo "  日志:         journalctl -u shareweb"
    echo "----------------------------------------------"
    echo ""
}

#=============================================================
# 主流程
#=============================================================
main() {
    echo ""
    echo "============================================"
    echo "  Htkis_shareweb 一键部署脚本"
    echo "  适用于 Debian 13 系统"
    echo "============================================"
    echo ""

    check_root
    check_debian13
    setup_env

    phase1_system_deps
    phase2_rust
    phase3_nodejs
    phase4_samba
    phase5_build
    phase6_config
    phase7_systemd
    phase8_nginx
    phase9_firewall
    phase10_ip_monitor

    verify

    info "部署完成!"
}

main "$@"
