terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "monosite/vps/staging/terraform.tfstate"
    region = "eu-west-0"
  }
}
