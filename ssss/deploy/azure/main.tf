terraform {
  backend "azurerm" {
    resource_group_name = "escrin-ssss-tfstate"
    container_name      = "terraform"
    key                 = "terraform.tfstate"
  }
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy = true
    }
  }
}

variable "instance_type" {
  description = "The Azure VM instance type"
  default     = "Standard_B2pts_v2"
}

variable "instance_arch" {
  description = "The Azure VM instance CPU architecture (e.g., amd64, arm64)"
  default     = "arm64"
}

variable "ssss_tag" {
  description = "The tag of the ghcr.io/escrin/ssss image to use"
}

variable "ssh_key" {
  description = "Path to the key pair for SSH access to the Azure VM instance."
  default     = ""
}

variable "location" {
  description = "The location of the resources (e.g. 'eastus')"
}

variable "hostname" {
  description = "The hostname of your SSSS deployment (e.g., ssss.example.com)"
  validation {
    condition     = can(regex("^([a-zA-Z0-9]([a-zA-Z0-9\\-]{0,61}[a-zA-Z0-9])?\\.)+[a-zA-Z]{2,}$", var.hostname))
    error_message = "Invalid hostname format. Please provide a valid hostname (e.g., ssss.example.com, ssss.xyz)."
  }
}

variable "cloudflare" {
  description = "Whether to restrict ingress to only Cloudflare IPs. Makes instance unreachable except through Cloudflare's relays."
  default     = false
}

locals {
  tags = {
    Vendor      = "escrin",
    Component   = "infra",
    Environment = "${terraform.workspace}",
  }

  domain_hash = substr(sha256(var.hostname), 0, 16)
  unique_name = "${terraform.workspace}${local.domain_hash}"

  storage_account_name = local.unique_name
  kv_name              = local.unique_name
}

