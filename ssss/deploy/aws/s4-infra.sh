#!/bin/sh

# s4-infra.sh - A script to manage SSSS infrastructure (using Terraform or OpenTofu)

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

cdd() {
	cd "$1" || die "Error: failed to chdir to $1"
}

log_do() {
	log "%s..." "$1"
	shift
	set +e
	out="$( ("$@" >/dev/null) 2>&1)"
	status=$?
	if [ $status != 0 ]; then
		log "❌\n"
		log "$out"
		exit 1
	fi
	set -e
	log "✅\n"
}

if command -v terraform >/dev/null; then
	terraform_cmd="terraform"
elif command -v tofu >/dev/null; then
	terraform_cmd="tofu"
else
	die "Error: terraform or tofu must be installed to run this script."
fi

script_dir=$(cdd "$(dirname "$0")" && pwd)
if [ ! -f "$script_dir/main.tf" ] || [ ! -f "$script_dir/tf_state/main.tf" ]; then
	die "Error: unknown context. Please ensure that you run this script from the escrin/escrin repo."
fi

ensure_aws_creds() {
	if [ -z "${AWS_PROFILE:-}" ] && [ -z "${AWS_DEFAULT_PROFILE:-}" ]; then
		printf "Enter your AWS profile name (e.g., %s): " "$(whoami)"
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
	current_workspace="$(tf workspace show)"
	if [ "$current_workspace" != "dev" ] && [ "$current_workspace" != "prod" ]; then
		log_do "➡️  Switching to prod workspace" tf workspace select -or-create prod
	fi
}

apply() {
	case "$1" in
	"-h" | "--help")
		die "${0} apply [--only-tfstate]"
		;;
	"--only-tfstate")
		ensure_aws_creds
		ensure_aws_region
		ensure_tfstate
		;;
	*)
		ensure_aws_creds
		ensure_aws_region
		ensure_tfstate
		ensure_infra
		tf output
		;;
	esac
}

state_bucket=""

obtain_state_bucket() {
	# Navigate to the directory containing TF configuration for the state
	cdd "$script_dir/tf_state"

	# Check for an existing state file
	if [ -f "terraform.tfstate" ]; then
		# Check for an existing state bucket (and assume the locks table was created properly)
		existing_bucket=$(tf state show aws_s3_bucket.tf_state 2>/dev/null | grep " bucket " | cut -d\" -f2)
		if [ -n "$existing_bucket" ]; then
			state_bucket="$existing_bucket"
			return 0
		fi
	elif [ ! -d ".terraform" ]; then
		log_do "🆕 Initializing local backend" tf init
	fi

	printf "❓ What is the domain name of your SSSS (e.g., ssss.example.com)? "
	read -r ssss_domain
	state_bucket="tfstate.$ssss_domain"
	return 1
}

ensure_tfstate() {
	# Navigate to the directory containing TF configuration for the state
	cdd "$script_dir/tf_state"

	if obtain_state_bucket; then
		return 0
	fi

	# Attempt to import the state bucket in case it was created but the local state file was lost.
	log "🔎 Detecting existing state..."
	if tf import -var "bucket_name=$state_bucket" aws_s3_bucket.tf_state "$state_bucket" >/dev/null 2>&1; then
		# Next, import the locks table. If it doesn't exist, continue to state application.
		if tf import -var "bucket_name=$state_bucket" aws_dynamodb_table.tf_locks 'escrin.tflocks' >/dev/null 2>&1; then
			log "✅\n"
			return 0
		fi
	fi
	log "❎\n"

	# Run the TF in the tf_state folder to get the backend installed
	log_do "🔨 Creating state infra" tf apply -var "bucket_name=$state_bucket" -auto-approve
}

ensure_infra() {
	# Navigate to the directory containing TF configuration for the actual infra
	cdd "$script_dir"

	# Check for an existing initialization
	if [ ! -d ".terraform" ]; then
		ensure_aws_region
		log_do "🆕 Initializing remote backend" tf init -backend-config="bucket=$state_bucket"
	fi

	ssss_tag=$(git tag -l 'ssss/v*.*.*' --sort=-taggerdate | head -n 1 | sed 's?ssss/v??')

	ensure_workspace
	log_do "🔨 Creating SSSS infra" tf apply -var "ssss_tag=$ssss_tag" -auto-approve
}

warn_destroy() {
	cdd "$script_dir"
	if [ "$(tf workspace show)" = "prod" ]; then
		log_do "⚠️  Confirming destruction of production infra" sleep 10
		log_do "‼️  Preparing to destroy production infra" sleep 10
	fi
}

destroy() {
	case "$1" in
	"-h" | "--help")
		die "${0} destroy [--all]\n Call \`${0} unlock\` first"
		;;
	"--all")
		warn_destroy
		ensure_aws_creds
		ensure_aws_region
		destroy_infra
		destroy_tfstate
		;;
	*)
		warn_destroy
		ensure_aws_creds
		ensure_aws_region
		destroy_infra
		;;
	esac
}

destroy_infra() {
	cdd "$script_dir"
	if [ -d ".terraform" ]; then
		ensure_workspace
		log_do "🧹 Destroying infra" tf apply -var "ssss_tag=" -destroy -auto-approve
	fi
}

destroy_tfstate() {
	cdd "$script_dir/tf_state"
	if [ -d ".terraform" ]; then
		obtain_state_bucket
		log_do "🧹 Destroying state" tf apply -var "bucket_name=$state_bucket" -destroy -auto-approve
	fi
}

unlock() {
	case "$1" in
	"-h" | "--help")
		die "${0} unlock [--all]"
		;;
	"--all")
		ensure_unlocked "$script_dir"
		ensure_unlocked "$script_dir/tf_state"
		;;
	*)
		ensure_unlocked "$script_dir"
		;;
	esac
}

lock() {
	case "$1" in
	"-h" | "--help")
		die "${0} lock"
		;;
	*)
		ensure_locked "$script_dir"
		ensure_locked "$script_dir/tf_state"
		;;
	esac
}

ensure_unlocked() {
	cdd "$1"
	perl -pi -e 's/prevent_destroy = true/prevent_destroy = false/' ./*.tf
}

ensure_locked() {
	cdd "$1"
	perl -pi -e 's/prevent_destroy = false/prevent_destroy = true/' ./*.tf
}

case "${1:-}" in
"apply")
	shift
	apply "${1:-}"
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
