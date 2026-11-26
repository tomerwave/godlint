import * as aws from "@pulumi/aws";

const database = new aws.rds.Instance("database", { instanceClass: "db.t4g.micro" }, { protect: true });
const bucket = new aws.s3.Bucket("bucket", {}, { protect: true, retainOnDelete: true });
