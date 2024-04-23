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

if command -v sha256sum >/dev/null; then
	sha256_cmd="sha256sum"
elif command -v shasum >/dev/null; then
	sha256_cmd="shasum -a 256"
else
	die "Error: unable to locate sha256sum or shasum."
fi

script_dir=$(cdd "$(dirname "$0")" && pwd)
if [ ! -f "$script_dir/main.tf" ] || [ ! -f "$script_dir/tf_state/main.tf" ]; then
	die "Error: unknown context. Please ensure that you run this script from the escrin/escrin repo."
fi

subid=""
ssss_hostname=""
ssss_location=""
tfstate_storage_account=""

ensure_subscription() {
	subid="$(az account show --query 'id' -o tsv)"
}

set_globals() {
	ssss_hostname="$1"
	ssss_location="$2"
	tfstate_storage_account="escrintf$(printf "%s" "$ssss_hostname" | "$sha256_cmd" | head -c 16)"
}

obtain_existing_state() {
	cdd "$script_dir/tf_state"

	if [ ! -f "terraform.tfstate" ]; then
		return 1
	fi

	hostname=$(tf output -raw hostname 2>/dev/null)
	location=$(tf state show azurerm_storage_account.sa 2>/dev/null | grep " location " | cut -d\" -f2)
	if [ -n "$hostname" ] && [ -n "$location" ]; then
		set_globals "$hostname" "$location"
		return 0
	fi

	return 1
}

obtain_ssss_hostname_and_location() {
	cdd "$script_dir/tf_state"

	if obtain_existing_state; then
		return 0
	fi

	if [ -z "$ssss_hostname" ]; then
		printf "❓ What is the domain name of your SSSS (e.g., ssss.example.com)? "
		read -r hostname
	fi

	if [ -z "$ssss_location" ]; then
		printf "❓ Into what Azure location do you want to deploy (e.g., eastus)? "
		read -r location
	fi

	set_globals "$hostname" "$location"

	return 1
}

tfv() {
	cmd="$1"
	shift
	tf "$cmd" -var "hostname=$ssss_hostname" -var "location=$ssss_location" "$@"
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
		ensure_subscription
		ensure_tfstate
		;;
	*)
		ensure_subscription
		ensure_tfstate
		ensure_infra
		tf output
		;;
	esac
}

ensure_tfstate() {
	cdd "$script_dir/tf_state"

	if obtain_ssss_hostname_and_location; then
		return 0
	fi

	if [ -f "terraform.tfstate" ]; then
		log "🔎 Detecting existing state..."
		import() {
			tfv import "$1" "$2" >/dev/null 2>&1
		}
		rgid="/subscriptions/$subid/resourceGroups/escrin-ssss-tfstate"
		said="$rgid/providers/Microsoft.Storage/storageAccounts/${tfstate_storage_account}"
		scid="https://${tfstate_storage_account}.blob.core.windows.net/terraform"
		if import azurerm_resource_group.rg "$rgid"; then
			if import azurerm_storage_account.sa "$said"; then
				if import azurerm_storage_container.sc "$scid"; then
					log "✅\n"
					return 0
				fi
			fi
		fi
		log "❎\n"
	elif [ ! -d ".terraform" ]; then
		log_do "🆕 Initializing local backend" tf init
	fi

	# Run the TF in the tf_state folder to get the backend installed
	log_do "🔨 Creating state infra" tfv apply -auto-approve
}

ensure_infra() {
	# Navigate to the directory containing TF configuration for the actual infra
	cdd "$script_dir"

	# Check for an existing initialization
	if [ ! -d ".terraform" ]; then
		log_do "🆕 Initializing remote backend" tf init -backend-config="storage_account_name=$tfstate_storage_account"
	fi

	ssss_tag=$(git tag -l 'ssss/v*.*.*' --sort=-taggerdate | head -n 1 | sed 's?ssss/v??')

	ensure_workspace
	log_do "🔨 Creating SSSS infra" tfv apply -var "ssss_tag=$ssss_tag" -auto-approve
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
		ensure_subscription
		if ! obtain_existing_state; then
			return 0
		fi
		destroy_infra
		destroy_tfstate
		;;
	*)
		warn_destroy
		ensure_subscription
		if ! obtain_existing_state; then
			return 0
		fi
		destroy_infra
		;;
	esac
}

destroy_infra() {
	cdd "$script_dir"
	if [ -d ".terraform" ]; then
		ensure_workspace
		log_do "🧹 Destroying infra" tfv apply -var "ssss_tag=" -destroy -auto-approve
	fi
}

destroy_tfstate() {
	cdd "$script_dir/tf_state"
	if [ -d ".terraform" ]; then
		log_do "🧹 Destroying state" tfv apply -destroy -auto-approve
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
