variable "tags" {
  type        = map
  description = "resource tags"
  default     = {}
}

variable "vpc_cidr" {
  description = "cidr for VPC"
  type        = string
}
