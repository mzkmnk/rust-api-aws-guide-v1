# Day 2: セッション 4 - デプロイ検証と監視

[← 前へ: セッション 3](./06-day2-session3-ecs-deploy.md) | [概要](./00-overview.md)

---

## 4.1 スタック状態の確認

```bash
# 全スタックの状態確認
aws cloudformation describe-stacks \
  --query 'Stacks[?starts_with(StackName, `user-api`)].{Name:StackName,Status:StackStatus}' \
  --output table \
  --region ap-northeast-1

# 特定スタックの詳細
aws cloudformation describe-stack-events \
  --stack-name user-api-ecs \
  --query 'StackEvents[0:5].{Resource:LogicalResourceId,Status:ResourceStatus,Reason:ResourceStatusReason}' \
  --output table \
  --region ap-northeast-1
```

---

## 4.2 ECS サービスの確認

```bash
# サービス状態
aws ecs describe-services \
  --cluster user-api-cluster \
  --services user-api-service \
  --query 'services[0].{Status:status,Running:runningCount,Desired:desiredCount}' \
  --output table \
  --region ap-northeast-1

# タスクの IP アドレス取得
TASK_ARN=$(aws ecs list-tasks --cluster user-api-cluster --query 'taskArns[0]' --output text --region ap-northeast-1)

aws ecs describe-tasks \
  --cluster user-api-cluster \
  --tasks $TASK_ARN \
  --query 'tasks[0].attachments[0].details[?name==`privateIPv4Address`].value' \
  --output text \
  --region ap-northeast-1
```

---

## 4.3 ログの確認

```bash
# 最新ログを表示
aws logs tail /ecs/user-api --follow --region ap-northeast-1

# 直近 10 分のログ
aws logs filter-log-events \
  --log-group-name /ecs/user-api \
  --start-time $(date -v-10M +%s000) \
  --query 'events[].message' \
  --output text \
  --region ap-northeast-1
```

---

## 4.4 API 動作確認

```bash
# タスクの Public IP を取得
ENI_ID=$(aws ecs describe-tasks \
  --cluster user-api-cluster \
  --tasks $TASK_ARN \
  --query 'tasks[0].attachments[0].details[?name==`networkInterfaceId`].value' \
  --output text \
  --region ap-northeast-1)

PUBLIC_IP=$(aws ec2 describe-network-interfaces \
  --network-interface-ids $ENI_ID \
  --query 'NetworkInterfaces[0].Association.PublicIp' \
  --output text \
  --region ap-northeast-1)

# ヘルスチェック
curl http://$PUBLIC_IP:3000/health

# ユーザー作成
curl -X POST http://$PUBLIC_IP:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "test@example.com"}'

# ユーザー一覧
curl http://$PUBLIC_IP:3000/users
```

---

## 4.5 トラブルシューティング

### スタック作成失敗時

```bash
# エラー原因を確認
aws cloudformation describe-stack-events \
  --stack-name user-api-ecs \
  --query 'StackEvents[?ResourceStatus==`CREATE_FAILED`].{Resource:LogicalResourceId,Reason:ResourceStatusReason}' \
  --output table \
  --region ap-northeast-1
```

### タスクが起動しない場合

```bash
# 停止理由を確認
aws ecs describe-tasks \
  --cluster user-api-cluster \
  --tasks $TASK_ARN \
  --query 'tasks[0].{Status:lastStatus,Reason:stoppedReason}' \
  --region ap-northeast-1
```

---

## 4.6 リソースの削除

学習後は必ず削除してコストを抑えましょう。**逆順で削除**します。

```bash
# 1. ECS
aws cloudformation delete-stack --stack-name user-api-ecs --region ap-northeast-1
aws cloudformation wait stack-delete-complete --stack-name user-api-ecs --region ap-northeast-1

# 2. Database
aws cloudformation delete-stack --stack-name user-api-database --region ap-northeast-1
aws cloudformation wait stack-delete-complete --stack-name user-api-database --region ap-northeast-1

# 3. ECR（イメージがある場合は先に削除）
aws ecr delete-repository --repository-name user-api --force --region ap-northeast-1
aws cloudformation delete-stack --stack-name user-api-ecr --region ap-northeast-1

# 4. Network
aws cloudformation delete-stack --stack-name user-api-network --region ap-northeast-1
```

---

## 4.7 本番運用チェックリスト

- [ ] 全スタックが `CREATE_COMPLETE` 状態
- [ ] ECS タスクが `RUNNING` 状態
- [ ] CloudWatch Logs にエラーがない
- [ ] API エンドポイントが応答する
- [ ] 学習後にリソースを削除した

---

[次へ: リファレンス →](./08-reference.md)
