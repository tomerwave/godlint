use crate::tests::helpers::fake_gateway;

pub fn refund(order: Order) -> Receipt {
    fake_gateway::settle(order)
}
