resource "aws_launch_template" "main" {
  name_prefix   = var.name
  image_id      = var.image_id
  instance_type = var.instance_type
  key_name      = "jack"

  iam_instance_profile {
    arn = aws_iam_instance_profile.asg.arn
  }
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

// IAM resources used to enabled SSM on ec2 instances in the ASG
resource "aws_iam_instance_profile" "asg" {
  name = var.name
  role = aws_iam_role.asg.name
}

resource "aws_iam_role" "asg" {
  name = var.name

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": "sts:AssumeRole",
      "Principal": {
        "Service": "ec2.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}

// get managed aws SSM policy
data "aws_iam_policy" "ssm" {
  arn = "arn:aws:iam::aws:policy/service-role/AmazonEC2RoleforSSM"
}

resource "aws_iam_role_policy_attachment" "asg" {
  role       = aws_iam_role.asg.name
  policy_arn = data.aws_iam_policy.ssm.arn
}
