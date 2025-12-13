# Day 2: セッション 4 - デプロイ検証と監視

[← 前へ: セッション 3](./06-day2-session3-ecs-deploy.md) | [概要](./00-overview.md)

---

## 4.1 デプロイ確認コマンド

```bash
# ECSサービスのステータス確認
aws ecs describe-services \
  --cluster user-api-cluster \
  --services user-api-service \
  --region ap-northeast-1

# タスク一覧確認
aws ecs list-tasks \
  --cluster user-api-cluster \
  --region ap-northeast-1

# ログ確認
aws logs tail /ecs/user-api --follow --region ap-northeast-1

# CloudWatch メトリクス確認
aws cloudwatch get-metric-statistics \
  --namespace AWS/ECS \
  --metric-name CPUUtilization \
  --dimensions Name=ServiceName,Value=user-api-service \
  --statistics Average \
  --start-time 2025-12-13T00:00:00Z \
  --end-time 2025-12-14T00:00:00Z \
  --period 300 \
  --region ap-northeast-1
```

---

## 4.2 ヘルスチェック設定

```bash
# ALB（Application Load Balancer）経由でのヘルスチェック
# ヘルスチェック用エンドポイント追加（main.rs）
app = app.route("/health", get(|| async { "OK" }))
```

```bash
# ALB ターゲットグループ設定
aws elbv2 create-target-group \
  --name user-api-targets \
  --protocol HTTP \
  --port 3000 \
  --vpc-id vpc-xxxxxxxx \
  --health-check-path /health \
  --health-check-interval-seconds 30 \
  --health-check-timeout-seconds 5 \
  --healthy-threshold-count 2 \
  --unhealthy-threshold-count 2 \
  --region ap-northeast-1
```

---

## 4.3 本番運用チェックリスト

- [ ] ECS タスクが両方起動している
- [ ] CloudWatch Logs にエラーがない
- [ ] RDS コネクション数が正常
- [ ] API Gateway が正常に動作している
- [ ] ヘルスチェック（/health）が通っている
- [ ] 定期的なバックアップが有効
- [ ] CloudWatch アラーム設定済み
- [ ] 本番環境用の.env 設定完了

---

[次へ: リファレンス →](./08-reference.md)
