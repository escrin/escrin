#!/bin/bash

# terra_genesis.sh - A script to set up Terraform backend and deploy resources with specified AWS profile and region.

echo "This process will set up a backend to house your Terraform state, and then create the necessary resources for an Escrin SSSS"

# Prompt and export environment variables
read -p "Enter your AWS profile name: " aws_profile
read -p "Enter your AWS region: " aws_region
export AWS_PROFILE=$aws_profile
export AWS_REGION=$aws_region

# Prompt for S3 bucket name
read -p "Enter a globally unique S3 bucket name for Terraform state storage: " bucket_name

# Navigate to the directory containing Terraform configuration for the state
cd tf_state || exit

# Run the terraform in the tf_state folder to get the backend installed
terraform init
terraform plan -var "bucket_name=${bucket_name}" -out out.tfstate
terraform apply out.tfstate
rm out.tfstate 

# Navigate to the directory containing the Terraform configuration for main resources
cd .. || exit

# Initialize Terraform with the backend configured in the previous step
terraform init -backend-config="bucket=$bucket_name" -backend-config="dynamodb_table=tflocks" -backend-config="region=$aws_region"

# Plan and apply Terraform configuration for resources
terraform plan -out out.tfstate
terraform apply out.tfstate
rm out.tfstate

echo "Successfully set up the backend and deployed the resources using the AWS profile '$aws_profile' and region '$aws_region'."
