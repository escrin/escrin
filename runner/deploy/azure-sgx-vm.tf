provider "azurerm" {
  features {}
}

variable "ssh_private_key_path" {
  type = string
}

variable "ssh_public_key_path" {
  type = string
}

variable "envfile_path" {
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


resource "azurerm_network_security_group" "escrin" {
  name                = "escrin-nsg"
  location            = azurerm_resource_group.escrin.location
  resource_group_name = azurerm_resource_group.escrin.name

  security_rule {
    name                       = "allow_ssh"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_address_prefix      = "*"
    source_port_range          = "*"
    destination_address_prefix = "*"
    destination_port_range     = "22"
  }

  security_rule {
    name                       = "allow_ipfs_p2p"
    priority                   = 101
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_address_prefix      = "*"
    source_port_range          = "*"
    destination_address_prefix = "*"
    destination_port_range     = "4001"
  }

  security_rule {
    name                       = "allow_ipfs_p2p_udp"
    priority                   = 102
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Udp"
    source_address_prefix      = "*"
    source_port_range          = "*"
    destination_address_prefix = "*"
    destination_port_range     = "4001"
  }

  security_rule {
    name                       = "allow_ipfs_gateway"
    priority                   = 103
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_address_prefixes = [
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
    ]
    source_port_range          = "*"
    destination_address_prefix = "*"
    destination_port_range     = "80"
  }
}

resource "azurerm_subnet_network_security_group_association" "escrin" {
  subnet_id                 = azurerm_subnet.escrin.id
  network_security_group_id = azurerm_network_security_group.escrin.id
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

  provisioner "file" {
    source      = var.envfile_path
    destination = "/opt/escrin/.env"
  }

  provisioner "remote-exec" {
    inline = [
      "cd /opt/escrin && sudo docker compose pull && sudo docker compose up --detach --no-build"
    ]
  }
}

output "public_ip_address" {
  value = azurerm_public_ip.escrin.ip_address
}
