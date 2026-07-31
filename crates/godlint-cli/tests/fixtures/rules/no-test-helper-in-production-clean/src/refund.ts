import { gateway } from "./gateway";
import { settle } from "some-lib/tests/util";

export function refund(order: Order) {
  return gateway.settle(order);
}
