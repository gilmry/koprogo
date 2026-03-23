terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "monosite/k3s/staging/terraform.tfstate"
    region = "eu-west-0"
  }
}
