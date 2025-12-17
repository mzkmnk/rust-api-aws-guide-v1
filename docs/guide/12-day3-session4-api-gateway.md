# Day 3: セッション 4 - API Gateway の追加

[← 前へ: セッション 3](./11-day3-session3-cicd.md) | [概要](./00-overview.md) | [次へ: リファレンス →](./08-reference.md)

---

## 4.1 学習目標

- API Gateway の役割を理解する
- CloudFormation で API Gateway を構築する
- ALB との連携を設定する

---

## 4.2 API Gateway を使う理由

| 機能 | 説明 |
|------|------|
| レート制限 | API の過負荷を防止 |
| 認証統合 | Cognito、IAM 認証 |
| API キー管理 | 利用者ごとのアクセス制御 |
| リクエスト変換 | ヘッダー追加、パス変換 |
| キャッシュ | レスポンスのキャッシュ |
| ログ・監視 | CloudWatch 連携 |

### アーキテクチャ

```
Internet → API Gateway → ALB → ECS → RDS
           ~~~~~~~~~~~~
           今回追加する部分
```

---

## 4.3 実装タスク

### Task 1: REST API リソース定義

**やること**: `cloudformation/api-gateway.yml` を新規作成し、REST API の基本構造を定義

**要件**:
- REST API を作成（名前: `user-api`）
- エンドポイントタイプは `REGIONAL`
- `/api` リソースを作成
- `/api/users` リソースを作成
- `/api/users/{id}` リソースを作成

<details>
<summary>ヒント</summary>

- `AWS::ApiGateway::RestApi` で API を作成
- `AWS::ApiGateway::Resource` でパスを定義
- `ParentId` で親リソースを指定（ルートは `!GetAtt RestApi.RootResourceId`）
- `PathPart` でパスの一部を指定（`{id}` でパスパラメータ）

</details>

<details>
<summary>コード例</summary>

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: API Gateway for User API

Parameters:
  ALBDnsName:
    Type: String
    Description: ALB DNS Name
  StageName:
    Type: String
    Default: prod
    Description: API Gateway Stage Name

Resources:
  # REST API
  UserApi:
    Type: AWS::ApiGateway::RestApi
    Properties:
      Name: user-api
      Description: User API Gateway
      EndpointConfiguration:
        Types:
          - REGIONAL

  # /api リソース
  ApiResource:
    Type: AWS::ApiGateway::Resource
    Properties:
      RestApiId: !Ref UserApi
      ParentId: !GetAtt UserApi.RootResourceId
      PathPart: api

  # /api/users リソース
  UsersResource:
    Type: AWS::ApiGateway::Resource
    Properties:
      RestApiId: !Ref UserApi
      ParentId: !Ref ApiResource
      PathPart: users

  # /api/users/{id} リソース
  UserIdResource:
    Type: AWS::ApiGateway::Resource
    Properties:
      RestApiId: !Ref UserApi
      ParentId: !Ref UsersResource
      PathPart: '{id}'
```

</details>

---

### Task 2: HTTP メソッド定義

**やること**: 各エンドポイントの HTTP メソッドを定義

**要件**:
- `GET /api/users` - ユーザー一覧
- `POST /api/users` - ユーザー作成
- `GET /api/users/{id}` - ユーザー取得
- `PATCH /api/users/{id}` - ユーザー更新
- `DELETE /api/users/{id}` - ユーザー削除
- 統合タイプは `HTTP_PROXY`（リクエストをそのまま ALB に転送）

<details>
<summary>ヒント</summary>

- `AWS::ApiGateway::Method` でメソッドを定義
- `Integration.Type: HTTP_PROXY` でプロキシ統合
- `Integration.Uri` で ALB の URL を指定
- パスパラメータは `RequestParameters` で定義し、`Integration.RequestParameters` でマッピング

</details>

<details>
<summary>コード例（GET/POST /api/users）</summary>

```yaml
  # GET /api/users
  UsersGetMethod:
    Type: AWS::ApiGateway::Method
    Properties:
      RestApiId: !Ref UserApi
      ResourceId: !Ref UsersResource
      HttpMethod: GET
      AuthorizationType: NONE
      Integration:
        Type: HTTP_PROXY
        IntegrationHttpMethod: GET
        Uri: !Sub 'http://${ALBDnsName}/api/users'
        IntegrationResponses:
          - StatusCode: '200'
      MethodResponses:
        - StatusCode: '200'

  # POST /api/users
  UsersPostMethod:
    Type: AWS::ApiGateway::Method
    Properties:
      RestApiId: !Ref UserApi
      ResourceId: !Ref UsersResource
      HttpMethod: POST
      AuthorizationType: NONE
      Integration:
        Type: HTTP_PROXY
        IntegrationHttpMethod: POST
        Uri: !Sub 'http://${ALBDnsName}/api/users'
        IntegrationResponses:
          - StatusCode: '201'
      MethodResponses:
        - StatusCode: '201'
