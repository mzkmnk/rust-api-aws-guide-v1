# User API - AWS Deployment

## デプロイ手順

### 前提条件

- AWS CLI 設定済み
- ACM 証明書を発行済み（ARN を控えておく）

### 1. Network Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-network \
  --template-file cloudformation/network.yml \
  --region ap-northeast-1
```

### 2. ECR Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-ecr \
  --template-file cloudformation/ecr.yml \
  --region ap-northeast-1
```

### 3. Database Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-db \
  --template-file cloudformation/database.yml \
  --parameter-overrides DBPassword=<your-password> \
  --region ap-northeast-1
```

### 4. ALB Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-alb \
  --template-file cloudformation/alb.yml \
  --parameter-overrides \
    CertificateArn=<your-certificate-arn> \
    HostedZoneId=<your-hosted-zone-id> \
    DomainName=<your-domain-name> \
  --region ap-northeast-1
```

### 5. ECS Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-ecs \
  --template-file cloudformation/ecs.yml \
  --parameter-overrides DBPassword=<your-password> \
  --capabilities CAPABILITY_IAM \
  --region ap-northeast-1
```

### 6. CICD Stack

```bash
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-cicd \
  --template-file cloudformation/cicd.yml \
  --capabilities CAPABILITY_IAM \
  --region ap-northeast-1
```

## 削除手順（逆順）

```bash
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-cicd --region ap-northeast-1
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-ecs --region ap-northeast-1
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-alb --region ap-northeast-1
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-db --region ap-northeast-1
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-ecr --region ap-northeast-1
aws cloudformation delete-stack --stack-name rust-api-aws-guide-v1-user-api-network --region ap-northeast-1
```
