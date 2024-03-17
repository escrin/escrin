# aws

## Prerequisites

- terraform

## AWS Authentication and Env Vars

```bash
export AWS_PROFILE=<your aws profile name here>
export AWS_REGION=<your aws region name here>
```

## Create Backend State

This will create the Terraform state bucket in AWS and DynamoDB table to manage terraform locks.

```bash
cd tf_state
terraform init
# You will be prompted to choose a globally unique S3 bucket name to save the Terraform state.
terraform plan -out out.tfstate
terraform apply out.tfstate
```

## Create Resources

```bash
# This is ran from the root of this directory: ./ssss/deploy/aws/
# You will be prompted for an S3 bucket name. Use the same bucket name for the
# backend that was set in the previous Create Backend State step.
terraform init
terraform plan -out out.tfstate
terraform apply out.tfstate
```
