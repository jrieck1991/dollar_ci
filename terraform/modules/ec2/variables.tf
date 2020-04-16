variable "tags" {
  type        = map
  description = "resource tags"
  default     = {}
}

variable "name" {
  description = "name of resources"
  type        = string
}

variable "subnet_ids" {
  description = "list of subnet ids"
  type        = list
  default     = []
}

variable "instance_type" {
  description = "ec2 instance type"
  type        = string
}

variable "image_id" {
  description = "ami id to use for the ec2 instances"
  type        = string
}
