# Day 2: セッション 3 - AWS ECS/Fargate へのデプロイ（1 時間）

[← 前へ: セッション 2](./05-day2-session2-docker.md) | [概要](./00-overview.md)

---

## 3.1 Amazon ECR（イメージレジストリ）へのプッシュ

```bash
# AWS CLIで認証
aws ecr get-login-password --region ap-northeast-1 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.ap-northeast-1.amazonaws.com

# リポジトリ作成
aws ecr create-repository --repository-name user-api --region ap-northeast-1

# イメージタグ付与
docker tag user-api:latest 123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest

# プッシュ
docker push 123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest
```

---

## 3.2 RDS PostgreSQL セットアップ

```bash
# AWS CLI でRDSインスタンス作成
aws rds create-db-instance \
  --db-instance-identifier user-api-db \
  --db-instance-class db.t4g.micro \
  --engine postgres \
  --master-username postgres \
  --master-user-password "GenerateStrongPassword123!" \
  --allocated-storage 20 \
  --vpc-security-group-ids sg-xxxxxxxx \
  --db-subnet-group-name default \
  --publicly-accessible false \
  --region ap-northeast-1

# RDS作成完了後、初期スキーマを流す
psql -h user-api-db.cxxxxxxx.ap-northeast-1.rds.amazonaws.com \
  -U postgres \
  < schema.sql
```

---

## 3.3 ECS クラスター・タスク定義

```bash
# クラスター作成
aws ecs create-cluster --cluster-name user-api-cluster --region ap-northeast-1

# タスク定義登録（task-definition.json から）
aws ecs register-task-definition \
  --cli-input-json file://task-definition.json \
  --region ap-northeast-1

# サービス作成
aws ecs create-service \
  --cluster user-api-cluster \
  --service-name user-api-service \
  --task-definition user-api-task:1 \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-xxx],securityGroups=[sg-xxx],assignPublicIp=ENABLED}" \
  --region ap-northeast-1
```

---

## 3.4 task-definition.json テンプレート

```json
{
  "family": "user-api-task",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "containerDefinitions": [
    {
      "name": "user-api",
      "image": "123456789.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest",
      "portMappings": [
        {
          "containerPort": 3000,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DATABASE_URL",
          "value": "postgresql://postgres:password@user-api-db.cxxxxxxx.ap-northeast-1.rds.amazonaws.com/userdb"
        },
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/user-api",
          "awslogs-region": "ap-northeast-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

---

## 3.5 API Gateway 設定（オプション）

```bash
# REST API作成
aws apigateway create-rest-api \
  --name user-api \
  --description "User Management API" \
  --region ap-northeast-1

# リソース・メソッド設定は AWSコンソール or Terraform推奨
```

---

[次へ: セッション 4 - デプロイ検証と監視 →](./07-day2-session4-monitoring.md)
