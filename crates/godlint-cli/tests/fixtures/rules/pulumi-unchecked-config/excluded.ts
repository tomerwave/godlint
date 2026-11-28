import * as pulumi from "@pulumi/pulumi";

const legacy = new pulumi.Config().getSecretObject<string>("legacy");
const config = new pulumi.Config();
const kept = config.getSecretObject<string>("kept");
