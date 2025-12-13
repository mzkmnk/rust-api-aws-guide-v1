# Day 2: セッション 1 - AWS アーキテクチャ設計（30 分）

[← 前へ: Day1 セッション 3](./03-day1-session3-implementation.md) | [概要](./00-overview.md)

---

## 1.1 本番環境構成

```mermaid
flowchart TB
    subgraph AWS["AWS Cloud"]
        Route53["Route 53 (DNS)<br/>api.example.com"]
        
        subgraph APIGateway["API Gateway (REST)"]
            Auth["Authentication (API Key)"]
            RateLimit["Rate Limiting"]
            CORS["CORS設定"]
        end
        
        subgraph ECS["ECS Cluster (Fargate)"]
            subgraph TaskDef["Task Definition (Rust Binary)"]
                CPU["CPU: 256 (0.25 vCPU)"]
                Memory["Memory: 512 MB"]
                Port["ContainerPort: 3000"]
                EnvVars["Environment Variables: DB_URL"]
            end
            AutoScale["Desired Count: 2 (Auto Scaling)"]
        end
        
        subgraph RDS["RDS (PostgreSQL 15)"]
            MultiAZ["Multi-AZ (本番要件)"]
            PerfInsights["Performance Insights有効"]
            Backups["Automated Backups (7日)"]
        end
        
        subgraph CloudWatch["CloudWatch Logs"]
            ECSLogs["ECS Container Logs"]
            LambdaLogs["Lambda Logs (必要に応じて)"]
        end
    end

    Route53 --> APIGateway
    APIGateway --> ECS
    ECS --> RDS
    ECS -.-> CloudWatch
```

---

## 1.2 コスト最適化のポイント

| リソース       | スペック                    | 月額コスト見積 |
| -------------- | --------------------------- | -------------- |
| ECS Fargate    | 0.25 vCPU × 512MB × 2 tasks | ~$15           |
| RDS PostgreSQL | db.t4g.micro (1 年契約)     | ~$25           |
| API Gateway    | 1M requests/月              | ~$3.50         |
| CloudWatch     | Logs retention 7 日         | ~$2            |
| **合計**       |                             | **~$45/月**    |

---

[次へ: セッション 2 - Docker コンテナ化 →](./05-day2-session2-docker.md)
