terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "multisite/k8s/dev/terraform.tfstate"
    region = "eu-west-0"
  }
}
