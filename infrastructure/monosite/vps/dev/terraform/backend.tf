terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "monosite/vps/dev/terraform.tfstate"
    region = "eu-west-0"
  }
}