resource "azurerm_resource_group" "rg" {
  name     = "escrin-ssss-${terraform.workspace}"
  location = var.location
  tags     = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

data "azurerm_client_config" "current" {}

resource "azurerm_user_assigned_identity" "uai" {
  name                = "escrin-ssss-identity-${terraform.workspace}"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_storage_account" "sa" {
  name                     = local.storage_account_name
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
  tags                     = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_role_assignment" "user" {
  count                = terraform.workspace == "dev" ? 1 : 0
  scope                = azurerm_storage_account.sa.id
  role_definition_name = "Storage Table Data Contributor"
  principal_id         = data.azurerm_client_config.current.object_id

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_role_assignment" "instance" {
  scope                = azurerm_storage_account.sa.id
  role_definition_name = "Storage Table Data Contributor"
  principal_id         = azurerm_user_assigned_identity.uai.principal_id

  lifecycle {
    prevent_destroy = true
  }
}


locals {
  storage_tables = ["secretversions", "verifiers"]
}

resource "azurerm_storage_table" "storage" {
  for_each             = toset(local.storage_tables)
  name                 = each.key
  storage_account_name = azurerm_storage_account.sa.name

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_key_vault" "kv" {
  name                        = local.kv_name
  location                    = azurerm_resource_group.rg.location
  resource_group_name         = azurerm_resource_group.rg.name
  enabled_for_disk_encryption = true
  tenant_id                   = data.azurerm_client_config.current.tenant_id
  tags                        = local.tags
  sku_name                    = "premium"

  lifecycle {
    prevent_destroy = true
  }
}

locals {
  kv_key_permissions = [
    "Get",
    "List",
    "Sign",
  ]
  kv_secret_permissions = [
    "Backup",
    "Delete",
    "Get",
    "List",
    "Purge",
    "Recover",
    "Restore",
    "Set",
  ]
}

resource "azurerm_key_vault_access_policy" "instance" {
  key_vault_id = azurerm_key_vault.kv.id
  tenant_id    = azurerm_user_assigned_identity.uai.tenant_id
  object_id    = azurerm_user_assigned_identity.uai.principal_id

  key_permissions    = local.kv_key_permissions
  secret_permissions = local.kv_secret_permissions

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_key_vault_access_policy" "client" {
  key_vault_id = azurerm_key_vault.kv.id
  tenant_id    = data.azurerm_client_config.current.tenant_id
  object_id    = data.azurerm_client_config.current.object_id

  key_permissions = concat(terraform.workspace == "dev" ? local.kv_key_permissions : [], [
    "Backup",
    "Create",
    "Delete",
    "Get",
    "GetRotationPolicy",
    "List",
    "Purge",
    "Recover",
    "SetRotationPolicy",
    "Update",
  ])
  secret_permissions = terraform.workspace == "dev" ? local.kv_secret_permissions : []

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_key_vault_key" "signer" {
  name         = "escrin-signer"
  key_vault_id = azurerm_key_vault.kv.id
  key_type     = "EC"
  curve        = "P-256K"
  key_opts     = ["sign"]
  tags         = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_virtual_network" "vnet" {
  name                = "escrin-vnet"
  resource_group_name = azurerm_resource_group.rg.name
  location            = azurerm_resource_group.rg.location
  address_space       = ["10.0.0.0/16"]
  tags                = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_network_security_group" "nsg" {
  name                = "escrin-nsg"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  tags                = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_network_interface" "ni" {
  name                = "escrin-ni"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  tags                = local.tags

  ip_configuration {
    name                          = "escrin-ip-config"
    subnet_id                     = azurerm_subnet.subnet.id
    private_ip_address_allocation = "Dynamic"
    public_ip_address_id          = azurerm_public_ip.public_ip.id
  }

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_subnet" "subnet" {
  name                 = "default"
  resource_group_name  = azurerm_resource_group.rg.name
  virtual_network_name = azurerm_virtual_network.vnet.name
  address_prefixes     = ["10.0.1.0/24"]

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_public_ip" "public_ip" {
  name                = "escrin-public-ip"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  allocation_method   = "Static"
  sku                 = "Standard"
  tags                = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "tls_private_key" "ssh" {
  algorithm = "RSA"
  rsa_bits  = 4096

  lifecycle {
    prevent_destroy = true
  }
}

output "ssh_private_key" {
  value     = terraform.workspace == "dev" ? tls_private_key.ssh.private_key_pem : "[redacted]"
  sensitive = true
}

resource "azurerm_virtual_machine" "vm" {
  name                             = "escrin-vm"
  location                         = azurerm_resource_group.rg.location
  resource_group_name              = azurerm_resource_group.rg.name
  network_interface_ids            = [azurerm_network_interface.ni.id]
  delete_os_disk_on_termination    = true
  vm_size                          = var.instance_type
  delete_data_disks_on_termination = true

  os_profile {
    computer_name  = "escrin-vm"
    admin_username = "escrin-administrator"
    custom_data    = <<-CUSTOM_DATA
      #!/bin/bash
      sudo apt-get -y update
      sudo apt-get -y install apt-transport-https ca-certificates curl gnupg lsb-release
      curl -fsSL https://download.docker.com/linux/debian/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
      echo "deb [arch=${var.instance_arch} signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
      sudo apt-get update -y
      sudo apt-get install -y docker-ce docker-ce-cli containerd.io
      sudo systemctl enable docker
      sudo systemctl start docker
      sudo usermod -aG docker $USER
      sudo systemctl enable docker.service
      sudo systemctl enable containerd.service
      sudo docker run -p 80:1075 -d --restart=always ghcr.io/escrin/ssss:${var.ssss_tag} -vv
    CUSTOM_DATA
  }

  os_profile_linux_config {
    disable_password_authentication = true
    ssh_keys {
      path     = "/home/escrin-administrator/.ssh/authorized_keys"
      key_data = tls_private_key.ssh.public_key_openssh
    }
  }

  storage_image_reference {
    publisher = "Debian"
    offer     = "debian-12"
    sku       = "12${var.instance_arch == "arm64" ? "-arm64" : ""}"
    version   = "latest"
  }

  storage_os_disk {
    name              = "escrin_vm_disk"
    caching           = "ReadWrite"
    create_option     = "FromImage"
    managed_disk_type = "Standard_LRS"
  }

  identity {
    type         = "UserAssigned"
    identity_ids = [azurerm_user_assigned_identity.uai.id]
  }
}

output "vm_ip" {
  value = azurerm_public_ip.public_ip.ip_address
}

locals {
  sg_cidrs = var.cloudflare ? [
    "173.245.48.0/20",
    "103.21.244.0/22",
    "103.22.200.0/22",
    "103.31.4.0/22",
    "141.101.64.0/18",
    "108.162.192.0/18",
    "190.93.240.0/20",
    "188.114.96.0/20",
    "197.234.240.0/22",
    "198.41.128.0/17",
    "162.158.0.0/15",
    "104.16.0.0/13",
    "104.24.0.0/14",
    "172.64.0.0/13",
    "131.0.72.0/22"
  ] : ["0.0.0.0/0"]
}

resource "azurerm_network_security_group" "sg" {
  name                = "escrin-ssss-sg-${terraform.workspace}"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_network_security_rule" "http_ingress" {
  for_each                    = toset(local.sg_cidrs)
  name                        = "HTTP_INGRESS_RULE_${replace(each.key, "/", "-")}"
  priority                    = 100
  direction                   = "Inbound"
  access                      = "Allow"
  protocol                    = "Tcp"
  source_port_range           = "*"
  destination_port_range      = "80"
  source_address_prefix       = each.key
  destination_address_prefix  = "*"
  resource_group_name         = azurerm_resource_group.rg.name
  network_security_group_name = azurerm_network_security_group.sg.name

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_network_security_rule" "ssh_ingress" {
  for_each                    = toset(local.sg_cidrs)
  name                        = "SSH_INGRESS_RULE_${replace(each.key, "/", "-")}"
  priority                    = 110
  direction                   = "Inbound"
  access                      = "Allow"
  protocol                    = "Tcp"
  source_port_range           = "*"
  destination_port_range      = "22"
  source_address_prefix       = each.key
  destination_address_prefix  = "*"
  resource_group_name         = azurerm_resource_group.rg.name
  network_security_group_name = azurerm_network_security_group.sg.name

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_network_security_rule" "egress" {
  name                        = "EGRESS_RULE"
  priority                    = 120
  direction                   = "Outbound"
  access                      = "Allow"
  protocol                    = "*"
  source_port_range           = "*"
  destination_port_range      = "*"
  source_address_prefix       = "*"
  destination_address_prefix  = "*"
  resource_group_name         = azurerm_resource_group.rg.name
  network_security_group_name = azurerm_network_security_group.sg.name

  lifecycle {
    prevent_destroy = true
  }
}
