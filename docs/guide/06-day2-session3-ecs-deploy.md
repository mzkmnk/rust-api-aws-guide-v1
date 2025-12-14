# Day 2: セッション 3 - CloudFormation でインフラ構築

[← 前へ: セッション 2](./05-day2-session2-docker.md) | [概要](./00-overview.md)

---

## 3.1 ディレクトリ構成

```bash
mkdir -p infrastructure/cloudformation
```

---

## 3.2 network.yaml - ネットワークリソース

VPC、サブネット、セキュリティグループを定義します。

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: Network resources for User API

Parameters:
  ProjectName:
    Type: String
    Default: user-api

Outputs:
  VpcId:
    Value: !Ref VPC
    Export:
      Name: !Sub ${ProjectName}-VpcId
  PublicSubnet1Id:
    Value: !Ref PublicSubnet1
    Export:
      Name: !Sub ${ProjectName}-PublicSubnet1Id
  PublicSubnet2Id:
    Value: !Ref PublicSubnet2
    Export:
      Name: !Sub ${ProjectName}-PublicSubnet2Id
  EcsSecurityGroupId:
    Value: !Ref EcsSecurityGroup
    Export:
      Name: !Sub ${ProjectName}-EcsSecurityGroupId
  RdsSecurityGroupId:
    Value: !Ref RdsSecurityGroup
    Export:
      Name: !Sub ${ProjectName}-RdsSecurityGroupId

Resources:
  # VPC
  VPC:
    Type: AWS::EC2::VPC
    Properties:
      CidrBlock: 10.0.0.0/16
      EnableDnsHostnames: true
      EnableDnsSupport: true
      Tags:
        - Key: Name
          Value: !Sub ${ProjectName}-vpc

  # Internet Gateway
  InternetGateway:
    Type: AWS::EC2::InternetGateway

  AttachGateway:
    Type: AWS::EC2::VPCGatewayAttachment
    Properties:
      VpcId: !Ref VPC
      InternetGatewayId: !Ref InternetGateway

  # Public Subnets (2 AZs required for RDS)
  PublicSubnet1:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref VPC
      CidrBlock: 10.0.1.0/24
      AvailabilityZone: !Select [0, !GetAZs '']
      MapPublicIpOnLaunch: true
      Tags:
        - Key: Name
          Value: !Sub ${ProjectName}-public-1

  PublicSubnet2:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref VPC
      CidrBlock: 10.0.2.0/24
      AvailabilityZone: !Select [1, !GetAZs '']
      MapPublicIpOnLaunch: true
      Tags:
        - Key: Name
          Value: !Sub ${ProjectName}-public-2

  # Route Table
  PublicRouteTable:
    Type: AWS::EC2::RouteTable
    Properties:
      VpcId: !Ref VPC

  PublicRoute:
    Type: AWS::EC2::Route
    DependsOn: AttachGateway
    Properties:
      RouteTableId: !Ref PublicRouteTable
      DestinationCidrBlock: 0.0.0.0/0
      GatewayId: !Ref InternetGateway

  PublicSubnet1RouteTableAssociation:
    Type: AWS::EC2::SubnetRouteTableAssociation
    Properties:
      SubnetId: !Ref PublicSubnet1
      RouteTableId: !Ref PublicRouteTable

  PublicSubnet2RouteTableAssociation:
    Type: AWS::EC2::SubnetRouteTableAssociation
    Properties:
      SubnetId: !Ref PublicSubnet2
      RouteTableId: !Ref PublicRouteTable

  # Security Group for ECS (port 3000)
  EcsSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Allow traffic to ECS tasks
      VpcId: !Ref VPC
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 3000
          ToPort: 3000
          CidrIp: 0.0.0.0/0

  # Security Group for RDS (only from ECS)
  RdsSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Allow traffic from ECS to RDS
      VpcId: !Ref VPC
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 5432
          ToPort: 5432
          SourceSecurityGroupId: !Ref EcsSecurityGroup
```

**学習ポイント**:
- `!Ref` で同一テンプレート内のリソースを参照
- `!Sub` で文字列内に変数を埋め込み
- `Export` で他スタックから参照可能にする
- `!Select` と `!GetAZs` で AZ を動的に取得

**デプロイ**:
```bash
aws cloudformation create-stack \
  --stack-name user-api-network \
  --template-body file://infrastructure/cloudformation/network.yaml \
  --region ap-northeast-1
