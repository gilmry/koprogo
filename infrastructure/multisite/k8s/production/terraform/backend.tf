terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "multisite/k8s/production/terraform.tfstate"
    region = "eu-west-0"
  }
}
