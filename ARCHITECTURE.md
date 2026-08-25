# trade 量化交易系统架构

## 目录结构

```
trade/
├── Cargo.toml                 # Cargo workspace 根配置
├── crates/
│   ├── common/                # 共享库（trade-common）
│   ├── collector/             # 数据采集器
│   └── strategy/              # 策略引擎
├── web/
│   ├── web-backend/           # Web 服务后端
│   └── web-client/                 # 前端页面（纯展示）
└── tradedata/                 # SQLite 数据库
```

## 核心架构原则：交易与展示彻底解耦

**交易程序（collector + strategy）与 Web 前后端（web-backend + web-client）零耦合。**

删除 `web-backend` 和 `web-client` 后，交易程序照常运行，不受任何影响。

### 依赖关系（单向）

```
                 trade-common（共享库）
                      ▲
        ┌─────────────┼─────────────┐
        │             │             │
   collector      strategy      web-backend
   （采集器）      （策略引擎）     （Web 服务）
        │             │             │
        └─────────────┼─────────────┘
                      ▼
                 SQLite（market.db）
                      ▲
                      │
                   web-client（纯展示，通过 HTTP 读 web-backend）
```

关键点：
- `common` 是唯一的共享库，被三个程序依赖，但它**不依赖任何 Web 组件**
- 交易程序（collector、strategy）**只依赖 `trade-common`**，不依赖 `web-backend`
- Web 后端（web-backend）只读库，**不参与任何交易决策**
- 前端（web-client）只做展示，**完全不接触数据库和币安**

## 各程序职责

| 程序 | 依赖 | 职责 | 数据流 |
|------|------|------|--------|
| collector | trade-common | 采集行情 | 币安 → 写库 |
| strategy | trade-common | 交易决策 | 读库 → 算指标 → 下单 |
| web-backend | trade-common + axum | 数据展示接口 | 读库 → 返回 JSON |
| web-client | 无（静态） | 图表展示 | HTTP → web-backend |

## 运行方式

```bash
cd /Users/wangbinmac/Desktop/trade

# 采集器（常驻后台，持续积累数据）
cargo run -p collector

# 策略引擎（需 API Key）
BINANCE_API_KEY=xxx BINANCE_SECRET_KEY=yyy cargo run -p strategy

# Web 服务（供前端展示）
cargo run -p web-backend

# 前端（另开终端）
cd web/web-client && python3 -m http.server 8080
```

## 为什么这样设计

1. **交易是核心，展示是附属**：交易程序运行时不依赖 Web，即使前端挂了、后端 Web 服务崩了，交易照常进行。
2. **职责单一**：采集管数据、策略管决策、Web 管展示，互不干扰。
3. **可独立部署**：生产环境可以只部署 collector + strategy（交易），Web 前后端按需部署。
4. **共享代码复用**：binance/indicators/storage 三块核心逻辑集中在 common，避免重复。
