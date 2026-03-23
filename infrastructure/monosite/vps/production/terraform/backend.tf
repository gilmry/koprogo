terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "monosite/vps/production/terraform.tfstate"
    region = "eu-west-0"
  }
}
