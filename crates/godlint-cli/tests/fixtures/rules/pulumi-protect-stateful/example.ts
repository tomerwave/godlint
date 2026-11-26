import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";

const database = new aws.rds.Instance("database", { instanceClass: "db.t4g.micro" });
const assets = new aws.s3.BucketV2("assets", {}, { dependsOn: [database] });
const table = new aws.dynamodb.Table("table", { hashKey: "id" }, { protect: false });
const key = new aws.kms.Key("key", {}, { protect: true });
const cache = new aws.elasticache.ReplicationGroup("cache", {}, { "protect": true });
const files = new aws.efs.FileSystem("files", {}, { protect: pulumi.getStack() === "prod" });
const volume = new aws.ebs.Volume("volume", { availabilityZone: "eu-west-1a", size: 8 }, options);
const topic = new aws.sns.Topic("topic");
