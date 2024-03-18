terraform {
  backend "s3" {
    key            = "terraform.tfstate"
    dynamodb_table = "tflocks"
  }
}

resource "aws_kms_key" "sek" {
  description             = "Escrin secret share encryption key (${terraform.workspace})"
  deletion_window_in_days = 7

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_kms_alias" "sek" {
  name          = "alias/escrin-sek-${terraform.workspace}"
  target_key_id = aws_kms_key.sek.key_id
}

resource "aws_dynamodb_table" "shares" {
  name         = "escrin-shares-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "id"
  range_key    = "version"

  attribute {
    name = "id"
    type = "S"
  }

  attribute {
    name = "version"
    type = "N"
  }

  point_in_time_recovery {
    enabled = terraform.workspace != "dev"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_dynamodb_table" "keys" {
  name         = "escrin-keys-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "id"
  range_key    = "version"

  attribute {
    name = "id"
    type = "S"
  }

  attribute {
    name = "version"
    type = "N"
  }

  point_in_time_recovery {
    enabled = terraform.workspace != "dev"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_dynamodb_table" "permits" {
  name         = "escrin-permits-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "identity"
  range_key    = "recipient"

  attribute {
    name = "identity"
    type = "S"
  }

  attribute {
    name = "recipient"
    type = "S"
  }

  ttl {
    attribute_name = "expiry"
    enabled        = true
  }

  point_in_time_recovery {
    enabled = terraform.workspace != "dev"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_dynamodb_table" "nonces" {
  name         = "escrin-nonces-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "nonce"

  attribute {
    name = "nonce"
    type = "B"
  }

  ttl {
    attribute_name = "expiry"
    enabled        = true
  }

  point_in_time_recovery {
    enabled = terraform.workspace != "dev"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_dynamodb_table" "chain_state" {
  name         = "escrin-chain-state-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "chain"

  attribute {
    name = "chain"
    type = "N"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_dynamodb_table" "verifiers" {
  name         = "escrin-verifiers-${terraform.workspace}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "permitter"
  range_key    = "identity"

  attribute {
    name = "permitter"
    type = "S"
  }

  attribute {
    name = "identity"
    type = "S"
  }

  point_in_time_recovery {
    enabled = terraform.workspace != "dev"
  }

  deletion_protection_enabled = terraform.workspace != "dev"

  lifecycle {
    prevent_destroy = true
  }
}

data "aws_iam_policy_document" "km_policy_doc" {
  statement {
    effect = "Allow"
    actions = [
      "kms:Encrypt",
      "kms:ReEncrypt",
      "kms:Decrypt",
    ]
    resources = [
      "${aws_kms_key.sek.arn}",
    ]
  }

  statement {
    effect = "Allow"
    actions = [
      "dynamodb:ConditionCheckItem",
      "dynamodb:DeleteItem",
      "dynamodb:GetItem",
      "dynamodb:PutItem",
      "dynamodb:Query",
    ]
    resources = [
      "${aws_dynamodb_table.shares.arn}",
      "${aws_dynamodb_table.keys.arn}",
      "${aws_dynamodb_table.permits.arn}",
      "${aws_dynamodb_table.nonces.arn}",
      "${aws_dynamodb_table.verifiers.arn}",
      "${aws_dynamodb_table.chain_state.arn}",
    ]
  }
}

resource "aws_iam_policy" "km_policy" {
  name        = "km_policy"
  description = "Escrin KM access policy"
  policy      = data.aws_iam_policy_document.km_policy_doc.json
}

data "aws_iam_policy_document" "ec2_assume_role_policy" {
  statement {
    effect = "Allow"

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }

    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_role" "ec2_role" {
  name               = "EC2Role"
  assume_role_policy = data.aws_iam_policy_document.ec2_assume_role_policy.json
}

resource "aws_iam_role_policy_attachment" "attach_ec2_policy" {
  role       = aws_iam_role.ec2_role.name
  policy_arn = aws_iam_policy.km_policy.arn
}

resource "aws_iam_group" "dev" {
  count = terraform.workspace == "dev" ? 1 : 0
  name  = "dev"
}

resource "aws_iam_group_policy_attachment" "attach_dev_policy" {
  count      = terraform.workspace == "dev" ? 1 : 0
  group      = aws_iam_group.dev[count.index].name
  policy_arn = aws_iam_policy.km_policy.arn
}
