provider "azurerm" {
  features {}
}

variable "hostname" {
  description = "The hostname of your SSSS deployment (e.g., ssss.example.org)"
  validation {
    condition     = can(regex("^([a-zA-Z0-9]([a-zA-Z0-9\\-]{0,61}[a-zA-Z0-9])?\\.)+[a-zA-Z]{2,}$", var.hostname))
    error_message = "Invalid hostname format. Please provide a valid hostname (e.g., ssss.example.com, ssss.xyz)."
  }
}

variable "location" {
  description = "The location of the resource group and storage account (e.g. 'eastus')"
}

variable "resource_group_name" {
  description = "The name of the resource group to create"
  default     = "escrin-ssss-tfstate"
}

variable "container_name" {
  description = "The name of the blob container to create."
  default     = "terraform"
}

locals {
  tags = {
    Vendor    = "escrin",
    Component = "infra",
  }

  domain_hash = substr(sha256(var.hostname), 0, 16)
}

resource "azurerm_resource_group" "rg" {
  name     = var.resource_group_name
  location = var.location
  tags     = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_storage_account" "sa" {
  name                     = "escrintf${local.domain_hash}"
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
  tags                     = local.tags

  lifecycle {
    prevent_destroy = true
  }
}

resource "azurerm_storage_container" "sc" {
  name                  = var.container_name
  storage_account_name  = azurerm_storage_account.sa.name
  container_access_type = "private"

  lifecycle {
    prevent_destroy = true
  }
}

output "hostname" {
  value = var.hostname
}
