import * as pulumi from "@pulumi/pulumi";
import { z } from "zod";

const Network = z.object({ cidr: z.string() });

const config = new pulumi.Config();
const network = Network.parse(config.requireObject("network"));
const zones = z.array(z.string()).parse(config.getObject("zones") ?? []);
const name = config.require<string>("name");