```

---

## 3.3 ecr.yaml - コンテナレジストリ

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: ECR Repository for User API

Parameters:
  ProjectName:
    Type: String
    Default: user-api

Outputs:
  RepositoryUri:
    Value: !GetAtt Repository.RepositoryUri
    Export:
      Name: !Sub ${ProjectName}-RepositoryUri

Resources:
  Repository:
    Type: AWS::ECR::Repository
    Properties:
      RepositoryName: !Ref ProjectName
      LifecyclePolicy:
        LifecyclePolicyText: |
          {
            "rules": [{
              "rulePriority": 1,
              "description": "Keep only 3 images",
              "selection": {
                "tagStatus": "any",
                "countType": "imageCountMoreThan",
                "countNumber": 3
              },
              "action": { "type": "expire" }
            }]
          }
```

**学習ポイント**:
- `LifecyclePolicy` で古いイメージを自動削除（コスト削減）
- `!GetAtt` でリソースの属性を取得

**デプロイ**:
```bash
aws cloudformation create-stack \
  --stack-name user-api-ecr \
  --template-body file://infrastructure/cloudformation/ecr.yaml \
  --region ap-northeast-1
```

**イメージのプッシュ**:
```bash
# ECR にログイン
aws ecr get-login-password --region ap-northeast-1 | \
  docker login --username AWS --password-stdin <ACCOUNT_ID>.dkr.ecr.ap-northeast-1.amazonaws.com

# イメージをビルド・タグ付け・プッシュ
docker build -t user-api .
docker tag user-api:latest <ACCOUNT_ID>.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest
docker push <ACCOUNT_ID>.dkr.ecr.ap-northeast-1.amazonaws.com/user-api:latest
```

---

## 3.4 database.yaml - RDS PostgreSQL

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: RDS PostgreSQL for User API

Parameters:
  ProjectName:
    Type: String
    Default: user-api
  DBPassword:
    Type: String
    NoEcho: true
    Description: Database master password

Outputs:
  RdsEndpoint:
    Value: !GetAtt RdsInstance.Endpoint.Address
    Export:
      Name: !Sub ${ProjectName}-RdsEndpoint

Resources:
  DBSubnetGroup:
    Type: AWS::RDS::DBSubnetGroup
    Properties:
      DBSubnetGroupDescription: Subnet group for RDS
      SubnetIds:
        - !ImportValue
          Fn::Sub: ${ProjectName}-PublicSubnet1Id
        - !ImportValue
          Fn::Sub: ${ProjectName}-PublicSubnet2Id

  RdsInstance:
    Type: AWS::RDS::DBInstance
    DeletionPolicy: Delete
    Properties:
      DBInstanceIdentifier: !Sub ${ProjectName}-db
      DBInstanceClass: db.t4g.micro
      Engine: postgres
      EngineVersion: '15'
      MasterUsername: postgres
      MasterUserPassword: !Ref DBPassword
      DBName: userdb
      AllocatedStorage: 20
      StorageType: gp2
      PubliclyAccessible: false
      VPCSecurityGroups:
        - !ImportValue
          Fn::Sub: ${ProjectName}-RdsSecurityGroupId
      DBSubnetGroupName: !Ref DBSubnetGroup
      BackupRetentionPeriod: 0
```

**学習ポイント**:
- `!ImportValue` で他スタックの Export を参照
- `NoEcho: true` でパスワードをログに出力しない
- `DeletionPolicy: Delete` で学習後の削除を容易に

**デプロイ**:
```bash
aws cloudformation create-stack \
  --stack-name user-api-database \
  --template-body file://infrastructure/cloudformation/database.yaml \
  --parameters ParameterKey=DBPassword,ParameterValue=YourSecurePassword123 \
  --region ap-northeast-1
