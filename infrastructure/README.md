# Inferno Infrastructure

This directory contains Infrastructure as Code (IaC) configurations for deploying and managing the Inferno AI/ML platform in production environments.

## 📁 Directory Structure

```
infrastructure/
├── terraform/           # Terraform configurations for AWS infrastructure
│   ├── main.tf         # Main Terraform configuration
│   ├── variables.tf    # Variable definitions
│   └── outputs.tf      # Output definitions
├── kubernetes/         # Kubernetes manifests
│   ├── namespace.yaml  # Namespace definitions
│   ├── configmap.yaml  # Configuration maps
│   ├── deployment.yaml # Application deployments
│   ├── service.yaml    # Service definitions
│   └── ingress.yaml    # Ingress configurations
├── helm/               # Helm chart for Kubernetes deployment
│   ├── Chart.yaml      # Helm chart metadata
│   ├── values.yaml     # Default values
│   └── templates/      # Helm templates
└── README.md           # This file
```

## 🏗️ Infrastructure Components

### AWS Infrastructure (Terraform)

The Terraform configuration provisions:

- **EKS Cluster**: Managed Kubernetes cluster with auto-scaling node groups
- **VPC**: Virtual Private Cloud with public/private subnets across 3 AZs
- **RDS PostgreSQL**: Managed database for application data
- **ElastiCache Redis**: Managed Redis for caching and session storage
- **S3 Buckets**: Object storage for models and logs
- **ALB**: Application Load Balancer for HTTP/HTTPS traffic
- **ECR**: Container registry for Docker images
- **CloudWatch**: Logging and monitoring
- **IAM**: Identity and Access Management roles and policies

### Kubernetes Resources

The Kubernetes manifests include:

- **Namespaces**: Environment separation (staging, production)
- **Deployments**: API server, worker processes, GPU workers
- **Services**: Internal and external service definitions
- **Ingress**: HTTP routing and SSL termination
- **ConfigMaps**: Application configuration
- **Secrets**: Sensitive data management
- **PVCs**: Persistent storage for models and data

### Helm Chart

The Helm chart provides:

- **Templated Deployments**: Customizable for different environments
- **Dependencies**: PostgreSQL, Redis, Prometheus, Grafana
- **Auto-scaling**: Horizontal Pod Autoscaler configurations
- **Monitoring**: ServiceMonitor for Prometheus integration
- **Security**: RBAC, Network Policies, Pod Security Standards

## 🚀 Getting Started

### Prerequisites

1. **AWS CLI** configured with appropriate permissions
2. **Terraform** >= 1.0
3. **kubectl** configured for Kubernetes access
4. **Helm** >= 3.0
5. **Docker** for container operations

### AWS Infrastructure Deployment

1. **Initialize Terraform**:
   ```bash
   cd terraform
   terraform init
   ```

2. **Plan Infrastructure**:
   ```bash
   terraform plan -var-file="environments/production.tfvars"
   ```

3. **Apply Infrastructure**:
   ```bash
   terraform apply -var-file="environments/production.tfvars"
   ```

4. **Configure kubectl**:
   ```bash
   aws eks update-kubeconfig --region us-west-2 --name inferno-production-cluster
   ```

### Kubernetes Application Deployment

#### Using kubectl (Manual)

1. **Apply Kubernetes manifests**:
   ```bash
   kubectl apply -f kubernetes/namespace.yaml
   kubectl apply -f kubernetes/configmap.yaml
   kubectl apply -f kubernetes/deployment.yaml
   kubectl apply -f kubernetes/service.yaml
   kubectl apply -f kubernetes/ingress.yaml
   ```

#### Using Helm (Recommended)

1. **Install Helm dependencies**:
   ```bash
   cd helm
   helm dependency update
   ```

2. **Deploy to staging**:
   ```bash
   helm upgrade --install inferno-staging . \
     --namespace inferno-staging \
     --create-namespace \
     --values values-staging.yaml
   ```

