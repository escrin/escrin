#!/bin/sh

# terra_genesis.sh - A script to set up TF backend and deploy resources with specified AWS profile and region.

set -eu

terraform_cmd=""
tf() {
	$terraform_cmd "$@"
}

log() {
	# shellcheck disable=SC2059
	printf "$@" 1>&2
}

die() {
	log "$@"
	log "\n"
	exit 1
}

log_do() {
	log "%s..." "$1"
	shift
	if ! "$@" >/dev/null; then
		exit 1
	fi
	log "✅\n"
}

if command -v terraform >/dev/null; then
	terraform_cmd="terraform"
elif command -v tofu >/dev/null; then
	terraform_cmd="tofu"
else
	die "Error: terraform or tofu must be installed to run this script."
fi

script_dir=$(cd "$(dirname "$0")" && pwd)
if [ ! -f "$script_dir/main.tf" ] || [ ! -f "$script_dir/tf_state/main.tf" ]; then
	die "Error: unknown context. Please ensure that you run this script from the escrin/escrin repo."
fi

ensure_aws_creds() {
	if [ -z "${AWS_PROFILE:-}" ] && [ -z "${AWS_DEFAULT_PROFILE:-}" ]; then
		printf 'Enter your AWS profile name: (e.g., "%s")' "$(whoami)"
		read -r aws_profile
		export AWS_PROFILE="$aws_profile"
	fi
}

ensure_aws_region() {
	if [ -z "${AWS_REGION:-}" ] && [ -z "${AWS_DEFAULT_REGION:-}" ]; then
		printf "Enter your AWS region code (e.g., us-west-1): "
		read -r aws_region
		export AWS_REGION="$aws_region"
	fi
}

ensure_workspace() {
	cd "$script_dir"
	log_do "Setting workspace" tf workspace select -or-create prod
}

apply() {
	ensure_aws_creds
	ensure_aws_region

	ensure_tfstate
	ensure_workspace
	ensure_infra
}

state_bucket=""

ensure_tfstate() {
	# Navigate to the directory containing TF configuration for the state
	cd "$script_dir/tf_state"

	# Check for an existing state file
	if [ -f "terraform.tfstate" ]; then
		# Check for an existing state bucket (and assume the locks table was created properly)
		existing_bucket=$(tf state show aws_s3_bucket.tf_state | grep " bucket " | cut -d\" -f2)
		if [ -n "$existing_bucket" ]; then
			state_bucket="$existing_bucket"
			return 0
		fi
	elif [ ! -d ".terraform" ]; then
		log_do "🆕 Initializing local backend" tf init
	fi

	printf "What is the domain name of your SSSS (e.g., ssss.escrin.org)? "
	read -r ssss_domain
	state_bucket="escrin.tfstate.${ssss_domain}"

	# First attempt to import the state bucket in case it was created but the local state file was lost.
	if log_do "🔎 Detecting state bucket" tf import -var "bucket_name=$state_bucket" aws_s3_bucket.tf_state "$state_bucket"; then
		# Next, import the locks table. If it doesn't exist, continue to state application.
		if log_do "🔎 Detecting locks table" tf import -var "bucket_name=$state_bucket" aws_dynamodb_table.tf_locks 'tflocks'; then
			return 0
		fi
	fi

	# Run the TF in the tf_state folder to get the backend installed
	log_do "🔨 Creating initial resources" tf apply -var "bucket_name=$state_bucket" -auto-approve
}

ensure_infra() {
	# Navigate to the directory containing TF configuration for the actual infra
	cd "$script_dir"

	# Check for an existing initialization
	if [ ! -d ".terraform" ]; then
		ensure_aws_region
		log_do "🚜 Initializing remote backend" tf init -backend-config="bucket=$state_bucket"
	fi

	log_do "🏗️ Creating infra" tf apply -auto-approve
}

destroy() {
	case "$1" in
	"--yes-i-really-mean-it")
		destroy_infra
		;;

	"--all")
		if [ "$2" != "--yes-i-really-really-mean-it" ]; then
			die "You must confirm with --yes-i-really-really-mean-it"
		fi
		die "Sorry, this is not implemented yet!"
		;;
	*)
		die "You must confirm with '--yes-i-really-mean-it' or '--all --yes-i-really-really-mean-it'"
		;;
	esac
}

destroy_infra() {
	ensure_workspace
	cd "$script_dir"
	log_do "🧨 Destroying infra" tf apply -destroy -auto-approve -json
}

case "${1:-}" in
"apply")
	apply
	;;
"destroy")
	shift
	destroy "${1:-}" "${2:-}"
	;;
*)
	die "Invalid command. The available commands are: apply, destroy"
	;;
esac
