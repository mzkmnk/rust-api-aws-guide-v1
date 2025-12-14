PROJECT_NAME := rust-api-aws-guide-v1-user-api

# network.yml
deploy-network:
	aws cloudformation create-stack \
	--stack-name $(PROJECT_NAME)-network \
	--template-body file://cloudformation/network.yml \
	--region ap-northeast-1

# ECR
deploy-ecr:
	aws cloudformation create-stack \
	--stack-name $(PROJECT_NAME)-ecr
	--template-body file://cloudformation/ecr.yml \
	--region ap-northeast-1

# push docker image

# RDS
deploy-rds:
	aws cloudformation create-stack \
	--stack-name $(PROJECT_NAME)-db \
	--template-body file://cloudformation/database.yml \
	--parameters ParameterKey=DBPassword,ParameterValue= \
	--region ap-northeast-1

# ECS
deploy-ecs:
	aws cloudformation create-stack \
	--stack-name $(PROJECT_NAME)-ecs \
	--template-body file://cloudformation/ecs.yml \
	--parameters ParameterKey=DBPassword,ParameterValue= \
	--capabilities CAPABILITY_IAM \
	--region ap-northeast-1