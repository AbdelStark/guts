# GCP Terraform Configuration for Guts Nodes
# Deploys Guts nodes on Google Cloud Platform

terraform {
  required_version = ">= 1.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

# Variables
variable "project_id" {
  description = "GCP Project ID"
  type        = string
}

variable "region" {
  description = "GCP region"
  type        = string
  default     = "us-central1"
}

variable "zone" {
  description = "GCP zone"
  type        = string
  default     = "us-central1-a"
}

variable "environment" {
  description = "Environment name (e.g., dev, staging, prod)"
  type        = string
  default     = "production"
}

variable "node_count" {
  description = "Number of Guts nodes to deploy"
  type        = number
  default     = 3
}

variable "machine_type" {
  description = "GCP machine type"
  type        = string
  default     = "c2-standard-8"
}

variable "disk_size_gb" {
  description = "Boot disk size in GB"
  type        = number
  default     = 500
}

variable "disk_type" {
  description = "Boot disk type"
  type        = string
  default     = "pd-ssd"
}

variable "network_name" {
  description = "VPC network name"
  type        = string
  default     = "guts-network"
}

# Provider configuration
provider "google" {
  project = var.project_id
  region  = var.region
}

# VPC Network
resource "google_compute_network" "guts_network" {
  name                    = var.network_name
  auto_create_subnetworks = false
}

# Subnet
resource "google_compute_subnetwork" "guts_subnet" {
  name          = "${var.network_name}-subnet"
  ip_cidr_range = "10.0.0.0/24"
  region        = var.region
  network       = google_compute_network.guts_network.id

  private_ip_google_access = true

  log_config {
    aggregation_interval = "INTERVAL_5_SEC"
    flow_sampling        = 0.5
    metadata             = "INCLUDE_ALL_METADATA"
  }
}

# Cloud Router for NAT
resource "google_compute_router" "guts_router" {
  name    = "${var.network_name}-router"
  region  = var.region
  network = google_compute_network.guts_network.id
}

# Cloud NAT
resource "google_compute_router_nat" "guts_nat" {
  name                               = "${var.network_name}-nat"
  router                             = google_compute_router.guts_router.name
  region                             = var.region
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"

  log_config {
    enable = true
    filter = "ERRORS_ONLY"
  }
}

# Firewall rules
resource "google_compute_firewall" "guts_api" {
  name    = "${var.network_name}-allow-api"
  network = google_compute_network.guts_network.name

  allow {
    protocol = "tcp"
    ports    = ["8080"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["guts-node"]
}

resource "google_compute_firewall" "guts_p2p" {
  name    = "${var.network_name}-allow-p2p"
  network = google_compute_network.guts_network.name

  allow {
    protocol = "tcp"
    ports    = ["9000"]
  }

  allow {
    protocol = "udp"
    ports    = ["9000"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["guts-node"]
}

resource "google_compute_firewall" "guts_internal" {
  name    = "${var.network_name}-allow-internal"
  network = google_compute_network.guts_network.name

  allow {
    protocol = "tcp"
  }

  allow {
    protocol = "udp"
  }

  allow {
    protocol = "icmp"
  }

  source_ranges = ["10.0.0.0/8"]
  target_tags   = ["guts-node"]
}

resource "google_compute_firewall" "guts_ssh" {
  name    = "${var.network_name}-allow-ssh"
  network = google_compute_network.guts_network.name

  allow {
    protocol = "tcp"
    ports    = ["22"]
  }

  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["guts-node"]
}

# Service Account
resource "google_service_account" "guts_node" {
  account_id   = "guts-node-sa"
  display_name = "Guts Node Service Account"
}

# IAM bindings
resource "google_project_iam_member" "guts_logs" {
  project = var.project_id
  role    = "roles/logging.logWriter"
  member  = "serviceAccount:${google_service_account.guts_node.email}"
}

resource "google_project_iam_member" "guts_metrics" {
  project = var.project_id
  role    = "roles/monitoring.metricWriter"
  member  = "serviceAccount:${google_service_account.guts_node.email}"
}

# Instance Template
resource "google_compute_instance_template" "guts_node" {
  name_prefix  = "guts-node-"
  machine_type = var.machine_type
  region       = var.region

  tags = ["guts-node"]

  disk {
    source_image = "ubuntu-os-cloud/ubuntu-2204-lts"
    auto_delete  = true
    boot         = true
    disk_size_gb = var.disk_size_gb
    disk_type    = var.disk_type
  }

  network_interface {
    network    = google_compute_network.guts_network.id
    subnetwork = google_compute_subnetwork.guts_subnet.id

    access_config {
      # Ephemeral public IP
    }
  }

  service_account {
    email  = google_service_account.guts_node.email
    scopes = ["cloud-platform"]
  }

  metadata_startup_script = <<-EOF
    #!/bin/bash
    set -e

    # Install dependencies
    apt-get update
    apt-get install -y curl jq

    # Install Guts node
    curl -sSL https://get.guts.network | sh

    # Configure node
    mkdir -p /etc/guts
    cat > /etc/guts/config.yaml << 'CONFIG'
    api:
      addr: "0.0.0.0:8080"
    p2p:
      addr: "0.0.0.0:9000"
    metrics:
      addr: "0.0.0.0:9090"
    storage:
      data_dir: "/var/lib/guts"
    logging:
      level: "info"
      format: "json"
    CONFIG

    # Enable and start service
    systemctl enable guts-node
    systemctl start guts-node

    echo "Guts node installation complete" | tee /var/log/guts-provision.log
  EOF

  labels = {
    environment = var.environment
    app         = "guts-node"
  }

  lifecycle {
    create_before_destroy = true
  }
}

# Managed Instance Group
resource "google_compute_region_instance_group_manager" "guts_nodes" {
  name               = "guts-nodes"
  base_instance_name = "guts-node"
  region             = var.region
  target_size        = var.node_count

  version {
    instance_template = google_compute_instance_template.guts_node.id
  }

  named_port {
    name = "http"
    port = 8080
  }

  named_port {
    name = "p2p"
    port = 9000
  }

  auto_healing_policies {
    health_check      = google_compute_health_check.guts_http.id
    initial_delay_sec = 300
  }

  update_policy {
    type                         = "PROACTIVE"
    minimal_action               = "REPLACE"
    max_surge_fixed              = 1
    max_unavailable_fixed        = 0
    instance_redistribution_type = "PROACTIVE"
  }
}

# Health Check
resource "google_compute_health_check" "guts_http" {
  name                = "guts-http-health-check"
  check_interval_sec  = 10
  timeout_sec         = 5
  healthy_threshold   = 2
  unhealthy_threshold = 3

  http_health_check {
    port         = 8080
    request_path = "/health/ready"
  }
}

# Load Balancer (Global HTTP)
resource "google_compute_global_address" "guts_lb" {
  name = "guts-lb-ip"
}

resource "google_compute_backend_service" "guts_api" {
  name                  = "guts-api-backend"
  port_name             = "http"
  protocol              = "HTTP"
  timeout_sec           = 30
  enable_cdn            = false
  load_balancing_scheme = "EXTERNAL"

  backend {
    group           = google_compute_region_instance_group_manager.guts_nodes.instance_group
    balancing_mode  = "UTILIZATION"
    capacity_scaler = 1.0
  }

  health_checks = [google_compute_health_check.guts_http.id]
}

resource "google_compute_url_map" "guts" {
  name            = "guts-url-map"
  default_service = google_compute_backend_service.guts_api.id
}

resource "google_compute_target_http_proxy" "guts" {
  name    = "guts-http-proxy"
  url_map = google_compute_url_map.guts.id
}

resource "google_compute_global_forwarding_rule" "guts_http" {
  name                  = "guts-http-forwarding"
  ip_protocol           = "TCP"
  load_balancing_scheme = "EXTERNAL"
  port_range            = "80"
  target                = google_compute_target_http_proxy.guts.id
  ip_address            = google_compute_global_address.guts_lb.id
}

# Outputs
output "load_balancer_ip" {
  description = "Load balancer IP address"
  value       = google_compute_global_address.guts_lb.address
}

output "api_endpoint" {
  description = "API endpoint URL"
  value       = "http://${google_compute_global_address.guts_lb.address}"
}

output "project_id" {
  description = "GCP project ID"
  value       = var.project_id
}

output "region" {
  description = "GCP region"
  value       = var.region
}
