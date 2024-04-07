# Micosoft Azure

These are instructions to use Terraform to get the necessary cloud resources for the SSSS into your
Azure account. [OpenTofu](https://opentofu.org) is an open-source tool to define and provision cloud
resources with simple, declarative configuration files.

## Prerequisites

- Azure account
- OpenTofu [(installation instructions)](https://opentofu.org/docs/intro/install/) or Terraform
- Azure CLI
  [(installation instructions)](https://learn.microsoft.com/en-us/cli/azure/install-azure-cli)

## Azure Configuration

In command line run:

```sh
az login
```

You'll need to know two things:

1. The region code you want to install the resources

There is not currently a good method to decide where an SSSS should live so you may pick one that is
in a place dear to your heart. (Or you have reason to believe it's underserved.) Find a
[list of Azure regions here](https://gist.github.com/ausfestivus/04e55c7d80229069bf3bc75870630ec8)

2. A domain name or subdomain where you will provide the expose SSSS API (e.g., `ssss.escrin.org`).

## Quick install

We need to install both a backend for our Terraform state file (read more about
[terraform state files](https://www.pluralsight.com/resources/blog/cloud/what-is-terraform-state))
as well as the resources needed for the SSSS. To install this in one go:

```sh
./s4-infra.sh apply
```

This script will ask you for the two pieces of information listed above and use sane defaults for
everything else. If you want more control over your terraform processes use the
[two-step install](#two-step-install).

If this script ran successfully, head over to [verify installation](#verify-installation).

To tear the environment down, run these

```sh
./s4-infra.sh unlock
./s4-infra.sh destroy # optionally pass --all to destroy everything
```

## Two-step install

Skip if you've done the quick install.

### Create Backend State

This will create the Terraform state bucket in AWS and DynamoDB table to manage terraform locks.

```sh
cd tf_state
terraform init
# You will be prompted to choose a globally unique Storage Account name to save the Terraform state.
terraform plan -out out.tfstate
terraform apply out.tfstate
```

### Create Resources

```sh
# This is ran from the root of this directory: ./ssss/deploy/aws/
# You will be prompted for an S3 bucket name. Use the same bucket name for the
# backend that was set in the previous Create Backend State step.
terraform init
terraform workspace switch -or-create prod # dev for development
terraform plan -out out.tfstate
terraform apply out.tfstate
```

## Verify installation

Let's check to see whether we have all the resources we need:

```sh
cd tf_state && terraform state list && cd .. && terraform state list
```

This should show the following output:

```
azurerm_resource_group.rg
azurerm_storage_account.sa
azurerm_storage_container.sc
data.azurerm_client_config.current
azurerm_key_vault.kv
azurerm_key_vault_access_policy.instance
azurerm_network_interface.ni
azurerm_network_security_group.nsg
azurerm_network_security_group.sg
azurerm_network_security_rule.egress
azurerm_network_security_rule.http_ingress["0.0.0.0/0"]
azurerm_network_security_rule.ssh_ingress["0.0.0.0/0"]
azurerm_public_ip.public_ip
azurerm_resource_group.rg
azurerm_role_assignment.instance
azurerm_storage_account.sa
azurerm_storage_table.storage["chainstate"]
azurerm_storage_table.storage["nonces"]
azurerm_storage_table.storage["permits"]
azurerm_storage_table.storage["secretversions"]
azurerm_storage_table.storage["verifiers"]
azurerm_subnet.subnet
azurerm_user_assigned_identity.uai
azurerm_virtual_machine.vm
azurerm_virtual_network.vnet
tls_private_key.ssh
```

## How to remove, patch, or otherwise deal with previously installed resources

To upgrade the infrastructure, pull the latest terraform modules and run `terraform apply` in the
main directory (the `tf_state` will likely never need to change).

To destroy the infrastructure run

```sh
terraform apply -auto-approve # if dirty
./s4-infra.sh unlock
terraform apply -auto-approve # apply the unlock
terraform destroy
cd tf_state
terraform apply -auto-approve # apply the unlock
terraform destroy
```
