// get custom ami built by packer
data "aws_ami" "main" {
  executable_users = ["self"]
  most_recent      = true
  name_regex       = "^http_handlers_*"
  owners           = ["self"]

  filter {
    name   = "root-device-type"
    values = ["ebs"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

// get subnets from vpc id
data "aws_subnet_ids" "public" {
  vpc_id = var.vpc_id
}

// a launch template for an autoscaling group with
// spot instances
resource "aws_launch_template" "main" {
  name_prefix   = var.name
  image_id      = data.aws_ami.main.image_id
  instance_type = var.instance_type

  iam_instance_profile {
    arn = aws_iam_instance_profile.asg.arn
  }

  instance_market_options {
    market_type = "spot"
  }

  vpc_security_group_ids = [aws_security_group.asg.id]

  user_data = filebase64("${path.module}/provision.sh")
}

// autoscaling group of ec2 spot instances
resource "aws_autoscaling_group" "main" {
  min_size = 1
  max_size = 1

  launch_template {
    id      = aws_launch_template.main.id
    version = aws_launch_template.main.latest_version
  }

  vpc_zone_identifier = data.aws_subnet_ids.public.ids
}

// IAM resources used to enabled SSM on ec2 instances in the ASG
resource "aws_iam_instance_profile" "asg" {
  name = var.name
  role = aws_iam_role.asg.name
}

// role for instance profile
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
  arn = "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore"
}

resource "aws_iam_role_policy_attachment" "asg" {
  role       = aws_iam_role.asg.name
  policy_arn = data.aws_iam_policy.ssm.arn
}

// ec2 instance security group
resource "aws_security_group" "asg" {
  name        = var.name
  description = "Allow http traffic on port 80"
  vpc_id = var.vpc_id

  // http in
  ingress {
    description = "http"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  // any out
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = var.tags
}