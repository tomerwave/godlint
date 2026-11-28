import * as pulumi from "@pulumi/pulumi";

const config = new pulumi.Config();
const network = config.requireObject("network");
