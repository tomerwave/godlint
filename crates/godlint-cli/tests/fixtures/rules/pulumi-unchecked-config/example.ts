import * as pulumi from "@pulumi/pulumi";

interface Network {
  cidr: string;
}

const config = new pulumi.Config();
const network = config.requireObject<Network>("network");
const zones = config.getObject<string[]>("zones");
const sizes = config.requireObject<{ min: number; max: number }>("sizes");
const tier = config.getObject<"small" | "large">("tier");
const token = config?.requireSecretObject<{ value: string }>("token");
const inline = new pulumi.Config("app").requireObject<Network>("network");
const plain = config.requireObject("plain");
