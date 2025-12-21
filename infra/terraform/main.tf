# ============================================================================
# Guts Infrastructure - Main Configuration
# ============================================================================

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  # Uncomment for remote state
  # backend "s3" {
  #   bucket = "guts-terraform-state"
  #   key    = "infrastructure/terraform.tfstate"
  #   region = "us-east-1"
  # }
}

# ============================================================================
# Provider Configuration
# ============================================================================

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "guts"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# ============================================================================
# Variables
# ============================================================================

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
}

variable "node_count" {
  description = "Number of Guts nodes to deploy"
  type        = number
  default     = 3
}

variable "instance_type" {
  description = "EC2 instance type for nodes"
  type        = string
  default     = "t3.medium"
}

variable "docker_image" {
  description = "Docker image for Guts node"
  type        = string
  default     = "ghcr.io/abdelstark/guts-node:latest"
}

variable "ssh_key_name" {
  description = "Name of the SSH key pair for EC2 instances"
  type        = string
  default     = ""
}

# ============================================================================
# Data Sources
# ============================================================================

data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# ============================================================================
# Networking
# ============================================================================

module "vpc" {
  source = "./modules/vpc"

  name        = "guts-${var.environment}"
  cidr        = "10.0.0.0/16"
  environment = var.environment
}

# ============================================================================
# Security Groups
# ============================================================================

resource "aws_security_group" "guts_node" {
  name        = "guts-node-${var.environment}"
  description = "Security group for Guts nodes"
  vpc_id      = module.vpc.vpc_id

  # SSH access
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "SSH access"
  }

  # API access
  ingress {
    from_port   = 8080
    to_port     = 8080
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "API access"
  }

  # P2P access
  ingress {
    from_port   = 9000
    to_port     = 9000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    description = "P2P access"
  }

  # Allow all outbound
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "guts-node-${var.environment}"
  }
}

# ============================================================================
# Guts Nodes
# ============================================================================

resource "aws_instance" "guts_node" {
  count = var.node_count

  ami                    = data.aws_ami.ubuntu.id
  instance_type          = var.instance_type
  subnet_id              = module.vpc.public_subnets[count.index % length(module.vpc.public_subnets)]
  vpc_security_group_ids = [aws_security_group.guts_node.id]

  root_block_device {
    volume_size = 100
    volume_type = "gp3"
  }

  user_data = base64encode(templatefile("${path.module}/templates/user_data.sh.tftpl", {
    node_index      = count.index
    node_count      = var.node_count
    environment     = var.environment
    docker_image    = var.docker_image
    bootstrap_nodes = count.index == 0 ? "" : join(",", [for i in range(count.index) : "guts-node-${var.environment}-${i}:9000"])
  }))

  tags = {
    Name = "guts-node-${var.environment}-${count.index}"
    Role = "guts-node"
  }
}

# ============================================================================
# Outputs
# ============================================================================

output "node_public_ips" {
  description = "Public IPs of Guts nodes"
  value       = aws_instance.guts_node[*].public_ip
}

output "node_private_ips" {
  description = "Private IPs of Guts nodes"
  value       = aws_instance.guts_node[*].private_ip
}

output "vpc_id" {
  description = "VPC ID"
  value       = module.vpc.vpc_id
}
