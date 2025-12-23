# Azure Terraform Configuration for Guts Nodes
# Deploys Guts nodes on Microsoft Azure

terraform {
  required_version = ">= 1.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

# Variables
variable "location" {
  description = "Azure region"
  type        = string
  default     = "eastus"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "resource_group_name" {
  description = "Resource group name"
  type        = string
  default     = "guts-rg"
}

variable "node_count" {
  description = "Number of Guts nodes"
  type        = number
  default     = 3
}

variable "vm_size" {
  description = "Azure VM size"
  type        = string
  default     = "Standard_F8s_v2"
}

variable "os_disk_size_gb" {
  description = "OS disk size in GB"
  type        = number
  default     = 500
}

# Provider
provider "azurerm" {
  features {}
}

# Resource Group
resource "azurerm_resource_group" "guts" {
  name     = var.resource_group_name
  location = var.location

  tags = {
    environment = var.environment
    app         = "guts-node"
  }
}

# Virtual Network
resource "azurerm_virtual_network" "guts" {
  name                = "guts-vnet"
  address_space       = ["10.0.0.0/16"]
  location            = azurerm_resource_group.guts.location
  resource_group_name = azurerm_resource_group.guts.name

  tags = {
    environment = var.environment
  }
}

# Subnet
resource "azurerm_subnet" "guts" {
  name                 = "guts-subnet"
  resource_group_name  = azurerm_resource_group.guts.name
  virtual_network_name = azurerm_virtual_network.guts.name
  address_prefixes     = ["10.0.1.0/24"]
}

# Network Security Group
resource "azurerm_network_security_group" "guts" {
  name                = "guts-nsg"
  location            = azurerm_resource_group.guts.location
  resource_group_name = azurerm_resource_group.guts.name

  # SSH
  security_rule {
    name                       = "SSH"
    priority                   = 1001
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "22"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # HTTP API
  security_rule {
    name                       = "HTTP-API"
    priority                   = 1002
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "8080"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # P2P TCP
  security_rule {
    name                       = "P2P-TCP"
    priority                   = 1003
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "9000"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  # P2P UDP
  security_rule {
    name                       = "P2P-UDP"
    priority                   = 1004
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Udp"
    source_port_range          = "*"
    destination_port_range     = "9000"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  tags = {
    environment = var.environment
  }
}

# Associate NSG with subnet
resource "azurerm_subnet_network_security_group_association" "guts" {
  subnet_id                 = azurerm_subnet.guts.id
  network_security_group_id = azurerm_network_security_group.guts.id
}

# Public IP for Load Balancer
resource "azurerm_public_ip" "guts_lb" {
  name                = "guts-lb-pip"
  location            = azurerm_resource_group.guts.location
  resource_group_name = azurerm_resource_group.guts.name
  allocation_method   = "Static"
  sku                 = "Standard"

  tags = {
    environment = var.environment
  }
}

# Load Balancer
resource "azurerm_lb" "guts" {
  name                = "guts-lb"
  location            = azurerm_resource_group.guts.location
  resource_group_name = azurerm_resource_group.guts.name
  sku                 = "Standard"

  frontend_ip_configuration {
    name                 = "PublicIPAddress"
    public_ip_address_id = azurerm_public_ip.guts_lb.id
  }

  tags = {
    environment = var.environment
  }
}

# Backend Pool
resource "azurerm_lb_backend_address_pool" "guts" {
  loadbalancer_id = azurerm_lb.guts.id
  name            = "guts-backend-pool"
}

# Health Probe
resource "azurerm_lb_probe" "guts_http" {
  loadbalancer_id     = azurerm_lb.guts.id
  name                = "guts-http-probe"
  protocol            = "Http"
  port                = 8080
  request_path        = "/health/ready"
  interval_in_seconds = 10
  number_of_probes    = 3
}

# Load Balancer Rule - HTTP
resource "azurerm_lb_rule" "guts_http" {
  loadbalancer_id                = azurerm_lb.guts.id
  name                           = "guts-http-rule"
  protocol                       = "Tcp"
  frontend_port                  = 80
  backend_port                   = 8080
  frontend_ip_configuration_name = "PublicIPAddress"
  backend_address_pool_ids       = [azurerm_lb_backend_address_pool.guts.id]
  probe_id                       = azurerm_lb_probe.guts_http.id
}

# Virtual Machine Scale Set
resource "azurerm_linux_virtual_machine_scale_set" "guts" {
  name                = "guts-vmss"
  resource_group_name = azurerm_resource_group.guts.name
  location            = azurerm_resource_group.guts.location
  sku                 = var.vm_size
  instances           = var.node_count
  admin_username      = "adminuser"

  admin_ssh_key {
    username   = "adminuser"
    public_key = file("~/.ssh/id_rsa.pub")
  }

  source_image_reference {
    publisher = "Canonical"
    offer     = "0001-com-ubuntu-server-jammy"
    sku       = "22_04-lts"
    version   = "latest"
  }

  os_disk {
    storage_account_type = "Premium_LRS"
    caching              = "ReadWrite"
    disk_size_gb         = var.os_disk_size_gb
  }

  network_interface {
    name    = "guts-nic"
    primary = true

    ip_configuration {
      name                                   = "internal"
      primary                                = true
      subnet_id                              = azurerm_subnet.guts.id
      load_balancer_backend_address_pool_ids = [azurerm_lb_backend_address_pool.guts.id]

      public_ip_address {
        name = "guts-pip"
      }
    }
  }

  custom_data = base64encode(<<-EOF
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

    echo "Guts node installation complete"
  EOF
  )

  tags = {
    environment = var.environment
    app         = "guts-node"
  }
}

# Auto-scaling
resource "azurerm_monitor_autoscale_setting" "guts" {
  name                = "guts-autoscale"
  resource_group_name = azurerm_resource_group.guts.name
  location            = azurerm_resource_group.guts.location
  target_resource_id  = azurerm_linux_virtual_machine_scale_set.guts.id

  profile {
    name = "defaultProfile"

    capacity {
      default = var.node_count
      minimum = 3
      maximum = 10
    }

    rule {
      metric_trigger {
        metric_name        = "Percentage CPU"
        metric_resource_id = azurerm_linux_virtual_machine_scale_set.guts.id
        time_grain         = "PT1M"
        statistic          = "Average"
        time_window        = "PT5M"
        time_aggregation   = "Average"
        operator           = "GreaterThan"
        threshold          = 75
      }

      scale_action {
        direction = "Increase"
        type      = "ChangeCount"
        value     = "1"
        cooldown  = "PT5M"
      }
    }

    rule {
      metric_trigger {
        metric_name        = "Percentage CPU"
        metric_resource_id = azurerm_linux_virtual_machine_scale_set.guts.id
        time_grain         = "PT1M"
        statistic          = "Average"
        time_window        = "PT5M"
        time_aggregation   = "Average"
        operator           = "LessThan"
        threshold          = 25
      }

      scale_action {
        direction = "Decrease"
        type      = "ChangeCount"
        value     = "1"
        cooldown  = "PT5M"
      }
    }
  }
}

# Outputs
output "load_balancer_ip" {
  description = "Load balancer public IP"
  value       = azurerm_public_ip.guts_lb.ip_address
}

output "api_endpoint" {
  description = "API endpoint URL"
  value       = "http://${azurerm_public_ip.guts_lb.ip_address}"
}

output "resource_group" {
  description = "Resource group name"
  value       = azurerm_resource_group.guts.name
}

output "location" {
  description = "Azure location"
  value       = var.location
}
