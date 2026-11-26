import { s3, rds } from "@pulumi/aws/index.js";

const bucket = new s3.Bucket("bucket");
const cluster = new rds.Cluster("cluster", {}, { ...defaults });
