from aws_cdk import (
    Stack,
    aws_ec2,
    aws_rds,
    aws_secretsmanager,
    aws_ecs,
    aws_ecs_patterns, SecretValue, Fn, CfnOutput, aws_s3,
)
from constructs import Construct
import os


image_tag = os.getenv("IMAGE_TAG", "latest")

DB_NAME = "hw2"
DB_USER = "dbadmin"
APP_DOCKER_IMAGE = f"danieltimko/recipes-rs:{image_tag}"


class Hw2Stack(Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs):
        super().__init__(scope, construct_id, **kwargs)

        # VPC for all resources
        vpc = aws_ec2.Vpc(self, "Hw2Vpc", max_azs=2)

        # Secret for database credentials
        db_credentials_secret = aws_rds.DatabaseSecret(self, "DbSecret", username=DB_USER)

        # Aurora PostgreSQL Cluster
        database = aws_rds.DatabaseCluster(self, "AuroraCluster",
            engine=aws_rds.DatabaseClusterEngine.aurora_postgres(version=aws_rds.AuroraPostgresEngineVersion.VER_16_6),
            credentials=aws_rds.Credentials.from_secret(db_credentials_secret),
            instances=2,
            instance_props=aws_rds.InstanceProps(
                instance_type=aws_ec2.InstanceType.of(
                    aws_ec2.InstanceClass.BURSTABLE3, aws_ec2.InstanceSize.MEDIUM),
                vpc=vpc
            ),
            default_database_name=DB_NAME
        )

        # Public S3 bucket for static content
        bucket = aws_s3.Bucket(
            self,
            "StaticContentBucket",
            bucket_name="recipes-rs-static-content",
            public_read_access=True,
            block_public_access=aws_s3.BlockPublicAccess(
                block_public_acls=False,
                ignore_public_acls=False,
                block_public_policy=False,
                restrict_public_buckets=False
            )
        )

        # Secret for database URL
        db_url = Fn.sub(
            body="postgres://${DBUser}:${DBPassword}@${DBHost}/${DBName}",
            variables={
                "DBUser": DB_USER,
                "DBPassword": db_credentials_secret.secret_value_from_json("password").unsafe_unwrap(),
                "DBHost": database.cluster_endpoint.hostname,
                "DBName": DB_NAME,
            },
        )
        db_url_secret = aws_secretsmanager.Secret(self, "DbUrlSecret",
            secret_string_value=SecretValue.unsafe_plain_text(db_url)
        )

        # ECS Cluster
        cluster = aws_ecs.Cluster(self, "EcsCluster", vpc=vpc)

        # Fargate service behind ALB
        fargate_service = aws_ecs_patterns.ApplicationLoadBalancedFargateService(self, "FargateService",
            cluster=cluster,
            cpu=512,
            memory_limit_mib=1024,
            desired_count=1,
            public_load_balancer=True,
            task_image_options=aws_ecs_patterns.ApplicationLoadBalancedTaskImageOptions(
                image=aws_ecs.ContainerImage.from_registry(APP_DOCKER_IMAGE),
                container_port=8080,
                secrets={
                    "DATABASE_URL": aws_ecs.Secret.from_secrets_manager(db_url_secret)
                }
            )
        )

        # Grant required permissions
        db_credentials_secret.grant_read(fargate_service.task_definition.task_role)
        database.connections.allow_default_port_from(fargate_service.service)
