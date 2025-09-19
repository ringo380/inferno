# Outputs for Inferno Infrastructure

# VPC Outputs
output "vpc_id" {
  description = "ID of the VPC"
  value       = module.vpc.vpc_id
}

output "vpc_cidr_block" {
  description = "The CIDR block of the VPC"
  value       = module.vpc.vpc_cidr_block
}

output "private_subnets" {
  description = "List of IDs of private subnets"
  value       = module.vpc.private_subnets
}

output "public_subnets" {
  description = "List of IDs of public subnets"
  value       = module.vpc.public_subnets
}

output "database_subnets" {
  description = "List of IDs of database subnets"
  value       = module.vpc.database_subnets
}

# EKS Outputs
output "cluster_name" {
  description = "Name of the EKS cluster"
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = module.eks.cluster_endpoint
}

output "cluster_security_group_id" {
  description = "Security group ids attached to the cluster control plane"
  value       = module.eks.cluster_security_group_id
}

output "cluster_iam_role_name" {
  description = "IAM role name associated with EKS cluster"
  value       = module.eks.cluster_iam_role_name
}

output "cluster_oidc_issuer_url" {
  description = "The URL on the EKS cluster OIDC Issuer"
  value       = module.eks.cluster_oidc_issuer_url
}

output "cluster_primary_security_group_id" {
  description = "The cluster primary security group ID created by EKS"
  value       = module.eks.cluster_primary_security_group_id
}

output "oidc_provider_arn" {
  description = "The ARN of the OIDC Provider if enabled"
  value       = module.eks.oidc_provider_arn
}

# Node Groups
output "eks_managed_node_groups" {
  description = "Map of attribute maps for all EKS managed node groups created"
  value       = module.eks.eks_managed_node_groups
  sensitive   = true
}

# Karpenter
output "karpenter_role_arn" {
  description = "The Amazon Resource Name (ARN) specifying the Karpenter role"
  value       = module.karpenter.role_arn
}

output "karpenter_instance_queue_name" {
  description = "The name of the SQS queue for Karpenter"
  value       = module.karpenter.queue_name
}

# Load Balancer
output "alb_dns_name" {
  description = "The DNS name of the load balancer"
  value       = aws_lb.main.dns_name
}

output "alb_arn" {
  description = "The ARN of the load balancer"
  value       = aws_lb.main.arn
}

output "alb_zone_id" {
  description = "The canonical hosted zone ID of the load balancer"
  value       = aws_lb.main.zone_id
}

# RDS Outputs
output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = module.rds.db_instance_endpoint
  sensitive   = true
}

output "rds_port" {
  description = "RDS instance port"
  value       = module.rds.db_instance_port
}

output "rds_database_name" {
  description = "RDS database name"
  value       = module.rds.db_instance_name
}

output "rds_username" {
  description = "RDS database username"
  value       = module.rds.db_instance_username
  sensitive   = true
}

# Redis Outputs
output "redis_endpoint" {
  description = "Redis endpoint"
  value       = aws_elasticache_replication_group.main.primary_endpoint_address
  sensitive   = true
}

output "redis_port" {
  description = "Redis port"
  value       = aws_elasticache_replication_group.main.port
}

# S3 Outputs
output "s3_models_bucket" {
  description = "Name of the S3 bucket for models"
  value       = aws_s3_bucket.models.bucket
}

output "s3_models_bucket_arn" {
  description = "ARN of the S3 bucket for models"
  value       = aws_s3_bucket.models.arn
}

output "s3_logs_bucket" {
  description = "Name of the S3 bucket for logs"
  value       = aws_s3_bucket.logs.bucket
}

output "s3_logs_bucket_arn" {
  description = "ARN of the S3 bucket for logs"
  value       = aws_s3_bucket.logs.arn
}

# ECR Outputs
output "ecr_repository_url" {
  description = "The repository URL"
  value       = aws_ecr_repository.inferno.repository_url
}

output "ecr_registry_id" {
  description = "The registry ID where the repository was created"
  value       = aws_ecr_repository.inferno.registry_id
}

# CloudWatch Outputs
output "cloudwatch_log_group_application" {
  description = "Name of the CloudWatch log group for application logs"
  value       = aws_cloudwatch_log_group.application.name
}

output "cloudwatch_log_group_eks" {
  description = "Name of the CloudWatch log group for EKS cluster logs"
  value       = aws_cloudwatch_log_group.eks_cluster.name
}

# Security Groups
output "alb_security_group_id" {
  description = "ID of the ALB security group"
  value       = aws_security_group.alb.id
}

output "rds_security_group_id" {
  description = "ID of the RDS security group"
  value       = aws_security_group.rds.id
}

output "redis_security_group_id" {
  description = "ID of the Redis security group"
  value       = aws_security_group.redis.id
}

# Environment Info
output "environment" {
  description = "Environment name"
  value       = var.environment
}

output "aws_region" {
  description = "AWS region"
  value       = var.aws_region
}

# Sensitive Information (for CI/CD)
output "database_connection_string" {
  description = "Database connection string"
  value       = "postgresql://${module.rds.db_instance_username}:${random_password.rds_password.result}@${module.rds.db_instance_endpoint}/${module.rds.db_instance_name}"
  sensitive   = true
}

output "redis_connection_string" {
  description = "Redis connection string"
  value       = "redis://${aws_elasticache_replication_group.main.primary_endpoint_address}:${aws_elasticache_replication_group.main.port}"
  sensitive   = true
}

# Kubectl configuration
output "kubectl_config" {
  description = "kubectl config as generated by the module"
  value = {
    cluster_name     = module.eks.cluster_name
    endpoint         = module.eks.cluster_endpoint
    region           = var.aws_region
    certificate_data = module.eks.cluster_certificate_authority_data
  }
  sensitive = true
}

# IAM Roles for Service Accounts (IRSA)
output "cluster_service_account_role_arn" {
  description = "ARN of the cluster service account IAM role"
  value       = module.eks.cluster_iam_role_arn
}

# Networking information for applications
output "networking_config" {
  description = "Networking configuration for applications"
  value = {
    vpc_id              = module.vpc.vpc_id
    private_subnet_ids  = module.vpc.private_subnets
    public_subnet_ids   = module.vpc.public_subnets
    database_subnet_ids = module.vpc.database_subnets
    cluster_sg_id       = module.eks.cluster_security_group_id
    node_sg_id          = module.eks.node_security_group_id
  }
}

# Complete configuration for external tools
output "inferno_config" {
  description = "Complete configuration for Inferno deployment"
  value = {
    # Infrastructure
    cluster_name = module.eks.cluster_name
    region       = var.aws_region
    environment  = var.environment

    # Database
    database_endpoint = module.rds.db_instance_endpoint
    database_name     = module.rds.db_instance_name
    database_port     = module.rds.db_instance_port

    # Cache
    redis_endpoint = aws_elasticache_replication_group.main.primary_endpoint_address
    redis_port     = aws_elasticache_replication_group.main.port

    # Storage
    models_bucket = aws_s3_bucket.models.bucket
    logs_bucket   = aws_s3_bucket.logs.bucket

    # Container Registry
    ecr_repository = aws_ecr_repository.inferno.repository_url

    # Load Balancer
    alb_dns_name = aws_lb.main.dns_name

    # Logging
    log_group = aws_cloudwatch_log_group.application.name
  }
  sensitive = true
}