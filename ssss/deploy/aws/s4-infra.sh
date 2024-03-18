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
	out="/dev/null"
	if [ -n "${VERBOSE:-}" ]; then
		out="/dev/stderr"
	fi
	if ! "$@" >"$out"; then
		log "❌\n"
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
	current_workspace="$(tf workspace show)"
	if [ "$current_workspace" != "dev" ] && [ "$current_workspace" != "prod" ]; then
		log_do "Switching to production workspace" tf workspace select -or-create prod
	fi
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
	"--help")
		die "${0} destroy [--all]\n Call \`${0} unlock\` first"
		;;
	"--all")
		destroy_infra
		destroy_tfstate
		;;
	*)
		destroy_infra
		;;
	esac
}

destroy_infra() {
	ensure_workspace
	cd "$script_dir"
	log_do "🧨 Destroying infra" tf apply -destroy -auto-approve -json
}

destroy_tfstate() {
	cd "$script_dir/tf_state"
	log_do "🌋 Destroying infra state" tf apply -destroy -auto-approve -json
}

unlock() {
	case "$1" in
	"--help")
		die "${0} unlock [--all]"
		;;
	"--all")
		ensure_workspace
		ensure_unlocked "$script_dir"
		ensure_unlocked "$script_dir/tf_state"
		;;
	*)
		ensure_workspace
		ensure_unlocked "$script_dir"
		;;
	esac
}

ensure_unlocked() {
	cd "$1"
	sed -i '' -e 's/prevent_destroy = true/prevent_destroy = false/' ./*.tf
	if [ "$(tf workspace show)" = "dev" ]; then
		return 0
	fi
	sed -i '' -e 's/deletion_protection_enabled = terraform.workspace != "dev"/deletion_protection_enabled = false/' ./*.tf
	log_do "🔧 Applying unlock in $(basename "$1")" tf apply -auto-approve
}

lock() {
	case "$1" in
	"--help")
		die "${0} lock [--all]"
		;;
	"--all")
		ensure_workspace
		ensure_locked "$script_dir"
		ensure_locked "$script_dir/tf_state"
		;;
	*)
		ensure_workspace
		ensure_locked "$script_dir"
		;;
	esac
}

ensure_locked() {
	cd "$1"
	sed -i '' -e 's/prevent_destroy = false/prevent_destroy = true/' ./*.tf ./*/*.tf
	if [ "$(tf workspace show)" = "dev" ]; then
		return 0
	fi
	sed -i '' -e 's/deletion_protection_enabled = false/deletion_protection_enabled = terraform.workspace != "dev"/' ./*.tf ./*/*.tf
	log_do "🔧 Applying lock in $(basename "$1")" tf apply -auto-approve
}

case "${1:-}" in
"apply")
	apply
	;;
"destroy")
	shift
	destroy "${1:-}"
	;;
"unlock")
	shift
	unlock "${1:-}"
	;;
"lock")
	shift
	lock "${1:-}"
	;;
*)
	die "${0} (apply|destroy|unlock|lock)"
	;;
esac