```

> **注意**: RDS の作成には 5〜10 分かかります。

---

## 3.5 ecs.yaml - ECS Fargate

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: ECS Fargate for User API

Parameters:
  ProjectName:
    Type: String
    Default: user-api
  ImageTag:
    Type: String
    Default: latest
  DBPassword:
    Type: String
    NoEcho: true

Resources:
  EcsCluster:
    Type: AWS::ECS::Cluster
    Properties:
      ClusterName: !Sub ${ProjectName}-cluster

  LogGroup:
    Type: AWS::Logs::LogGroup
    Properties:
      LogGroupName: !Sub /ecs/${ProjectName}
      RetentionInDays: 3

  TaskExecutionRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Effect: Allow
            Principal:
              Service: ecs-tasks.amazonaws.com
            Action: sts:AssumeRole
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy

  TaskDefinition:
    Type: AWS::ECS::TaskDefinition
    Properties:
      Family: !Sub ${ProjectName}-task
      NetworkMode: awsvpc
      RequiresCompatibilities: [FARGATE]
      Cpu: '256'
      Memory: '512'
      ExecutionRoleArn: !GetAtt TaskExecutionRole.Arn
      ContainerDefinitions:
        - Name: !Ref ProjectName
          Image: !Sub
            - ${RepositoryUri}:${ImageTag}
            - RepositoryUri: !ImportValue
                Fn::Sub: ${ProjectName}-RepositoryUri
          PortMappings:
            - ContainerPort: 3000
          Environment:
            - Name: DATABASE_URL
              Value: !Sub
                - postgresql://postgres:${DBPassword}@${RdsEndpoint}:5432/userdb
                - RdsEndpoint: !ImportValue
                    Fn::Sub: ${ProjectName}-RdsEndpoint
            - Name: RUST_LOG
              Value: info
          LogConfiguration:
            LogDriver: awslogs
            Options:
              awslogs-group: !Ref LogGroup
              awslogs-region: !Ref AWS::Region
              awslogs-stream-prefix: ecs

  EcsService:
    Type: AWS::ECS::Service
    Properties:
      ServiceName: !Sub ${ProjectName}-service
      Cluster: !Ref EcsCluster
      TaskDefinition: !Ref TaskDefinition
      DesiredCount: 1
      LaunchType: FARGATE
      NetworkConfiguration:
        AwsvpcConfiguration:
          AssignPublicIp: ENABLED
          Subnets:
            - !ImportValue
              Fn::Sub: ${ProjectName}-PublicSubnet1Id
            - !ImportValue
              Fn::Sub: ${ProjectName}-PublicSubnet2Id
          SecurityGroups:
            - !ImportValue
              Fn::Sub: ${ProjectName}-EcsSecurityGroupId
```

**学習ポイント**:
- `TaskExecutionRole` は ECS がイメージ取得・ログ出力に使用
- `awsvpc` モードで各タスクに ENI が割り当てられる
- `!Sub` のリスト形式で複数の変数を置換

**デプロイ**:
```bash
aws cloudformation create-stack \
  --stack-name user-api-ecs \
  --template-body file://infrastructure/cloudformation/ecs.yaml \
  --parameters ParameterKey=DBPassword,ParameterValue=YourSecurePassword123 \
  --capabilities CAPABILITY_IAM \
  --region ap-northeast-1
```

> `--capabilities CAPABILITY_IAM` は IAM リソース作成時に必要です。

---

## 3.6 デプロイ順序まとめ

```bash
# 1. ネットワーク
aws cloudformation create-stack --stack-name user-api-network \
  --template-body file://infrastructure/cloudformation/network.yaml \
  --region ap-northeast-1

# 2. ECR
aws cloudformation create-stack --stack-name user-api-ecr \
  --template-body file://infrastructure/cloudformation/ecr.yaml \
  --region ap-northeast-1

# 3. Docker イメージをプッシュ（ECR 作成後）

# 4. RDS
aws cloudformation create-stack --stack-name user-api-database \
  --template-body file://infrastructure/cloudformation/database.yaml \
  --parameters ParameterKey=DBPassword,ParameterValue=YourSecurePassword123 \
  --region ap-northeast-1

# 5. ECS（RDS 作成完了後）
aws cloudformation create-stack --stack-name user-api-ecs \
  --template-body file://infrastructure/cloudformation/ecs.yaml \
  --parameters ParameterKey=DBPassword,ParameterValue=YourSecurePassword123 \
  --capabilities CAPABILITY_IAM \
  --region ap-northeast-1
```

---

[次へ: セッション 4 - デプロイ検証と監視 →](./07-day2-session4-monitoring.md)
