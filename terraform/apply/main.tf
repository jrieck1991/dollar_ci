provider "aws" {
  region = "us-east-1"
}

locals {
  tags = {
    app = "dollar_ci"
  }
}

module "network" {
  source = "../modules/network"

  vpc_cidr = "10.0.0.0/16"

  tags = local.tags
}

module "ec2" {
  source = "../modules/ec2"

  name          = "dollar_ci"
  instance_type = "t2.micro"
  subnet_ids    = module.network.public_subnet_ids

  tags = local.tags
}
