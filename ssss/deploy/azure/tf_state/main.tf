provider "azurerm" {
  features {}
  skip_provider_registration = true
}

locals {
  tags = {
    Vendor    = "escrin",
    Component = "infra",
  }
}

variable "location" {
  description = "The location of the resource group and storage account (e.g. 'eastus')"
}

variable "resource_group_name" {
  description = "The name of the resource group to create"
  default     = "escrin-ssss-tfstate"
}

variable "storage_account_name" {
  description = "The name of the storage account to create. It must be globally unique, can only consist of lowercase letters and numbers, and must be between 3 and 24 characters long"
}

variable "container_name" {
  description = "The name of the blob container to create."
  default     = "backend-blob"
}

resource "azurerm_resource_group" "rg" {
  name     = var.resource_group_name
  location = var.location
  tags = local.tags
}

resource "azurerm_storage_account" "sa" {
  name                     = var.storage_account_name
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"

  tags = local.tags
}

resource "azurerm_storage_container" "sc" {
  name                  = var.container_name
  storage_account_name  = azurerm_storage_account.sa.name
  container_access_type = "private"
}