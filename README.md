# Htkis_shareweb

基于 Web 的 SMB 共享目录文件管理系统，支持文件浏览、上传、下载、预览和权限控制。

## 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 后端 | Rust + Actix-Web | rustc 1.96.0 / actix-web 4 |
| 前端 | Vue3 + TypeScript + Element Plus | Node.js 20.x / Vite |
| 数据库 | SQLite | 3.46+ |
| 文件存储 | 直接操作本地/SMB 挂载目录 | ext4 / cifs |
| 反向代理 | Nginx | 1.26+ |
| 进程管理 | systemd | Restart=always |

## 功能特性

- **文件浏览**：目录树导航、列表/网格视图切换、文件搜索、排序
- **文件上传**：拖拽上传、大文件分片上传（5MB 分片）、上传进度显示
- **文件下载**：单文件下载、批量 ZIP 打包、Range 断点续传
- **文件操作**：新建文件夹、重命名、删除、移动、复制
- **文件预览**：图片、文本、PDF、视频、音频在线预览
- **权限控制**：基于角色的访问控制（RBAC）、目录级权限规则、权限继承
- **用户管理**：用户增删改查、角色分配（管理员/读写/只读）
- **审计日志**：操作记录查询、只追加写入
- **断电恢复**：systemd Restart=always，异常断电后开机自动恢复
- **IP 变化检测**：cron 定时检测，IP 变更自动重启服务

## 项目结构

```
shareweb/
├── src/                        # Rust 后端源码
│   ├── main.rs                 # 入口，路由注册，PayloadConfig(100MB)
│   ├── config.rs               # 配置加载（TOML + 环境变量覆盖）
│   ├── error.rs                # 统一错误定义(ApiError) + ApiResponse
│   ├── handlers/               # HTTP 请求处理器
│   │   ├── auth.rs             # 认证接口（登录/登出/修改密码）
│   │   ├── files.rs            # 文件浏览/操作接口
│   │   ├── upload.rs           # 分片上传接口（init/chunk/complete/status）
│   │   ├── download.rs         # 下载接口（断点续传/ZIP打包）
│   │   ├── preview.rs          # 6类文件预览接口
│   │   ├── users.rs            # 用户管理接口（Arc<Mutex<Connection>>）
│   │   ├── permissions.rs      # 权限规则 CRUD 接口
│   │   ├── audit.rs            # 审计日志查询接口
│   │   └── health.rs           # 健康检查接口
│   ├── services/               # 业务逻辑层
│   │   ├── auth_service.rs     # JWT+bcrypt 认证
│   │   └── file_service.rs     # 文件浏览/操作，路径映射到 base_dir
│   ├── repositories/           # 数据访问层（SQLite）
│   │   ├── user_repo.rs        # 用户 CRUD，init_admin 用 UPSERT
│   │   ├── audit_repo.rs       # 只追加写入 + 多条件查询
│   │   ├── permission_repo.rs  # 权限规则 CRUD
│   │   └── upload_task_repo.rs # 上传任务 CRUD
│   ├── models/                 # 数据模型定义
│   ├── middleware/              # 中间件（认证/RBAC/审计）
│   └── utils/
│       ├── path_sanitizer.rs   # 虚拟路径→实际路径映射 + 遍历防护
│       └── file_type.rs        # MIME 类型 + 预览类型判断
├── migrations/
│   └── 001_init.sql            # 4 张表 DDL + 索引
├── frontend/                   # Vue3 前端
│   ├── src/
│   │   ├── api/                # Axios 实例 + 拦截器（401 跳转）
│   │   ├── views/              # 5 个页面组件
│   │   ├── stores/             # Pinia 认证状态 + localStorage 持久化
│   │   └── router/             # 路由配置与守卫
│   └── dist/                   # 构建产物
├── deploy/                     # 部署配置
│   ├── deploy.sh               # 一键部署脚本（Debian 13，10 阶段）
│   ├── web-file-manager.service # systemd 服务模板
│   └── nginx.conf              # Nginx 配置模板
├── config.toml                 # 应用配置文件（开发用）
└── Cargo.toml                  # Rust 依赖配置
```

## 快速开始

### 一键部署（推荐，适用于全新 Debian 13）

```bash
# 克隆项目
git clone https://github.com/wujupeng/shareweb.git
cd shareweb

# 以 root 执行一键部署脚本
sudo bash deploy/deploy.sh
```

脚本自动完成 10 个阶段：

| 阶段 | 内容 | 说明 |
|------|------|------|
| 1 | 系统基础依赖 | 自动切换清华 TUNA 镜像源，安装 curl/git/nginx/samba/sqlite3 等 |
| 2 | Rust 工具链 | rustup + TUNA 镜像加速，安装 rustc 1.96+ |
| 3 | Node.js | apt 安装 v20.x + npm |
| 4 | SMB 共享 | 创建 /mnt/share，配置 smbpasswd，**只保留 [share] 共享**，移除默认的 [homes]/[printers]/[print$]，禁止用户自建共享 |
| 5 | 编译构建 | git clone → cargo build --release → vite build（均使用国内镜像） |
| 6 | 应用配置 | 生成 config.toml（随机 jwt_secret，端口 8888） |
| 7 | systemd 服务 | Restart=always + ExecStartPre，断电自动恢复 |
| 8 | Nginx 代理 | listen 88 → proxy 8888，删除默认站点，修复文件权限 |
| 9 | UFW 防火墙 | 开放 SSH/88/80/443/SMB |
| 10 | IP 变化检测 | cron 每 5 分钟检测，IP 变更自动重启服务 |

部署完成后访问 `http://<服务器IP>:88`，管理员账号 `admin / Admin@2026`。

