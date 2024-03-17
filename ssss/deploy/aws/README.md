# Amazon Web Services (AWS)

These are instructions to use Terraform to get the necessary cloud resources for the SSSS into your AWS account. [Terraform](https://www.terraform.io/) is an open-source tool to define and provision cloud resources with simple, declarative configuration files.

## Prerequisites

- Terraform [(installation instructions)](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli)
- AWS CLI [(installation instructions)](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)

## AWS Configuration

Terraform needs your AWS credentials to install resources on your behalf. [Here](https://www.msp360.com/resources/blog/how-to-find-your-aws-access-key-id-and-secret-access-key/#:~:text=1%20Go%20to%20Amazon%20Web,and%20Secret%20Access%20Key%20option.) is a good tutorial on how to get your Access Key ID & Secret Access Key.

Then run:

```bash
aws configure
```

And put in your account and secret key, as well as a default region code and an output format that you may leave blank. For region codes refer to [this list](https://www.aws-services.info/regions.html).

You'll need to know the following three things:

1. The name of your AWS profile that you just configured

It's probably `default`. If you care to check, get this by running `cat ~/.aws/credentials`. This should return something like:

```bash
[default]
aws_access_key_id = ABC123
aws_secret_access_key = ABC123
```

Where `default` is your profile name.

2. The region code you want to install the resources

There is not currently a good method to decide where an SSSS should live so you may pick one that is in a place dear to your heart. (Or you have reason to believe it's underserved.) *The region does not have to match your profile's default.*

3. A *globally unique* name for your s3 storage bucket

## Quick install

We need to install both a backend for our Terraform state file (read more about [terraform state files](https://www.pluralsight.com/resources/blog/cloud/what-is-terraform-state)) as well as the resources needed for the SSSS. To install this in one go:

```bash
sh terra_genesis.sh
```

This script will ask you for the above mentioned profile name, region, and bucket name.

If you want more control over your terraform processes use the [two-step install](#two-step-install). 

If this script ran successfully, head over to [verify installation](#verify-installation).

## Two-step install

Skip if you've done the quick install.

```bash
export AWS_PROFILE=<your aws profile name here>
export AWS_REGION=<your aws region name here>
```

### Create Backend State

This will create the Terraform state bucket in AWS and DynamoDB table to manage terraform locks.

```bash
cd tf_state
terraform init
# You will be prompted to choose a globally unique S3 bucket name to save the Terraform state.
terraform plan -out out.tfstate
terraform apply out.tfstate
```

### Create Resources

```bash
# This is ran from the root of this directory: ./ssss/deploy/aws/
# You will be prompted for an S3 bucket name. Use the same bucket name for the
# backend that was set in the previous Create Backend State step.
terraform init
terraform plan -out out.tfstate
terraform apply out.tfstate
```

## Verify installation

Let's check to see whether we have all the resources we need:

```bash 
cd tf_state && terraform state list && cd .. && terraform state list
```

This should show the following output:

```bash
aws_dynamodb_table.tf_locks
aws_s3_bucket.tf_state
aws_s3_bucket_acl.tf_state
aws_s3_bucket_ownership_controls.tf_state
aws_s3_bucket_server_side_encryption_configuration.tf_state_sse
aws_s3_bucket_versioning.tf_state
aws_s3_bucket_versioning.tf_state_versioning
data.aws_iam_policy_document.ec2_assume_role_policy
data.aws_iam_policy_document.km_policy_doc
aws_dynamodb_table.chain_state
aws_dynamodb_table.keys
aws_dynamodb_table.nonces
aws_dynamodb_table.permits
aws_dynamodb_table.shares
aws_dynamodb_table.verifiers
aws_iam_policy.km_policy
aws_iam_role.ec2_role
aws_iam_role_policy_attachment.attach_ec2_policy
aws_kms_key.sek
```

And check whether the state files are present in your bucket:

```bash
aws s3 ls s3://<your-globally-unique-bucket-name>
```

## How to remove, patch or otherwise deal with, previously installed resources

- Coming soon