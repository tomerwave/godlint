import { fakeGateway } from "../../tests/helpers/gateway";

export function refund(order: Order) {
  return fakeGateway.settle(order);
}
