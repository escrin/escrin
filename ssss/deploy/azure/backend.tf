terraform {
    backend "azurerm" {
    # Configure terraform storage to store state files 
    storage_account_name = "dirkescrin"
    container_name       = "backend-blob"
    key                  = "environment.terraform.tfstate"
    access_key           = "<get it in azure>"
    }
}