3. **Deploy to production**:
   ```bash
   helm upgrade --install inferno-production . \
     --namespace inferno-production \
     --create-namespace \
     --values values-production.yaml
   ```

## 🔧 Configuration

### Environment Variables

The following environment variables need to be configured:

| Variable | Description | Default |
|----------|-------------|---------|
| `AWS_REGION` | AWS region for deployment | `us-west-2` |
| `ENVIRONMENT` | Environment name (dev/staging/prod) | `development` |
| `DOMAIN_NAME` | Domain name for the application | `inferno-ai.dev` |
| `DATABASE_PASSWORD` | PostgreSQL password | Auto-generated |
| `REDIS_PASSWORD` | Redis password | Auto-generated |
| `JWT_SECRET` | JWT signing secret | Required |

### Terraform Variables

Create environment-specific `.tfvars` files:

```hcl
# environments/production.tfvars
environment = "production"
aws_region = "us-west-2"
domain_name = "inferno-ai.dev"

# EKS Configuration
kubernetes_version = "1.28"
node_instance_types = ["t3.large", "t3.xlarge"]
node_group_desired_size = 5
enable_gpu_nodes = true

# Database Configuration
rds_instance_class = "db.r5.large"
rds_allocated_storage = 100

# Redis Configuration
redis_node_type = "cache.r5.large"
redis_num_cache_nodes = 3
```

### Helm Values

Create environment-specific values files:

```yaml
# values-production.yaml
environment: production

api:
  replicaCount: 5
  resources:
    requests:
      memory: "1Gi"
      cpu: "1000m"
    limits:
      memory: "4Gi"
      cpu: "4000m"

gpu:
  enabled: true
  replicaCount: 2

ingress:
  hosts:
    - host: api.inferno-ai.dev
      paths:
        - path: /
          pathType: Prefix

monitoring:
  enabled: true
  prometheus:
    enabled: true
  grafana:
    enabled: true
```

## 📊 Monitoring and Observability

### Prometheus Metrics

The infrastructure includes comprehensive monitoring:

- **Application Metrics**: Request latency, throughput, error rates
- **Infrastructure Metrics**: CPU, memory, disk, network usage
- **Custom Metrics**: Model inference metrics, cache hit rates
- **Alerts**: Automated alerting for critical issues

### Grafana Dashboards

Pre-configured dashboards for:

- **Application Overview**: High-level application health
- **Infrastructure Overview**: Cluster and node metrics
- **Model Performance**: Inference latency and throughput
- **Error Analysis**: Error rates and failure patterns

### Logging

Centralized logging with:

- **Fluent Bit**: Log collection and forwarding
- **CloudWatch Logs**: Centralized log storage
- **Structured Logging**: JSON-formatted logs for analysis
- **Log Retention**: Configurable retention policies

## 🔒 Security

### Network Security

- **VPC**: Isolated network environment
- **Security Groups**: Restrictive ingress/egress rules
- **Network Policies**: Kubernetes-level network isolation
- **WAF**: Web Application Firewall for external traffic

### Identity and Access Management

- **IAM Roles**: Least-privilege access for services
- **RBAC**: Kubernetes Role-Based Access Control
- **Service Accounts**: Dedicated accounts for workloads
- **Pod Security Standards**: Enforced security policies

### Data Security

- **Encryption at Rest**: EBS, RDS, S3 encryption
- **Encryption in Transit**: TLS for all communications
- **Secrets Management**: Kubernetes secrets and AWS Secrets Manager
- **Certificate Management**: Automated SSL/TLS certificate provisioning

## 🔄 CI/CD Integration

### Automated Deployment

The infrastructure integrates with GitHub Actions for:

- **Infrastructure Validation**: Terraform plan on PRs
- **Automated Deployment**: Apply changes on merge to main
- **Environment Promotion**: Staging → Production workflow
- **Rollback Capabilities**: Quick rollback on failures

### GitOps Workflow

