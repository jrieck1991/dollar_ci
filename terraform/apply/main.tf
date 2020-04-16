module "network" {
  source = "../../modules/network"

  vpc_cidr = "10.0.1.0/16"
}

module "ec2" {
  source = "../../modules/ec2"

  name          = "dollar_ci"
  instance_type = "t2.micro"
  image_id      = "ami-0323c3dd2da7fb37d"
  subnet_ids    = module.network.subnet_ids
}
