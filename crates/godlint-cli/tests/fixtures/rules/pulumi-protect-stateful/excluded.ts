import * as aws from "@pulumi/aws";

const scratch = new aws.s3.Bucket("scratch");