```

</details>

<details>
<summary>コード例（/api/users/{id} のメソッド）</summary>

```yaml
  # GET /api/users/{id}
  UserGetMethod:
    Type: AWS::ApiGateway::Method
    Properties:
      RestApiId: !Ref UserApi
      ResourceId: !Ref UserIdResource
      HttpMethod: GET
      AuthorizationType: NONE
      RequestParameters:
        method.request.path.id: true
      Integration:
        Type: HTTP_PROXY
        IntegrationHttpMethod: GET
        Uri: !Sub 'http://${ALBDnsName}/api/users/{id}'
        RequestParameters:
          integration.request.path.id: method.request.path.id
        IntegrationResponses:
          - StatusCode: '200'
      MethodResponses:
        - StatusCode: '200'

  # PATCH /api/users/{id}
  UserPatchMethod:
    Type: AWS::ApiGateway::Method
    Properties:
      RestApiId: !Ref UserApi
      ResourceId: !Ref UserIdResource
      HttpMethod: PATCH
      AuthorizationType: NONE
      RequestParameters:
        method.request.path.id: true
      Integration:
        Type: HTTP_PROXY
        IntegrationHttpMethod: PATCH
        Uri: !Sub 'http://${ALBDnsName}/api/users/{id}'
        RequestParameters:
          integration.request.path.id: method.request.path.id
        IntegrationResponses:
          - StatusCode: '200'
      MethodResponses:
        - StatusCode: '200'

  # DELETE /api/users/{id}
  UserDeleteMethod:
    Type: AWS::ApiGateway::Method
    Properties:
      RestApiId: !Ref UserApi
      ResourceId: !Ref UserIdResource
      HttpMethod: DELETE
      AuthorizationType: NONE
      RequestParameters:
        method.request.path.id: true
      Integration:
        Type: HTTP_PROXY
        IntegrationHttpMethod: DELETE
        Uri: !Sub 'http://${ALBDnsName}/api/users/{id}'
        RequestParameters:
          integration.request.path.id: method.request.path.id
        IntegrationResponses:
          - StatusCode: '204'
      MethodResponses:
        - StatusCode: '204'
```

</details>

---

### Task 3: デプロイメントとステージ定義

**やること**: API をデプロイしてアクセス可能にする

**要件**:
- デプロイメントリソースを作成
- ステージを作成（`prod`）
- ログとメトリクスを有効化
- エンドポイント URL を出力

<details>
<summary>ヒント</summary>

- `AWS::ApiGateway::Deployment` でデプロイメント作成
- `DependsOn` で全メソッドの作成完了を待つ
- `AWS::ApiGateway::Stage` でステージ作成
- `Outputs` でエンドポイント URL をエクスポート

</details>

<details>
<summary>コード例</summary>

```yaml
  # デプロイメント
  ApiDeployment:
    Type: AWS::ApiGateway::Deployment
    DependsOn:
      - UsersGetMethod
      - UsersPostMethod
      - UserGetMethod
      - UserPatchMethod
      - UserDeleteMethod
    Properties:
      RestApiId: !Ref UserApi

  # ステージ
  ApiStage:
    Type: AWS::ApiGateway::Stage
    Properties:
      RestApiId: !Ref UserApi
      DeploymentId: !Ref ApiDeployment
      StageName: !Ref StageName
      MethodSettings:
        - ResourcePath: '/*'
          HttpMethod: '*'
          LoggingLevel: INFO
          DataTraceEnabled: true
          MetricsEnabled: true

Outputs:
  ApiEndpoint:
    Description: API Gateway Endpoint URL
    Value: !Sub 'https://${UserApi}.execute-api.${AWS::Region}.amazonaws.com/${StageName}'
    Export:
      Name: UserApiEndpoint

  ApiId:
    Description: API Gateway ID
    Value: !Ref UserApi
    Export:
      Name: UserApiId
