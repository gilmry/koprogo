terraform {
  backend "s3" {
    bucket = "koprogo-tfstate"
    key    = "monosite/vps/dev/terraform.tfstate"
    region = "eu-west-0"

    # CyFun PR.DS-1: state holds IPs, outputs and potentially secrets.
    # encrypt sends x-amz-server-side-encryption: AES256 (SSE-OMK on OVH).
    encrypt = true
    # Native S3 locking via conditional writes (Terraform >= 1.10,
    # supported by OVHcloud Object Storage): no concurrent applies.
    use_lockfile = true
  }
}
