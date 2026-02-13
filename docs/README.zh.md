[English](../README.md) | [Deutsch](README.de.md) | **中文**

# FHIR R4 Patient Server

用 Rust 打造的生产级 FHIR R4 **Patient** 资源服务器。

内含 PostgreSQL 自定义扩展（PGRX）做存储，完整 CRUD + 搜索 + 历史版本，通过 Claude API 实现 AI 功能，Docker Compose 一键部署。

## 技术栈

| 层级              | 技术                                |
| ----------------- | ----------------------------------- |
| HTTP 服务器       | Rust + [Axum](https://github.com/tokio-rs/axum) |
| FHIR 类型         | [fhir-sdk](https://crates.io/crates/fhir-sdk) (R4B) |
| Postgres 扩展     | Rust + [PGRX](https://github.com/pgcentralfoundation/pgrx) |
| LLM 集成          | Claude API (Anthropic)              |
| 容器化            | Docker Compose                      |
| CI/CD             | GitHub Actions                      |

## 架构

<p align="center">
  <img src="../diagrams/system-architecture.svg" alt="系统架构" width="800"/>
</p>

### 数据库模型

<p align="center">
  <img src="../diagrams/db-schema.drawio.svg" alt="数据库模型" width="700"/>
</p>

## 项目结构

```
fhir/
├── Cargo.toml                    # 工作区根
├── crates/
│   ├── core/                     # 共享 FHIR 类型与工具
│   │   └── src/
│   │       ├── lib.rs            # 重导出 Patient, HumanName, Identifier
│   │       ├── bundle.rs         # FHIR Bundle（searchset、history）
│   │       ├── outcome.rs        # OperationOutcome 错误响应
│   │       ├── error.rs          # FhirError 枚举
│   │       └── capability.rs     # CapabilityStatement
│   ├── server/                   # Axum HTTP 服务器
│   │   └── src/
│   │       ├── main.rs           # 入口，路由配置
│   │       ├── config.rs         # 环境变量配置
│   │       ├── routes/           # 端点处理器
│   │       ├── middleware/        # 认证、审计、请求 ID、限流、指标
│   │       ├── db/               # 连接池 & PatientRepository
│   │       ├── ai/               # Claude API 客户端、自然语言搜索、生成器、聊天
│   │       └── error.rs          # AppError → OperationOutcome
│   └── pg-ext/                   # PGRX PostgreSQL 扩展
│       └── src/
│           ├── lib.rs            # 扩展入口
│           ├── storage.rs        # fhir_put, fhir_get, fhir_update, fhir_delete
│           ├── search.rs         # fhir_search 过滤 & 分页
│           ├── history.rs        # fhir_history, fhir_get_version
│           └── schema.sql        # 表和索引定义
├── docker/
│   ├── postgres/
│   │   ├── Dockerfile            # 多阶段构建：PGRX 扩展 → postgres:17
│   │   └── init.sql              # 首次启动时 CREATE EXTENSION
│   └── server/
│       └── Dockerfile            # 多阶段构建：编译服务器 → debian-slim
├── docker-compose.yml
├── examples/                     # 示例数据 & curl 脚本
├── postman/                      # Postman 集合
├── Makefile
└── .github/workflows/ci.yml
```

## 快速开始

### 用 Docker Compose（推荐）

```bash
# 克隆仓库
git clone https://github.com/hwang-fu/fhir.git
cd fhir

# 复制并编辑环境变量
cp .env.example .env
# 编辑 .env，设置 API_KEY，可选设置 ANTHROPIC_API_KEY

# 启动所有服务
docker compose up -d --build

# 确认两个服务都在跑
docker compose ps

# 创建一个患者
curl -X POST http://localhost:8080/fhir/Patient \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"resourceType": "Patient", "name": [{"family": "Smith", "given": ["John"]}], "gender": "male", "birthDate": "1990-05-15"}'

# 停止服务
docker compose down
```

### 本地开发（不用 Docker）

前置条件：Rust 1.92+、PostgreSQL 17、[cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.16.x

```bash
# 终端 1 — 启动带 PGRX 扩展的 PostgreSQL
cargo pgrx run pg17 -p fhir-pg-ext
# 在 psql 里执行：
#   DROP EXTENSION IF EXISTS fhir_pg_ext CASCADE;
#   CREATE EXTENSION fhir_pg_ext;
# 保持这个终端开着。

# 终端 2 — 启动服务器
API_KEY="secret123" \
DATABASE_URL="postgres://$USER@localhost:28817/fhir_pg_ext" \
cargo run -p fhir-server

# 终端 3 — 测试
curl http://localhost:8080/health
curl http://localhost:8080/metadata | jq
```

## API 参考

所有 `/fhir/*` 端点需要 `X-API-Key` 头（除非认证已关闭）。
公开端点（`/metadata`、`/health`、`/metrics`）不需要认证。

### CRUD 操作

| 方法 | 端点 | 说明 | 成功响应 |
| ---- | ---- | ---- | -------- |
| `POST` | `/fhir/Patient` | 创建患者 | `201` + `Location` + `ETag` |
| `GET` | `/fhir/Patient/{id}` | 读取患者 | `200` + `ETag` + JSON |
| `PUT` | `/fhir/Patient/{id}` | 更新患者 | `200` + `ETag` |
| `DELETE` | `/fhir/Patient/{id}` | 删除患者（软删除） | `204` |

<details>
<summary>CRUD 请求流程图</summary>

<p align="center">
  <img src="../diagrams/crud-req-flow.drawio.svg" alt="CRUD 请求流程" width="800"/>
</p>
</details>

### 搜索 & 历史

| 方法 | 端点 | 说明 |
| ---- | ---- | ---- |
| `GET` | `/fhir/Patient?name=&gender=&birthdate=&_count=&_offset=&_sort=` | 带分页的搜索 |
| `GET` | `/fhir/Patient/{id}/_history` | 版本历史 |

**搜索参数：**

| 参数 | 类型 | 示例 |
| ---- | ---- | ---- |
| `name` | 字符串（子串匹配） | `name=Smith` |
| `gender` | token | `gender=male` |
| `birthdate` | 带前缀的日期 | `birthdate=ge1990-01-01` |
| `_count` | 整数 | `_count=10`（默认 10） |
| `_offset` | 整数 | `_offset=0` |
| `_sort` | 字段名 | `_sort=-birthdate`（前缀 `-` = 降序） |

### 扩展功能

| 方法 | 端点 | 说明 |
| ---- | ---- | ---- |
| `POST` | `/fhir/Patient/$validate` | 只验证不存储 |
| `GET` | `/metadata` | CapabilityStatement |

### AI 功能（需要 `ANTHROPIC_API_KEY`）

| 方法 | 端点 | Body | 说明 |
| ---- | ---- | ---- | ---- |
| `POST` | `/fhir/Patient/$nl-search` | `{"query": "..."}` | 自然语言 → FHIR 搜索 |
| `POST` | `/fhir/Patient/$generate` | `{"count": 5}` | 生成合成患者数据（最多 50） |
| `POST` | `/fhir/$chat` | `{"message": "..."}` | AI 聊天机器人（带 tool calling） |

<details>
<summary>AI 聊天功能图</summary>

<p align="center">
  <img src="../diagrams/ai-chat-feature.drawio.svg" alt="AI 聊天功能" width="700"/>
</p>
</details>

### 可观测性

| 方法 | 端点 | 说明 |
| ---- | ---- | ---- |
| `GET` | `/health` | 数据库连通性检查（`200`/`503`） |
| `GET` | `/metrics` | Prometheus 文本格式 |

## 配置

所有配置通过环境变量：

| 变量 | 必填 | 默认值 | 说明 |
| ---- | ---- | ------ | ---- |
| `DATABASE_URL` | 是 | `host=localhost user=postgres dbname=fhir` | PostgreSQL 连接字符串 |
| `BIND_ADDRESS` | 否 | `0.0.0.0:8080` | 服务器监听地址 |
| `API_KEY` | 否 | _（关闭）_ | `X-API-Key` 认证密钥 |
| `ANTHROPIC_API_KEY` | 否 | _（关闭）_ | 启用 AI 功能 |
| `CORS_ORIGINS` | 否 | `*` | 逗号分隔的允许来源 |
| `RATE_LIMIT_RPS` | 否 | `100` | 每秒最大请求数 |
| `RUST_LOG` | 否 | `info` | 日志级别过滤 |

## 中间件

请求按以下顺序经过这些层（最外层优先）：

1. **Prometheus Metrics** — 计数请求，记录延迟直方图
2. **Tracing** — 通过 `tower-http` 进行 HTTP 级别的链路追踪
3. **CORS** — 可配置的来源白名单
4. **Request ID** — 生成或传播 `X-Request-ID`
5. **Audit** — 记录 POST/PUT/DELETE 变更操作
6. **Rate Limit** — 令牌桶限流器（仅受保护路由）
7. **Auth** — 验证 `X-API-Key` 头（仅受保护路由）

<p align="center">
  <img src="../diagrams/middleware-pipeline.drawio.svg" alt="中间件流水线" width="600"/>
</p>

## 测试

集成测试用 [testcontainers](https://github.com/testcontainers/testcontainers-rs) 启动一个真实的 PostgreSQL 实例（带 PGRX 扩展），然后通过 Axum 路由器测试所有 HTTP 端点。

```bash
# 构建测试数据库镜像（只需第一次）
make test-db-image

# 跑全部测试
make integration-test

# 或者直接
cargo test -p fhir-server -- --test-threads=1
```

| 测试 | 验证内容 |
| ---- | -------- |
| `test_auth` | 缺少 / 错误 / 正确的 API Key |
| `test_crud_lifecycle` | Create → Read → Update → Delete → 404 |
| `test_health` | `GET /health` → 200 healthy |
| `test_history` | Create + Update → `/_history` 含 2 个版本 |
| `test_metadata` | `GET /metadata` → CapabilityStatement |
| `test_pagination` | `_count` / `_offset` + 分页链接 |
| `test_search` | 名称、性别、出生日期过滤 + 组合 |
| `test_validate` | 有效 → 200，无效 → 400 |

## CI/CD

GitHub Actions 在每次 push 和 PR 时运行：

| 任务 | 触发条件 | 说明 |
| ---- | -------- | ---- |
| **Format** | Push + PR | `cargo fmt --check` |
| **Build** | Push + PR | `cargo build`（core + server） |
| **Clippy** | 仅 PR | `cargo clippy -- -D warnings` |
| **Docker** | 仅 PR | `docker compose build` |
| **Integration** | 仅 PR | 构建测试 DB 镜像 + `cargo test` |

## 许可证

[MIT](../LICENSE)