```

</details>

---

### Task 4: スタックのデプロイ

**やること**: CloudFormation でスタックをデプロイ

**手順**:

```bash
# Step 1: ALB の DNS 名を取得
ALB_DNS=$(aws cloudformation describe-stacks \
  --stack-name rust-api-aws-guide-v1-user-api-alb \
  --query 'Stacks[0].Outputs[?OutputKey==`ALBDnsName`].OutputValue' \
  --output text \
  --region ap-northeast-1)

echo $ALB_DNS

# Step 2: API Gateway スタックをデプロイ
aws cloudformation deploy \
  --stack-name rust-api-aws-guide-v1-user-api-gateway \
  --template-file cloudformation/api-gateway.yml \
  --parameter-overrides ALBDnsName=$ALB_DNS \
  --region ap-northeast-1

# Step 3: エンドポイント URL を取得
API_ENDPOINT=$(aws cloudformation describe-stacks \
  --stack-name rust-api-aws-guide-v1-user-api-gateway \
  --query 'Stacks[0].Outputs[?OutputKey==`ApiEndpoint`].OutputValue' \
  --output text \
  --region ap-northeast-1)

echo $API_ENDPOINT
```

---

## 4.4 動作確認

```bash
# ユーザー一覧
curl $API_ENDPOINT/api/users

# ユーザー作成
curl -X POST $API_ENDPOINT/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "API Gateway Test", "email": "apigw@example.com"}'

# ユーザー取得
curl $API_ENDPOINT/api/users/1

# ユーザー更新
curl -X PATCH $API_ENDPOINT/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated via API Gateway"}'

# ユーザー削除
curl -X DELETE $API_ENDPOINT/api/users/1
```

---

## 4.5 学習ポイント

<details>
<summary>API Gateway の統合タイプ</summary>

| タイプ | 説明 |
|--------|------|
| HTTP_PROXY | リクエストをそのまま転送 |
| HTTP | リクエスト/レスポンスを変換可能 |
| AWS | Lambda など AWS サービス連携 |
| MOCK | モックレスポンスを返す |

</details>

<details>
<summary>パスパラメータの受け渡し</summary>

```yaml
# メソッドでパラメータを定義
RequestParameters:
  method.request.path.id: true

# 統合でバックエンドに渡す
Integration:
  RequestParameters:
    integration.request.path.id: method.request.path.id
```

</details>

<details>
<summary>ステージ変数</summary>

```yaml
# 環境ごとに異なる設定
ApiStage:
  StageName: prod  # dev, staging, prod など
```

</details>

---

## 4.6 オプション: API キー認証の追加

<details>
<summary>API キー認証の実装例</summary>

```yaml
# API キー
ApiKey:
  Type: AWS::ApiGateway::ApiKey
  Properties:
    Name: user-api-key
    Enabled: true

# 使用量プラン
UsagePlan:
  Type: AWS::ApiGateway::UsagePlan
  Properties:
    UsagePlanName: user-api-plan
    Throttle:
      RateLimit: 100
      BurstLimit: 200
    Quota:
      Limit: 10000
      Period: MONTH
    ApiStages:
      - ApiId: !Ref UserApi
        Stage: !Ref ApiStage

# メソッドで API キーを要求
UsersGetMethod:
  Properties:
    ApiKeyRequired: true
```

</details>

---

## 4.7 リソースの削除

```bash
# API Gateway スタックを削除
aws cloudformation delete-stack \
  --stack-name rust-api-aws-guide-v1-user-api-gateway \
  --region ap-northeast-1
```

---

## 4.8 完了チェックリスト

- [ ] `cloudformation/api-gateway.yml` を作成
- [ ] ALB の DNS 名を取得
- [ ] API Gateway スタックをデプロイ
- [ ] エンドポイント URL を取得
- [ ] 各 HTTP メソッドの動作を確認
- [ ] 学習後にリソースを削除

---

## 4.9 Day 3 まとめ

Day 3 で学んだこと:

| セッション | 内容 | 学習ポイント |
|-----------|------|-------------|
| 1 | Update 機能 | Option 型、COALESCE、部分更新 |
| 2 | JWT 認証 | ミドルウェア、Bearer トークン |
| 3 | CI/CD | GitHub Actions、ECR/ECS デプロイ |
| 4 | API Gateway | CloudFormation、HTTP_PROXY 統合 |

---

[リファレンスへ →](./08-reference.md)
