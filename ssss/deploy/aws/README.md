# Amazon Web Services (AWS)

These are instructions to use Terraform to get the necessary cloud resources for the SSSS into your
AWS account. [OpenTofu](https://opentofu.org) is an open-source tool to define and provision cloud
resources with simple, declarative configuration files.

## Prerequisites

- OpenTofu [(installation instructions)](https://opentofu.org/docs/intro/install/) or Terraform
- AWS CLI
  [(installation instructions)](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)

## AWS Configuration

Terraform needs your AWS credentials to install resources on your behalf. Follow the instructions in
the [guide](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_root-user_manage_add-key.html#),
ignoring any "Alternatives to root user access keys". Record the new access key and secret key.

Then in your command line run:

```sh
aws configure
```

When it asks, put in your account and secret key, as well as a default region code and an output
format that you may leave blank. For region codes refer to
[this list](https://www.aws-services.info/regions.html).

You'll need to know the following three things:

1. The name of your AWS profile that you just configured

It's probably `default` or your user name. If you care to check, get this by running
`head -n 1 ~/.aws/credentials`. This should return something like:

```sh
[<profile name>]
```

2. The region code you want to install the resources

There is not currently a good method to decide where an SSSS should live so you may pick one that is
in a place dear to your heart. (Or you have reason to believe it's underserved.) _The region does
not have to match your profile's default._

3. A domain name or subdomain where you will provide the expose SSSS API (e.g., `ssss.escrin.org`).

## Quick install

We need to install both a backend for our Terraform state file (read more about
[terraform state files](https://www.pluralsight.com/resources/blog/cloud/what-is-terraform-state))
as well as the resources needed for the SSSS. To install this in one go:

```sh
./s4-infra.sh apply
```

This script will ask you for the above mentioned profile name, region, domain name, and a globally unique s3 bucket name.

If you want more control over your terraform processes use the
[two-step install](#two-step-install).

If this script ran successfully, head over to [verify installation](#verify-installation).

To tear the environment down, run these

```sh
./s4-infra.sh unlock
./s4-infra.sh destroy # optionally pass --all to destroy everything
./s4-infra.sh lock
```

## Two-step install

Skip if you've done the quick install.

```sh
export AWS_PROFILE=<your aws profile name here>
export AWS_REGION=<your aws region name here>
```

### Create Backend State

This will create the Terraform state bucket in AWS and DynamoDB table to manage terraform locks.

```sh
cd tf_state
terraform init
# You will be prompted to choose a globally unique S3 bucket name to save the Terraform state.
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
aws_dynamodb_table.tf_locks
aws_s3_bucket.tf_state
aws_s3_bucket_acl.tf_state
aws_s3_bucket_ownership_controls.tf_state
aws_s3_bucket_server_side_encryption_configuration.tf_state_sse
aws_s3_bucket_versioning.tf_state
aws_s3_bucket_versioning.tf_state_versioning
data.aws_iam_policy_document.ec2_assume_role_policy
aws_dynamodb_table.chain_state
aws_dynamodb_table.secrets
aws_dynamodb_table.nonces
aws_dynamodb_table.permits
aws_dynamodb_table.verifiers
aws_iam_role.ec2_role
aws_iam_role_policy_attachment.attach_ec2_policy
```

And check whether the state files are present in your bucket:

```sh
# <your-globally-unique-bucket-name> is `escrin.tfstate.${domain}` if created using the script
aws s3 ls s3://<your-globally-unique-bucket-name>
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
