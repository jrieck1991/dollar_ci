variable "tags" {
  type        = "map"
  description = "resource tags"
  default     = {}
}

variable "name" {
  description = "name of resources"
  type = "string"
}

variable "subnet_ids" {
  description = "list of subnet ids"
  type = "list"
  default = []
}
