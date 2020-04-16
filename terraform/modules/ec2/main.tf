resource "aws_launch_template" "main" {
  name_prefix   = var.name
  image_id      = var.image_id
  instance_type = "t2.micro"
}

resource "aws_autoscaling_group" "main" {
  desired_capacity   = 1
  max_size           = 1
  min_size           = 1

  launch_template {
    id      = aws_launch_template.main.id
    version = "$Latest"
  }

  vpc_zone_identifier = [var.subnet_ids]
}