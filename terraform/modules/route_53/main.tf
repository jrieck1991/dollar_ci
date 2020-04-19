// get dollar-ci.com hosted zone
// manually created
data "aws_route53_zone" "main" {
  name = "dollar-ci.com."
}

// endpoint to receive webhooks from git events
resource "aws_route53_record" "http" {
  zone_id = data.aws_route53_zone.http.zone_id
  name    = "www.dollar-ci.com/events"
  type    = "A"
  ttl     = "300"
  records = [var.http_ip]
}
