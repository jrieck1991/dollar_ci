variable "tags" {
  type        = map
  description = "resource tags"
  default     = {}
}

variable "name" {
  description = "name of resources"
  type        = string
}

variable "vpc_id" {
  description = "id of vpc to use"
  type        = string
}

variable "instance_type" {
  description = "ec2 instance type"
  type        = string
}
