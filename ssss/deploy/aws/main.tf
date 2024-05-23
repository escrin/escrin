terraform {
  backend "s3" {
    key            = "terraform.tfstate"
    dynamodb_table = "tflocks"
  }
}

variable "instance_type" {
  description = "The SSSS EC2 instance type"
  default     = "t2.nano"
}

variable "ssss_tag" {
  description = "The tag of the ghcr.io/escrin/ssss image to use"
}

variable "ssh_key" {
  description = "Name of the key pair for SSH access to the EC2 instance."
  default     = ""
}

variable "cloudflare" {
  description = "Whether to restrict ingress to only Cloudflare IPs. Makes instance unreachable except through Cloudflare's relays."
  default     = false
}

locals {
  tags = {
    Vendor      = "escrin",
    Component   = "ssss",
    Environment = "${terraform.workspace}",
  }
}

resource "aws_kms_key" "signer" {
  description              = "Escrin SSSS permit signer (${terraform.workspace})"
  key_usage                = "SIGN_VERIFY"
  customer_master_key_spec = "ECC_SECG_P256K1"
  deletion_window_in_days  = 7
  tags                     = local.tags

  lifecycle {
    prevent_destroy = false
  }
}

resource "aws_kms_alias" "signer" {
  name          = "alias/escrin-signer-${terraform.workspace}"
  target_key_id = aws_kms_key.signer.key_id
}

resource "aws_dynamodb_table" "secrets" {
  name         = "escrin-secrets-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "id"
  range_key    = "version"
  tags         = local.tags

  attribute {
    name = "id"
    type = "S"
  }

  attribute {
    name = "version"
    type = "N"
  }

  ttl {
    attribute_name = "expiry"
    enabled        = true
  }

  lifecycle {
    prevent_destroy = false
  }
}

resource "aws_dynamodb_table" "verifiers" {
  name         = "escrin-verifiers-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "permitter"
  range_key    = "identity"
  tags         = local.tags

  attribute {
    name = "permitter"
    type = "S"
  }

  attribute {
    name = "identity"
    type = "S"
  }

  lifecycle {
    prevent_destroy = false
  }
}

data "aws_iam_policy_document" "policy" {
  statement {
    effect = "Allow"
    actions = [
      "kms:GetPublicKey",
      "kms:Sign",
    ]
    resources = [
      "${aws_kms_key.signer.arn}",
    ]
  }

  statement {
    effect = "Allow"
    actions = [
      "dynamodb:ConditionCheckItem",
      "dynamodb:DeleteItem",
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:UpdateItem",
      "dynamodb:Query",
    ]
    resources = [
      "${aws_dynamodb_table.secrets.arn}",
      "${aws_dynamodb_table.verifiers.arn}",
    ]
  }
}

resource "aws_iam_policy" "policy" {
  name        = "escrin_policy_${terraform.workspace}"
  description = "Escrin KM access policy"
  policy      = data.aws_iam_policy_document.policy.json
  tags        = local.tags
}

data "aws_iam_policy_document" "ec2_assume_role_policy" {
  statement {
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "ec2_role" {
  name               = "escrin_ec2_role_${terraform.workspace}"
  assume_role_policy = data.aws_iam_policy_document.ec2_assume_role_policy.json
  tags               = local.tags
}

resource "aws_iam_role_policy_attachment" "attach_ec2_policy" {
  role       = aws_iam_role.ec2_role.name
  policy_arn = aws_iam_policy.policy.arn
}

resource "aws_iam_group_policy_attachment" "attach_dev_policy" {
  count      = terraform.workspace == "dev" ? 1 : 0
  group      = "dev"
  policy_arn = aws_iam_policy.policy.arn
}

data "aws_ec2_instance_type" "instance" {
  instance_type = var.instance_type
}

locals {
  instance_archs = data.aws_ec2_instance_type.instance.supported_architectures
  instance_arch  = try(element([for v in local.instance_archs : v if can(regex("64$", v))], 0), null)
}

data "aws_ami" "ami" {
  most_recent = true
  owners      = ["amazon"]
  filter {
    name   = "name"
    values = ["al2023-ami-2023*-${local.instance_arch}"]
  }
}

resource "aws_instance" "instance" {
  ami                         = data.aws_ami.ami.id
  instance_type               = var.instance_type
  iam_instance_profile        = aws_iam_instance_profile.profile.name
  vpc_security_group_ids      = [aws_security_group.sg.id]
  user_data_replace_on_change = true
  key_name                    = var.ssh_key
  tags                        = merge(local.tags, { Name = "escrin-ssss-${terraform.workspace}" })

  user_data = <<-EOF
    #!/bin/bash
    yum -yq update
    yum -yq install containerd nerdctl cni-plugins iptables
    systemctl enable containerd
    systemctl start containerd
    nerdctl run -p 80:1075 -d --restart=always ghcr.io/escrin/ssss:${var.ssss_tag} -vv
    EOF

  root_block_device {
    volume_size = 8
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_iam_instance_profile" "profile" {
  name = "escrin_ec2_instance_profile_${terraform.workspace}"
  role = aws_iam_role.ec2_role.name
  tags = local.tags
}

resource "aws_eip" "eip" {
  instance = aws_instance.instance.id
  tags     = local.tags
}

output "instance_ip" {
  value = aws_eip.eip.public_ip
}

locals {
  sg_cidrs = var.cloudflare ? [
    # From https://www.cloudflare.com/ips-v4
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

resource "aws_security_group" "sg" {
  name        = "escrin-ssss-sg-${terraform.workspace}"
  description = "Allow HTTP & SSH from anywhere"
  tags        = local.tags

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = local.sg_cidrs
  }

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = local.sg_cidrs
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}
