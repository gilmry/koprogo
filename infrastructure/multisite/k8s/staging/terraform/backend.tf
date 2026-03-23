terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "multisite/k8s/staging/terraform.tfstate"
    region = "eu-west-0"
  }
}
