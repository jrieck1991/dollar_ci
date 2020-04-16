resource "aws_launch_template" "main" {
  name_prefix   = var.name
  image_id      = var.image_id
  instance_type = var.instance_type
}

resource "aws_autoscaling_group" "main" {
  min_size = 1
  max_size = 1

  launch_template {
    id      = aws_launch_template.main.id
    version = "$Latest"
  }

  vpc_zone_identifier = var.subnet_ids
}