1. **Infrastructure Changes**: Update Terraform configurations
2. **Application Changes**: Update Helm values or Kubernetes manifests
3. **Pull Request**: Review and validate changes
4. **Automated Testing**: Run infrastructure and application tests
5. **Deployment**: Automated deployment to appropriate environment
6. **Monitoring**: Validate deployment success and monitor health

## 💰 Cost Optimization

### Right-sizing

- **Auto-scaling**: Scale resources based on demand
- **Spot Instances**: Use spot instances for non-critical workloads
- **Reserved Instances**: Use reserved instances for predictable workloads
- **Resource Limits**: Set appropriate resource limits

### Storage Optimization

- **Storage Classes**: Use appropriate storage classes for different needs
- **Lifecycle Policies**: Automated cleanup of old data
- **Compression**: Enable compression for logs and backups
- **Data Archival**: Move old data to cheaper storage tiers

## 🛠️ Troubleshooting

### Common Issues

1. **EKS Cluster Access**:
   ```bash
   aws eks update-kubeconfig --region us-west-2 --name cluster-name
   ```

2. **Pod Startup Issues**:
   ```bash
   kubectl describe pod <pod-name> -n <namespace>
   kubectl logs <pod-name> -n <namespace>
   ```

3. **Ingress Issues**:
   ```bash
   kubectl describe ingress -n <namespace>
   kubectl logs -n kube-system -l app.kubernetes.io/name=aws-load-balancer-controller
   ```

4. **Database Connection Issues**:
   ```bash
   kubectl exec -it <pod-name> -n <namespace> -- env | grep DATABASE
   ```

### Health Checks

```bash
# Check cluster status
kubectl cluster-info

# Check node status
kubectl get nodes

# Check application pods
kubectl get pods -n inferno-production

# Check services
kubectl get svc -n inferno-production

# Check ingress
kubectl get ingress -n inferno-production
```

## 📈 Scaling

### Horizontal Scaling

- **Application Pods**: Increase replica count in Helm values
- **Database**: Use read replicas for read-heavy workloads
- **Cache**: Use Redis clustering for larger datasets
- **Load Balancing**: ALB automatically scales with traffic

### Vertical Scaling

- **Resource Limits**: Increase CPU/memory limits in deployments
- **Node Instance Types**: Use larger instance types for nodes
- **Database Instance**: Scale up RDS instance class
- **Cache Instance**: Scale up Redis instance type

## 🔐 Backup and Disaster Recovery

### Automated Backups

- **RDS**: Automated daily backups with point-in-time recovery
- **EBS**: Snapshot lifecycle policies for persistent volumes
- **S3**: Cross-region replication for critical data
- **Configuration**: GitOps ensures infrastructure reproducibility

### Disaster Recovery

- **Multi-AZ**: Resources deployed across multiple availability zones
- **Cross-Region**: Optional cross-region backup and replication
- **Recovery Procedures**: Documented recovery processes
- **Testing**: Regular disaster recovery testing

## 📚 Additional Resources

- [Terraform AWS Provider Documentation](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Helm Documentation](https://helm.sh/docs/)
- [AWS EKS Best Practices](https://aws.github.io/aws-eks-best-practices/)
- [Prometheus Monitoring](https://prometheus.io/docs/)

## 🤝 Contributing

1. **Infrastructure Changes**: Create feature branch and submit PR
2. **Testing**: Test changes in development environment first
3. **Documentation**: Update documentation for any changes
4. **Review**: All changes require code review and approval
5. **Deployment**: Follow GitOps workflow for deployments

## 📞 Support

For infrastructure-related issues:

1. **Check Documentation**: Review this README and linked resources
2. **Search Issues**: Check existing GitHub issues
3. **Create Issue**: Create new issue with detailed description
4. **Emergency**: Contact on-call engineer for production issues

---

**Note**: This infrastructure is designed for production use with enterprise-grade security, monitoring, and reliability features. Always test changes in development/staging environments before applying to production.