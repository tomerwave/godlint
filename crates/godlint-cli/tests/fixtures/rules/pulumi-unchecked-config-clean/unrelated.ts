import { settings } from "./settings";

const network = settings.requireObject<{ cidr: string }>("network");