### 部署验证

在 192.168.2.97（全新 Debian 13，3.7GB 内存）上实际验证通过：

```
后端服务:    运行正常 (端口 8888)
Nginx代理:   运行正常 (端口 88)
SMB共享:     运行正常
SQLite数据库: 正常
管理员登录:   正常（JWT token 签发成功）
```

### 部署注意事项

- **apt 源**：全新 Debian 13 可能只有 cdrom 源，deploy.sh 会自动切换为清华 TUNA 镜像
- **编译时间**：cargo build --release 在 4GB 内存机器上约 30-60 分钟
- **首次部署总耗时**：约 60-90 分钟（含下载+编译+构建）
- **sudo 权限**：部署用户需有 sudo 权限（`echo 9090 | sudo -S` 方式）

### 手动生产部署

```bash
# 编译后端
cargo build --release

# 构建前端
cd frontend && npm install && npm run build

# 配置 config.toml
# 修改 storage.base_dir 为 SMB 挂载目录路径

# 启动服务
./target/release/web-file-manager config.toml
```

### 本地开发

```bash
# 克隆项目
git clone https://github.com/wujupeng/shareweb.git
cd shareweb

# 后端编译运行
cargo run -- config.toml

# 前端开发
cd frontend
npm install
npm run dev
```

## API 接口

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/auth/login | 用户登录 |
| POST | /api/auth/logout | 用户登出 |
| GET | /api/auth/profile | 获取当前用户信息 |
| PUT | /api/auth/password | 修改密码 |
| GET | /api/files | 文件列表 |
| GET | /api/files/tree | 目录树 |
| GET | /api/files/search | 文件搜索 |
| POST | /api/files/mkdir | 新建文件夹 |
| PUT | /api/files/rename | 重命名 |
| DELETE | /api/files/delete | 删除 |
| POST | /api/files/move | 移动 |
| POST | /api/files/copy | 复制 |
| GET | /api/files/download | 文件下载 |
| POST | /api/files/download/batch | 批量下载（ZIP） |
| GET | /api/files/preview | 文件预览 |
| POST | /api/upload/init | 初始化上传任务 |
| POST | /api/upload/chunk | 上传分片 |
| POST | /api/upload/complete | 完成上传 |
| GET | /api/upload/status | 上传状态查询 |
| GET | /api/users | 用户列表 |
| POST | /api/users | 创建用户 |
| PUT | /api/users/{username} | 更新用户 |
| DELETE | /api/users/{username} | 删除用户 |
| GET | /api/permissions | 权限规则列表 |
| POST | /api/permissions | 创建权限规则 |
| DELETE | /api/permissions/{id} | 删除权限规则 |
| GET | /api/audit-logs | 审计日志查询 |
| GET | /api/health | 健康检查 |

## 默认账号

| 用户名 | 密码 | 角色 |
|--------|------|------|
| admin | Admin@2026 | 管理员 |

## 配置说明

编辑 `config.toml`：

```toml
[server]
host = "0.0.0.0"          # 监听地址
port = 8888                # 后端监听端口（Nginx 代理此端口）

[storage]
base_dir = "/mnt/share"    # 共享目录路径
max_file_size = 10737418240 # 最大文件大小（10GB）
chunk_size = 5242880        # 分片大小（5MB）
tmp_dir = "./data/upload_tmp" # 上传临时目录（必须在磁盘上，不能放 /tmp 即 tmpfs）

[database]
path = "./data/web-file-manager.db"

[auth]
jwt_secret = "your-secret"  # JWT 签名密钥（生产环境务必更换）
jwt_expiration_hours = 24
max_login_attempts = 5
lock_duration_minutes = 30
bcrypt_cost = 12

[log]
level = "info"
file_path = "./data/app.log"
```

## 运维命令

```bash
# 查看服务状态
systemctl status shareweb

# 查看实时日志
journalctl -u shareweb -f

# 重启服务
sudo systemctl restart shareweb

# 查看 Nginx 日志
sudo tail -f /var/log/nginx/error.log

# 测试后端健康
curl http://127.0.0.1:8888/api/health

# 测试完整链路
curl http://127.0.0.1:88/api/health
```

## 已知问题与解决方案

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| SMB 暴露多余共享（homes/printers） | Debian 默认 smb.conf 含 [homes]/[printers]/[print$] | deploy.sh 覆盖 smb.conf，只保留 [share]，设置 usershare allow guests = no |
| 大文件上传 "No space left on device" | /tmp 是 tmpfs（内存盘），默认仅 2GB，无法容纳大文件分片 | tmp_dir 改为磁盘路径 ./data/upload_tmp（175GB 可用） |
| 上传超过 2MB 失败 | Actix-Web 默认 Payload 限制 | 已配置 PayloadConfig 为 100MB |
| bcrypt 登录缓慢 | cost=12 计算耗时 10-30s | 正常行为，已在 Nginx 设置 proxy_read_timeout 300s |
| Nginx 500 错误 | /home/debian 权限 750，www-data 无法读取 | deploy.sh 已执行 chmod 755 /home/debian |
| 数据库初始化失败 | data/ 目录属主为 root | deploy.sh 已执行 chown -R debian:debian data/ |
| apt-get 卡住 | 全新 Debian 13 只有 cdrom 源 | deploy.sh 自动切换清华 TUNA 镜像 |
| rustup 下载极慢 | static.rust-lang.org 国内访问慢 | deploy.sh 使用 TUNA RUSTUP_DIST_SERVER 镜像 |
| cargo 下载极慢 | crates.io 国内访问慢 | deploy.sh 使用 TUNA sparse registry 镜像 |

## 许可证

MIT
