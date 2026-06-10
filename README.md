# Htkis_shareweb

基于 Web 的 SMB 共享目录文件管理系统，支持文件浏览、上传、下载、预览和权限控制。

## 技术栈

| 层级 | 技术 |
|------|------|
| 后端 | Rust + Actix-Web |
| 前端 | Vue3 + TypeScript + Element Plus |
| 数据库 | SQLite |
| 文件存储 | 直接操作 SMB 挂载目录 |
| 反向代理 | Nginx |

## 功能特性

- **文件浏览**：目录树导航、列表/网格视图切换、文件搜索、排序
- **文件上传**：拖拽上传、大文件分片上传（5MB 分片）、上传进度显示
- **文件下载**：单文件下载、批量 ZIP 打包、Range 断点续传
- **文件操作**：新建文件夹、重命名、删除、移动、复制
- **文件预览**：图片、文本、PDF、视频、音频在线预览
- **权限控制**：基于角色的访问控制（RBAC）、目录级权限规则、权限继承
- **用户管理**：用户增删改查、角色分配（管理员/读写/只读）
- **审计日志**：操作记录查询、只追加写入

## 项目结构

```
shareweb/
├── src/                        # Rust 后端源码
│   ├── main.rs                 # 入口，路由注册，数据库初始化
│   ├── config.rs               # 配置加载（TOML + 环境变量覆盖）
│   ├── error.rs                # 统一错误定义与响应格式
│   ├── handlers/               # HTTP 请求处理器
│   │   ├── auth.rs             # 认证接口（登录/登出/修改密码）
│   │   ├── files.rs            # 文件浏览/操作接口
│   │   ├── upload.rs           # 分片上传接口
│   │   ├── download.rs         # 下载接口（断点续传/ZIP打包）
│   │   ├── preview.rs          # 文件预览接口
│   │   ├── users.rs            # 用户管理接口
│   │   ├── permissions.rs      # 权限管理接口
│   │   ├── audit.rs            # 审计日志查询接口
│   │   └── health.rs           # 健康检查接口
│   ├── services/               # 业务逻辑层
│   ├── repositories/           # 数据访问层（SQLite）
│   ├── models/                 # 数据模型定义
│   ├── middleware/              # 中间件（认证/RBAC/审计）
│   └── utils/                  # 工具函数（路径安全/文件类型判断）
├── migrations/                 # 数据库迁移脚本
│   └── 001_init.sql            # 初始化表结构
├── frontend/                   # Vue3 前端
│   ├── src/
│   │   ├── api/                # API 调用封装
│   │   ├── views/              # 页面组件
│   │   ├── stores/             # Pinia 状态管理
│   │   ├── router/             # 路由配置与守卫
│   │   └── components/         # 通用组件
│   └── dist/                   # 构建产物
├── deploy/                     # 部署配置
│   ├── deploy.sh               # 一键部署脚本（Debian 13）
│   ├── web-file-manager.service # systemd 服务文件
│   └── nginx.conf              # Nginx 配置模板
├── config.toml                 # 应用配置文件
└── Cargo.toml                  # Rust 依赖配置
```

## 快速开始

### 环境要求

- Rust 1.88+
- Node.js 18+
- SQLite 3
- Nginx（生产部署）

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

### 一键部署（推荐，适用于全新 Debian 13）

```bash
# 克隆项目
git clone https://github.com/wujupeng/shareweb.git
cd shareweb

# 以 root 执行一键部署脚本
sudo bash deploy/deploy.sh
```

脚本自动完成 10 个阶段：
1. 安装系统基础依赖（curl/git/nginx/sqlite3/samba 等）
2. 安装 Rust 工具链（rustup，需 1.88+）
3. 安装 Node.js（v20.x）
4. 配置 SMB 共享（/mnt/share，smbpasswd）
5. 获取源码并编译后端 + 构建前端
6. 生成应用配置（config.toml，随机 jwt_secret）
7. 配置 systemd 服务（Restart=always，断电自动恢复）
8. 配置 Nginx 反向代理（端口 88 → 8888）
9. 配置 UFW 防火墙规则
10. 配置 IP 变化自动检测（cron 每5分钟）

部署完成后访问 `http://<服务器IP>:88`，管理员账号 `admin / Admin@2026`。

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
port = 8888                # 监听端口

[storage]
base_dir = "/mnt/share"    # SMB 共享目录路径
max_file_size = 10737418240 # 最大文件大小（10GB）
chunk_size = 5242880        # 分片大小（5MB）

[database]
path = "./data/web-file-manager.db"

[auth]
jwt_secret = "your-secret"  # JWT 签名密钥
jwt_expiration_hours = 24
max_login_attempts = 5
lock_duration_minutes = 30
bcrypt_cost = 12
```

## 许可证

MIT
