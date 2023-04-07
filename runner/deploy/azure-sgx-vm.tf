provider "azurerm" {
  features {}
}

variable "ssh_private_key_path" {
  type = string
}

variable "ssh_public_key_path" {
  type = string
}

variable "estuary_api_key" {
  type = string
}

variable "wallet_private_key" {
  type = string
}

variable "bacalhau_branch" {
  type = string
}

variable "lilypad_branch" {
  type = string
}

variable "lilypad_events_addr" {
  type = string
}

resource "azurerm_resource_group" "escrin" {
  name     = "escrin-rg"
  location = "eastus"
}

resource "azurerm_virtual_network" "escrin" {
  name                = "escrin-vnet"
  address_space       = ["10.0.0.0/16"]
  location            = azurerm_resource_group.escrin.location
  resource_group_name = azurerm_resource_group.escrin.name
}

resource "azurerm_subnet" "escrin" {
  name                 = "escrin-subnet"
  resource_group_name  = azurerm_resource_group.escrin.name
  virtual_network_name = azurerm_virtual_network.escrin.name
  address_prefixes     = ["10.0.1.0/24"]
}

resource "azurerm_public_ip" "escrin" {
  name                = "escrin-pip"
  location            = azurerm_resource_group.escrin.location
  resource_group_name = azurerm_resource_group.escrin.name
  allocation_method   = "Static"
}

resource "azurerm_network_interface" "escrin" {
  name                = "escrin-nic"
  location            = azurerm_resource_group.escrin.location
  resource_group_name = azurerm_resource_group.escrin.name

  ip_configuration {
    name                          = "escrin-ipc"
    subnet_id                     = azurerm_subnet.escrin.id
    public_ip_address_id          = azurerm_public_ip.escrin.id
    private_ip_address_allocation = "Dynamic"
  }
}

resource "azurerm_linux_virtual_machine" "escrin" {
  name                = "escrin-vm"
  location            = azurerm_resource_group.escrin.location
  resource_group_name = azurerm_resource_group.escrin.name
  size                = "Standard_DC1s_v2"

  source_image_reference {
    publisher = "Canonical"
    offer     = "0001-com-ubuntu-server-focal"
    sku       = "20_04-lts-gen2"
    version   = "latest"
  }

  admin_username = "ubuntu"

  admin_ssh_key {
    username   = "ubuntu"
    public_key = file(var.ssh_public_key_path)
  }

  network_interface_ids = [azurerm_network_interface.escrin.id]

  priority = "Spot"
  eviction_policy = "Deallocate"


  os_disk {
    name              = "escrin-osdisk"
    caching           = "ReadWrite"
    storage_account_type = "Standard_LRS"
    disk_size_gb      = 30
  }
}

resource "null_resource" "install_deps" {
  depends_on = [
    azurerm_linux_virtual_machine.escrin
  ]

  connection {
    type        = "ssh"
    user        = "ubuntu"
    host        = "${azurerm_public_ip.escrin.ip_address}"
    private_key = file(var.ssh_private_key_path)
  }

  provisioner "remote-exec" {
    inline = [
      "sudo apt-get update",
      "sudo apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release",
      "curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor | sudo tee /usr/share/keyrings/docker-archive-keyring.gpg > /dev/null",
      "echo \"deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null",
      "wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | gpg --dearmor | sudo tee /usr/share/keyrings/intel-sgx-archive-keyring.gpg >/dev/null",
      "echo \"deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx-archive-keyring.gpg] https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main\" | sudo tee /etc/apt/sources.list.d/intelsgx.list",
      "sudo apt-get update",
      "sudo apt-get install -y docker-ce docker-ce-cli containerd.io libsgx-dcap-ql",
      "sudo usermod -aG docker $USER"
    ]
  }
}

resource "null_resource" "run_containers" {
  depends_on = [
    null_resource.install_deps
  ]

  connection {
    type        = "ssh"
    user        = "ubuntu"
    host        = "${azurerm_public_ip.escrin.ip_address}"
    private_key = file(var.ssh_private_key_path)
  }

  provisioner "remote-exec" {
    inline = [
      "sudo mkdir -p /opt/escrin && sudo chmod a+rwx /opt/escrin"
    ]
  }

  provisioner "file" {
    source      = "./docker-compose.yaml"
    destination = "/opt/escrin/docker-compose.yaml"
  }

  provisioner "remote-exec" {
    inline = [
      "cd /opt/escrin && export BACALHAU_BRANCH=${var.bacalhau_branch} LILYPAD_BRANCH=${var.lilypad_branch} ESTUARY_API_KEY=${var.estuary_api_key} WALLET_PRIVATE_KEY=${var.wallet_private_key} LILYPAD_EVENTS_ADDR=${var.lilypad_events_addr} && docker compose pull && docker compose up --detach --no-build"
    ]
  }
}

output "public_ip_address" {
  value = azurerm_public_ip.escrin.ip_address
}
