resource "aws_vpc" "main" {
  cidr_block = var.vpc_cidr
  tags = var.tags
}

resource "aws_subnet" "main" {
  vpc_id     = "${aws_vpc.main.id}"
  cidr_block = cidrsubnets(aws_vpc.main.cidr_block, 8)

  tags = var.tags
}